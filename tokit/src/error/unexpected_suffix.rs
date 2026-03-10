use core::ops::{Add, AddAssign};

use crate::{
  span::SimpleSpan,
  utils::{CharLen, Lexeme, PositionedChar, human_display::DisplayHuman},
};

/// An error indicating that an unexpected suffix was found after a valid token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnexpectedSuffix<Char, Knowledge, O = usize> {
  token: SimpleSpan<O>,
  suffix: Lexeme<Char, O>,
  knowledge: Option<Knowledge>,
}

impl<Char, Knowledge, O> UnexpectedSuffix<Char, Knowledge, O> {
  /// Creates a new `UnexpectedSuffix` error with the span of the valid token and the unexpected suffix.
  ///
  /// ## Panics
  /// - If the suffix span starts before the token span ends.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///     SimpleSpan::new(0, 5),
  ///     Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(token: SimpleSpan<O>, suffix: Lexeme<Char, O>) -> Self
  where
    O: Ord,
  {
    assert!(
      suffix.start_ref() >= token.end_ref(),
      "suffix starts before token ends"
    );

    Self {
      token,
      suffix,
      knowledge: None,
    }
  }

  /// Create a new `UnexpectedSuffix` error from the given token span and character with position.
  ///
  /// ## Panics
  /// - If the positioned character's position is before the token span ends.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_char(
  ///    SimpleSpan::new(0, 5),
  ///    5,
  ///   'x'
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn from_char(token: SimpleSpan<O>, pos: O, ch: Char) -> Self
  where
    O: Ord,
  {
    Self::from_positioned_char(token, PositionedChar::with_position(ch, pos))
  }

  /// Create a new `UnexpectedSuffix` error from the given token span and character with position.
  ///
  /// ## Panics
  /// - If the positioned character's position is before the token span ends.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_positioned_char(
  ///    SimpleSpan::new(0, 5),
  ///    PositionedChar::with_position('x', 5)
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn from_positioned_char(token: SimpleSpan<O>, ch: PositionedChar<Char, O>) -> Self
  where
    O: Ord,
  {
    Self::new(token, Lexeme::Char(ch))
  }

  /// Adds knowledge to the `UnexpectedSuffix` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_knowledge_const(mut self, knowledge: Knowledge) -> Self
  where
    Knowledge: Copy,
  {
    self.knowledge = Some(knowledge);
    self
  }

  /// Adds knowledge to the `UnexpectedSuffix` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_knowledge(mut self, knowledge: Knowledge) -> Self {
    self.knowledge = Some(knowledge);
    self
  }

  /// Create a new `UnexpectedSuffix` error from the given token span and the suffix span
  ///
  /// ## Panics
  /// - If the suffix span starts before the token span ends.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{Lexeme}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_suffix(
  ///   SimpleSpan::new(0, 5),
  ///   SimpleSpan::new(5, 10)
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn from_suffix(token: SimpleSpan<O>, span: SimpleSpan<O>) -> Self
  where
    O: Ord,
  {
    Self::new(token, Lexeme::Range(span))
  }

  /// Returns the full span since the start of the valid token to the end of the unexpected suffix.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///   SimpleSpan::new(0, 5),
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// assert_eq!(error.span(), SimpleSpan::new(0, 6));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span(&self) -> SimpleSpan<O>
  where
    Char: CharLen,
    for<'a> &'a O: Add<usize, Output = O>,
    O: Clone + Ord,
  {
    let start = self.token.start_ref();
    let end = match &self.suffix {
      Lexeme::Char(positioned_char) => {
        positioned_char.position_ref() + positioned_char.char_ref().char_len()
      }
      Lexeme::Range(span) => span.end_ref().clone(),
    };
    SimpleSpan::new(start.clone(), end)
  }

  /// Returns the span of the valid token before the unexpected suffix.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///     SimpleSpan::new(0, 5),
  ///     Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// assert_eq!(error.token(), SimpleSpan::new(0, 5));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn token(&self) -> SimpleSpan<O>
  where
    O: Copy,
  {
    self.token
  }

  /// Returns the span of the valid token before the unexpected suffix.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///     SimpleSpan::new(0, 5),
  ///     Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// assert_eq!(error.token_ref(), SimpleSpan::new(&0, &5));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn token_ref(&self) -> SimpleSpan<&O> {
    self.token.as_ref()
  }

  /// Returns the unexpected suffix found after the valid token.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///    SimpleSpan::new(0, 5),
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  ///
  /// assert_eq!(
  ///   error.suffix(),
  ///   &Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn suffix(&self) -> &Lexeme<Char, O> {
    &self.suffix
  }

  /// Consumes the error and returns the unexpected suffix.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///   SimpleSpan::new(0, 5),
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// let (token, suffix) = error.into_components();
  /// assert_eq!(token, SimpleSpan::new(0, 5));
  /// assert_eq!(
  ///   suffix,
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (SimpleSpan<O>, Lexeme<Char, O>) {
    (self.token, self.suffix)
  }

  /// Bumps both the start and end positions of the token span by the given offset.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining spans from different contexts.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use tokit::{SimpleSpan, utils::{Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let mut error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///   SimpleSpan::new(0, 5),
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// error.bump(&10);
  /// assert_eq!(error.token(), SimpleSpan::new(10, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &O) -> &mut Self
  where
    O: for<'a> AddAssign<&'a O> + Clone,
  {
    self.token.bump(offset);
    self
  }
}

