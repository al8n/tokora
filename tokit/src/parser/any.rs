use crate::{error::UnexpectedEot, located::Located, span::Span, utils::marker::PhantomLocated};

use super::*;

/// A parser that accepts any single token from the input stream.
///
/// This is the most fundamental parser - it consumes one token regardless of its type.
/// It succeeds if a token is available, and fails only on end-of-input or lexer errors.
///
/// `Any` comes in several variants that determine what information is captured:
/// - **Basic**: Returns just the token value
/// - **Spanned**: Returns token with its [`Span`] (position information)
/// - **Sliced**: Returns token with its source text slice
/// - **Located**: Returns token with both span and slice
///
/// # Type Parameters
///
/// - `L`: Lexer type
/// - `Ctx`: Parse context
/// - `Lang`: Language marker type (default `()`)
///
/// # Examples
///
/// ## Basic Token Consumption
///
/// ```ignore
/// use tokit::parser::{any, ParseInput};
///
/// // Accept any token
/// let parser = any::<MyLexer>();
///
/// // Input: Number(42)      → Ok(Number(42))
/// // Input: Identifier("x") → Ok(Identifier("x"))
/// // Input: (end of input)  → Err(UnexpectedEot)
/// ```
///
/// ## With Span Information
///
/// ```ignore
/// // Capture token with its position
/// let parser = any::<MyLexer>().spanned();
///
/// // Returns: Spanned { data: Token, span: Span { start, end } }
/// ```
///
/// ## With Source Text
///
/// ```ignore
/// // Capture token with its source text
/// let parser = any::<MyLexer>().sliced();
///
/// // Input: "foo" → Ok(Sliced { data: Identifier("foo"), slice: "foo" })
/// ```
///
/// ## With Full Location Info
///
/// ```ignore
/// // Capture token, span, and source
/// let parser = any::<MyLexer>().located();
///
/// // Returns: Located { data: Token, span: Span, slice: &str }
/// ```
///
/// ## Filtering Specific Tokens
///
/// ```ignore
/// // Accept any token, then filter for numbers
/// let parser = any::<MyLexer>()
///     .filter(|tok| {
///         if matches!(tok, Token::Number(_)) {
///             Ok(())
///         } else {
///             Err(ExpectedNumberError::new())
///         }
///     });
///
/// // More efficient alternative: use `expect` instead
/// let parser = expect(|tok| matches!(tok, Token::Number(_)));
/// ```
///
/// # Error Handling
///
/// `Any` can fail with:
/// - `UnexpectedEot`: No more tokens available (end of input)
/// - Lexer errors: The lexer produced an error token
///
/// # When to Use
///
/// - **Building blocks**: As the foundation for more complex parsers
/// - **Generic parsing**: When you need to consume any token
/// - **With filtering**: Combined with `.filter()` or `.filter_map()`
/// - **Development**: Quick prototyping before adding specific token checks
///
/// **Prefer `expect`** when you know which token you want - it provides better error messages.
///
/// # Performance
///
/// - **Memory**: Zero-sized type (no runtime overhead)
/// - **Runtime**: O(1) - single token consumption
/// - **Variants**: `.spanned()`, `.sliced()`, `.located()` have minimal overhead
///
/// # See Also
///
/// - [`Expect`] - Parse a specific token (better error messages)
/// - [`Filter`] - Validate after parsing
/// - [`Spanned`], [`Sliced`], [`Located`] - Output wrapper types
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Any<L, Ctx, Lang: ?Sized = ()> {
  _lxr: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<L, Ctx> Any<L, Ctx> {
  /// Creates a parser that accepts any token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self::of()
  }

  /// Creates a parser that yields any token with its span
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn spanned() -> With<Self, PhantomSpan> {
    Self::spanned_of()
  }

  /// Creates a parser that yields any token with its source
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn sliced() -> With<Self, PhantomSliced> {
    Self::sliced_of()
  }

  /// Creates a parser that yields any token without its source and span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn located() -> With<Self, PhantomLocated> {
    Self::located_of()
  }
}

impl<L, Ctx, Lang> Any<L, Ctx, Lang> {
  /// Creates a parser that accepts any token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of() -> Self {
    Any {
      _lxr: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a parser that yields any token with its span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn spanned_of() -> With<Self, PhantomSpan> {
    With::new(Self::of(), PhantomSpan::PHANTOM)
  }

  /// Creates a parser that yields any token with its source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn sliced_of() -> With<Self, PhantomSliced> {
    With::new(Self::of(), PhantomSliced::PHANTOM)
  }

