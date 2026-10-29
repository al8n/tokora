use crate::{
  Lexer,
  parser::{DelimiterHandler, SeparatorHandler},
  span::Spanned,
};

const _: () = {
  use tinyvec_1::{Array, ArrayVec, SliceVec};

  impl<'inp, L, T, N> SeparatorHandler<'inp, L> for ArrayVec<N>
  where
    N: Array<Item = T>,
  {
    const OBSERVES_SEPARATORS: bool = false;

    #[inline(always)]
    fn on_separator(&mut self, _: Spanned<L::Token, L::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  impl<'inp, L, T> SeparatorHandler<'inp, L> for SliceVec<'_, T> {
    const OBSERVES_SEPARATORS: bool = false;

    #[inline(always)]
    fn on_separator(&mut self, _: Spanned<L::Token, L::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  const _: () = {
    impl<'inp, L, T, A> SeparatorHandler<'inp, L> for tinyvec_1::TinyVec<A>
    where
      A: Array<Item = T>,
    {
      const OBSERVES_SEPARATORS: bool = false;

      #[inline(always)]
      fn on_separator(&mut self, _: Spanned<L::Token, L::Span>)
      where
        L: Lexer<'inp>,
      {
      }
    }
  };
};

const _: () = {
  use tinyvec_1::{Array, ArrayVec, SliceVec};

  impl<'inp, L, T, N> DelimiterHandler<'inp, L> for ArrayVec<N>
  where
    N: Array<Item = T>,
  {
    #[inline(always)]
    fn on_open_delimiter(&mut self, _: Spanned<L::Token, L::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[inline(always)]
    fn on_close_delimiter(&mut self, _: Spanned<L::Token, L::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  impl<'inp, L, T> DelimiterHandler<'inp, L> for SliceVec<'_, T> {
    #[inline(always)]
    fn on_open_delimiter(&mut self, _: Spanned<L::Token, L::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[inline(always)]
    fn on_close_delimiter(&mut self, _: Spanned<L::Token, L::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  const _: () = {
    impl<'inp, L, T, A> DelimiterHandler<'inp, L> for tinyvec_1::TinyVec<A>
    where
      A: Array<Item = T>,
    {
      #[inline(always)]
      fn on_open_delimiter(&mut self, _: Spanned<L::Token, L::Span>)
      where
        L: Lexer<'inp>,
      {
      }

      #[inline(always)]
      fn on_close_delimiter(&mut self, _: Spanned<L::Token, L::Span>)
      where
        L: Lexer<'inp>,
      {
      }
    }
  };
};
