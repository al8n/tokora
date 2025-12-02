/// A parser that applies its inner parser if a peeked condition is false.
pub struct OrNot<P>(pub(super) P);

impl<P> OrNot<P> {
  /// Creates a new `OrNot` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: P) -> Self {
    Self(parser)
  }
}
