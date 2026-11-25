#![cfg(all(feature = "std", feature = "combinators", feature = "logos_0_16"))]

//! The flat driver against the typed driver, over one table and generated expressions: `folding`
//! with the typed driver's folds, `parse_into` with folds that read nothing on, and the bare mode
//! against the checked one.
//!
//! The typed side is `pratt(TABLE.lhs(operand), TABLE.rhs(), ..)`, recursive and unchanged. For
//! each input `folding` must build the typed driver's tree, with the same folds in the same order,
//! or fail with the same error at the same offset, and end where the typed driver ends.
//! `parse_into` must end where it ends, and tell each token it consumed once and in order. Its
//! items, fed to a machine alone with no input, must resolve to the typed driver's tree. A missing
//! operand is compared by its offset alone: the flat driver says which of the two errors it is,
//! and the typed driver's left-hand-side channel cannot.
//!
//! Over a lexer that skips trivia every position is the typed driver's. Over one that surfaces it,
//! the flat driver commits the trivia it reads before an ending, and the typed driver hands it
//! back: an `Ok` ends, and a chain is refused, at `past` the typed driver's position, which is the
//! position after every trivia token from it. A prefix or an infix fold reads `span().end()` at
//! `past` the typed driver's too, unless it read on, and a postfix fold reads the typed driver's.
//! A missing operand's offset is the typed driver's. `past` is the test's own: it walks the source
//! the generator wrote.
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
  Fail, FatalIr, Fold, Landing, Operand, TABLE, fold_infix, fold_postfix, fold_prefix,
  forget_lexes, landing, lexes, read_infix, read_postfix, read_prefix, take_folds,
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

/// What a generator writes at each position, and what a mutation draws from.
struct Alphabet {
  prefixes: &'static [u8],
  infixes: &'static [u8],
  postfixes: &'static [u8],
  any: &'static [u8],
  /// Whether a term may be followed by the tail an `i` reads, and a `[` by its rest.
  tails: bool,
}

/// Every row of the table but `@` and the four whose folds read on. A mutation also draws the
/// terminator, and a byte no row names.
const QUIET: Alphabet = Alphabet {
  prefixes: b"-~",
  infixes: b"+-&*^=<>",
  postfixes: b"!?%",
  any: b"0123456789-~+&*^=<>!?%;#",
  tails: false,
};

/// Every row but `@`, the four whose folds read on among them, and their tails.
const READS: Alphabet = Alphabet {
  prefixes: b"-~p",
  infixes: b"+-&*^=<>it",
  postfixes: b"!?%[",
  any: b"0123456789-~p+&*^=<>it!?%[]e;#",
  tails: true,
};

/// A well-formed expression: terms joined by infix operators, each term an operand with prefix
/// operators before it and postfix operators after it.
fn well_formed(rng: &mut Rng, alphabet: &Alphabet) -> Vec<u8> {
  let mut tokens = Vec::new();
  for term in 0..1 + rng.below(4) {
    if term > 0 {
      tokens.push(rng.pick(alphabet.infixes));
    }
    for _ in 0..rng.below(3) {
      tokens.push(rng.pick(alphabet.prefixes));
    }
    tokens.push(rng.pick(OPERANDS));
    for _ in 0..rng.below(3) {
      let postfix = rng.pick(alphabet.postfixes);
      tokens.push(postfix);
      if alphabet.tails && postfix == b'[' {
        tokens.extend([rng.pick(OPERANDS), b']']);
      }
    }
    if alphabet.tails && rng.below(4) == 0 {
      tokens.extend([b'e', rng.pick(OPERANDS)]);
    }
  }
  tokens
}