impl<Char, Knowledge> core::fmt::Display for UnexpectedSuffix<Char, Knowledge>
where
  Char: DisplayHuman,
  Knowledge: DisplayHuman,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.suffix {
      Lexeme::Char(positioned_char) => match &self.knowledge {
        None => write!(
          f,
          "unexpected suffix '{}' at position {} found after {}",
          positioned_char.char_ref().display(),
          positioned_char.position(),
          self.token
        ),
        Some(knowledge) => {
          write!(
            f,
            "unexpected suffix '{}' at position {} found after '{}'@({})",
            positioned_char.char_ref().display(),
            positioned_char.position(),
            knowledge.display(),
            self.token
          )
        }
      },
      Lexeme::Range(span) => match &self.knowledge {
        Some(knowledge) => write!(
          f,
          "unexpected suffix at {} found after '{}'@({})",
          span,
          knowledge.display(),
          self.token
        ),
        None => write!(
          f,
          "unexpected suffix at {} found after {}",
          span, self.token
        ),
      },
    }
  }
}

impl<Char, Knowledge> core::error::Error for UnexpectedSuffix<Char, Knowledge>
where
  Char: DisplayHuman + core::fmt::Debug,
  Knowledge: DisplayHuman + core::fmt::Debug,
{
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(feature = "std")]
mod tests {
  use super::*;
  use core::hash::Hash;

  type UsSuffix = UnexpectedSuffix<u8, ()>;

  fn make_char_error() -> UsSuffix {
    UnexpectedSuffix::new(
      SimpleSpan::new(0, 5),
      Lexeme::Char(PositionedChar::with_position(b'x', 5)),
    )
  }

  fn make_range_error() -> UsSuffix {
    UnexpectedSuffix::from_suffix(SimpleSpan::new(0, 5), SimpleSpan::new(5, 10))
  }

  #[test]
  fn new_creates_error() {
    let e = make_char_error();
    assert_eq!(e.token(), SimpleSpan::new(0, 5));
  }

  #[test]
  fn from_char_creates_error() {
    let e: UsSuffix = UnexpectedSuffix::from_char(SimpleSpan::new(0, 5), 5, b'x');
    assert_eq!(e.token(), SimpleSpan::new(0, 5));
  }

  #[test]
  fn from_positioned_char_creates_error() {
    let e: UsSuffix = UnexpectedSuffix::from_positioned_char(
      SimpleSpan::new(0, 5),
      PositionedChar::with_position(b'x', 5),
    );
    assert_eq!(e.token(), SimpleSpan::new(0, 5));
  }

  #[test]
  fn from_suffix_creates_error() {
    let e = make_range_error();
    assert_eq!(e.token(), SimpleSpan::new(0, 5));
  }

  #[test]
  fn with_knowledge_method() {
    let e = make_char_error().with_knowledge(());
    assert_eq!(e.token(), SimpleSpan::new(0, 5));
  }

  #[test]
  fn with_knowledge_const_method() {
    let e = make_char_error().with_knowledge_const(());
    assert_eq!(e.token(), SimpleSpan::new(0, 5));
  }

  #[test]
  fn span_char_variant() {
    let e = make_char_error();
    assert_eq!(e.span(), SimpleSpan::new(0, 6));
  }

  #[test]
  fn span_range_variant() {
    let e = make_range_error();
    assert_eq!(e.span(), SimpleSpan::new(0, 10));
  }

  #[test]
  fn token_ref_method() {
    let e = make_char_error();
    assert_eq!(e.token_ref(), SimpleSpan::new(&0, &5));
  }

  #[test]
  fn suffix_accessor() {
    let e = make_char_error();
    assert_eq!(
      e.suffix(),
      &Lexeme::Char(PositionedChar::with_position(b'x', 5))
    );
  }

  #[test]
  fn into_components_test() {
    let e = make_char_error();
    let (token, suffix) = e.into_components();
    assert_eq!(token, SimpleSpan::new(0, 5));
    assert_eq!(suffix, Lexeme::Char(PositionedChar::with_position(b'x', 5)));
  }

  #[test]
  fn bump_test() {
    let mut e = make_char_error();
    e.bump(&10);
    assert_eq!(e.token(), SimpleSpan::new(10, 15));
  }

  #[test]
  fn display_char_no_knowledge() {
    extern crate alloc;
    let e = make_char_error();
    let s = alloc::format!("{e}");
    assert!(s.contains("unexpected suffix"));
    assert!(s.contains("position 5"));
  }

  #[test]
  fn display_char_with_knowledge() {
    extern crate alloc;
    let e = make_char_error().with_knowledge(());
    let s = alloc::format!("{e}");
    assert!(s.contains("unexpected suffix"));
  }

  #[test]
  fn display_range_no_knowledge() {
    extern crate alloc;
    let e = make_range_error();
    let s = alloc::format!("{e}");
    assert!(s.contains("unexpected suffix at"));
  }

  #[test]
  fn display_range_with_knowledge() {
    extern crate alloc;
    let e = make_range_error().with_knowledge(());
    let s = alloc::format!("{e}");
    assert!(s.contains("unexpected suffix at"));
  }

  #[test]
  fn clone_and_eq() {
    let e = make_char_error();
    let e2 = e.clone();
    assert_eq!(e, e2);
  }

  #[test]
  fn debug_impl() {
    extern crate alloc;
    let e = make_char_error();
    let s = alloc::format!("{e:?}");
    assert!(s.contains("UnexpectedSuffix"));
  }

  #[test]
  fn hash_impl() {
    let e = make_char_error();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    e.hash(&mut hasher);
  }
}
