use core::ops::{Add, AddAssign};

use crate::utils::{CharLen, Lexeme, PositionedChar, Span, human_display::DisplayHuman};


/// An error indicating that an unexpected suffix was found after a valid token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnexpectedSuffix<Char, Knowledge, O = usize> {
  token: Span<O>,
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
  /// use logosky::{utils::{Span, Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///     Span::new(0, 5),
  ///     Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(token: Span<O>, suffix: Lexeme<Char, O>) -> Self
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
  /// use logosky::{utils::{Span, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_char(
  ///    Span::new(0, 5),
  ///    5,
  ///   'x'
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn from_char(token: Span<O>, pos: O, ch: Char) -> Self
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
  /// use logosky::{utils::{Span, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_positioned_char(
  ///    Span::new(0, 5),
  ///    PositionedChar::with_position('x', 5)
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn from_positioned_char(token: Span<O>, ch: PositionedChar<Char, O>) -> Self
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
  /// use logosky::{utils::{Span, Lexeme}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_suffix(
  ///   Span::new(0, 5),
  ///   Span::new(5, 10)
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn from_suffix(token: Span<O>, span: Span<O>) -> Self
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
  /// use logosky::{utils::{Span, Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///   Span::new(0, 5),
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// assert_eq!(error.span(), Span::new(0, 6));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span(&self) -> Span<O>
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
    Span::new(start.clone(), end)
  }

  /// Returns the span of the valid token before the unexpected suffix.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{utils::{Span, Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///     Span::new(0, 5),
  ///     Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// assert_eq!(error.token(), Span::new(0, 5));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn token(&self) -> Span<O>
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
  /// use logosky::{utils::{Span, Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///     Span::new(0, 5),
  ///     Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// assert_eq!(error.token_ref(), Span::new(&0, &5));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn token_ref(&self) -> Span<&O> {
    self.token.as_ref()
  }

  /// Returns the unexpected suffix found after the valid token.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{utils::{Span, Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///    Span::new(0, 5),
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
  /// use logosky::{utils::{Span, Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///   Span::new(0, 5),
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// let (token, suffix) = error.into_components();
  /// assert_eq!(token, Span::new(0, 5));
  /// assert_eq!(
  ///   suffix,
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (Span<O>, Lexeme<Char, O>) {
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
  /// use logosky::{utils::{Span, Lexeme, PositionedChar}, error::UnexpectedSuffix};
  ///
  /// let mut error: UnexpectedSuffix<char, ()> = UnexpectedSuffix::new(
  ///   Span::new(0, 5),
  ///   Lexeme::Char(PositionedChar::with_position('x', 5))
  /// );
  /// error.bump(10);
  /// assert_eq!(error.token(), Span::new(10, 15));
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
