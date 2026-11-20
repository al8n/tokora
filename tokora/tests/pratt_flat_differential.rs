#![cfg(all(feature = "std", feature = "combinators", feature = "logos_0_16"))]

//! The flat driver's `parse_into` against the typed driver, over one table and generated
//! expressions, and its bare mode against its checked mode.
//!
//! The typed side is `pratt(TABLE.lhs(operand), TABLE.rhs(), ..)`, recursive and unchanged, with
//! folds that read nothing on. For each input the flat driver must end where the typed driver
//! ends, or fail with the same error at the same offset, and tell each token it consumed once and
//! in order. Its items, fed to a machine alone with no input, must resolve to the typed driver's
//! tree. A missing operand is compared by its offset alone: the flat driver says which of the two
//! errors it is, and the typed driver's left-hand-side channel cannot.
//!
//! Over a lexer that skips trivia every position is the typed driver's. Over one that surfaces it,
//! the flat driver commits the trivia it reads before an ending, and the typed driver hands it
//! back: an `Ok` ends, and a chain is refused, at `past` the typed driver's position, which is the
//! position after every trivia token from it. A missing operand's offset is the typed driver's.
//! `past` is the test's own: it walks the source the generator wrote.
//!
//! The generator writes no `@`. The typed driver's channel refuses an `Adjacent` row below the
//! floor too, where the flat driver declines it, and the loop-contract suite pins each.

mod flat_fixture;
mod flat_items;

use tokora::{
  Parse, ParseInput, Parser, ParserContext,
  emitter::Fatal,
  pratt::{flat, pratt},
};

use flat_fixture::{
  Fail, FatalIr, Landing, Operand, TABLE, fold_infix, fold_postfix, fold_prefix, forget_lexes,
  landing, lexes,
};
use flat_items::{Item, Items, replay};

/// How many sources each leg generates. Miri interprets each parse, so it reads a few.
const CASES: usize = if cfg!(miri) { 4 } else { 2000 };

/// The floors each source is parsed at: one below every power, one between the infix powers, one
/// at the non-associative power, and one above every infix power but `^`.
const FLOORS: [u8; 4] = [0, 11, 15, 25];

// ── The generator ─────────────────────────────────────────────────────────────────────────────

/// A small deterministic generator: xorshift64.
struct Rng(u64);

impl Rng {
  fn below(&mut self, n: usize) -> usize {
    self.0 ^= self.0 << 13;
    self.0 ^= self.0 >> 7;
    self.0 ^= self.0 << 17;
    (self.0 % n as u64) as usize
  }

  fn pick(&mut self, bytes: &[u8]) -> u8 {
    bytes[self.below(bytes.len())]
  }
}

const OPERANDS: &[u8] = b"0123456789";
const PREFIXES: &[u8] = b"-~";
const INFIXES: &[u8] = b"+-&*^=<>";
const POSTFIXES: &[u8] = b"!?%";
/// What a mutation draws from: every row of the table but `@`, the terminator, and a byte no row
/// names.
const ANY: &[u8] = b"0123456789-~+&*^=<>!?%;#";

/// A well-formed expression: terms joined by infix operators, each term an operand with prefix
/// operators before it and postfix operators after it.
fn well_formed(rng: &mut Rng) -> Vec<u8> {
  let mut tokens = Vec::new();
  for term in 0..1 + rng.below(4) {
    if term > 0 {
      tokens.push(rng.pick(INFIXES));
    }
    for _ in 0..rng.below(3) {
      tokens.push(rng.pick(PREFIXES));
    }
    tokens.push(rng.pick(OPERANDS));
    for _ in 0..rng.below(3) {
      tokens.push(rng.pick(POSTFIXES));
    }
  }
  tokens
}

/// A well-formed expression with one to three tokens deleted, inserted or replaced.
fn mutated(rng: &mut Rng) -> Vec<u8> {
  let mut tokens = well_formed(rng);
  for _ in 0..1 + rng.below(3) {
    let at = rng.below(tokens.len() + 1);
    match (rng.below(3), at < tokens.len()) {
      (0, true) => {
        tokens.remove(at);
      }
      (1, _) | (_, false) => tokens.insert(at, rng.pick(ANY)),
      (_, true) => tokens[at] = rng.pick(ANY),
    }
  }
  tokens
}

/// The source of `tokens` and a terminator, with up to `spaces` spaces before each token.
fn write(rng: &mut Rng, tokens: &[u8], spaces: usize) -> String {
  let mut src = String::new();
  for &byte in tokens.iter().chain(b";") {
    for _ in 0..rng.below(spaces + 1) {
      src.push(' ');
    }
    src.push(char::from(byte));
  }
  src
}

