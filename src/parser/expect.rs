use crate::{
  Check,
  error::{UnexpectedEot, token::UnexpectedToken},
  span::Span,
};

use super::*;

/// A parser that expects a token matching a specific criterion.
///
/// This parser consumes one token and checks if it matches the provided classifier.
/// If the token matches, parsing succeeds; otherwise, an `UnexpectedToken` error is
/// emitted with information about what was expected and what was found.
///
/// Unlike [`Any`] which accepts any token, `Expect` provides **better error messages**
/// by specifying what token was expected when a mismatch occurs.
///
/// # Type Parameters
///
/// - `Classifier`: A function or closure that checks if a token matches (implements [`Check`])
/// - `Ctx`: Parse context
/// - `Lang`: Language marker type (default `()`)
///
/// # Examples
///
/// ## Expect Specific Token Type
///
/// ```ignore
/// use tokit::parser::{expect, ParseInput};
///
/// // Expect a semicolon token
/// let parser = expect(|tok| {
///     if matches!(tok, Token::Semicolon) {
///         Ok(())
///     } else {
///         Err(Expected::token(TokenKind::Semicolon))
///     }
/// });
///
/// // Input: Token::Semicolon   → Ok(Token::Semicolon)
/// // Input: Token::Comma       → Err(UnexpectedToken { expected: Semicolon, found: Comma })
/// ```
///
/// ## Expect Token with Value Check
///
/// ```ignore
/// // Expect identifier starting with uppercase
/// let parser = expect(|tok| {
///     match tok {
///         Token::Identifier(name) if name.starts_with(char::is_uppercase) => Ok(()),
///         _ => Err(Expected::description("uppercase identifier")),
///     }
/// });
/// ```
///
/// ## Multiple Valid Tokens
///
/// ```ignore
/// // Accept either semicolon or newline
/// let parser = expect(|tok| {
///     match tok {
///         Token::Semicolon | Token::Newline => Ok(()),
///         _ => Err(Expected::one_of(&[TokenKind::Semicolon, TokenKind::Newline])),
///     }
/// });
/// ```
///
/// ## With Span Information
///
/// ```ignore
/// // Capture token and its position
/// let parser = expect(classifier).spanned();
///
/// // Returns: Spanned { data: Token, span: Span }
/// ```
///
/// ## With Source Text
///
/// ```ignore
/// // Capture token and source slice
/// let parser = expect(|tok| matches!(tok, Token::Number(_)))
///     .sliced()
///     .map(|Sliced { data, slice }| {
///         // Use slice to parse the exact number format
///         parse_number(slice)
///     });
/// ```
///
/// # Error Handling
///
/// `Expect` provides detailed error information:
/// - **What was expected**: Based on the `Expected` value from the classifier
/// - **What was found**: The actual token that didn't match
/// - **Position**: Span information for the unexpected token
///
/// Errors:
/// - `UnexpectedToken`: Token didn't match the classifier
/// - `UnexpectedEot`: No more tokens available (end of input)
/// - Lexer errors: The lexer produced an error token
///
/// # Classifier Pattern
///
/// The classifier should return `Result<(), Expected>`:
/// - `Ok(())`: Token matches, parsing succeeds
/// - `Err(Expected::...)`: Token doesn't match, error describes what was expected
///
/// ```ignore
/// // Good error messages
/// |tok| match tok {
///     Token::If => Ok(()),
///     _ => Err(Expected::keyword("if")),
/// }
///
/// // Generic error (less helpful)
/// |tok| match tok {
///     Token::If => Ok(()),
///     _ => Err(Expected::description("keyword")),
/// }
/// ```
///
/// # Comparison with Any
///
/// | Parser | Accepts | Error Message Quality |
/// |--------|---------|----------------------|
/// | [`Any`] | Any token | Generic (just "unexpected token") |
/// | **`Expect`** | Specific tokens | Detailed (expected vs found) |
///
/// **When to use**:
/// - `Any`: Consume any token, filter later
/// - `Expect`: Know what token you want, need good error messages
///
/// # Performance
///
/// - **Memory**: Size of the classifier closure (often zero-sized)
/// - **Runtime**: O(1) - single token check
/// - **Error construction**: Only on mismatch
///
/// # See Also
///
/// - [`Any`] - Accept any token
/// - [`Filter`] - Validate after parsing (less specific errors)
/// - [`Check`] - The trait for token classifiers
/// - [`Expected`] - Type for describing expected tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Expect<Classifier, Ctx, Lang: ?Sized = ()> {
  is: Classifier,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<Classifier, Ctx> Expect<Classifier, Ctx> {
  /// Creates a parser that accepts a specific token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(classifier: Classifier) -> Self {
    Self::of(classifier)
  }

  /// Creates a parser that yields specific token with its span
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn spanned(classifier: Classifier) -> With<Self, PhantomSpan> {
    Self::spanned_of(classifier)
  }

  /// Creates a parser that yields specific token with its source
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn sliced(classifier: Classifier) -> With<Self, PhantomSliced> {
    Self::sliced_of(classifier)
  }

  /// Creates a parser that yields specific token without its source and span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn located(classifier: Classifier) -> With<Self, PhantomLocated> {
    Self::located_of(classifier)
  }
}

