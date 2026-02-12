use generic_arraydeque::{ArrayLength, GenericArrayDeque};

use crate::{
  Emitter, Lexer, ParseContext,
  input::{Checkpoint, InputRef},
  span::Spanned,
};

mod allow_leading;
mod allow_leading_require_trailing;
mod allow_surrounded;
mod allow_trailing;
mod bounded;
mod maximum;
mod minimum;
mod require_leading;
mod require_leading_allow_trailing;
mod require_surrounded;
mod require_trailing;
mod unbounded;

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

macro_rules! blackhole_separator_handler {
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

blackhole_separator_handler!(());
blackhole_separator_handler!(@generic core::marker::PhantomData<T>);
blackhole_separator_handler!(@generic crate::utils::marker::Ignored<T>);

impl<'inp, L, T, N> SeparatorHandler<'inp, L> for GenericArrayDeque<T, N>
where
  N: ArrayLength,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_separator(&mut self, _: Spanned<<L>::Token, <L>::Span>)
  where
    L: Lexer<'inp>,
  {
  }
}

#[cfg(feature = "heapless_0_9")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless_0_9")))]
const _: () = {
  use heapless_0_9::{Deque, Vec};

  impl<'inp, L, T, const N: usize, LenT> SeparatorHandler<'inp, L> for Vec<T, N, LenT>
  where
    LenT: heapless_0_9::LenType,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_separator(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  impl<'inp, L, T, const N: usize> SeparatorHandler<'inp, L> for Deque<T, N> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_separator(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }
};

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
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

  #[cfg(feature = "smallvec_1")]
  #[cfg_attr(docsrs, doc(cfg(feature = "smallvec_1")))]
  impl<'inp, L, T, N> SeparatorHandler<'inp, L> for smallvec_1::SmallVec<N>
  where
    N: smallvec_1::Array<Item = T>,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_separator(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }
};

/// A handler for delimiter events during parsing.
pub trait DelimiterHandler<'inp, L> {
  /// Called when a delimiter is encountered.
  fn on_open_delimiter(&mut self, open: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>;

  /// Called when a closing delimiter is encountered.
  fn on_close_delimiter(&mut self, close: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>;
}

impl<'inp, L, T> DelimiterHandler<'inp, L> for &mut T
where
  T: ?Sized + DelimiterHandler<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_open_delimiter(&mut self, open: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>,
  {
    (**self).on_open_delimiter(open);
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_close_delimiter(&mut self, close: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>,
  {
    (**self).on_close_delimiter(close);
  }
}

macro_rules! blackhole_delimiter_handler {
  ($ty:ty) => {
    impl<'inp, L> DelimiterHandler<'inp, L> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
      where
        L: Lexer<'inp>,
      {
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
      where
        L: Lexer<'inp>,
      {
      }
    }
  };
  (@generic $ty:ty) => {
    impl<'inp, L, T> DelimiterHandler<'inp, L> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
      where
        L: Lexer<'inp>,
      {
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
      where
        L: Lexer<'inp>,
      {
      }
    }
  };
}

blackhole_delimiter_handler!(());
blackhole_delimiter_handler!(@generic core::marker::PhantomData<T>);
blackhole_delimiter_handler!(@generic crate::utils::marker::Ignored<T>);

impl<'inp, L, T, N> DelimiterHandler<'inp, L> for GenericArrayDeque<T, N>
where
  N: ArrayLength,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
  where
    L: Lexer<'inp>,
  {
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
  where
    L: Lexer<'inp>,
  {
  }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
const _: () = {
  use std::{collections::vec_deque::VecDeque, vec::Vec};

  impl<'inp, L, T> DelimiterHandler<'inp, L> for Vec<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  impl<'inp, L, T> DelimiterHandler<'inp, L> for VecDeque<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  #[cfg(feature = "smallvec_1")]
  #[cfg_attr(docsrs, doc(cfg(feature = "smallvec_1")))]
  impl<'inp, L, T, N> DelimiterHandler<'inp, L> for smallvec_1::SmallVec<N>
  where
    N: smallvec_1::Array<Item = T>,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }
};

#[cfg(feature = "heapless_0_9")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless_0_9")))]
const _: () = {
  use heapless_0_9::{Deque, Vec};

  impl<'inp, L, T, const N: usize, LenT> DelimiterHandler<'inp, L> for Vec<T, N, LenT>
  where
    LenT: heapless_0_9::LenType,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  impl<'inp, L, T, const N: usize> DelimiterHandler<'inp, L> for Deque<T, N> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }
};

pub(super) trait EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized> {
  fn handle_start_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  fn handle_element_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  fn handle_leading_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    leading_sep: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  fn handle_separator_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    sep: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

pub(super) trait ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized> {
  fn handle_start_state(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    off: L::Offset,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_too_many_element(
    &self,
    _: usize,
    _: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    _: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(())
  }
}

pub(super) trait SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized> {
  fn handle_start_state(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    sep_tok: &Spanned<L::Token, L::Span>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

pub(super) trait RepeatedHandler<'inp, 'closure, O, L, Ctx, Lang: ?Sized> {
  fn on_element(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  fn on_stop(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}
