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
pub struct Then<F, ThenFn, O> {
  parser: F,
  then_fn: ThenFn,
  _marker: PhantomData<O>,
}

impl<F, ThenFn, O> Then<F, ThenFn, O> {
  /// Creates a new `Then` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(parser: F, then_fn: ThenFn) -> Self {
    Self {
      parser,
      then_fn,
      _marker: PhantomData,
    }
  }
}

impl<'inp, F, ThenFn, L, O, NO, E, C, NextParser> ParseInput<'inp, L, NO, E, C>
  for Then<F, ThenFn, O>
where
  F: ParseInput<'inp, L, O, E, C>,
  ThenFn: FnMut(O) -> NextParser,
  NextParser: ParseInput<'inp, L, NO, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> NO {
    let result = self.parser.parse_input(input);
    let mut next_parser = (self.then_fn)(result);
    next_parser.parse_input(input)
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
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> O2 {
    let _first_result = self.first.parse_input(input);
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
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> O1 {
    let first_result = self.first.parse_input(input);
    let _second_result = self.second.parse_input(input);
    first_result
  }
}

impl<F, L, O, Error, E, C> With<F, Parser<(), L, O, Error, ParserOptions<L, E, C>>> {
  /// Sequence this parser with another, using the first result to determine the second parser.
  ///
  /// This creates a new parser that runs this parser first, then uses its result
  /// to determine and run the next parser.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let parser = Any::parser()
  ///     .then(|tok| match tok.kind() {
  ///         TokenKind::Number => parse_number(),
  ///         TokenKind::String => parse_string(),
  ///         _ => parse_value(),
  ///     });
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn then<ThenFn, NextParser, NO>(
    self,
    then_fn: ThenFn,
  ) -> With<Then<F, ThenFn, O>, Parser<(), L, NO, Error, ParserOptions<L, E, C>>>
  where
    ThenFn: FnMut(O) -> NextParser,
  {
    With::new(
      Then::new(self.primary, then_fn),
      Parser {
        f: (),
        opts: self.secondary.opts,
        _marker: PhantomData,
      },
    )
  }

  /// Sequence this parser with another, keeping only the second result.
  ///
  /// This runs this parser first (discarding its result), then runs the
  /// second parser and returns its result.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let parser = Expect::parser(is_keyword("let"))
  ///     .ignore_then(Any::parser());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn ignore_then<G, NO, E2, C2>(
    self,
    second: With<G, Parser<(), L, NO, Error, ParserOptions<L, E2, C2>>>,
  ) -> With<IgnoreThen<F, G, O>, Parser<(), L, NO, Error, ParserOptions<L, E, C>>> {
    With::new(
      IgnoreThen::new(self.primary, second.primary),
      Parser {
        f: (),
        opts: self.secondary.opts,
        _marker: PhantomData,
      },
    )
  }

  /// Sequence this parser with another, keeping only the first result.
  ///
  /// This runs this parser first, then runs the second parser (discarding
  /// its result), and returns the first result.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let parser = Any::parser()
  ///     .then_ignore(Expect::parser(is_semicolon));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn then_ignore<G, NO, E2, C2>(
    self,
    second: With<G, Parser<(), L, NO, Error, ParserOptions<L, E2, C2>>>,
  ) -> With<ThenIgnore<F, G, NO>, Parser<(), L, O, Error, ParserOptions<L, E, C>>> {
    With::new(
      ThenIgnore::new(self.primary, second.primary),
      Parser {
        f: (),
        opts: self.secondary.opts,
        _marker: PhantomData,
      },
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  fn assert_ignore_then_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()>
  {
    Any::parser().ignore_then(Any::parser())
  }

  fn assert_then_ignore_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()>
  {
    Any::parser().then_ignore(Any::parser())
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_ignore_then_parse_impl();
    let _ = assert_then_ignore_parse_impl();
  }
}
