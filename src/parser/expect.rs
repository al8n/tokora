use crate::{
  Check, TryParseInput,
  error::{UnexpectedEot, token::UnexpectedToken},
  try_parse_input::ParseAttempt,
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
#[repr(transparent)]
pub struct Expect<Classifier, Ctx, Lang: ?Sized = ()> {
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  is: Classifier,
}

impl<Classifier, Ctx, Lang: ?Sized> Expect<Classifier, Ctx, Lang> {
  /// Creates a parser that accepts a specific token of a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn of(classifier: Classifier) -> Self {
    Self {
      is: classifier,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn from_ref(classifier: &Classifier) -> &Self {
    // SAFETY: Self is #[repr(transparent)]
    unsafe { &*(classifier as *const Classifier as *const Self) }
  }
}

impl<'inp, L, Ctx, Classifier, Lang: ?Sized> ParseInput<'inp, L, L::Token, Ctx, Lang>
  for Expect<Classifier, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Token, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Expect::from_ref(&self.is)
      .parse_input(inp)
      .map(|spanned| spanned.into_data())
  }
}

impl<'inp, L, Ctx, Classifier, Lang: ?Sized>
  ParseInput<'inp, L, Spanned<L::Token, L::Span>, Ctx, Lang> for &Expect<Classifier, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    inp.expect(|tok, _| match self.is.check(tok.data()) {
      Ok(_) => Ok(()),
      Err(exp) => Err(UnexpectedToken::with_expected_of(tok.span().clone(), exp).into()),
    })
  }
}

impl<'inp, L, Ctx, Classifier, Lang: ?Sized>
  ParseInput<'inp, L, Spanned<L::Token, L::Span>, Ctx, Lang>
  for With<Expect<Classifier, Ctx, Lang>, PhantomSpan>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Expect::from_ref(&self.primary.is).parse_input(inp)
  }
}

impl<'inp, L, Ctx, Classifier, Lang: ?Sized>
  ParseInput<'inp, L, Sliced<L::Token, <L::Source as Source<L::Offset>>::Slice<'inp>>, Ctx, Lang>
  for With<Expect<Classifier, Ctx, Lang>, PhantomSliced>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
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
    self
      .primary
      .parse_input(inp)
      .map(|tok| Sliced::new(inp.slice(), tok))
  }
}

impl<'inp, L, Ctx, Classifier, Lang: ?Sized>
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
    Expect::from_ref(&self.primary.is)
      .parse_input(inp)
      .map(|spanned| {
        let (span, tok) = spanned.into_components();
        Located::new(inp.slice(), span, tok)
      })
  }
}

impl<'inp, L, Ctx, Classifier, Lang: ?Sized>
  TryParseInput<'inp, L, Spanned<L::Token, L::Span>, Ctx, Lang> for Expect<Classifier, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Classifier: Check<L::Token>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    ParseAttempt<Spanned<L::Token, L::Span>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    inp
      .try_expect(|tok| self.is.check(tok.data()))
      .map(|opt_tok| match opt_tok {
        Some(tok) => ParseAttempt::Accept(tok),
        None => ParseAttempt::Decline,
      })
  }
}

/// Creates a parser that expects a token matching a specific criterion.
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn try_expect<'inp, Classifier, L, Ctx>(classifier: Classifier) -> Expect<Classifier, Ctx>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L>,
  Classifier: Check<L::Token>,
{
  try_expect_of(classifier)
}

/// Creates a parser that expects a token matching a specific criterion for a specific language.
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn try_expect_of<'inp, Classifier, L, Ctx, Lang>(
  classifier: Classifier,
) -> Expect<Classifier, Ctx, Lang>
where
  Lang: ?Sized,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Classifier: Check<L::Token>,
{
  Expect::of(classifier)
}

/// Creates a parser that expects a token matching a specific criterion.
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn expect<'inp, Classifier, L, Ctx>(classifier: Classifier) -> Expect<Classifier, Ctx>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L>,
  <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>
    + From<UnexpectedEot<L::Offset>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  expect_of(classifier)
}

/// Creates a parser that expects a token matching a specific criterion for a specific language.
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn expect_of<'inp, Classifier, L, Ctx, Lang>(
  classifier: Classifier,
) -> Expect<Classifier, Ctx, Lang>
where
  Lang: ?Sized,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: Emitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  Expect::of(classifier)
}
