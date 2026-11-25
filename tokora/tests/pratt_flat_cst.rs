#![cfg(all(
  feature = "std",
  feature = "combinators",
  feature = "logos_0_16",
  feature = "rowan_0_17"
))]

//! The flat driver's CST seam over a recording sink: one node around every expression, flat by
//! construction, none left on a failing exit, with leading trivia outside it and trailing trivia
//! inside it, at each way an expression ends, under both modes of `parse_into` and under
//! `folding`.

mod flat_fixture;

use tokora::{
  Emitter, InputRef, ParseAttempt, ParseContext, ParseInput, SimpleSpan, Token as _, TryParseInput,
  cache::DefaultCache,
  cst::{
    CstProfile, FinishError, KindValidator, Sink, parse_lossless, parse_lossless_with_context,
  },
  emitter::{CstEmitter, Verbose},
  input::{InputContext, TokenBudget},
  pratt::{PrattFoldOp, PrattHandler, PrattInfix, PrattLHS, Precedenced, flat, pratt},
  rowan::{GreenNode, Language, NodeOrToken, SyntaxKind, SyntaxNode},
  span::Spanned,
  token::PrattToken,
};

use flat_fixture::{
  Expr, Fail, InfixOp, Mode, Surface, TABLE, Tok, fold_infix, fold_postfix, fold_prefix,
  read_infix, read_postfix, read_prefix,
};

// ── The kinds, the profile and the tree ──────────────────────────────────────────────────────

const K_ROOT: u16 = 1;
const K_EXPR: u16 = 2;
const K_LIT: u16 = 3;
const K_BIN: u16 = 4;
const K_HOLE: u16 = 5;
const K_ERR: u16 = 90;
const K_GAP: u16 = 91;

/// Token images: `100 + byte`.
fn map_tok(tok: &Tok) -> u16 {
  100 + u16::from(tok.0)
}

fn in_kind_space(kind: u16) -> bool {
  matches!(kind, K_ROOT..=K_HOLE | K_ERR | K_GAP) || kind >= 100
}

fn profile() -> CstProfile<Tok> {
  CstProfile::new(map_tok, KindValidator::new(in_kind_space), K_ERR, K_GAP)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Raw {}

impl Language for Raw {
  type Kind = u16;

  fn kind_from_raw(raw: SyntaxKind) -> u16 {
    raw.0
  }

  fn kind_to_raw(kind: u16) -> SyntaxKind {
    SyntaxKind(kind)
  }
}

fn name(kind: u16) -> &'static str {
  match kind {
    K_ROOT => "Root",
    K_EXPR => "Expr",
    K_LIT => "Lit",
    K_BIN => "Bin",
    K_HOLE => "Err",
    _ => "?",
  }
}

/// `Kind[child, child]`, with each token as its text and a space as `ws`.
fn render(node: &SyntaxNode<Raw>) -> String {
  let children: Vec<String> = node
    .children_with_tokens()
    .map(|child| match child {
      NodeOrToken::Node(node) => render(&node),
      NodeOrToken::Token(token) if token.text() == " " => String::from("ws"),
      NodeOrToken::Token(token) => String::from(token.text()),
    })
    .collect();
  format!("{}[{}]", name(node.kind()), children.join(", "))
}

/// How many nodes deep the tree is, its root included.
fn depth(node: &SyntaxNode<Raw>) -> usize {
  1 + node
    .children()
    .map(|child| depth(&child))
    .max()
    .unwrap_or(0)
}

type Ctx<'inp> = (
  Sink<'inp, Surface<'inp>, Verbose<Fail>>,
  DefaultCache<'inp, Surface<'inp>>,
);
type Ir<'inp, 'c> = InputRef<'inp, 'c, Surface<'inp>, Ctx<'inp>>;

/// What a lossless parse of `src` returned, the tree it materializes, and how many diagnostics it
/// left.
fn run<O>(
  src: &str,
  parser: impl FnMut(&mut Ir<'_, '_>) -> Result<O, Fail>,
) -> (Result<O, Fail>, Result<GreenNode, FinishError>, usize) {
  let (cst, result) = parse_lossless(
    src,
    Mode(0),
    Verbose::new(),
    profile(),
    DefaultCache::<Surface<'_>>::default(),
    parser,
  );
  let (green, emitter) = cst.finish(K_ROOT);
  let diagnostics = emitter.errors().values().map(Vec::len).sum();
  (result, green, diagnostics)
}

