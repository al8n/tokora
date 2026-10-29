use crate::{
  Lexer,
  parser::{DelimiterHandler, SeparatorHandler},
  span::Spanned,
};

impl<'inp, L, T, N> SeparatorHandler<'inp, L> for smallvec_1::SmallVec<N>
where
  N: smallvec_1::Array<Item = T>,
{
  const OBSERVES_SEPARATORS: bool = false;

  #[inline(always)]
  fn on_separator(&mut self, _: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>,
  {
  }
}

impl<'inp, L, T, N> DelimiterHandler<'inp, L> for smallvec_1::SmallVec<N>
where
  N: smallvec_1::Array<Item = T>,
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
