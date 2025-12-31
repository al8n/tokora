use core::marker::PhantomData;

use super::*;

/// A parser that sequentially composes two parsers.
///
/// This combinator runs the first parser, then uses its output to determine
/// the second parser to run. This enables context-dependent parsing where
/// the result of one parser influences what comes next.
///
/// # Type Parameters
///
/// - `F`: The first parser
/// - `ThenFn`: A function that takes the first parser's output and returns the second parser
/// - `O`: The output type of the first parser
///
/// # Examples
///
/// ```ignore
/// // Parse a token, then parse different content based on what we got
/// let parser = Any::parser()
///     .then(|tok| {
///         match tok.kind() {
///             TokenKind::BraceOpen => parse_object(),
///             TokenKind::BracketOpen => parse_array(),
///             _ => parse_value(),
///         }
///     });
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Then<F, T, O, U, L, Ctx, Lang: ?Sized = ()> {
  parser: F,
  then: T,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _lang: PhantomData<Lang>,
  _ctx: PhantomData<Ctx>,
}

impl<F, T, O, U, L, Ctx, Lang: ?Sized> Then<F, T, O, U, L, Ctx, Lang> {
  /// Creates a new `Then` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(parser: F, then: T) -> Self {
    Self {
      parser,
      then,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
      _u: PhantomData,
    }
  }
}

impl<'inp, F, T, L, O, U, Ctx, Lang> ParseInput<'inp, L, (O, U), Ctx, Lang>
  for Then<F, T, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  T: ParseInput<'inp, L, U, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<(O, U), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let a = self.parser.parse_input(input)?;
    let b = self.then.parse_input(input)?;
    Ok((a, b))
  }
}

/// A parser that sequences two parsers, keeping only the second result.
///
/// This combinator runs the first parser, discards its result, then runs
/// the second parser and returns its result. Useful for skipping over
/// expected tokens or syntax.
///
/// # Type Parameters
///
/// - `F`: The first parser (result will be discarded)
/// - `G`: The second parser (result will be returned)
/// - `O1`: The output type of the first parser
///
/// # Examples
///
/// ```ignore
/// // Parse an opening brace, then parse the content
/// let parser = Expect::parser(is_brace_open)
///     .ignore_then(parse_object_content());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IgnoreThen<F, G, O, U, L, Ctx, Lang: ?Sized> {
  first: F,
  second: G,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, G, O, U, L, Ctx, Lang: ?Sized> IgnoreThen<F, G, O, U, L, Ctx, Lang> {
  /// Creates a new `IgnoreThen` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(first: F, second: G) -> Self {
    Self {
      first,
      second,
      _o: PhantomData,
      _u: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, F, G, L, O, U, Ctx, Lang: ?Sized> ParseInput<'inp, L, U, Ctx, Lang>
  for IgnoreThen<F, G, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  G: ParseInput<'inp, L, U, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let _ = self.first.parse_input(input)?;
    self.second.parse_input(input)
  }
}

/// A parser that sequences two parsers, keeping only the first result.
///
/// This combinator runs the first parser, then runs the second parser,
/// but only returns the first parser's result. Useful for parsing required
/// trailing tokens or syntax that you want to validate but don't need.
///
/// # Type Parameters
///
/// - `F`: The first parser (result will be returned)
/// - `G`: The second parser (result will be discarded)
/// - `O2`: The output type of the second parser
///
/// # Examples
///
/// ```ignore
/// // Parse object content, then expect a closing brace
/// let parser = parse_object_content()
///     .then_ignore(Expect::parser(is_brace_close));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThenIgnore<F, G, O, U, L, Ctx, Lang: ?Sized> {
  first: F,
  second: G,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, G, O, U, L, Ctx, Lang: ?Sized> ThenIgnore<F, G, O, U, L, Ctx, Lang> {
  /// Creates a new `ThenIgnore` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(first: F, second: G) -> Self {
    Self {
      first,
      second,
      _o: PhantomData,
      _u: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, F, G, L, O, U, Ctx, Lang: ?Sized> ParseInput<'inp, L, O, Ctx, Lang>
  for ThenIgnore<F, G, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  G: ParseInput<'inp, L, U, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let first_result = self.first.parse_input(input)?;
    self.second.parse_input(input).map(|_| first_result)
  }
}

/// A parser that sequentially composes two parsers.
///
/// This combinator runs the first parser, then uses its output to determine
/// the second parser to run. This enables context-dependent parsing where
/// the result of one parser influences what comes next.
///
/// # Type Parameters
///
/// - `F`: The first parser
/// - `AndThenFn`: A function that takes the first parser's output and returns the second parser
/// - `O`: The output type of the first parser
///
/// # Examples
///
/// ```ignore
/// // Parse a token, then parse different content based on what we got
/// let parser = Any::parser()
///     .then(|tok| {
///         match tok.kind() {
///             TokenKind::BraceOpen => parse_object(),
///             TokenKind::BracketOpen => parse_array(),
///             _ => parse_value(),
///         }
///     });
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AndThen<F, T, O, U, L, Ctx, Lang: ?Sized = ()> {
  parser: F,
  then: T,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, O, T, U, L, Ctx, Lang: ?Sized> AndThen<F, T, O, U, L, Ctx, Lang> {
  /// Creates a new `AndThen` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(parser: F, then: T) -> Self {
    Self {
      parser,
      then,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
      _u: PhantomData,
    }
  }
}

impl<'inp, F, T, L, O, U, Ctx, Lang> ParseInput<'inp, L, U, Ctx, Lang>
  for AndThen<F, T, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  T: FnMut(O) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self.parser.parse_input(input).and_then(&mut self.then)
  }
}

/// A parser that sequentially composes two parsers.
///
/// This combinator runs the first parser, then uses its output to determine
/// the second parser to run. This enables context-dependent parsing where
/// the result of one parser influences what comes next.
///
/// # Type Parameters
///
/// - `F`: The first parser
/// - `AndThenFn`: A function that takes the first parser's output and returns the second parser
/// - `O`: The output type of the first parser
///
/// # Examples
///
/// ```ignore
/// // Parse a token, then parse different content based on what we got
/// let parser = Any::parser()
///     .then(|tok| {
///         match tok.kind() {
///             TokenKind::BraceOpen => parse_object(),
///             TokenKind::BracketOpen => parse_array(),
///             _ => parse_value(),
///         }
///     });
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AndThenWith<F, T, O, U, L, Ctx, Lang: ?Sized = ()> {
  parser: F,
  then: T,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, O, T, U, L, Ctx, Lang: ?Sized> AndThenWith<F, T, O, U, L, Ctx, Lang> {
  /// Creates a new `AndThen` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(parser: F, then: T) -> Self {
    Self {
      parser,
      then,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
      _u: PhantomData,
    }
  }
}

impl<'inp, F, T, L, O, U, Ctx, Lang> ParseInput<'inp, L, U, Ctx, Lang>
  for AndThenWith<F, T, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  T: FnMut(
    O,
    ParseState<'_, 'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let start = input.cursor().clone();
    self
      .parser
      .parse_input(input)
      .and_then(|output| (self.then)(output, ParseState::new(input, start)))
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_ignore_then_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::new().ignore_then(Any::new()))
  }

  fn assert_then_ignore_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::new().then_ignore(Any::new()))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_ignore_then_parse_impl();
    let _ = assert_then_ignore_parse_impl();
  }
}
