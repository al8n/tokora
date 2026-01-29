use crate::{
  span::{SimpleSpan, Span},
  utils::human_display::DisplayHuman,
};

/// An incomplete token
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IncompleteToken<Knowledge, S = SimpleSpan> {
  span: S,
  knowledge: Option<Knowledge>,
}

impl<Knowledge> core::fmt::Display for IncompleteToken<Knowledge>
where
  Knowledge: DisplayHuman,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.knowledge {
      Some(knowledge) => write!(
        f,
        "incomplete {} token at {}",
        knowledge.display(),
        self.span
      ),
      None => write!(f, "incomplete token at {}", self.span),
    }
  }
}

impl<Knowledge> core::error::Error for IncompleteToken<Knowledge> where
  Knowledge: DisplayHuman + core::fmt::Debug
{
}

impl<Knowledge, S> IncompleteToken<Knowledge, S> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new_in(span: S, knowledge: Option<Knowledge>) -> Self {
    Self { span, knowledge }
  }

  /// Create a new `IncompleteToken` without any knowledge.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S) -> Self {
    Self::new_in(span, None)
  }

  /// Create a new `IncompleteToken` with knowledge.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_knowledge(span: S, knowledge: Knowledge) -> Self {
    Self::new_in(span, Some(knowledge))
  }

  /// Get the span of the incomplete token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    self.span
  }

  /// Get the knowledge for the incomplete token, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn knowledge(&self) -> Option<&Knowledge> {
    self.knowledge.as_ref()
  }

  /// Decompose the `IncompleteToken` into its components.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (S, Option<Knowledge>) {
    (self.span, self.knowledge)
  }

  /// Bumps both the start and end positions of the span by the given offset.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining spans from different contexts.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &S::Offset)
  where
    S: Span,
  {
    self.span.bump(offset);
  }
}
