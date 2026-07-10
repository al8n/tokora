use super::{super::many::Unbounded, *};
use crate::{
  emitter::{
    SeparatedEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  error::token::UnexpectedTokenOf,
  punct::Punctuator,
};

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for Unbounded
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang> + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    _: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    anchor: &Cursor<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(inp.span_since(anchor))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_element_state(
    &self,
    _: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    anchor: &Cursor<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(inp.span_since(anchor))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_leading_state(
    &self,
    _: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    anchor: &Cursor<'inp, 'closure, L>,
    _: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    // nothing to do, the unexpected leading separator should be handled by SeparatorStateHandler or ContinueStateHandler
    Ok(inp.span_since(anchor))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_separator_state(
    &self,
    _: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    anchor: &Cursor<'inp, 'closure, L>,
    sep: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    // emit unexpected trailing separator
    let (span, tok) = sep.into_components();
    inp.emitter().emit_unexpected_trailing_separator(
      Sep::name(),
      UnexpectedTokenOf::<'_, L, Lang>::of(span).with_found(tok),
    )?;

    Ok(inp.span_since(anchor))
  }
}

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for Unbounded
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang> + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    _: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    _: L::Offset,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(())
  }
}

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for Unbounded
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang> + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    sep_tok: &Spanned<L::Token, L::Span>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let (span, tok) = sep_tok.clone().into_components();
    inp.emitter().emit_unexpected_leading_separator(
      Sep::name(),
      UnexpectedTokenOf::<'_, L, Lang>::of(span).with_found(tok),
    )
  }
}

impl<'inp, 'closure, O, L, Ctx, Lang: ?Sized> RepeatedHandler<'inp, 'closure, O, L, Ctx, Lang>
  for Unbounded
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_element(
    &self,
    _: usize,
    _: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    _: &Cursor<'inp, 'closure, L>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_stop(
    &self,
    _: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    anchor: &Cursor<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(inp.span_since(anchor))
  }
}