/// The tree of a parse that must balance, rendered, with its source checked against its text.
fn rendered(src: &str, green: Result<GreenNode, FinishError>) -> String {
  let root = SyntaxNode::<Raw>::new_root(green.expect("the parse balances and covers the source"));
  assert_eq!(
    root.text().to_string(),
    src,
    "the tree's text is the source"
  );
  render(&root)
}

/// What a lossless parse of `src`, within a budget of `budget` items, returned.
fn run_budgeted<O>(
  src: &str,
  budget: usize,
  parser: impl FnMut(&mut Ir<'_, '_>) -> Result<O, Fail>,
) -> Result<O, Fail> {
  let context = InputContext::new(Verbose::new(), DefaultCache::<Surface<'_>>::default())
    .with_token_budget(TokenBudget::with_limitation(budget));
  let (_cst, result) = parse_lossless_with_context(src, Mode(0), context, profile(), parser);
  result
}

/// Consumes what is left of the input, so that the tree covers the whole source.
fn drain(input: &mut Ir<'_, '_>) -> Result<(), Fail> {
  while input.next()?.is_some() {}
  Ok(())
}

// ── The operand parser ────────────────────────────────────────────────────────────────────────

/// A digit, wrapped in a node of its own. With `recovering`, a head that begins no operand and is
/// neither trivia nor a prefix operator is a hole: one diagnostic, and an empty error node. With
/// `failing`, a `#` is an error of the operand parser's own.
#[derive(Debug, Clone, Copy)]
struct Lit {
  recovering: bool,
  failing: bool,
}

const PLAIN: Lit = Lit {
  recovering: false,
  failing: false,
};

impl<'inp, Ctx> TryParseInput<'inp, Surface<'inp>, String, Ctx> for Lit
where
  Ctx: ParseContext<'inp, Surface<'inp>>,
  Ctx::Emitter: CstEmitter<'inp, Surface<'inp>> + Emitter<'inp, Surface<'inp>, Error = Fail>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, Surface<'inp>, Ctx>,
  ) -> Result<ParseAttempt<String>, Fail> {
    let mark = input.cst_mark();
    if let Some(tok) = input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
      input.cst_start_at(mark, K_LIT);
      input.cst_finish(K_LIT);
      return Ok(ParseAttempt::Accept(
        char::from(tok.into_data().0).to_string(),
      ));
    }
    let mut head = None;
    input.try_expect(|tok| {
      head = Some(Tok(tok.data().0));
      false
    })?;
    let Some(head) = head else {
      return Ok(ParseAttempt::Decline);
    };
    if self.failing && head.0 == b'#' {
      return Err(Fail::Own("the operand parser"));
    }
    let prefix = matches!(
      <Tok as PrattToken<'_, Expr, u8>>::try_pratt_lhs(&head),
      Some(PrattLHS::Prefix(_))
    );
    if !self.recovering || head.is_trivia() || prefix {
      return Ok(ParseAttempt::Decline);
    }
    let at = input.span().end();
    input.emit_error(Spanned::new(
      SimpleSpan::new(at, at),
      Fail::Own("an operand"),
    ))?;
    input.cst_start_at(mark, K_HOLE);
    input.cst_finish(K_HOLE);
    Ok(ParseAttempt::Accept(String::from("?")))
  }
}

/// A digit, and an error of its own where none is: an operand parser that takes a miss for an
/// error rather than declining it.
#[derive(Debug, Clone, Copy)]
struct Atom;

impl<'inp, Ctx> TryParseInput<'inp, Surface<'inp>, String, Ctx> for Atom
where
  Ctx: ParseContext<'inp, Surface<'inp>>,
  Ctx::Emitter: Emitter<'inp, Surface<'inp>, Error = Fail>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, Surface<'inp>, Ctx>,
  ) -> Result<ParseAttempt<String>, Fail> {
    match input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
      Some(tok) => Ok(ParseAttempt::Accept(
        char::from(tok.into_data().0).to_string(),
      )),
      None => Err(Fail::Own("an atom")),
    }
  }
}

/// A checked driver with a node of `K_EXPR`, and then the rest of the input drained.
fn checked(input: &mut Ir<'_, '_>, lit: Lit, floor: u8) -> Result<Result<(), Fail>, Fail> {
  let result = flat(TABLE, lit)
    .min_precedence(floor)
    .with_cst_kind(K_EXPR)
    .parse_into(input, &mut ());
  drain(input)?;
  Ok(result)
}

