use crate::{
  InputRef, ParseInput, Token,
  cache::DefaultCache,
  emitter::Verbose,
  error::token::UnexpectedToken,
  input::Input,
  lexer::LogosLexer,
  span::{SimpleSpan, Spanned},
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
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
    f.write_str("number")
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
type Cx<'a> = (Verbose<Err>, DefaultCache<'a, Lex<'a>>);

/// Emits one diagnostic at `span`, so the caller can read back the labels that were open.
fn mark<'inp>(inp: &mut InputRef<'inp, '_, Lex<'inp>, Cx<'inp>>, at: SimpleSpan) {
  let _ = <Verbose<Err> as crate::Emitter<'inp, Lex<'inp>>>::emit_error(
    inp.emitter(),
    Spanned::new(at, Err::Any),
  );
}

fn boom<'inp>(_inp: &mut InputRef<'inp, '_, Lex<'inp>, Cx<'inp>>) -> Result<(), Err> {
  panic!("D26: the inner parser panics inside the labelled scope")
}

#[test]
fn labelled_pops_its_label_when_the_inner_parser_panics() {
  let mut input = Input::<Lex<'_>, Cx<'_>>::with_state_and_context(
    "12 34",
    (),
    crate::input::InputContext::new(
      Verbose::<Err>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    let mut p = crate::labelled("audit-ctx", boom);
    let _ = p.parse_input(&mut inp);
  }));
  assert!(caught.is_err(), "the panic was caught by this host");

  // The host emits ONE diagnostic after the catch, outside every label scope.
  let after = SimpleSpan::new(0usize, 1usize);
  mark(&mut inp, after);

  assert_eq!(
    inp.emitter().labels()[&after],
    std::vec![std::vec::Vec::<&'static str>::new()],
    "a diagnostic emitted after the caught panic carries no label — `labelled` popped its \
     own scope on the unwind edge"
  );
}

/// Runs a panicking `labelled("inner", ..)` under a catch, then — still inside the caller's
/// own `labelled("outer", ..)` scope — emits one diagnostic so the open-label set is readable.
fn catch_inner_then_mark<'inp>(
  inp: &mut InputRef<'inp, '_, Lex<'inp>, Cx<'inp>>,
) -> Result<(), Err> {
  let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    let mut inner = crate::labelled("inner", boom);
    let _ = inner.parse_input(inp);
  }));
  assert!(caught.is_err(), "the inner panic was caught inside `outer`");
  mark(inp, SimpleSpan::new(0usize, 1usize));
  Ok(())
}

#[test]
fn nested_labels_unwind_pops_exactly_the_unwound_scopes() {
  let mut input = Input::<Lex<'_>, Cx<'_>>::with_state_and_context(
    "12 34",
    (),
    crate::input::InputContext::new(
      Verbose::<Err>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let inside = SimpleSpan::new(0usize, 1usize);
  let mut outer = crate::labelled("outer", catch_inner_then_mark);
  outer.parse_input(&mut inp).expect("outer completes");

  assert_eq!(
    inp.emitter().labels()[&inside],
    std::vec![std::vec!["outer"]],
    "the unwind popped `inner` and only `inner` — the guard pops per scope, not clear-all"
  );
}
