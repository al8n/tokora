//! Syntax definition and incomplete syntax error types.
//!
//! This module provides types for representing syntax elements with a known number
//! of components, and errors for tracking missing components during parsing.
//!
//! Two implementations are provided:
//! - **Const-generic** (default): Uses `const COMPONENTS: usize` for component count
//! - **Type-level** (with `generic-array` feature): Uses `typenum` for type-level component count
//!
//! The implementation is chosen at compile time based on feature flags.
//!
//! # Feature Flags
//!
//! - Without `generic-array`: Uses const-generic implementation
//! - With `generic-array`: Uses type-level implementation with `generic_arraydeque::ArrayLength`
//!
//! # Design Philosophy
//!
//! When parsing syntax elements that require multiple components (like variable declarations,
//! function definitions, etc.), it's valuable to track *all* missing components rather than
//! failing on the first missing one. This enables:
//!
//! - Better error messages showing all missing parts
//! - Faster development iteration (see all errors at once)
//! - More helpful IDE diagnostics
//!
//! # Examples
//!
//! ```rust
//! # {
//! use tokit::{
//!     utils::{typenum::U3, GenericArrayDeque},
//!     syntax::Syntax,
//!     error::IncompleteSyntax
//! };
//! use core::fmt;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
//! struct MyLanguage;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! enum WhileComponent {
//!     WhileKeyword,
//!     Condition,
//!     Body,
//! }
//!
//! impl fmt::Display for WhileComponent {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         match self {
//!             Self::WhileKeyword => write!(f, "'while' keyword"),
//!             Self::Condition => write!(f, "condition"),
//!             Self::Body => write!(f, "body"),
//!         }
//!     }
//! }
//!
//! struct WhileLoop;
//!
//! impl Syntax for WhileLoop {
//!     type Component = WhileComponent;
//!     type COMPONENTS = U3;
//!     type REQUIRED = U3;
//!     type Lang = MyLanguage;
//!
//!     fn possible_components() -> &'static GenericArrayDeque<Self::Component, U3> {
//!         const COMPONENTS: &GenericArrayDeque<WhileComponent, U3> = &GenericArrayDeque::from_array([
//!             WhileComponent::WhileKeyword,
//!             WhileComponent::Condition,
//!             WhileComponent::Body,
//!         ]);
//!         COMPONENTS
//!     }
//!
//!     fn required_components() -> &'static GenericArrayDeque<Self::Component, U3> {
//!         const REQUIRED: &GenericArrayDeque<WhileComponent, U3> = &GenericArrayDeque::from_array([
//!             WhileComponent::WhileKeyword,
//!             WhileComponent::Condition,
//!             WhileComponent::Body,
//!         ]);
//!         REQUIRED
//!     }
//! }
//!
//! let mut error = IncompleteSyntax::<WhileLoop>::new(
//!     tokit::utils::SimpleSpan::new(10, 15),
//!     WhileComponent::Condition
//! );
//! assert_eq!(error.len(), 1);
//! # }
//! ```

use crate::{syntax::Syntax, utils::SimpleSpan};
use generic_arraydeque::{GenericArrayDeque, typenum::Unsigned};

use core::{
  fmt::{Debug, Display},
  hash::Hash,
};

