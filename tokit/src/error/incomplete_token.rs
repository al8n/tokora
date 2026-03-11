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

#[cfg(test)]
#[allow(warnings)]
#[cfg(feature = "std")]
mod tests {
  use super::*;
  use core::hash::Hash;

  #[test]
  fn new_without_knowledge() {
    let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
    assert_eq!(e.span(), SimpleSpan::new(0, 5));
    assert_eq!(e.knowledge(), None);
  }

  #[test]
  fn with_knowledge_test() {
    let e: IncompleteToken<&str> = IncompleteToken::with_knowledge(SimpleSpan::new(0, 5), "int");
    assert_eq!(e.span(), SimpleSpan::new(0, 5));
    assert_eq!(e.knowledge(), Some(&"int"));
  }

  #[test]
  fn into_components_no_knowledge() {
    let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(2, 8));
    let (span, knowledge) = e.into_components();
    assert_eq!(span, SimpleSpan::new(2, 8));
    assert_eq!(knowledge, None);
  }

  #[test]
  fn into_components_with_knowledge() {
    let e: IncompleteToken<&str> = IncompleteToken::with_knowledge(SimpleSpan::new(2, 8), "float");
    let (span, knowledge) = e.into_components();
    assert_eq!(span, SimpleSpan::new(2, 8));
    assert_eq!(knowledge, Some("float"));
  }

  #[test]
  fn bump_adjusts_span() {
    let mut e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
    e.bump(&10);
    assert_eq!(e.span(), SimpleSpan::new(10, 15));
  }

  #[test]
  fn display_no_knowledge() {
    extern crate alloc;
    let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(3, 7));
    let s = alloc::format!("{e}");
    assert!(s.contains("incomplete token at"));
  }

  #[test]
  fn display_with_knowledge() {
    extern crate alloc;
    use crate::utils::knowledge::IntLiteral;
    let e: IncompleteToken<IntLiteral> =
      IncompleteToken::with_knowledge(SimpleSpan::new(3, 7), IntLiteral(()));
    let s = alloc::format!("{e}");
    assert!(s.contains("incomplete"));
    assert!(s.contains("token at"));
  }

  #[test]
  fn clone_and_eq() {
    let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
    assert_eq!(e, e.clone());
  }

  #[test]
  fn debug_impl() {
    extern crate alloc;
    let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
    let s = alloc::format!("{e:?}");
    assert!(s.contains("IncompleteToken"));
  }

  #[test]
  fn hash_impl() {
    let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    e.hash(&mut hasher);
  }
}