/// A well-formed expression with one to three tokens deleted, inserted or replaced.
fn mutated(rng: &mut Rng, alphabet: &Alphabet) -> Vec<u8> {
  let mut tokens = well_formed(rng, alphabet);
  for _ in 0..1 + rng.below(3) {
    let at = rng.below(tokens.len() + 1);
    match (rng.below(3), at < tokens.len()) {
      (0, true) => {
        tokens.remove(at);
      }
      (1, _) | (_, false) => tokens.insert(at, rng.pick(alphabet.any)),
      (_, true) => tokens[at] = rng.pick(alphabet.any),
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
fn sources(seed: u64, spaces: usize, alphabet: &Alphabet) -> Vec<String> {
  let mut rng = Rng(seed);
  (0..CASES)
    .map(|case| {
      let tokens = if case % 2 == 0 {
        well_formed(&mut rng, alphabet)
      } else {
        mutated(&mut rng, alphabet)
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

/// What a driver that folds did with one source, under the fixture's recorded folds: what it
/// returned, its folds in order, where it left the input, what the input had charged once the next
/// token was read, and every lex its parse made.
#[derive(Debug)]
struct Folded {
  result: Result<String, Fail>,
  folds: Vec<Fold>,
  landing: Landing,
  total: usize,
  lexed: Vec<(usize, u8)>,
}

/// The typed driver, with the recorded folds.
fn traced<const SKIP: bool>(src: &str, operand: Operand, floor: u8) -> Folded {
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(|input: &mut FatalIr<'_, '_, SKIP>| {
      forget_lexes();
      take_folds();
      let result = pratt(
        TABLE.lhs(operand),
        TABLE.rhs(),
        read_prefix,
        read_infix,
        read_postfix,
      )
      .min_precedence(floor)
      .parse_input(input);
      let (folds, lexed) = (take_folds(), lexes());
      let landing = landing(input)?;
      Ok(Folded {
        result,
        folds,
        landing,
        total: input.token_budget().spent(),
        lexed,
      })
    })
    .parse_str(src)
    .expect("the parse's own error is caught")
}

/// `folding`, with the recorded folds.
fn folded<const SKIP: bool>(src: &str, operand: Operand, floor: u8) -> Folded {
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(|input: &mut FatalIr<'_, '_, SKIP>| {
      forget_lexes();
      take_folds();
      let result = flat(TABLE, operand)
        .min_precedence(floor)
        .folding(read_prefix, read_infix, read_postfix)
        .parse_input(input);
      let (folds, lexed) = (take_folds(), lexes());
      let landing = landing(input)?;
      Ok(Folded {
        result,
        folds,
        landing,
        total: input.token_budget().spent(),
        lexed,
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

/// `folding` against the typed driver, on one source: the same tree, or the same error at its
/// offset, and the same ending; and the same folds in the same order, each at its position.
fn assert_folds_as_typed<const SKIP: bool>(src: &str, typed: &Folded, folded: &Folded, case: &str) {
  let moved = |at: usize| if SKIP { at } else { past(src, at) };
  match (&typed.result, &folded.result) {
    (Ok(tree), Ok(value)) => {
      assert_eq!(value, tree, "{case}: the tree");
      let handback = moved(typed.landing.handback);
      assert_eq!(folded.landing.handback, handback, "{case}: the ending");
      let next = if SKIP {
        typed.landing.next
      } else {
        token_at(src, handback)
      };
      assert_eq!(folded.landing.next, next, "{case}: the next token");
    }
    (Err(Fail::Chain { at }), Err(Fail::Chain { at: folded_at })) => {
      assert_eq!(*folded_at, moved(*at), "{case}: the chain's offset");
      assert_eq!(
        folded.landing.handback,
        moved(*at),
        "{case}: the chain's handback"
      );
    }
    (Err(error @ (Fail::EoLhs { at, .. } | Fail::EoRhs { at, .. })), Err(folded_error)) => {
      assert!(
        folded_error.is_missing_at(*at),
        "{case}: the typed driver's {error:?}, and folding's {folded_error:?}"
      );
    }
    (typed_result, folded_result) => {
      panic!("{case}: the typed driver returned {typed_result:?}, and folding {folded_result:?}")
    }
  }
  assert_eq!(
    folded.folds.len(),
    typed.folds.len(),
    "{case}: {:?}, where the typed driver folds {:?}",
    folded.folds,
    typed.folds
  );
  for (fold, typed_fold) in folded.folds.iter().zip(&typed.folds) {
    // A prefix or an infix fold runs once the head after its last operand is read, past the trivia
    // before it, unless the fold read on. A postfix fold runs where its operator was read.
    let postfix = matches!(fold.byte, b'!' | b'?' | b'%' | b'[');
    let end = if SKIP || fold.read || postfix {
      typed_fold.end
    } else {
      past(src, typed_fold.end)
    };
    assert_eq!(
      (fold.byte, fold.at, fold.read, fold.end),
      (typed_fold.byte, typed_fold.at, typed_fold.read, end),
      "{case}: {:?}, where the typed driver folds {:?}",
      folded.folds,
      typed.folds
    );
  }
}

/// What the two drivers lexed, with folds that read nothing on and a lexer that skips trivia.
/// `folding` lexes no token twice, so its charge is the number of tokens it read. The typed
/// driver's is at least that once the next token is read, and above it wherever the parse returns
/// `Ok`: that parse ends on a probe it rolled back, whose token the next read lexes again.
fn assert_lexes(typed: &Folded, folded: &Folded, case: &str) {
  let mut offsets: Vec<usize> = folded.lexed.iter().map(|&(at, _)| at).collect();
  offsets.sort_unstable();
  offsets.dedup();
  assert_eq!(
    offsets.len(),
    folded.lexed.len(),
    "{case}: no token is lexed twice"
  );
  assert_eq!(
    folded.landing.spent,
    folded.lexed.len(),
    "{case}: the charge is the tokens read"
  );
  assert!(
    typed.total >= folded.total,
    "{case}: the typed driver charged {}, and folding {}",
    typed.total,
    folded.total
  );
  if typed.result.is_ok() {
    assert!(
      typed.total > folded.total,
      "{case}: the typed driver charged {}, and folding {}",
      typed.total,
      folded.total
    );
  }
}

/// Every source of one leg, under both operand parsers, at each floor: `folding` against the typed
/// driver, and with `count_lexes`, what each lexed.
fn folding_leg<const SKIP: bool>(seed: u64, spaces: usize, alphabet: &Alphabet, count_lexes: bool) {
  for (index, src) in sources(seed, spaces, alphabet).iter().enumerate() {
    let operand = if index % 2 == 0 {
      Operand::STRICT
    } else {
      Operand::CROSSING
    };
    for floor in FLOORS {
      let case = format!("{src:?} at {floor}, {operand:?}");
      let typed = traced::<SKIP>(src, operand, floor);
      let folded = folded::<SKIP>(src, operand, floor);
      assert_folds_as_typed::<SKIP>(src, &typed, &folded, &case);
      if count_lexes {
        assert_lexes(&typed, &folded, &case);
      }
    }
  }
}

/// Every source of one leg, under both operand parsers, at each floor, in both modes.
fn leg<const SKIP: bool>(seed: u64, spaces: usize) {
  for (index, src) in sources(seed, spaces, &QUIET).iter().enumerate() {
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
  let folded = folded::<false>(src, Operand::STRICT, 0);
  assert_eq!(folded.result, Ok(String::from("(1+2)")));
  assert_eq!(folded.landing.handback, 8);
  assert_eq!(folded.landing.next, Some((b';', 8)));
}

/// `folding` against the typed driver over a lexer that skips trivia, with folds that read nothing
/// on: every position is the typed driver's, and what each lexed is compared.
#[test]
fn folding_against_the_typed_driver_over_a_skipping_lexer() {
  folding_leg::<true>(0x2545_f491_4f6c_dd1d, 1, &QUIET, true);
}

/// Over a lexer that surfaces trivia, each position by the rule of `past`.
#[test]
fn folding_against_the_typed_driver_over_a_surfacing_lexer() {
  folding_leg::<false>(0x6a09_e667_f3bc_c908, 3, &QUIET, false);
}

/// With the four rows whose folds read on: an infix operator that reads a tail after its right
/// operand, one that takes an operator that follows, a postfix operator that reads a bracketed
/// rest, and a prefix operator that takes an operator that follows its operand.
#[test]
fn folding_against_the_typed_driver_with_folds_that_read_on() {
  folding_leg::<true>(0xbb67_ae85_84ca_a73b, 1, &READS, false);
  folding_leg::<false>(0x3c6e_f372_fe94_f82b, 3, &READS, false);
}

/// The two shapes where a driver that reduced every frame an operator does not admit would part
/// from the typed driver: `1 i 2 e 3 + 4`, where the `i`'s fold reads `e 3`, is
/// `(1 i 2 e 3) + 4`; and `~ 1 t 2 + ^ 3`, where the `t`'s fold takes the `+`, is
/// `~((1 t 2 +) ^ 3)`.
#[test]
fn the_shapes_where_a_fold_reads_on_group_as_the_typed_driver_groups_them() {
  for (src, tree) in [("1i2e3+4;", "((1i2e3)+4)"), ("~1t2+^3;", "(~((1t2+)^3))")] {
    let typed = traced::<true>(src, Operand::STRICT, 0);
    let folded = folded::<true>(src, Operand::STRICT, 0);
    assert_eq!(typed.result.as_deref(), Ok(tree), "{src}");
    assert_folds_as_typed::<true>(src, &typed, &folded, src);
  }
}

/// On an input that fails before any probe is rolled back, the two drivers lex alike: `+1;` is
/// one lex under both, and `1+;` is three.
#[test]
fn where_no_probe_is_rolled_back_the_two_drivers_lex_alike() {
  for (src, lexed) in [("+1;", 1), ("1+;", 3)] {
    let typed = traced::<true>(src, Operand::STRICT, 0);
    let folded = folded::<true>(src, Operand::STRICT, 0);
    assert_eq!((typed.landing.spent, typed.total), (lexed, lexed), "{src}");
    assert_eq!(
      (folded.landing.spent, folded.total),
      (lexed, lexed),
      "{src}"
    );
  }
}
