use crate::utils::{CharLen, Lexeme, PositionedChar, Span, human_display::DisplayHuman};

/// An error indicating that an unexpected suffix was found after a valid token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnexpectedSuffix<Char, Knowledge, S = Span> {
  token: Span,
  suffix: Lexeme<Char>,
  knowledge: Option<Knowledge>,
}

impl<Char, Knowledge, S> UnexpectedSuffix<Char, Knowledge, S> {
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
  pub const fn new(token: Span, suffix: Lexeme<Char>) -> Self {
    assert!(
      suffix.start() >= token.end(),
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
  pub const fn from_char(token: Span, pos: usize, ch: Char) -> Self {
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
  pub const fn from_positioned_char(token: Span, ch: PositionedChar<Char>) -> Self {
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
  pub const fn from_suffix(token: Span, span: Span) -> Self {
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
  pub fn span(&self) -> Span
  where
    Char: CharLen,
  {
    let start = self.token.start();
    let end = match &self.suffix {
      Lexeme::Char(positioned_char) => {
        positioned_char.position() + positioned_char.char_ref().char_len()
      }
      Lexeme::Range(span) => span.end(),
    };
    Span::new(start, end)
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
  pub const fn token(&self) -> Span {
    self.token
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
  pub const fn suffix(&self) -> &Lexeme<Char> {
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
  pub fn into_components(self) -> (Span, Lexeme<Char>) {
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
  pub fn bump(&mut self, offset: usize) {
    self.token.bump(offset);
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
