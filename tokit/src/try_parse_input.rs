use derive_more::{IsVariant, TryUnwrap, Unwrap};

use crate::{
  input::InputRef,
  parser::{Accepted, ByRef, Fold, Repeated, Separated, TryFold, TryFoldWith},
  punct::*,
  token::PunctuatorToken,
};

#[cfg(any(feature = "alloc", feature = "std"))]
use crate::parser::RFold;

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
/// returns `Ok(ParseAttempt::Decline)`, **no valid tokens are consumed** - the input position only advances
/// past any error tokens that were consumed by the emitter.
///
/// **IMPORTANT:**
/// Implicit backtracking may occur when a parser returns `Ok(ParseAttempt::Decline)`.
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

  /// Creates a `Fold` combinator that accumulates results using the provided initializer and accumulator.
  ///
  /// See also [`try_fold`](TryParseInput::try_fold), [`fold_while`](crate::ParseInput::fold_while), [try_fold_while](crate::ParseInput::try_fold_while).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fold<Init, Acc>(self, init: Init, acc: Acc) -> Fold<Self, Init, Acc, L, O, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Init: FnMut() -> O,
    Acc: FnMut(O, O) -> O,
  {
    Fold::new(self, init, acc)
  }

  /// Creates a `TryFold` combinator that accumulates results using the provided initializer and fallible accumulator.
  ///
  /// See also [`try_fold_with`](Self::try_fold_with), [`fold_while`](crate::ParseInput::fold_while), [try_fold_while](crate::ParseInput::try_fold_while).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_fold<Init, Acc>(self, init: Init, acc: Acc) -> TryFold<Self, Init, Acc, L, O, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Init: FnMut() -> O,
    Acc: FnMut(O, O) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    TryFold::new(self, init, acc)
  }

  /// Creates a `TryFoldWith` combinator that accumulates results using the provided initializer,
  /// fallible accumulator, and parsing state.
  ///
  /// See also [`try_fold`](Self::try_fold), [`fold_while`](crate::ParseInput::fold_while), [try_fold_while](crate::ParseInput::try_fold_while).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_fold_with<Init, Acc>(
    self,
    init: Init,
    acc: Acc,
  ) -> TryFoldWith<Self, Init, Acc, L, O, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Init: FnMut() -> O,
    Acc: FnMut(
      O,
      O,
      ParseState<'_, 'inp, '_, L, Ctx, Lang>,
    ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    TryFoldWith::new(self, init, acc)
  }

  /// Creates a `RFold` combinator that accumulates results in reverse order using the provided
  /// initializer and accumulator.
  ///
  /// This buffers all parsed outputs before folding them from right to left.
  /// Parsing stops when this parser returns `Ok(ParseAttempt::Decline)`.
  ///
  /// See also [`fold`](Self::fold).
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rfold<Init, Acc>(self, init: Init, acc: Acc) -> RFold<Self, Init, Acc, L, O, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Init: FnMut() -> O,
    Acc: FnMut(O, O) -> O,
  {
    RFold::new(self, init, acc)
  }

  /// Creates a `Separated` combinator that parses separated elements, where **this parser
  /// handles the lookahead**.
  ///
  /// This parser (the element parser) will be called repeatedly to parse elements separated
  /// by the given separator. Since this parser implements [`TryParseInput`], it can peek ahead
  /// and return `Ok(ParseAttempt::Decline)` when it doesn't match, **without consuming any tokens**.
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
  fn separated<Sep>(self) -> Separated<Self, Sep, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Sep: Punctuator<'inp, L, Lang>,
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

#[cfg(test)]
mod tests {
  use super::*;

  // --- ParseAttempt tests ---

  #[test]
  fn accept_is_accepted() {
    let pa = Accept(42);
    assert!(pa.is_accept());
    assert!(!pa.is_decline());
  }