  /// Creates a parser that yields any token without its source and span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn located_of() -> With<Self, PhantomLocated> {
    With::new(Self::of(), PhantomLocated::PHANTOM)
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> ParseInput<'inp, L, L::Token, Ctx, Lang> for Any<L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<<L::Token as Token<'inp>>::Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Token, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Ctx: ParseContext<'inp, L, Lang>,
  {
    match inp.next()? {
      Some(Spanned { data: tok, .. }) => Ok(tok),
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> ParseInput<'inp, L, Spanned<L::Token, L::Span>, Ctx, Lang>
  for With<Any<L, Ctx, Lang>, PhantomSpan>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<<L::Token as Token<'inp>>::Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Ctx: ParseContext<'inp, L, Lang>,
  {
    match inp.next()? {
      Some(Spanned { data: tok, span }) => Ok(Spanned::new(span, tok)),
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Sliced<L::Token, <L::Source as Source<L::Offset>>::Slice<'inp>>, Ctx, Lang>
  for With<Any<L, Ctx, Lang>, PhantomSliced>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<<L::Token as Token<'inp>>::Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Sliced<L::Token, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    Ctx: ParseContext<'inp, L, Lang>,
  {
    match inp.next()? {
      Some(Spanned { data: tok, .. }) => Ok(Sliced::new(inp.slice(), tok)),
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang: ?Sized>
  ParseInput<
    'inp,
    L,
    Located<L::Token, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    Ctx,
    Lang,
  > for With<Any<L, Ctx, Lang>, PhantomLocated>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<<L::Token as Token<'inp>>::Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Located<L::Token, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    Ctx: ParseContext<'inp, L, Lang>,
  {
    match inp.next()? {
      Some(Spanned { data: tok, span }) => Ok(Located::new(inp.slice(), span, tok)),
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_any_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::spanned().map(Spanned::into_data))
  }

  fn assert_any_parse_with_context_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::with_context(()).apply(Any::new().spanned().map(Spanned::into_data))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_any_parse_impl();
    let _ = assert_any_parse_with_context_impl();
  }
}

#[cfg(all(test, feature = "std", feature = "logos"))]
mod slice_tests {
  use super::*;

  use crate::{
    ParseInput, ParserContext,
    error::token::{UnexpectedToken, UnexpectedTokenOf},
    input::Cursor,
    lexer::LogosLexer,
    logos::{self, Logos},
    span::Spanned,
    token::Token as TokenTrait,
  };

  #[derive(Debug, Clone, Logos, PartialEq)]
  #[logos(crate = logos, skip r"[ \t\r\n]+")]
  enum Token {
    #[regex(r"[0-9]+")]
    Num,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum TokenKind {
    Num,
  }

  impl core::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TokenKind::Num => write!(f, "number"),
      }
    }
  }

  impl TokenTrait<'_> for Token {
    type Kind = TokenKind;
    type Error = ();

    fn kind(&self) -> TokenKind {
      match self {
        Token::Num => TokenKind::Num,
      }
    }

    fn is_trivia(&self) -> bool {
      false
    }
  }

  type TestLexer<'a> = LogosLexer<'a, Token>;

  #[derive(Debug, PartialEq)]
  enum E {
    Lex,
    Eot,
  }

  impl From<()> for E {
    fn from(_: ()) -> Self {
      E::Lex
    }
  }

  impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for E {
    fn from(_: UnexpectedEot<O, Lang>) -> Self {
      E::Eot
    }
  }

  impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
    fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
      E::Lex
    }
  }

  struct TestEm;

  impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEm {
    type Error = E;

    fn emit_lexer_error(
      &mut self,
      _: Spanned<
        <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
        <TestLexer<'inp> as Lexer<'inp>>::Span,
      >,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(E::Lex)
    }

    fn emit_unexpected_token(
      &mut self,
      _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(E::Lex)
    }

    fn emit_error(
      &mut self,
      err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(err.into_data())
    }

    fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>)
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
    }
  }

  fn ctx() -> ParserContext<'static, TestLexer<'static>, TestEm> {
    ParserContext::new(TestEm)
  }

  // `Any::sliced()` captures the current token's text via `slice()`, so the
  // second token in a row must slice to its own text, not the whole prefix.
  #[test]
  fn any_sliced_slices_each_current_token() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<(&'inp str, &'inp str), E> {
      let first = Any::<TestLexer<'inp>, _>::sliced().parse_input(inp)?;
      let second = Any::<TestLexer<'inp>, _>::sliced().parse_input(inp)?;
      Ok((first.slice(), second.slice()))
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("12 34");
    assert_eq!(r.unwrap(), ("12", "34"));
  }
}