/// Represents an incomplete syntax with missing components.
///
/// This error type is used to track which components are missing from a syntax
/// construct during parsing. It stores components as a set (no duplicates) and
/// always contains at least one missing component.
///
/// # Design Philosophy
///
/// When parsing fails, it's valuable to report *all* missing components rather
/// than just the first one encountered. This type accumulates missing components
/// up to the syntax's maximum component count.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// # {
/// use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
/// use typenum::U3;
/// use core::fmt;
///
/// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// struct MyLanguage;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum IfStatementComponent {
///     IfKeyword,
///     Condition,
///     ThenBlock,
/// }
///
/// impl fmt::Display for IfStatementComponent {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         match self {
///             Self::IfKeyword => write!(f, "'if' keyword"),
///             Self::Condition => write!(f, "condition"),
///             Self::ThenBlock => write!(f, "then block"),
///         }
///     }
/// }
///
/// struct IfStatement;
///
/// impl Syntax for IfStatement {
///     type Lang = MyLanguage;
///     type Component = IfStatementComponent;
///     type COMPONENTS = U3;
///     type REQUIRED = U3;
///
///     fn possible_components() -> &'static GenericArrayDeque<Self::Component, U3> {
///         const COMPONENTS: &GenericArrayDeque<IfStatementComponent, U3> = &GenericArrayDeque::from_array([
///            IfStatementComponent::IfKeyword,
///            IfStatementComponent::Condition,
///            IfStatementComponent::ThenBlock,
///         ]);
///         COMPONENTS
///     }
///
///     fn required_components() -> &'static GenericArrayDeque<Self::Component, U3> {
///         const REQUIRED: &GenericArrayDeque<IfStatementComponent, U3> = &GenericArrayDeque::from_array([
///             IfStatementComponent::IfKeyword,
///             IfStatementComponent::Condition,
///             IfStatementComponent::ThenBlock,
///         ]);
///         REQUIRED
///     }
/// }
///
/// // Report a missing component at a specific location
/// let error = IncompleteSyntax::<IfStatement>::new(
///     tokit::utils::SimpleSpan::new(10, 15),
///     IfStatementComponent::Condition
/// );
/// assert_eq!(error.len(), 1);
///
/// // Add more missing components
/// let mut error = error;
/// error.push(IfStatementComponent::ThenBlock);
/// assert_eq!(error.len(), 2);
/// # }
/// ```
///
/// ## Error Message Formatting
///
/// ```rust
/// # {
/// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
/// # use typenum::U2;
/// # use core::fmt;
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// # enum Component { A, B }
/// # impl fmt::Display for Component {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
/// #     }
/// # }
/// # struct MyLang;
/// # struct MySyntax;
/// # impl Syntax for MySyntax {
/// #     type Component = Component;
/// #     type COMPONENTS = U2;
/// #     type REQUIRED = U2;
/// #     type Lang = MyLang;
/// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
/// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
/// #         COMPONENTS
/// #     }
/// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
/// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
/// #         REQUIRED
/// #     }
/// # }
/// let mut error = IncompleteSyntax::<MySyntax>::new(
///     tokit::utils::SimpleSpan::new(10, 15),
///     Component::A
/// );
/// assert_eq!(format!("{}", error), "incomplete syntax: component A is missing");
///
/// error.push(Component::B);
/// assert_eq!(format!("{}", error), "incomplete syntax: components A, B are missing");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct IncompleteSyntax<S: Syntax, Sp = SimpleSpan> {
  span: Sp,
  components: GenericArrayDeque<S::Component, S::COMPONENTS>,
}

impl<S, Sp> PartialEq for IncompleteSyntax<S, Sp>
where
  S: Syntax,
  Sp: PartialEq,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Self) -> bool {
    self.span == other.span && self.components == other.components
  }
}

impl<S, Sp> Eq for IncompleteSyntax<S, Sp>
where
  S: Syntax,
  Sp: Eq,
{
}

impl<S, Sp> Hash for IncompleteSyntax<S, Sp>
where
  S: Syntax,
  Sp: Hash,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.span.hash(state);
    self.components.hash(state);
  }
}

impl<S, Sp> AsRef<[S::Component]> for IncompleteSyntax<S, Sp>
where
  S: Syntax,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[S::Component] {
    self.as_slice()
  }
}

impl<S, Sp> AsMut<[S::Component]> for IncompleteSyntax<S, Sp>
where
  S: Syntax,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_mut(&mut self) -> &mut [S::Component] {
    self.as_mut_slice()
  }
}

