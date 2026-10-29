use crate::{
  InputRef, ParseInput, Token, cache::DefaultCache, emitter::Silent, error::UnexpectedToken,
  input::Input, lexer::LogosLexer,
};

#[derive(Debug, Clone, PartialEq)]
enum Err {
  Any,
}
impl From<()> for Err {
  fn from(_: ()) -> Self {
    Err::Any
  }
}
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for Err {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    Err::Any
  }
}

#[derive(Debug, Clone, PartialEq, crate::logos::Logos)]
#[logos(crate = crate::logos, skip r"[ \t\r\n]+")]
enum Tok {
  #[regex(r"[0-9]+")]
  Num,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
  Num,
}
impl core::fmt::Display for Kind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

impl Token<'_> for Tok {
  type Kind = Kind;
  type Error = Err;

  const SCAN_LOOKAHEAD: crate::ScanLookahead = crate::ScanLookahead::Unbounded;
  fn kind(&self) -> Kind {
    Kind::Num
  }
  fn is_trivia(&self) -> bool {
    false
  }
}

type Lex<'a> = LogosLexer<'a, Tok>;
type Cx<'a> = (Silent<Err>, DefaultCache<'a, Lex<'a>>);

fn eat_num<'inp>(inp: &mut InputRef<'inp, '_, Lex<'inp>, Cx<'inp>>) -> Result<bool, Err> {
  inp.try_expect(|_| true).map(|tok| tok.is_some())
}

// The capture test: a one-token parse wrapped in `traced` must print exactly enter, the
// internal `try_expect` leaf indented one level, then exit-ok with the consumed span.
#[test]
fn traced_emits_enter_leaf_exit_with_indentation() {
  let mut input = Input::<Lex<'_>, Cx<'_>>::with_state_and_context(
    "12",
    (),
    crate::input::InputContext::new(Silent::<Err>::new(), DefaultCache::<'_, Lex<'_>>::default()),
  );
  let mut inp = input.as_ref();

  let mut parser = crate::combinator::traced("num", eat_num);
  let (res, lines) = crate::trace::capture(|| parser.parse_input(&mut inp));

  assert_eq!(res, Ok(true));
  assert_eq!(lines.len(), 3, "unexpected event sequence: {lines:#?}");
  // enter at depth 0
  assert!(lines[0].starts_with("> num"), "enter line: {:?}", lines[0]);
  // the internal try_expect hook fires one level deeper (two-space indent)
  assert!(
    lines[1].starts_with("  \u{b7} try_expect"),
    "leaf line: {:?}",
    lines[1]
  );
  // exit-ok back at depth 0, carrying the consumed span
  assert!(lines[2].starts_with("< num"), "exit line: {:?}", lines[2]);
  assert!(
    lines[2].contains("ok"),
    "exit carries ok+span: {:?}",
    lines[2]
  );
  assert!(
    lines[2].contains("0..2"),
    "consumed span 0..2: {:?}",
    lines[2]
  );
}

fn boom<'inp>(_inp: &mut InputRef<'inp, '_, Lex<'inp>, Cx<'inp>>) -> Result<bool, Err> {
  panic!("D45/traced: the inner parser panics inside the traced scope")
}

// The label-scope class, member for member: `trace_enter` bumps the depth, the exit arms
// decrement it, and an unwind through the sub-parse skips them, so every later trace line
// is indented one level too deep for the emitter's whole life.
#[test]
fn traced_unwind_restores_depth() {
  let mut input = Input::<Lex<'_>, Cx<'_>>::with_state_and_context(
    "12 34",
    (),
    crate::input::InputContext::new(Silent::<Err>::new(), DefaultCache::<'_, Lex<'_>>::default()),
  );
  let mut inp = input.as_ref();

  let (_, lines) = crate::trace::capture(|| {
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      let mut p = crate::combinator::traced("boom", boom);
      let _ = p.parse_input(&mut inp);
    }));
    assert!(caught.is_err(), "the panic was caught by this host");
    let mut after = crate::combinator::traced("after", eat_num);
    let _ = after.parse_input(&mut inp);
  });

  let enter = lines
    .iter()
    .find(|l| l.contains("> after"))
    .unwrap_or_else(|| panic!("no post-unwind enter line: {lines:#?}"));
  assert!(
    enter.starts_with("> after"),
    "the unwound scope leaked its depth: {enter:?} (all lines: {lines:#?})"
  );
}
