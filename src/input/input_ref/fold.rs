use core::ops::ControlFlow;

use generic_arraydeque::typenum::Unsigned;

use super::*;

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Folds over the input tokens using the provided accumulator function.
  pub fn fold<O, Init, Op>(&mut self, init: Init, mut op: Op) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Init: FnOnce() -> O,
    Op: FnMut(O, Spanned<L::Token, L::Span>) -> ControlFlow<O, O>,
  {
    let mut output = init();

    loop {
      match self.next()? {
        Some(spanned) => match op(output, spanned) {
          ControlFlow::Continue(o) => output = o,
          ControlFlow::Break(o) => return Ok(o),
        },
        None => return Ok(output),
      }
    }
  }

  /// Folds n tokens over the input using the provided accumulator function.
  pub fn foldn<O, Init, Op>(&mut self, init: Init, mut op: Op, num: usize) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Init: FnOnce() -> O,
    Op: FnMut(O, Spanned<L::Token, L::Span>) -> O,
  {
    let mut output = init();

    let mut n = 0;
    
    loop {
      if n >= num {
        return Ok(output);
      }

      match self.next()? {
        Some(spanned) => {
          output = op(output, spanned);
          n += 1;
        },
        None => return Ok(output),
      }
    }
  }

  /// Right-folds over the input tokens using the provided accumulator function.
  /// 
  /// The maximum number of tokens folded is determined by the capacity of the specified `W`.
  /// 
  /// See also [`foldrn`](Self::foldrn).
  pub fn foldr_within<O, W, Init, Op>(&mut self, init: Init, mut op: Op) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Init: FnOnce() -> O,
    Op: FnMut(O, Spanned<L::Token, L::Span>) -> O,
    W: Window,
  {
    let mut output = init();
    let mut buf = GenericArrayDeque::<_, W::CAPACITY>::new();

    loop {
      if buf.len() >= <W::CAPACITY as Unsigned>::USIZE {
        break;
      }

      match self.next()? {
        Some(spanned) =>{
          buf.push_back(spanned);
        },
        None => return Ok(output),
      }
    }

    while let Some(spanned) = buf.pop_back() {
      output = op(output, spanned);
    }

    Ok(output)
  }

  /// Right-folds over the input tokens using the provided accumulator function.
  /// 
  /// This method folds up to `num` tokens, and this will lead to implicit allocation.
  /// 
  /// See also [`foldr_within`](Self::foldr_within).
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  pub fn foldrn<O, Init, Op>(&mut self, init: Init, mut op: Op, num: usize) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Init: FnOnce() -> O,
    Op: FnMut(O, Spanned<L::Token, L::Span>) -> O,
  {
    let mut output = init();
    let mut buf = std::vec::Vec::with_capacity(num);

    let mut n = 0;
    loop {
      if n >= num {
        break;
      }

      match self.next()? {
        Some(spanned) =>{
          buf.push(spanned);
          n += 1;
        },
        None => break,
      }
    }

    while let Some(spanned) = buf.pop() {
      output = op(output, spanned);
    }

    Ok(output)
  }
}
