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
pub struct Then<F, T> {
  parser: F,
  then: T,
}

impl<F, T> Then<F, T> {
  /// Creates a new `Then` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(parser: F, then: T) -> Self {
    Self { parser, then }
  }
}

impl<'inp, F, T, L, O, U, E, C> ParseInput<'inp, L, (O, U), E, C> for Then<F, T>
where
  F: ParseInput<'inp, L, O, E, C>,
  T: ParseInput<'inp, L, U, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> Result<(O, U), E::Error> {
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
pub struct IgnoreThen<F, G, O1> {
  first: F,
  second: G,
  _marker: PhantomData<O1>,
}

impl<F, G, O1> IgnoreThen<F, G, O1> {
  /// Creates a new `IgnoreThen` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(first: F, second: G) -> Self {
    Self {
      first,
      second,
      _marker: PhantomData,
    }
  }
}

impl<'inp, F, G, L, O1, O2, E, C> ParseInput<'inp, L, O2, E, C> for IgnoreThen<F, G, O1>
where
  F: ParseInput<'inp, L, O1, E, C>,
  G: ParseInput<'inp, L, O2, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> Result<O2, E::Error> {
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
pub struct ThenIgnore<F, G, O2> {
  first: F,
  second: G,
  _marker: PhantomData<O2>,
}

impl<F, G, O2> ThenIgnore<F, G, O2> {
  /// Creates a new `ThenIgnore` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(first: F, second: G) -> Self {
    Self {
      first,
      second,
      _marker: PhantomData,
    }
  }
}

impl<'inp, F, G, L, O1, O2, E, C> ParseInput<'inp, L, O1, E, C> for ThenIgnore<F, G, O2>
where
  F: ParseInput<'inp, L, O1, E, C>,
  G: ParseInput<'inp, L, O2, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> Result<O1, E::Error> {
    let first_result = self.first.parse_input(input)?;
    self.second.parse_input(input).map(|_| first_result)
  }
}

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  fn assert_ignore_then_parse_impl<'inp>()
  -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::new().ignore_then(Any::new()))
  }

  fn assert_then_ignore_parse_impl<'inp>()
  -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::new().then_ignore(Any::new()))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_ignore_then_parse_impl();
    let _ = assert_then_ignore_parse_impl();
  }
}
