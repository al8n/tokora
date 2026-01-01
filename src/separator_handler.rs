use crate::{Lexer, utils::Spanned};

/// A handler for separator events during parsing.
pub trait SeparatorHandler<'inp, L> {
  /// Called when a separator is encountered.
  fn on_separator(&mut self, sep: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>;
}

impl<'inp, L, T> SeparatorHandler<'inp, L> for &mut T
where
  T: ?Sized + SeparatorHandler<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_separator(&mut self, sep: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>,
  {
    (**self).on_separator(sep)
  }
}

macro_rules! blackhole {
  ($ty:ty) => {
    impl<'inp, L> SeparatorHandler<'inp, L> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_separator(&mut self, _: Spanned<L::Token, L::Span>)
      where
        L: Lexer<'inp>,
      {
      }
    }
  };
  (@generic $ty:ty) => {
    impl<'inp, L, T> SeparatorHandler<'inp, L> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_separator(&mut self, _: Spanned<L::Token, L::Span>)
      where
        L: Lexer<'inp>,
      {
      }
    }
  };
}

blackhole!(());
blackhole!(crate::lexer::BlackHole);
blackhole!(@generic core::marker::PhantomData<T>);
blackhole!(@generic crate::utils::marker::Ignored<T>);

#[cfg(any(feature = "alloc", feature = "std"))]
const _: () = {
  use std::{collections::vec_deque::VecDeque, vec::Vec};

  impl<'inp, L, T> SeparatorHandler<'inp, L> for Vec<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_separator(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  impl<'inp, L, T> SeparatorHandler<'inp, L> for VecDeque<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_separator(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  #[cfg(feature = "smallvec")]
  impl<'inp, L, T, N> SeparatorHandler<'inp, L> for smallvec::SmallVec<N>
  where
    N: smallvec::Array<Item = T>,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_separator(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }
};
