use super::*;

/// An iterator over the tokens produced by a [`Input`].
#[derive(derive_more::From, derive_more::Into)]
pub struct IntoIter<'a, T: Token<'a>, L: Lexer<'a, T>, C> {
  stream: Input<'a, T, L, C>,
}

impl<'a, T, L, C> IntoIter<'a, T, L, C>
where
  T: Token<'a>,
  L: Lexer<'a, T>,
{
  pub(super) const fn new(stream: Input<'a, T, L, C>) -> Self {
    Self { stream }
  }
}

impl<'a, T: Token<'a>, L: Lexer<'a, T>, C> Clone for IntoIter<'a, T, L, C>
where
  L::State: Clone,
  C: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    Self {
      stream: self.stream.clone(),
    }
  }
}

impl<'a, T, L, C> core::fmt::Debug for IntoIter<'a, T, L, C>
where
  T: Token<'a>,
  L: Lexer<'a, T>,
  L::Source: core::fmt::Debug,
  L::State: core::fmt::Debug,
  C: core::fmt::Debug,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.stream.fmt(f)
  }
}

impl<'a, T, L, C> IntoIterator for Input<'a, T, L, C>
where
  T: Token<'a>,
  L: Lexer<'a, T>,
  L::State: Clone,
  C: Cache<'a, T, L>,
{
  type Item = Spanned<Lexed<'a, T>>;
  type IntoIter = IntoIter<'a, T, L, C>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.into_iter()
  }
}

impl<'a, T, L, C> Iterator for IntoIter<'a, T, L, C>
where
  T: Token<'a>,
  L: Lexer<'a, T>,
  L::State: Clone,
  C: Cache<'a, T, L>,
{
  type Item = Spanned<Lexed<'a, T>>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(&mut self) -> Option<Self::Item> {
    Input::next(&mut self.stream)
  }
}

/// An iterator over the tokens produced by a [`Input`].
#[derive(derive_more::From, derive_more::Into)]
pub struct Iter<'a, 'b, T: Token<'a>, L: Lexer<'a, T>, C> {
  stream: &'b mut Input<'a, T, L, C>,
}

impl<'a, 'b, T, L, C> Iter<'a, 'b, T, L, C>
where
  T: Token<'a>,
  L: Lexer<'a, T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(stream: &'b mut Input<'a, T, L, C>) -> Self {
    Self { stream }
  }
}

impl<'a, 'b, T, L, C> IntoIterator for &'b mut Input<'a, T, L, C>
where
  T: Token<'a>,
  L: Lexer<'a, T>,
  L::State: Clone,
  C: Cache<'a, T, L>,
{
  type Item = Spanned<Lexed<'a, T>>;
  type IntoIter = Iter<'a, 'b, T, L, C>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'a, T, L, C> Iterator for Iter<'a, '_, T, L, C>
where
  T: Token<'a>,
  L: Lexer<'a, T>,
  L::State: Clone,
  C: Cache<'a, T, L>,
{
  type Item = Spanned<Lexed<'a, T>>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(&mut self) -> Option<Self::Item> {
    Input::next(self.stream)
  }
}