/// A bare driver with a node of `K_EXPR`, and then the rest of the input drained.
fn bare(input: &mut Ir<'_, '_>, lit: Lit) -> Result<Result<(), Fail>, Fail> {
  let result = flat(TABLE, lit)
    .bare()
    .with_cst_kind(K_EXPR)
    .parse_into(input, &mut ());
  drain(input)?;
  Ok(result)
}

/// A driver made by `folding` at `floor` with a node of `K_EXPR`, over the fixture's folds that
/// read on, and then the rest of the input drained.
fn folded(input: &mut Ir<'_, '_>, lit: Lit, floor: u8) -> Result<Result<(), Fail>, Fail> {
  let result = flat(TABLE, lit)
    .min_precedence(floor)
    .with_cst_kind(K_EXPR)
    .folding(read_prefix, read_infix, read_postfix)
    .parse_input(input)
    .map(|_| ());
  drain(input)?;
  Ok(result)
}

/// The tree of `src` under each mode and under `folding`, which must agree, with the parse's
/// result.
fn both(src: &str) -> (Result<(), Fail>, String) {
  let (result, green, _) = run(src, |input| checked(input, PLAIN, 0));
  let result = result.expect("the drain succeeds");
  let tree = rendered(src, green);
  let (bare_result, bare_green, _) = run(src, |input| bare(input, PLAIN));
  assert_eq!(
    bare_result.expect("the drain succeeds"),
    result,
    "{src}: the two modes agree"
  );
  assert_eq!(
    rendered(src, bare_green),
    tree,
    "{src}: the two modes agree"
  );
  let (folded_result, folded_green, _) = run(src, |input| folded(input, PLAIN, 0));
  assert_eq!(
    folded_result.expect("the drain succeeds"),
    result,
    "{src}: folding agrees"
  );
  assert_eq!(rendered(src, folded_green), tree, "{src}: folding agrees");
  (result, tree)
}

// ── Flat, and always one node ─────────────────────────────────────────────────────────────────

/// The expression is one node whose children are the operands' nodes and the operator tokens, in
/// written order, with the trivia between them. No node stands for an application.
#[test]
fn an_expression_is_one_flat_node() {
  let (result, tree) = both("1 + 2 * 3 - -4");
  assert_eq!(result, Ok(()));
  assert_eq!(
    tree,
    "Root[Expr[Lit[1], ws, +, ws, Lit[2], ws, *, ws, Lit[3], ws, -, ws, -, Lit[4]]]"
  );
}

/// A bare operand gets its node, and so does a prefix operator with its operand.
#[test]
fn every_expression_gets_its_node() {
  assert_eq!(both("7"), (Ok(()), String::from("Root[Expr[Lit[7]]]")));
  assert_eq!(both("-7"), (Ok(()), String::from("Root[Expr[-, Lit[7]]]")));
}

// ── The failing exit ──────────────────────────────────────────────────────────────────────────

/// Each of the loop's own errors leaves no node: the tokens the expression consumed are the
/// parent's, and the tree balances. A missing operand of either side, and an `Adjacent` row, in
/// both modes and under `folding`; a refused chain and a full stack, which a checked driver and
/// `folding` raise.
#[test]
fn an_error_of_the_loop_leaves_no_node() {
  let cases = [
    (
      "1 + ;",
      Fail::EoRhs {
        at: 4,
        terminal: false,
      },
      "Root[Lit[1], ws, +, ws, ;]",
    ),
    (
      "1+-;",
      Fail::EoLhs {
        at: 3,
        terminal: false,
      },
      "Root[Lit[1], +, -, ;]",
    ),
    (
      "1@",
      Fail::EoRhs {
        at: 1,
        terminal: false,
      },
      "Root[Lit[1], @]",
    ),
  ];
  for (src, error, expected) in cases {
    let (result, tree) = both(src);
    assert_eq!(result, Err(error), "{src}");
    assert_eq!(tree, expected, "{src}");
  }

  for folding in [false, true] {
    let (result, green, _) = run("1<2<3", |input| {
      if folding {
        folded(input, PLAIN, 0)
      } else {
        checked(input, PLAIN, 0)
      }
    });
    assert_eq!(result, Ok(Err(Fail::Chain { at: 3 })), "folding: {folding}");
    assert_eq!(
      rendered("1<2<3", green),
      "Root[Lit[1], <, Lit[2], <, 3]",
      "folding: {folding}"
    );

    let (result, green, _) = run("1^2^3", |input| {
      let mut driver = flat(TABLE, PLAIN).max_depth(1).with_cst_kind(K_EXPR);
      let result = if folding {
        driver
          .folding(read_prefix, read_infix, read_postfix)
          .parse_input(input)
          .map(|_| ())
      } else {
        driver.parse_into(input, &mut ())
      };
      drain(input)?;
      Ok(result)
    });
    assert_eq!(
      result,
      Ok(Err(Fail::Full { at: 4, capacity: 1 })),
      "folding: {folding}"
    );
    assert_eq!(
      rendered("1^2^3", green),
      "Root[Lit[1], ^, Lit[2], ^, 3]",
      "folding: {folding}"
    );
  }
}