/// The sources of one leg: half well-formed and half mutated.
fn sources(seed: u64, spaces: usize) -> Vec<String> {
  let mut rng = Rng(seed);
  (0..CASES)
    .map(|case| {
      let tokens = if case % 2 == 0 {
        well_formed(&mut rng)
      } else {
        mutated(&mut rng)
      };
      write(&mut rng, &tokens, spaces)
    })
    .collect()
}

/// The offset after every space from `at`.
fn past(src: &str, at: usize) -> usize {
  at + src.as_bytes()[at..]
    .iter()
    .take_while(|&&byte| byte == b' ')
    .count()
}

/// The token at `at`, and its offset.
fn token_at(src: &str, at: usize) -> Option<(u8, usize)> {
  src.as_bytes().get(at).map(|&byte| (byte, at))
}

// ── The drivers ───────────────────────────────────────────────────────────────────────────────

/// What the typed driver did with one source.
#[derive(Debug)]
struct Typed {
  result: Result<String, Fail>,
  landing: Landing,
}

/// What a flat driver did with one source: what it returned, what its handler was told, where it
/// left the input, and every lex made, the next token's read included.
#[derive(Debug)]
struct Told {
  result: Result<(), Fail>,
  items: Vec<Item>,
  landing: Landing,
  lexed: Vec<(usize, u8)>,
}

fn typed<const SKIP: bool>(src: &str, operand: Operand, floor: u8) -> Typed {
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(|input: &mut FatalIr<'_, '_, SKIP>| {
      let result = pratt(
        TABLE.lhs(operand),
        TABLE.rhs(),
        fold_prefix,
        fold_infix,
        fold_postfix,
      )
      .min_precedence(floor)
      .parse_input(input);
      Ok(Typed {
        result,
        landing: landing(input)?,
      })
    })
    .parse_str(src)
    .expect("the parse's own error is caught")
}

/// A checked driver at `Some(floor)`, or a bare driver at `None`.
fn told<const SKIP: bool>(src: &str, operand: Operand, floor: Option<u8>) -> Told {
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(|input: &mut FatalIr<'_, '_, SKIP>| {
      forget_lexes();
      let mut items = Items::default();
      let result = match floor {
        Some(floor) => flat(TABLE, operand)
          .min_precedence(floor)
          .parse_into(input, &mut items),
        None => flat(TABLE, operand).bare().parse_into(input, &mut items),
      };
      let landing = landing(input)?;
      Ok(Told {
        result,
        items: items.0,
        landing,
        lexed: lexes(),
      })
    })
    .parse_str(src)
    .expect("the parse's own error is caught")
}

// ── The checks ────────────────────────────────────────────────────────────────────────────────

/// Each token the flat driver consumed, but trivia, is told once and in written order, and each
/// operator is told at its own offset. No token is lexed twice.
fn assert_told_once_in_order(src: &str, told: &Told, case: &str) {
  let consumed: Vec<(u8, usize)> = src.as_bytes()[..told.landing.handback]
    .iter()
    .enumerate()
    .filter(|&(_, &byte)| byte != b' ')
    .map(|(at, &byte)| (byte, at))
    .collect();
  assert_eq!(
    told.items.len(),
    consumed.len(),
    "{case}: each consumed token is told once: {:?}",
    told.items
  );
  for (item, &(byte, at)) in told.items.iter().zip(&consumed) {
    assert_eq!(item.byte(), byte, "{case}: in written order");
    let told_at = match item {
      Item::Prefix(_, at, _) | Item::Infix(_, at, ..) | Item::Postfix(_, at, _) => Some(*at),
      Item::Operand(_) => None,
    };
    assert!(
      told_at.is_none_or(|told_at| told_at == at),
      "{case}: {item:?} is told at its own offset, {at}"
    );
  }
  let mut offsets: Vec<usize> = told.lexed.iter().map(|&(at, _)| at).collect();
  offsets.sort_unstable();
  offsets.dedup();
  assert_eq!(
    offsets.len(),
    told.lexed.len(),
    "{case}: no token is lexed twice"
  );
}

