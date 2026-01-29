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
/// use tokit::Require;
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
///     fn matched(&self) -> bool {
///         matches!(self, Punct::Dot)
///     }
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

  /// Returns `true` if the `Self` matches, and can safely be converted to `O`.
  fn matched(&self) -> bool;

  /// Attempts to extract the desired output from `Self`, returning `Err(Self::Err)` if not possible.
  fn require(self) -> Result<O, Self::Err>
  where
    Self: Sized;
}