/// `1+2` folds to `(1+2)`, and the fold of `*` fails.
fn fails_at_star<'inp, Ctx>(
  input: &mut InputRef<'inp, '_, Surface<'inp>, Ctx>,
  left: String,
  right: String,
  operator: InfixOp,
) -> Result<String, Fail>
where
  Ctx: ParseContext<'inp, Surface<'inp>>,
  Ctx::Emitter: Emitter<'inp, Surface<'inp>, Error = Fail>,
{
  let (PrattInfix::Left(tok) | PrattInfix::Right(tok) | PrattInfix::Neither(tok)) =
    operator.token_ref();
  if tok.data().0 == b'*' {
    return Err(Fail::Own("the fold of `*` fails"));
  }
  fold_infix(input, left, right, operator)
}

/// An error out of a fold leaves no node either: what the expression consumed, the trivia before
/// the `;` among it, is the parent's.
#[test]
fn an_error_of_a_fold_leaves_no_node() {
  let src = "1 + 2 * 3 ;";
  let (result, green, _) = run(src, |input| {
    let result = flat(TABLE, PLAIN)
      .with_cst_kind(K_EXPR)
      .folding(read_prefix, fails_at_star, read_postfix)
      .parse_input(input);
    drain(input)?;
    Ok(result)
  });
  assert_eq!(result, Ok(Err(Fail::Own("the fold of `*` fails"))));
  assert_eq!(
    rendered(src, green),
    "Root[Lit[1], ws, +, ws, Lit[2], ws, *, ws, Lit[3], ws, ;]"
  );
}

/// An error out of the operand parser leaves no node either.
#[test]
fn an_error_of_the_operand_parser_leaves_no_node() {
  let failing = Lit {
    recovering: false,
    failing: true,
  };
  let (result, green, _) = run("1+#", |input| checked(input, failing, 0));
  assert_eq!(result, Ok(Err(Fail::Own("the operand parser"))));
  assert_eq!(rendered("1+#", green), "Root[Lit[1], +, #]");
}

/// Fails at the item numbered `at`.
struct FailsAt {
  told: usize,
  at: usize,
}

impl FailsAt {
  fn tell(&mut self) -> Result<(), Fail> {
    self.told += 1;
    if self.told == self.at {
      return Err(Fail::Own("the handler"));
    }
    Ok(())
  }
}

impl<'inp, Ctx> PrattHandler<'inp, u8, Surface<'inp>, String, Ctx> for FailsAt
where
  Ctx: ParseContext<'inp, Surface<'inp>>,
  Ctx::Emitter: Emitter<'inp, Surface<'inp>, Error = Fail>,
{
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Surface<'inp>, Ctx>,
    _operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), Fail> {
    self.tell()
  }

  fn on_operand(
    &mut self,
    _input: &mut InputRef<'inp, '_, Surface<'inp>, Ctx>,
    _operand: String,
  ) -> Result<(), Fail> {
    self.tell()
  }

  fn on_infix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Surface<'inp>, Ctx>,
    _operator: Precedenced<
      PrattInfix<Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>>,
      u8,
    >,
  ) -> Result<(), Fail> {
    self.tell()
  }

  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Surface<'inp>, Ctx>,
    _operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), Fail> {
    self.tell()
  }
}