  #[test]
  fn decline_is_declined() {
    let pa: ParseAttempt<i32> = Decline;
    assert!(!pa.is_accept());
    assert!(pa.is_decline());
  }

  #[test]
  fn accept_map() {
    let pa = Accept(10);
    let mapped = pa.map(|v| v + 1);
    assert_eq!(mapped, Accept(11));
  }

  #[test]
  fn decline_map() {
    let pa: ParseAttempt<i32> = Decline;
    let mapped = pa.map(|v: i32| v + 1);
    assert_eq!(mapped, Decline);
  }

  #[test]
  fn accept_as_ref() {
    let pa = Accept(42);
    let r = pa.as_ref();
    assert_eq!(r, Accept(&42));
  }

  #[test]
  fn decline_as_ref() {
    let pa: ParseAttempt<i32> = Decline;
    let r = pa.as_ref();
    assert!(r.is_decline());
  }

  #[test]
  fn accept_as_mut() {
    let mut pa = Accept(42);
    let r = pa.as_mut();
    assert!(r.is_accept());
  }

  #[test]
  fn decline_as_mut() {
    let mut pa: ParseAttempt<i32> = Decline;
    let r = pa.as_mut();
    assert!(r.is_decline());
  }

  #[test]
  fn accept_and_then_ok() {
    let pa = Accept(10);
    let result: Result<ParseAttempt<i32>, &str> = pa.and_then(|v| Ok(v + 1));
    assert_eq!(result, Ok(Accept(11)));
  }

  #[test]
  fn accept_and_then_err() {
    let pa = Accept(10);
    let result: Result<ParseAttempt<i32>, &str> = pa.and_then(|_| Err("fail"));
    assert_eq!(result, Err("fail"));
  }

  #[test]
  fn decline_and_then() {
    let pa: ParseAttempt<i32> = Decline;
    let result: Result<ParseAttempt<i32>, &str> = pa.and_then(|v| Ok(v + 1));
    assert_eq!(result, Ok(Decline));
  }

  // --- From/Into conversions ---

  #[test]
  fn from_some_to_accept() {
    let pa: ParseAttempt<i32> = Some(42).into();
    assert_eq!(pa, Accept(42));
  }

  #[test]
  fn from_none_to_decline() {
    let pa: ParseAttempt<i32> = None.into();
    assert_eq!(pa, Decline);
  }

  #[test]
  fn accept_into_some() {
    let opt: Option<i32> = Accept(42).into();
    assert_eq!(opt, Some(42));
  }

  #[test]
  fn decline_into_none() {
    let opt: Option<i32> = Decline.into();
    assert_eq!(opt, None);
  }

  // --- Derived methods ---

  #[test]
  fn accept_unwrap() {
    let pa = Accept(42);
    assert_eq!(pa.unwrap_accept_ref(), &42);
  }

  #[test]
  fn accept_try_unwrap() {
    let pa = Accept(42);
    assert_eq!(pa.try_unwrap_accept_ref(), Ok(&42));
  }

  #[test]
  fn decline_try_unwrap() {
    let pa: ParseAttempt<i32> = Decline;
    assert!(pa.try_unwrap_accept_ref().is_err());
  }

  #[test]
  fn accept_clone_eq() {
    let pa = Accept(42);
    let cloned = pa.clone();
    assert_eq!(pa, cloned);
  }

  #[test]
  fn decline_clone_eq() {
    let pa: ParseAttempt<i32> = Decline;
    let cloned = pa.clone();
    assert_eq!(pa, cloned);
  }

  #[test]
  fn accept_debug() {
    let pa = Accept(42);
    let dbg = format!("{:?}", pa);
    assert!(dbg.contains("Accept"));
    assert!(dbg.contains("42"));
  }

  #[test]
  fn decline_debug() {
    let pa: ParseAttempt<i32> = Decline;
    let dbg = format!("{:?}", pa);
    assert!(dbg.contains("Decline"));
  }
}
