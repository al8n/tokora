//! What the flat driver's `parse_into` tells its caller.

use crate::{
  Emitter, InputRef, Lexer, ParseContext,
  input::{Complete, Completeness},
  span::Spanned,
};

use super::super::{PrattInfix, Precedenced};

/// What [`parse_into`](super::Flat::parse_into) tells its caller: each operator and each operand of
/// one expression, in written order, resolving nothing.
///
/// - **When each method is called.** `on_operand` once the operand parser has accepted, and the
///   three operator methods once the operator's token is committed and the mode has taken it. Each
///   item of the expression is told once.
/// - **What the input is, at each.** The committed frontier is the end of the item just told, so a
///   method may read on, and what it commits lands in the expression. The driver then reads the
///   next head afresh.
/// - **What it is not told.** A trivia token, and an operator the mode did not take. Those are
///   committed, or left on the input, by the driver.
/// - **An `Err` from any method ends the parse** with that error. What was consumed is kept, and
///   what a failed parse leaves in a handler is the handler's own to discard.
///
/// Implemented for `()`, which keeps nothing, and for `&mut H`.
pub trait PrattHandler<'inp, Power, L, O, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  /// A prefix operator, committed: its token, at its power.
  fn on_prefix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    operator: Precedenced<Spanned<L::Token, L::Span>, Power>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;

  /// An operand, as the operand parser accepted it.
  fn on_operand(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    operand: O,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;

  /// An infix operator, committed: its token in its associativity, at its power.
  fn on_infix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    operator: Precedenced<
      PrattInfix<
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
      >,
      Power,
    >,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;

  /// A postfix operator, committed: its token, at its power.
  fn on_postfix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    operator: Precedenced<Spanned<L::Token, L::Span>, Power>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;
}

/// Keeps nothing.
impl<'inp, Power, L, O, Ctx, Lang: ?Sized, Cmpl> PrattHandler<'inp, Power, L, O, Ctx, Lang, Cmpl>
  for ()
{
  #[inline(always)]
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    _operator: Precedenced<Spanned<L::Token, L::Span>, Power>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness,
  {
    Ok(())
  }

  #[inline(always)]
  fn on_operand(
    &mut self,
    _input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    _operand: O,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness,
  {
    Ok(())
  }

  #[inline(always)]
  fn on_infix(
    &mut self,
    _input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    _operator: Precedenced<
      PrattInfix<
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
      >,
      Power,
    >,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness,
  {
    Ok(())
  }

  #[inline(always)]
  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    _operator: Precedenced<Spanned<L::Token, L::Span>, Power>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness,
  {
    Ok(())
  }
}

/// Tells the handler it borrows.
impl<'inp, Power, L, O, Ctx, Lang: ?Sized, Cmpl, H> PrattHandler<'inp, Power, L, O, Ctx, Lang, Cmpl>
  for &mut H
where
  H: PrattHandler<'inp, Power, L, O, Ctx, Lang, Cmpl> + ?Sized,
{
  #[inline(always)]
  fn on_prefix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    operator: Precedenced<Spanned<L::Token, L::Span>, Power>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness,
  {
    (**self).on_prefix(input, operator)
  }

  #[inline(always)]
  fn on_operand(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    operand: O,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness,
  {
    (**self).on_operand(input, operand)
  }

  #[inline(always)]
  fn on_infix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    operator: Precedenced<
      PrattInfix<
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
      >,
      Power,
    >,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness,
  {
    (**self).on_infix(input, operator)
  }

  #[inline(always)]
  fn on_postfix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    operator: Precedenced<Spanned<L::Token, L::Span>, Power>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness,
  {
    (**self).on_postfix(input, operator)
  }
}