impl<S, Sp> IncompleteSyntax<S, Sp>
where
  S: Syntax,
{
  /// Creates a new incomplete syntax error with the specified span and missing component.
  ///
  /// The error always starts with at least one missing component.
  ///
  /// # Panics
  ///
  /// Panics if `S::COMPONENTS::USIZE` is 0 (which would be a malformed Syntax implementation).
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, SimpleSpan, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U1;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "A") }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U1;
  /// #     type REQUIRED = U1;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U1> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U1> = &GenericArrayDeque::from_array([Component::A]);
  /// #         COMPONENTS
  /// #     }
  /// #
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U1> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U1> = &GenericArrayDeque::from_array([Component::A]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let error = IncompleteSyntax::<MySyntax>::new(SimpleSpan::new(10, 15), Component::A);
  /// assert_eq!(error.len(), 1);
  /// assert_eq!(error.span(), SimpleSpan::new(10, 15));
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(span: Sp, component: S::Component) -> Self {
    if S::COMPONENTS::USIZE == 0 {
      panic!("IncompleteSyntax requires S::COMPONENTS to be non-zero");
    }
    let mut components = GenericArrayDeque::new();
    components.push_back(component);
    Self { span, components }
  }

  /// Tries to create an incomplete syntax error from a span and an iterator of components.
  ///
  /// Returns `None` if:
  /// - The iterator yields no components
  /// - The iterator yields more unique components than the buffer can hold
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, SimpleSpan, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  /// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
  /// #     }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         &COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let components = vec![Component::A, Component::B];
  /// let error = IncompleteSyntax::<MySyntax>::from_iter(SimpleSpan::new(10, 15), components).unwrap();
  /// assert_eq!(error.len(), 2);
  /// assert_eq!(error.span(), SimpleSpan::new(10, 15));
  ///
  /// // Empty iterator returns None
  /// let error = IncompleteSyntax::<MySyntax>::from_iter(SimpleSpan::new(10, 15), std::iter::empty());
  /// assert!(error.is_none());
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::should_implement_trait)]
  pub fn from_iter(span: Sp, iter: impl IntoIterator<Item = S::Component>) -> Option<Self> {
    let mut components = GenericArrayDeque::new();
    for component in iter {
      Self::try_push_impl(&mut components, component);
    }
    (!components.is_empty()).then_some(Self { span, components })
  }

  /// Helper function that tries to push a component with deduplication logic.
  ///
  /// Returns `None` if the component was added or already exists (success),
  /// `Some(component)` if the buffer is full (failure).
  #[inline(always)]
  fn try_push_impl(
    components: &mut GenericArrayDeque<S::Component, S::COMPONENTS>,
    component: S::Component,
  ) -> Option<S::Component> {
    if components.contains(&component) {
      None
    } else {
      components.push_back(component)
    }
  }

  /// Helper function that tries to push a component with deduplication logic.
  ///
  /// Returns `None` if the component was added or already exists (success),
  /// `Some(component)` if the buffer is full (failure).
  #[inline(always)]
  fn try_push_front_impl(
    components: &mut GenericArrayDeque<S::Component, S::COMPONENTS>,
    component: S::Component,
  ) -> Option<S::Component> {
    if components.contains(&component) {
      None
    } else {
      components.push_front(component)
    }
  }

  /// Returns the number of missing components.
  ///
  /// The length is always at least 1.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  /// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
  /// #     }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// assert_eq!(error.len(), 1);
  /// error.push(Component::B);
  /// assert_eq!(error.len(), 2);
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::len_without_is_empty)]
  pub fn len(&self) -> usize {
    self.components.len()
  }

  /// Returns the maximum number of components this error can hold.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U3;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B, C }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "X") }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U3;
  /// #     type REQUIRED = U3;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U3> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U3> = &GenericArrayDeque::from_array([Component::A, Component::B, Component::C]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U3> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U3> = &GenericArrayDeque::from_array([Component::A, Component::B, Component::C]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// assert_eq!(error.capacity(), 3);
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn capacity(&self) -> usize {
    self.components.capacity()
  }

  /// Returns `true` if the error is at full capacity.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  /// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
  /// #     }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// assert!(!error.is_full());
  /// error.push(Component::B);
  /// assert!(error.is_full());
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_full(&self) -> bool {
    self.components.is_full()
  }

  /// Pushes a new missing component into the error.
  ///
  /// If the component already exists in the error, this is a no-op (silently succeeds).
  /// This maintains the set semantics where each component appears at most once.
  ///
  /// # Panics
  ///
  /// Panics if the error is already full and the component is not already present.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  /// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
  /// #     }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// error.push(Component::B);
  /// // Pushing the same component again is a no-op
  /// error.push(Component::A);
  /// assert_eq!(error.len(), 2);
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn push(&mut self, component: S::Component) {
    if self.try_push(component).is_some() {
      panic!("IncompleteSyntax buffer overflow: cannot push more components")
    }
  }

  /// Pushes a new missing component into the error from the front.
  ///
  /// If the component already exists in the error, this is a no-op (silently succeeds).
  /// This maintains the set semantics where each component appears at most once.
  ///
  /// # Panics
  ///
  /// Panics if the error is already full and the component is not already present.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  /// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
  /// #     }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// error.push_front(Component::B);
  /// // Pushing the same component again is a no-op
  /// error.push_front(Component::A);
  /// assert_eq!(error.len(), 2);
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn push_front(&mut self, component: S::Component) {
    if self.try_push_front(component).is_some() {
      panic!("IncompleteSyntax buffer overflow: cannot push more components")
    }
  }

  /// Tries to push a new missing component into the error.
  ///
  /// Returns:
  /// - `None` if the component was added or already exists (success)
  /// - `Some(component)` if the buffer is full and the component is not present (failure)
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B, C }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "X") }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// assert!(error.try_push(Component::B).is_none()); // Success
  /// assert_eq!(error.try_push(Component::C), Some(Component::C)); // Full!
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_push(&mut self, component: S::Component) -> Option<S::Component> {
    Self::try_push_impl(&mut self.components, component)
  }

  /// Tries to push a new missing component into the error from the front.
  ///
  /// Returns:
  /// - `None` if the component was added or already exists (success)
  /// - `Some(component)` if the buffer is full and the component is not present (failure)
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B, C }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "X") }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// assert!(error.try_push_front(Component::B).is_none()); // Success
  /// assert_eq!(error.try_push_front(Component::C), Some(Component::C)); // Full!
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_push_front(&mut self, component: S::Component) -> Option<S::Component> {
    Self::try_push_front_impl(&mut self.components, component)
  }

  /// Returns a slice of the missing components.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  /// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
  /// #     }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// error.push(Component::B);
  /// assert_eq!(error.as_slice(), &[Component::A, Component::B]);
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_slice(&self) -> &[S::Component] {
    self.components.as_slices().0
  }

  /// Returns a mutable slice of the missing components.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  /// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
  /// #     }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Lang = MyLang;
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(
  ///     tokit::utils::SimpleSpan::new(10, 15),
  ///     Component::A
  /// );
  /// error.as_mut_slice()[0] = Component::B;
  /// assert_eq!(error.as_slice()[0], Component::B);
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_mut_slice(&mut self) -> &mut [S::Component] {
    self.components.as_mut_slices().0
  }

  /// Returns an iterator over the missing components.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, SimpleSpan, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U2;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A, B }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  /// #         match self { Self::A => write!(f, "A"), Self::B => write!(f, "B") }
  /// #     }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U2;
  /// #     type REQUIRED = U2;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U2> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U2> = &GenericArrayDeque::from_array([Component::A, Component::B]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(SimpleSpan::new(10, 15), Component::A);
  /// error.push(Component::B);
  /// let collected: Vec<_> = error.iter().collect();
  /// assert_eq!(collected, vec![&Component::A, &Component::B]);
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn iter(&self) -> generic_arraydeque::Iter<'_, S::Component> {
    self.components.iter()
  }

  /// Returns the span of the incomplete syntax.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, SimpleSpan, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U1;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "A") }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U1;
  /// #     type REQUIRED = U1;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U1> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U1> = &GenericArrayDeque::from_array([Component::A]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U1> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U1> = &GenericArrayDeque::from_array([Component::A]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let error = IncompleteSyntax::<MySyntax>::new(SimpleSpan::new(10, 15), Component::A);
  /// assert_eq!(error.span(), SimpleSpan::new(10, 15));
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> Sp
  where
    Sp: Copy,
  {
    self.span
  }

  /// Returns a reference to the span of the incomplete syntax.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &Sp {
    &self.span
  }

  /// Returns a mutable reference to the span of the incomplete syntax.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut Sp {
    &mut self.span
  }

  /// Bumps the span by the given offset.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining errors from different parsing contexts.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # {
  /// # use tokit::{utils::{typenum, SimpleSpan, GenericArrayDeque}, syntax::Syntax, error::IncompleteSyntax};
  /// # use typenum::U1;
  /// # use core::fmt;
  /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  /// # enum Component { A }
  /// # impl fmt::Display for Component {
  /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "A") }
  /// # }
  /// # struct MyLang;
  /// # struct MySyntax;
  /// # impl Syntax for MySyntax {
  /// #     type Component = Component;
  /// #     type COMPONENTS = U1;
  /// #     type REQUIRED = U1;
  /// #     type Lang = MyLang;
  /// #     fn possible_components() -> &'static GenericArrayDeque<Component, U1> {
  /// #         const COMPONENTS: &GenericArrayDeque<Component, U1> = &GenericArrayDeque::from_array([Component::A]);
  /// #         COMPONENTS
  /// #     }
  /// #     fn required_components() -> &'static GenericArrayDeque<Component, U1> {
  /// #         const REQUIRED: &GenericArrayDeque<Component, U1> = &GenericArrayDeque::from_array([Component::A]);
  /// #         REQUIRED
  /// #     }
  /// # }
  /// let mut error = IncompleteSyntax::<MySyntax>::new(SimpleSpan::new(10, 15), Component::A);
  /// error.bump(5);
  /// assert_eq!(error.span(), SimpleSpan::new(15, 20));
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &Sp::Offset) -> &mut Self
  where
    Sp: crate::lexer::Span,
  {
    self.span.bump(offset);
    self
  }
}

impl<S, Sp> Display for IncompleteSyntax<S, Sp>
where
  S: Syntax,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let components = self.as_slice();

    if components.len() == 1 {
      write!(
        f,
        "incomplete syntax: component {} is missing",
        components[0]
      )
    } else {
      write!(f, "incomplete syntax: components ")?;
      for (i, component) in components.iter().enumerate() {
        if i > 0 {
          write!(f, ", ")?;
        }
        write!(f, "{}", component)?;
      }
      write!(f, " are missing")
    }
  }
}

impl<S, Sp> core::error::Error for IncompleteSyntax<S, Sp>
where
  S: Syntax + Debug,
  Sp: Debug,
{
}
