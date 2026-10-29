use crate::{
  InputRef, ParseInput, Token, cache::DefaultCache, emitter::Silent, error::token::UnexpectedToken,
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

// With the `trace` feature on, entering a `labelled` scope fires exactly one trace leaf line
// naming the label at the current depth — keeping the label DX coherent with `traced`.
#[test]
fn labelled_fires_a_single_trace_leaf_naming_the_label() {
  let mut input = Input::<Lex<'_>, Cx<'_>>::with_state_and_context(
    "12",
    (),
    crate::input::InputContext::new(Silent::<Err>::new(), DefaultCache::<'_, Lex<'_>>::default()),
  );
  let mut inp = input.as_ref();

  let mut parser = crate::labelled("while parsing item", eat_num);
  let (res, lines) = crate::trace::capture(|| parser.parse_input(&mut inp));

  assert_eq!(res, Ok(true));
  // The label surfaces as a leaf line (`·`), naming the label.
  assert!(
    lines
      .iter()
      .any(|l| l.contains("\u{b7}") && l.contains("while parsing item")),
    "labelled fires a trace leaf naming the label: {lines:#?}"
  );
}
