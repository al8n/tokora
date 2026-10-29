use crate::{
  Lexer,
  parse::{DelimiterHandler, SeparatorHandler},
  span::Spanned,
};

const _: () = {
  use heapless_0_9::{Deque, Vec};

  impl<'inp, L, T, const N: usize, LenT> SeparatorHandler<'inp, L> for Vec<T, N, LenT>
  where
    LenT: heapless_0_9::LenType,
  {
    const OBSERVES_SEPARATORS: bool = false;

    #[inline(always)]
    fn on_separator(&mut self, _: Spanned<L::Token, L::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  impl<'inp, L, T, const N: usize> SeparatorHandler<'inp, L> for Deque<T, N> {
    const OBSERVES_SEPARATORS: bool = false;

    #[inline(always)]
    fn on_separator(&mut self, _: Spanned<L::Token, L::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }
};

const _: () = {
  use heapless_0_9::{Deque, Vec};

  impl<'inp, L, T, const N: usize, LenT> DelimiterHandler<'inp, L> for Vec<T, N, LenT>
  where
    LenT: heapless_0_9::LenType,
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

  impl<'inp, L, T, const N: usize> DelimiterHandler<'inp, L> for Deque<T, N> {
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
