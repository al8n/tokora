/// A trait for tokens that can be compared for equivalence against a reference.
/// A helper trait for ergonomically requiring specific token shapes.
///
/// `Require` is intended for tiny wrappers (e.g., `Dot`, `Comma`, `ParenOpen`) that want a
/// `try_into`-style API without consuming the token stream. Implementors typically return
/// `Ok(output)` when the token matches the desired pattern, or `Err(Self::Err)` to hand the
/// original token (or a custom error type) back to the caller so other logic can handle it.
///
/// ## Example
///
/// ```rust
/// use tokit::{Require, IdentifierToken};
///
/// #[derive(Debug, Clone)]
/// pub enum Punct {
///     Dot,
///     Comma,
///     Other(String),
/// }
///
/// #[derive(Debug, Clone)]
/// pub struct Dot(pub Punct);
///
/// impl Require<Dot> for Punct {
///     type Err = Self;
///
///     fn require(self) -> Result<Dot, Self::Err> {
///         match &self {
///             Punct::Dot => Ok(Dot(Self::Dot)),
///             _ => Err(self),
///         }
///     }
/// }
/// ```
pub trait Require<O> {
  /// The error type returned when a requirement is not met.
  type Err;

  /// Attempts to extract the desired output from the token, returning `Err(Self::Err)` if not possible.
  fn require(self) -> Result<O, Self::Err>
  where
    Self: Sized;
}
