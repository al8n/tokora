// use crate::utils::Spanned;

// use super::*;

// /// An iterator over the tokens produced by a [`Input`].
// #[derive(derive_more::From, derive_more::Into)]
// pub struct IntoIter<'inp, 'closure, T: Token<'inp>, L: Lexer<'inp, T>, C> {
//   stream: InputRef<'inp, 'closure, T, L, C>,
// }

// impl<'inp, 'closure, T, L, C> IntoIter<'inp, 'closure, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
// {
//   pub(super) const fn new(stream: InputRef<'inp, 'closure, T, L, C>) -> Self {
//     Self { stream }
//   }
// }

// impl<'inp, T: Token<'inp>, L: Lexer<'inp, T>, C> Clone for IntoIter<'inp, '_, T, L, C>
// where
//   L::State: Clone,
//   C: Clone,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn clone(&self) -> Self {
//     Self {
//       stream: self.stream.clone(),
//     }
//   }
// }

// impl<'inp, T, L, C> core::fmt::Debug for IntoIter<'inp, '_, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
//   L::Source: core::fmt::Debug,
//   L::State: core::fmt::Debug,
//   C: core::fmt::Debug,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//     self.stream.fmt(f)
//   }
// }

// impl<'inp, 'closure, T, L, C> IntoIterator for InputRef<'inp, 'closure, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
//   L::State: Clone,
//   C: Cache<'inp, T, L>,
// {
//   type Item = Spanned<Lexed<'inp, T>>;
//   type IntoIter = IntoIter<'inp, 'closure, T, L, C>;

//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn into_iter(self) -> Self::IntoIter {
//     self.into_iter()
//   }
// }

// impl<'inp, T, L, C> Iterator for IntoIter<'inp, '_, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
//   L::State: Clone,
//   C: Cache<'inp, T, L>,
// {
//   type Item = Spanned<Lexed<'inp, T>>;

//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn next(&mut self) -> Option<Self::Item> {
//     InputRef::next(&mut self.stream)
//   }
// }

// /// An iterator over the tokens produced by a [`Input`].
// #[derive(derive_more::From, derive_more::Into)]
// pub struct Iter<'inp, 'closure, T: Token<'inp>, L: Lexer<'inp, T>, C> {
//   stream: &'closure mut InputRef<'inp, 'closure, T, L, C>,
// }

// impl<'inp, 'closure, T, L, C> Iter<'inp, 'closure, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub(super) const fn new(stream: &'closure mut InputRef<'inp, 'closure, T, L, C>) -> Self {
//     Self { stream }
//   }
// }

// impl<'inp, 'closure, T, L, C> IntoIterator for &'closure mut InputRef<'inp, 'closure, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
//   L::State: Clone,
//   C: Cache<'inp, T, L>,
// {
//   type Item = Spanned<Lexed<'inp, T>>;
//   type IntoIter = Iter<'inp, 'closure, T, L, C>;

//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn into_iter(self) -> Self::IntoIter {
//     self.iter()
//   }
// }

// impl<'inp, T, L, C> Iterator for Iter<'inp, '_, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
//   L::State: Clone,
//   C: Cache<'inp, T, L>,
// {
//   type Item = Spanned<Lexed<'inp, T>>;

//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn next(&mut self) -> Option<Self::Item> {
//     Input::next(self.stream)
//   }
// }
