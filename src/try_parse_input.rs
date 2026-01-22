use derive_more::{IsVariant, TryUnwrap, Unwrap};

use crate::{
  input::InputRef,
  parser::{Accepted, ByRef, Repeated, Separated},
  punct::*,
  token::PunctuatorToken,
};

use super::*;

pub use ParseAttempt::{Accept, Decline};

/// Result type for tentative parsing attempts.
#[derive(Debug, Clone, PartialEq, Eq, IsVariant, TryUnwrap, Unwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum ParseAttempt<O> {
  /// Parser successfully matched and produced a value.
  Accept(O),
  /// Parser declined to match without consuming any valid tokens.
  #[unwrap(ignore)]
  #[try_unwrap(ignore)]
  Decline,
}

impl<O> From<Option<O>> for ParseAttempt<O> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(opt: Option<O>) -> Self {
    match opt {
      Some(value) => Self::Accept(value),
      None => Self::Decline,
    }
  }
}

impl<O> From<ParseAttempt<O>> for Option<O> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(result: ParseAttempt<O>) -> Self {
    match result {
      ParseAttempt::Accept(value) => Some(value),
      ParseAttempt::Decline => None,
    }
  }
}

impl<O> ParseAttempt<O> {
  /// Converts to a `ParseAttempt` with a reference to the inner value.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref(&self) -> ParseAttempt<&O> {
    match self {
      Self::Accept(value) => ParseAttempt::Accept(value),
      Self::Decline => ParseAttempt::Decline,
    }
  }

  /// Converts to a `ParseAttempt` with a mutable reference to the inner value.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> ParseAttempt<&mut O> {
    match self {
      Self::Accept(value) => ParseAttempt::Accept(value),
      Self::Decline => ParseAttempt::Decline,
    }
  }

  /// Maps the inner value using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map<U, F>(self, f: F) -> ParseAttempt<U>
  where
    F: FnOnce(O) -> U,
  {
    match self {
      Self::Accept(value) => ParseAttempt::Accept(f(value)),
      Self::Decline => ParseAttempt::Decline,
    }
  }

  /// Maps the inner value using the given fallible function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn and_then<U, F, E>(self, f: F) -> Result<ParseAttempt<U>, E>
  where
    F: FnOnce(O) -> Result<U, E>,
  {
    match self {
      Self::Accept(value) => Ok(ParseAttempt::Accept(f(value)?)),
      Self::Decline => Ok(ParseAttempt::Decline),
    }
  }
}

macro_rules! define_separated_by {
  ($($name:ident),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Creates a `Separated` combinator which separates elements by the `" $name:snake "` separator and applies this parser repeatedly."]
        ///
        /// See [`separated`](crate::TryParseInput::separated) for details.
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn [< separated_by_ $name:snake>](
          self,
        ) -> Separated<Self, $name, O, L, Ctx, Lang>
        where
          Self: Sized,
          L: Lexer<'inp>,
          L::Token: PunctuatorToken<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
        {
          Separated::new(self)
        }
      )*
    }
  };
}

/// Tentative parsing trait for optional token consumption with automatic backtracking.
///
/// Unlike [`ParseInput`] which must produce a value or error, `TryParseInput` allows parsers
/// to inspect the input and decide whether to consume it based on lookahead. If the parser
/// returns `Ok(None)`, **no valid tokens are consumed** - the input position only advances
/// past any error tokens that were consumed by the emitter.
///
/// **IMPORTANT:**
/// Implicit backtracking may occur when a parser returns `Ok(None)`.
pub trait TryParseInput<'inp, L, O, Ctx, Lang: ?Sized = ()> {
  /// Attempts to parse `O` from the input without committing.
  ///
  /// **IMPORTANT:**
  ///
  /// Implementations **must** uphold this contract:
  /// - ✅ `Ok(ParseAttempt::Accept(value))` - Parser succeeded, tokens consumed, value produced
  /// - ✅ `Ok(ParseAttempt::Decline)` - Parser declined, **no valid tokens consumed** (error tokens may be consumed by emitter)
  ///   - Backtracking may occur - input position restored to before parse attempt
  /// - ✅ `Err(error)` - Parser encountered an error (may have consumed tokens)
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  /// Applies combinator on [`ParseAttempt::Accept`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn accepted(self) -> Accepted<Self, L, O, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Accepted::new(self)
  }

  /// Create a parser over a mutable reference to this parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn by_ref(&mut self) -> &mut ByRef<Self> {
    ByRef::from_ref_mut(self)
  }

  /// Creates a `Repeated` combinator that applies this parser repeatedly until it signals completion.
  ///
  /// The parser will be called repeatedly until:
  /// - It returns `Ok(ParseAttempt::Decline)` - parser peeked ahead, didn't match (no tokens consumed)
  /// - It returns `Err(e)` - parse error
  ///
  /// ## Key Behavior
  ///
  /// Since this parser implements [`TryParseInput`], when it returns `Ok(ParseAttempt::Decline)`:
  /// - The parser **peeked ahead** and saw tokens it doesn't match
  /// - **No tokens were consumed** - input position unchanged
  /// - Repetition **stops cleanly**
  ///
  /// ## When to Use This
  ///
  /// Use `.repeated()` when:
  /// - You want to parse zero or more occurrences
  /// - The parser can look ahead and decide if it matches
  /// - You want automatic stopping based on parser's lookahead
  ///
  /// ## See Also
  ///
  /// - [`repeated_while`](crate::ParseInput::repeated_while) - When you want to provide explicit stopping condition
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn repeated(self) -> Repeated<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Repeated::new(self)
  }

  /// Creates a `Separated` combinator that parses separated elements, where **this parser
  /// handles the lookahead**.
  ///
  /// This parser (the element parser) will be called repeatedly to parse elements separated
  /// by the given separator. Since this parser implements [`TryParseInput`], it can peek ahead
  /// and return `Ok(None)` when it doesn't match, **without consuming any tokens**.
  ///
  /// ## Key Behavior
  ///
  /// The combinator stops when this element parser returns `Ok(ParseAttempt::Decline)`:
  /// - Parser **peeked ahead** and saw tokens it doesn't match
  /// - **No tokens consumed** - separator or closing delimiter left in input
  /// - Parsing **stops cleanly**
  ///
  /// ## When to Use This
  ///
  /// Use `.separated()` when:
  /// - Your element parser has built-in lookahead (implements `TryParseInput`)
  /// - You want the element parser to decide when to stop
  /// - The parser returns `Ok(ParseAttempt::Decline)` for non-matching tokens
  ///
  /// ## See Also
  ///
  /// - [`separated_while`](crate::ParseInput::separated_while) - When you want to provide the lookahead/stopping logic externally
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn separated<SepClassifier>(self) -> Separated<Self, SepClassifier, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    SepClassifier: Punctuator<'inp, L, Lang>,
  {
    Separated::new(self)
  }

  define_separated_by!(
    Comma,
    Semicolon,
    Dot,
    Colon,
    Pipe,
    Ampersand,
    Hyphen,
    Underscore,
    DoubleColon,
    Arrow,
    FatArrow,
    Tilde,
    Slash,
    Backslash,
    Percent,
    Dollar,
    Hash,
    At,
  );
}

impl<'inp, L, F, O, Ctx, Lang: ?Sized> TryParseInput<'inp, L, O, Ctx, Lang> for F
where
  F: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (self)(input)
  }
}

impl<'inp, L, F, O, Ctx, Lang: ?Sized> TryParseInput<'inp, L, O, Ctx, Lang> for &mut ByRef<F>
where
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (**self).try_parse_input(input)
  }
}