impl<Classifier, Ctx, Lang> Expect<Classifier, Ctx, Lang> {
  /// Creates a parser that accepts a specific token of a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(classifier: Classifier) -> Self {
    Self {
      is: classifier,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a parser that yields specific token with its span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn spanned_of(classifier: Classifier) -> With<Self, PhantomSpan> {
    With::new(Self::of(classifier), PhantomSpan::PHANTOM)
  }

  /// Creates a parser that yields specific token with its source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn sliced_of(classifier: Classifier) -> With<Self, PhantomSliced> {
    With::new(Self::of(classifier), PhantomSliced::PHANTOM)
  }

  /// Creates a parser that yields specific token without its source and span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn located_of(classifier: Classifier) -> With<Self, PhantomLocated> {
    With::new(Self::of(classifier), PhantomLocated::PHANTOM)
  }
}

impl<'inp, L, Ctx, Lang, Classifier> ParseInput<'inp, L, L::Token, Ctx, Lang>
  for Expect<Classifier, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Token, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.is.check(&tok) {
          Ok(()) => Ok(tok),
          Err(expected) => Err(
            UnexpectedToken::with_expected_of(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(From::from(err)),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang, Classifier> ParseInput<'inp, L, Spanned<L::Token, L::Span>, Ctx, Lang>
  for With<Expect<Classifier, Ctx, Lang>, PhantomSpan>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.primary.is.check(&tok) {
          Ok(()) => Ok(Spanned::new(span, tok)),
          Err(expected) => Err(
            UnexpectedToken::with_expected_of(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(From::from(err)),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang, Classifier>
  ParseInput<'inp, L, Sliced<L::Token, <L::Source as Source<L::Offset>>::Slice<'inp>>, Ctx, Lang>
  for With<Expect<Classifier, Ctx, Lang>, PhantomSliced>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Sliced<L::Token, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.primary.is.check(&tok) {
          Ok(()) => Ok(Sliced::new(inp.slice(), tok)),
          Err(expected) => Err(
            UnexpectedToken::with_expected_of(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(From::from(err)),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang, Classifier>
  ParseInput<
    'inp,
    L,
    Located<L::Token, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    Ctx,
    Lang,
  > for With<Expect<Classifier, Ctx, Lang>, PhantomLocated>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Located<L::Token, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.primary.is.check(&tok) {
          Ok(()) => Ok(Located::new(inp.slice(), span, tok)),
          Err(expected) => Err(
            UnexpectedToken::with_expected_of(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(From::from(err)),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_expect_parse_impl_with_ctx<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::with_context(()).apply(Expect::new(|_tok: &DummyToken| Ok(())))
  }

  fn assert_expect_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Expect::new(|_tok: &DummyToken| Ok(())))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_expect_parse_impl();
    let _ = assert_expect_parse_impl_with_ctx();
  }
}
