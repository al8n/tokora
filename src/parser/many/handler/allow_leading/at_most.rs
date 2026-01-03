use crate::{
  Emitter, Lexer, ParseContext,
  emitter::{SeparatedEmitter, TooManyEmitter, UnexpectedTrailingSeparatorEmitter},
  error::{syntax::MissingSyntaxOf, token::UnexpectedTrailingOf},
  input::{Checkpoint, InputRef},
  parser::{
    AllowLeading, Maximum,
    many::{ContinueStateHandler, EndStateHandler, SeparatorStateHandler},
  },
  span::{Span, Spanned},
};

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for AllowLeading<Maximum>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, Sep, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, Sep, L, Lang>
    + TooManyEmitter<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parser.check(inp, ckp, num_elems)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_element_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parser.check(inp, ckp, num_elems)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_leading_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    spanned: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    inp
      .emitter()
      .emit_missing_element(MissingSyntaxOf::<'_, L, Lang>::of(spanned.span_ref().end()))
      .and_then(|_| self.parser.check(inp, ckp, num_elems))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_separator_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    sep: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    // emit unexpected trailing separator
    let (span, tok) = sep.into_components();
    inp.emitter().emit_unexpected_trailing_separator(
      UnexpectedTrailingOf::<'_, Sep, L, Lang>::of(span).with_found(tok),
    )?;

    self.parser.check(inp, ckp, num_elems)
  }
}

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for AllowLeading<Maximum>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, Sep, L, Lang>,
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
  SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for AllowLeading<Maximum>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    _: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    _: &Spanned<L::Token, L::Span>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(())
  }
}
