/// A fallible owned projection to `T`.
pub trait Downcast<T> {
  /// Consumes `self` and returns `Some(T)` when the projection is valid.
  ///
  /// Returns `None` when this value cannot be projected to `T`.
  fn downcast(self) -> Option<T>;
}

/// A fallible borrowed projection to `T`.
pub trait DowncastRef<T> {
  /// Borrows `self` and returns `Some(T)` when the projection is valid.
  ///
  /// Returns `None` when this value cannot be projected to `T`.
  fn downcast_ref(&self) -> Option<T>;
}