/// The flat driver against the typed driver, on one source at one floor. With `SKIP`, every
/// position is the typed driver's; without it, the endings are at `past` the typed driver's.
fn assert_same_ending<const SKIP: bool>(
  src: &str,
  floor: u8,
  typed: &Typed,
  told: &Told,
  case: &str,
) {
  let moved = |at: usize| if SKIP { at } else { past(src, at) };
  match (&typed.result, &told.result) {
    (Ok(tree), Ok(())) => {
      assert_eq!(
        &replay(&told.items, floor),
        tree,
        "{case}: the items resolve to the tree"
      );
      let handback = moved(typed.landing.handback);
      assert_eq!(told.landing.handback, handback, "{case}: the ending");
      let next = if SKIP {
        typed.landing.next
      } else {
        token_at(src, handback)
      };
      assert_eq!(told.landing.next, next, "{case}: the next token");
    }
    (Err(Fail::Chain { at }), Err(Fail::Chain { at: told_at })) => {
      assert_eq!(*told_at, moved(*at), "{case}: the chain's offset");
      assert_eq!(
        told.landing.handback,
        moved(*at),
        "{case}: the chain's handback"
      );
    }
    (Err(error @ (Fail::EoLhs { at, .. } | Fail::EoRhs { at, .. })), Err(told_error)) => {
      assert!(
        told_error.is_missing_at(*at),
        "{case}: the typed driver's {error:?}, and the flat driver's {told_error:?}"
      );
    }
    (typed_result, told_result) => panic!(
      "{case}: the typed driver returned {typed_result:?}, and the flat driver {told_result:?}"
    ),
  }
}

/// A bare driver against a checked one at a floor below every power and with no cap: the same
/// outcome, items, position and lexes, unless the checked driver refuses a chain. There the bare
/// driver's items begin with the checked driver's, and the next is the repeated operator.
fn assert_bare_is_checked(checked: &Told, bare: &Told, case: &str) {
  if let Err(Fail::Chain { .. }) = checked.result {
    assert!(
      bare.items.starts_with(&checked.items),
      "{case}: {:?} begins with {:?}",
      bare.items,
      checked.items
    );
    let next = checked
      .landing
      .next
      .expect("a refused chain leaves its operator on the input");
    assert!(
      matches!(
        bare.items.get(checked.items.len()),
        Some(Item::Infix(byte, at, ..)) if (*byte, *at) == next
      ),
      "{case}: then the repeated operator, {next:?}: {:?}",
      bare.items
    );
    return;
  }
  assert_eq!(bare.result, checked.result, "{case}");
  assert_eq!(bare.items, checked.items, "{case}");
  assert_eq!(bare.landing, checked.landing, "{case}");
  assert_eq!(bare.lexed, checked.lexed, "{case}: the reads are equal");
}

/// Every source of one leg, under both operand parsers, at each floor, in both modes.
fn leg<const SKIP: bool>(seed: u64, spaces: usize) {
  for (index, src) in sources(seed, spaces).iter().enumerate() {
    // Half the sources run an operand parser that crosses no trivia itself, so the driver's own
    // crossing at a term is exercised.
    let operand = if index % 2 == 0 {
      Operand::STRICT
    } else {
      Operand::CROSSING
    };
    for floor in FLOORS {
      let case = format!("{src:?} at {floor}, {operand:?}");
      let typed = typed::<SKIP>(src, operand, floor);
      let checked = told::<SKIP>(src, operand, Some(floor));
      assert_told_once_in_order(src, &checked, &case);
      assert_same_ending::<SKIP>(src, floor, &typed, &checked, &case);
      if floor == 0 {
        let bare = told::<SKIP>(src, operand, None);
        assert_told_once_in_order(src, &bare, &case);
        assert_bare_is_checked(&checked, &bare, &case);
      }
    }
  }
}

/// Over a lexer that skips trivia, every position is the typed driver's.
#[test]
fn parse_into_against_the_typed_driver_over_a_skipping_lexer() {
  leg::<true>(0x9e37_79b9_7f4a_7c15, 1);
}

/// Over a lexer that surfaces trivia, the endings are at `past` the typed driver's.
#[test]
fn parse_into_against_the_typed_driver_over_a_surfacing_lexer() {
  leg::<false>(0xd1b5_4a32_d192_ed03, 3);
}

/// Pinned: `1 + 2   ; 3` ends at 8 under the flat driver, past the trivia it read, and at 5 under
/// the typed driver, which hands that trivia back.
#[test]
fn the_flat_driver_ends_past_the_trivia_it_read() {
  let src = "1 + 2   ; 3";
  let typed = typed::<false>(src, Operand::STRICT, 0);
  assert_eq!(typed.result, Ok(String::from("(1+2)")));
  assert_eq!(typed.landing.handback, 5);
  for floor in [Some(0), None] {
    let told = told::<false>(src, Operand::STRICT, floor);
    assert_eq!(told.result, Ok(()), "{floor:?}");
    assert_eq!(told.landing.handback, 8, "{floor:?}");
    assert_eq!(told.landing.next, Some((b';', 8)), "{floor:?}");
  }
}
