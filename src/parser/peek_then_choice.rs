use core::mem::MaybeUninit;

use mayber::MaybeRef;

use crate::CachedToken;

use super::*;

/// a
pub struct PeekThenChoice<P, H, T, const N: usize> {
  parser: P,
  handler: H,
  _token: PhantomData<T>,
}

impl<P, H, T, const N: usize> PeekThenChoice<P, H, T, N> {
  /// Creates a new `PeekThenChoice` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<'inp, L, O, Ctx>(parser: P, condition: H) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseChoice<'inp, L, O, Ctx, ()>,
    H: FnMut(
      &[MaybeRef<'_, CachedToken<'_, L>>],
      &mut Ctx::Emitter,
    ) -> Result<P::Id, <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::of(parser, condition)
  }

  /// Creates a new `PeekThenChoice` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of<'inp, L, O, Ctx, Lang>(parser: P, condition: H) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseChoice<'inp, L, O, Ctx, Lang>,
    H: FnMut(
      &[MaybeRef<'_, CachedToken<'_, L>>],
      &mut Ctx::Emitter,
    ) -> Result<P::Id, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Lang: ?Sized,
  {
    Self {
      parser,
      handler: condition,
      _token: PhantomData,
    }
  }

  /// Creates a new `PeekThenChoice` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn or_not<'inp, L, O, Ctx>(parser: P, condition: H) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseChoice<'inp, L, O, Ctx, ()>,
    H: FnMut(
      &[MaybeRef<'_, CachedToken<'_, L>>],
      &mut Ctx::Emitter,
    ) -> Result<Option<P::Id>, <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::or_not_of(parser, condition)
  }

  /// Creates a new `PeekThenChoice` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn or_not_of<'inp, L, O, Ctx, Lang>(parser: P, condition: H) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseChoice<'inp, L, O, Ctx, Lang>,
    H: FnMut(
      &[MaybeRef<'_, CachedToken<'_, L>>],
      &mut Ctx::Emitter,
    ) -> Result<Option<P::Id>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Lang: ?Sized,
  {
    OrNot::new(Self {
      parser,
      handler: condition,
      _token: PhantomData,
    })
  }
}

impl<'inp, P, H, L, O, Ctx, Lang, const N: usize> ParseInput<'inp, L, O, Ctx, Lang>
  for PeekThenChoice<P, H, L::Token, N>
where
  P: ParseChoice<'inp, L, O, Ctx, Lang>,
  H: FnMut(
    &[MaybeRef<'_, CachedToken<'_, L>>],
    &mut Ctx::Emitter,
  ) -> Result<P::Id, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let mut buf = [const { MaybeUninit::uninit() }; N];
    let id = {
      let (output, emitter) = inp.peek_with_emitter(&mut buf);
      (self.handler)(output, emitter)?
    };
    self.parser.parse_choice(inp, &id)
  }
}

impl<'inp, P, H, L, O, Ctx, Lang, const N: usize> ParseInput<'inp, L, Option<O>, Ctx, Lang>
  for or_not::OrNot<PeekThenChoice<P, H, L::Token, N>>
where
  P: ParseChoice<'inp, L, O, Ctx, Lang>,
  H: FnMut(
    &[MaybeRef<'_, CachedToken<'_, L>>],
    &mut Ctx::Emitter,
  ) -> Result<Option<P::Id>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<Option<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let mut buf = [const { MaybeUninit::uninit() }; N];
    let id = {
      let (output, emitter) = inp.peek_with_emitter(&mut buf);

      if output.is_empty() {
        return Ok(None);
      }

      (self.0.handler)(output, emitter)?
    };
    match id {
      Some(id) => self.0.parser.parse_choice(inp, &id).map(Some),
      None => Ok(None),
    }
  }
}
