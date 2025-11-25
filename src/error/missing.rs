use core::{fmt, marker::PhantomData};

use crate::{
  syntax::{Language, Syntax},
  utils::Span,
};

/// Describes a missing syntax element between two anchors.
///
/// `Missing<T, Lang>` remembers the span immediately **before** the missing syntax and,
/// optionally, the span immediately **after** it. This allows diagnostics to highlight the
/// precise gap where the parser expected a `T` node. The generic `T` refers to a
/// [`Syntax`] implementation so the error can report the exact syntax kind via
/// [`Syntax::KIND`], while `Lang` ties the error to a specific [`Language`].
///
/// # When to Use
///
/// - A required AST/CST node is completely absent (e.g., missing identifier or block)
/// - Delimited structures contain consecutive items without the expected syntax between them
/// - Recovery wants to surface “something should be here” while pointing to the surrounding
///   context instead of fabricating bogus spans
///
/// # Anchors
///
/// - **`before`**: Span of the last token confidently parsed before the missing node.
/// - **`after` (optional)**: Span of the first token parsed after the missing node.
///   When `after` is `None` the gap is assumed to be at end-of-input or before an unknown
///   boundary.
///
/// The [`span`](Self::span) method derives a zero-width span when only `before` is known, or
/// the exclusive range between `before.end()` and `after.start()` when both anchors exist.
///
/// # Examples
///
/// ```rust,ignore
/// use logosky::{
///     error::Missing,
///     syntax::{Language, Syntax},
///     utils::Span,
/// };
///
/// // Suppose `ParameterListSyntax` implements `Syntax<KIND = SyntaxKind::ParameterList>`
/// let before = Span::new(10, 11); // '('
/// let after = Span::new(12, 13);  // ')'
/// let error = Missing::<ParameterListSyntax, MyLang>::between(before, after);
///
/// assert_eq!(error.kind(), SyntaxKind::ParameterList);
/// assert_eq!(error.span(), Span::new(11, 12)); // gap between '(' and ')'
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Missing<T, Lang, S = Span> {
  before: S,
  after: Option<S>,
  span: Option<S>,
  _syntax: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T, Lang, S> Missing<T, Lang, S> {
  /// Creates a missing error that occurs **after** the provided span.
  ///
  /// Use this when the parser reached the end of a construct without finding the required
  /// syntax (e.g., missing trailing expression before `}` or end-of-input).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(before: S) -> Self {
    Self {
      before,
      after: None,
      span: None,
      _syntax: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a missing error bounded by both a `before` and `after` span.
  ///
  /// The resulting [`span`](Self::span) covers the gap between `before.end()` and
  /// `after.start()`. When the anchors overlap (e.g., consecutive tokens), the gap collapses
  /// to a zero-width span at `after.start()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn between(before: S, after: S) -> Self
  where
    S: crate::lexer::Span,
  {
    Self {
      before,
      after: Some(after),
      span: Self::gap_span(before, Some(after)),
      _syntax: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Updates the trailing anchor, returning `self` for chaining.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_after(mut self, after: S) -> Self {
    self.after = Some(after);
    self
  }

  /// Sets/overwrites the trailing anchor in-place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_after(&mut self, after: S) {
    self.after = Some(after);
  }

  /// Returns the span immediately preceding the missing syntax.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn before(&self) -> S
  where
    S: Copy,
  {
    self.before
  }

  /// Returns the optional span immediately following the missing syntax.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn after(&self) -> Option<S>
  where
    S: Copy,
  {
    self.after
  }

  /// Returns the span representing the gap where the syntax should have existed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    match self.span.as_ref() {
      Some(span) => *span,
      None => self.before,
    }
  }

  /// Returns the span representing the gap where the syntax should have existed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &S {
    match self.span.as_ref() {
      Some(span) => span,
      None => &self.before,
    }
  }

  /// Bumps the spans of the missing node by the specified offset.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &S::Offset) -> &mut Self
  where
    S: crate::lexer::Span,
  {
    self.before.bump(offset);
    if let Some(after) = &mut self.after {
      after.bump(offset);
    }
    self.span = Self::gap_span(self.before, self.after);
    self
  }

  /// Returns the syntax kind of the missing node.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn kind(&self) -> <Lang as Language>::SyntaxKind
  where
    T: Syntax<Lang = Lang>,
    Lang: Language,
  {
    T::KIND
  }

  const fn gap_span(before: S, after: Option<S>) -> S
  where
    S: crate::lexer::Span,
  {
    match after {
      Some(after_span) => {
        let start = before.end();
        let end = after_span.start();

        if end >= start {
          S::new(start, end)
        } else {
          S::new(end, end)
        }
      }
      None => {
        let pos = before.end();
        S::new(pos, pos)
      }
    }
  }
}

impl<T, Lang, S> fmt::Display for Missing<T, Lang, S>
where
  T: Syntax<Lang = Lang>,
  Lang: Language,
  <Lang as Language>::SyntaxKind: fmt::Display,
  S: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.after {
      Some(after) => write!(
        f,
        "missing {} between {} and {}",
        self.kind(),
        self.before,
        after
      ),
      None => write!(f, "missing {} after {}", self.kind(), self.before),
    }
  }
}

impl<T, Lang, S> fmt::Debug for Missing<T, Lang, S>
where
  S: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Missing")
      .field("before", &self.before)
      .field("after", &self.after)
      .finish()
  }
}

impl<T, Lang, S> core::error::Error for Missing<T, Lang, S>
where
  T: Syntax<Lang = Lang>,
  Lang: Language,
  <Lang as Language>::SyntaxKind: fmt::Display + fmt::Debug,
  S: fmt::Debug + fmt::Display,
{
}
