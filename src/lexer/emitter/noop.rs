use crate::{
  error::token::UnexpectedToken,
  lexer::Span as _,
  utils::{Spanned, marker::Noop},
};

use super::*;

impl<'a, L, E> Emitter<'a, L> for Noop<E>
where
  L: Lexer<'a>,
  E: From<<L::Token as Token<'a>>::Error>
    + From<UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span>>,
{
  type Error = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    Spanned { span, data: err }: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>> {
    Err(Spanned {
      span,
      data: err.into(),
    })
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(
    &mut self,
    err: Spanned<Self::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>> {
    Err(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(
    &mut self,
    err: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span>,
  ) -> Result<(), Spanned<Self::Error, <L>::Span>>
  where
    L: Lexer<'a>,
  {
    Err(Spanned::new(err.span_ref().clone(), err.into()))
  }
}

impl<'a, O, L, E> RepeatedEmitter<'a, O, L> for Noop<E>
where
  O: ?Sized,
  L: Lexer<'a>,
  E: From<TooFew<O, L::Span>> + From<TooMany<O, L::Span>>,
  Noop<E>: Emitter<'a, L, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, err: TooFew<O, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Err(Spanned::new(err.span().clone(), err.into()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(&mut self, err: TooMany<O, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Err(Spanned::new(err.span().clone(), err.into()))
  }
}

impl<'a, L, Any, E> BatchEmitter<'a, L, Any> for Noop<E>
where
  L: Lexer<'a>,
  E: From<Any>,
  Noop<E>: Emitter<'a, L, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch(&mut self, _: <L>::Span, _: Message)
  where
    L: Lexer<'a>,
  {
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch_with_error(
    &mut self,
    _: Message,
    err: Spanned<Any, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Err(err.map_data(Into::into))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_to_batch(
    &mut self,
    _: &<L>::Span,
    err: Spanned<Any, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Err(err.map_data(Into::into))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_batch(&mut self, _: &<L>::Span) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn drop_batch(&mut self, _: &<L>::Span)
  where
    L: Lexer<'a>,
  {
  }
}

impl<'inp, L, O, Sep, E> SeparatedByEmitter<'inp, O, Sep, L> for Noop<E>
where
  L: Lexer<'inp>,
  E: From<MissingTokenOf<'inp, Sep, L>>
    + From<MissingSyntaxOf<'inp, O, L>>
    + From<MissingLeadingOf<'inp, Sep, L>>
    + From<MissingTrailingOf<'inp, Sep, L>>
    + From<UnexpectedLeadingOf<'inp, Sep, L>>
    + From<UnexpectedTrailingOf<'inp, Sep, L>>
    + From<UnexpectedRepeatedOf<'inp, Sep, L>>
    + From<TooFew<O, L::Span>>
    + From<TooMany<O, L::Span>>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>
    + From<<L::Token as Token<'inp>>::Error>,
  Noop<E>: Emitter<'inp, L, Error = E>
    + BatchEmitter<'inp, L, UnexpectedLeadingOf<'inp, Sep, L>>
    + BatchEmitter<'inp, L, UnexpectedTrailingOf<'inp, Sep, L>>
    + BatchEmitter<'inp, L, UnexpectedRepeatedOf<'inp, Sep, L>>
    + BatchEmitter<'inp, L, <L::Token as Token<'inp>>::Error>
    + RepeatedEmitter<'inp, O, L, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    err: MissingTokenOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    let offset = err.offset_ref().clone();
    Err(Spanned::new(
      L::Span::new(offset.clone(), offset),
      err.into(),
    ))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    let offset = err.offset_ref().clone();
    Err(Spanned::new(
      L::Span::new(offset.clone(), offset),
      err.into(),
    ))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_leading_separator(
    &mut self,
    err: MissingLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    let offset = err.offset_ref().clone();
    Err(Spanned::new(
      L::Span::new(offset.clone(), offset),
      err.into(),
    ))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_trailing_separator(
    &mut self,
    err: MissingTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    let offset = err.offset_ref().clone();
    Err(Spanned::new(
      L::Span::new(offset.clone(), offset),
      err.into(),
    ))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_repeated_separator(
    &mut self,
    err: UnexpectedRepeatedOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Err(Spanned::new(err.span_ref().clone(), err.into()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Err(Spanned::new(err.span_ref().clone(), err.into()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Err(Spanned::new(err.span_ref().clone(), err.into()))
  }
}

#[cfg(test)]
const _: () = {
  use crate::{BlackHole, Check, DummyLexer, DummyToken, SeqSepAction};

  struct DummySep;

  impl<'inp> Check<DummyToken, SeqSepAction<'inp, DummyToken>> for DummySep {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn check(&self, _: &DummyToken) -> SeqSepAction<'inp, DummyToken> {
      unimplemented!()
    }
  }

  const fn assert_noop_batch_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: BatchEmitter<'a, L, Any, Error = Error>,
  {
  }

  const fn assert_noop_repeated_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: RepeatedEmitter<'a, Any, L, Error = Error>,
  {
  }

  const fn assert_noop_separated_by_emitter<'a, L, O, Sep, Error, E>()
  where
    L: Lexer<'a>,
    E: SeparatedByEmitter<'a, O, Sep, L, Error = Error>,
  {
  }

  assert_noop_batch_emitter::<'_, DummyLexer, (), (), Noop<()>>();
  assert_noop_batch_emitter::<'_, DummyLexer, (), BlackHole, Noop<BlackHole>>();

  assert_noop_repeated_emitter::<'_, DummyLexer, (), (), Noop<()>>();

  assert_noop_separated_by_emitter::<'_, DummyLexer, (), DummySep, (), Noop<()>>();
};