/// An error out of the handler leaves no node either.
#[test]
fn an_error_of_the_handler_leaves_no_node() {
  let (result, green, _) = run("1+2;", |input| {
    let result = flat(TABLE, PLAIN)
      .with_cst_kind(K_EXPR)
      .parse_into(input, &mut FailsAt { told: 0, at: 3 });
    drain(input)?;
    Ok(result)
  });
  assert_eq!(result, Ok(Err(Fail::Own("the handler"))));
  assert_eq!(rendered("1+2;", green), "Root[Lit[1], +, Lit[2], ;]");
}

// ── Depth ─────────────────────────────────────────────────────────────────────────────────────

/// A chain of 4096 operators, four times the tree's depth ceiling, materializes through `finish`,
/// as deep as one operator's. The same chain under the typed driver's `with_cst_kinds` nests a
/// node for each fold, and is refused as too deep.
#[test]
fn a_long_chain_is_as_deep_as_one_operator() {
  let chain = |n: usize| {
    let mut src = String::from("1");
    for _ in 0..n {
      src.push_str("+1");
    }
    src
  };

  let one = chain(1);
  let (result, green, _) = run(&one, |input| checked(input, PLAIN, 0));
  assert_eq!(result, Ok(Ok(())));
  let shallow = depth(&SyntaxNode::<Raw>::new_root(
    green.expect("one operator balances"),
  ));

  let long = chain(4096);
  let (result, green, _) = run(&long, |input| checked(input, PLAIN, 0));
  assert_eq!(result, Ok(Ok(())));
  let root = SyntaxNode::<Raw>::new_root(green.expect("a flat chain materializes"));
  assert_eq!(root.text().to_string(), long);
  assert_eq!(depth(&root), shallow);

  fn bin(_: PrattFoldOp<'_, Tk, Tk, Tk, Tk, Tk>) -> Option<u16> {
    Some(K_BIN)
  }
  type Tk = Spanned<Tok, SimpleSpan>;
  let (result, green, _) = run(&long, |input| {
    pratt(
      TABLE.lhs(PLAIN),
      TABLE.rhs(),
      fold_prefix,
      fold_infix,
      fold_postfix,
    )
    .with_cst_kinds(bin)
    .parse_input(input)
  });
  assert!(
    result.is_ok(),
    "the typed driver folds a left chain in one frame"
  );
  assert!(
    matches!(green, Err(FinishError::TooDeep { .. })),
    "{green:?}"
  );
}

// ── Round trip ────────────────────────────────────────────────────────────────────────────────

/// A small deterministic generator.
struct Rng(u64);

impl Rng {
  fn below(&mut self, n: usize) -> usize {
    self.0 ^= self.0 << 13;
    self.0 ^= self.0 >> 7;
    self.0 ^= self.0 << 17;
    (self.0 % n as u64) as usize
  }
}

/// For each generated source, the tree's text is the source, byte for byte, in both modes.
#[test]
fn every_tree_round_trips() {
  const ALPHABET: &[u8] = b"0123456789-~+&*^=<>!?%@; ";
  let mut rng = Rng(0x2545_f491_4f6c_dd1d);
  for _ in 0..500 {
    let len = 1 + rng.below(16);
    let src: String = (0..len)
      .map(|_| char::from(ALPHABET[rng.below(ALPHABET.len())]))
      .collect();
    let (result, green, _) = run(&src, |input| checked(input, PLAIN, 0));
    assert!(result.is_ok(), "{src:?}: the drain succeeds");
    let _ = rendered(&src, green);
    let (result, green, _) = run(&src, |input| bare(input, PLAIN));
    assert!(result.is_ok(), "{src:?}: the drain succeeds");
    let _ = rendered(&src, green);
  }
}

// ── Trivia, at the four ways an expression ends ───────────────────────────────────────────────

/// At a token that is no operator, the trailing trivia is inside the node and the token after it.
#[test]
fn trailing_trivia_before_a_token_that_is_no_operator_is_inside() {
  let (result, tree) = both("1 + 2  ;");
  assert_eq!(result, Ok(()));
  assert_eq!(tree, "Root[Expr[Lit[1], ws, +, ws, Lit[2], ws, ws], ;]");
}

/// At an operator below the floor, the trivia before it is inside the node. A bare driver has no
/// floor, so this ending is a checked driver's alone.
#[test]
fn trailing_trivia_before_an_operator_below_the_floor_is_inside() {
  let src = "1 * 2 + 3";
  for folding in [false, true] {
    let (result, green, _) = run(src, |input| {
      if folding {
        folded(input, PLAIN, 11)
      } else {
        checked(input, PLAIN, 11)
      }
    });
    assert_eq!(result, Ok(Ok(())), "folding: {folding}");
    assert_eq!(
      rendered(src, green),
      "Root[Expr[Lit[1], ws, *, ws, Lit[2], ws], +, ws, 3]",
      "folding: {folding}"
    );
  }
}

