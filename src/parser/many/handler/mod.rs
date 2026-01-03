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