/// Where a fold read on, what it read is inside the node, and so is the trivia after it: under
/// `folding`, `1 i 2 e 3 ;` holds the `e 3` the `i`'s fold read, and the space before the `;`.
#[test]
fn trailing_trivia_after_a_fold_that_read_on_is_inside() {
  let src = "1 i 2 e 3 ;";
  let (result, green, _) = run(src, |input| folded(input, PLAIN, 0));
  assert_eq!(result, Ok(Ok(())));
  assert_eq!(
    rendered(src, green),
    "Root[Expr[Lit[1], ws, i, ws, Lit[2], ws, e, ws, 3, ws], ;]"
  );
}

/// At the end of input, the trailing trivia is inside the node.
#[test]
fn trailing_trivia_at_the_end_of_input_is_inside() {
  let (result, tree) = both("1 + 2  ");
  assert_eq!(result, Ok(()));
  assert_eq!(tree, "Root[Expr[Lit[1], ws, +, ws, Lit[2], ws, ws]]");
}

/// Leading trivia is outside the node: it opens at the first token.
#[test]
fn leading_trivia_is_outside() {
  let (result, tree) = both("  1 + 2 ;");
  assert_eq!(result, Ok(()));
  assert_eq!(tree, "Root[ws, ws, Expr[Lit[1], ws, +, ws, Lit[2], ws], ;]");
}

/// A scanner stop met while the seam crosses leading trivia is the parse's terminal error, and no
/// node is opened over it: ` 1` within a budget of one item, behind an operand parser that would
/// take a miss for an error of its own.
#[test]
fn a_stop_met_crossing_leading_trivia_is_terminal() {
  let result = run_budgeted(" 1", 1, |input| {
    flat(TABLE, Atom)
      .with_cst_kind(K_EXPR)
      .parse_into(input, &mut ())
  });
  assert_eq!(
    result,
    Err(Fail::Eot {
      at: 1,
      terminal: true
    })
  );
}

/// At the end of the input the seam commits nothing: over `$`, a lone lexer error, the missing
/// operand is at 0 with a kind as without one.
#[test]
fn the_seam_moves_over_no_lexer_error() {
  let missing = Err(Fail::EoLhs {
    at: 0,
    terminal: false,
  });
  let (with_kind, _, _) = run("$", |input| {
    flat(TABLE, PLAIN)
      .with_cst_kind(K_EXPR)
      .parse_into(input, &mut ())
  });
  let (without, _, _) = run("$", |input| flat(TABLE, PLAIN).parse_into(input, &mut ()));
  assert_eq!(with_kind, missing, "with a kind");
  assert_eq!(without, missing, "without one");
}

// ── Recovery, and a rollback ──────────────────────────────────────────────────────────────────

/// Under a recovering operand parser, `1++2` is one node with an empty error node for the missing
/// operand, and one diagnostic.
#[test]
fn a_recovering_operand_parser_leaves_a_hole() {
  let recovering = Lit {
    recovering: true,
    failing: false,
  };
  let (result, green, diagnostics) = run("1++2", |input| checked(input, recovering, 0));
  assert_eq!(result, Ok(Ok(())));
  assert_eq!(
    rendered("1++2", green),
    "Root[Expr[Lit[1], +, Err[], +, Lit[2]]]"
  );
  assert_eq!(diagnostics, 1);
}

/// An enclosing attempt that fails after the expression takes the expression's events with it:
/// no token of it is in its node, nor is the node.
#[test]
fn a_rollback_takes_the_events() {
  let src = "1+2;";
  let (result, green, _) = run(src, |input| {
    let attempt = input.try_attempt(|input| {
      flat(TABLE, PLAIN)
        .with_cst_kind(K_EXPR)
        .parse_into(input, &mut ())?;
      Err::<(), Fail>(Fail::Own("after the expression"))
    });
    drain(input)?;
    Ok(attempt)
  });
  assert_eq!(result, Ok(Err(Fail::Own("after the expression"))));
  assert_eq!(rendered(src, green), "Root[1, +, 2, ;]");
}
