//! The flat expression driver: one loop over the machine, with no recursion of its own.
//!
//! Its loop reads `prefix* operand postfix* (infix prefix* operand postfix*)*`. Every head it reads,
//! it reads with `InputRef::try_expect_map_or_stop`, and the mode's admission runs inside that
//! read's closure, so a token is committed only once the mode has taken it, and a token that is
//! not taken is staged for the next reader. No head is lexed twice.

use core::marker::PhantomData;

use crate::{
  Emitter, InputRef, Lexer, ParseAttempt, ParseContext, Token as _, TryParseInput,
  cache::Cache as _,
  error::{NonAssociativeChain, PrattStackFull, UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot},
  input::{Completeness, SurfaceIncomplete},
  span::Span as _,
  token::PrattToken,
};

use super::{
  NoCst, PrattInfix, PrattLHS, PrattOffer, PrattPower, PrattRHS, PrattTable, Precedenced,
};

pub use handler::PrattHandler;
pub use mode::{Bare, Bounded, Checked, FlatMode, Open};
pub use seam::{FlatCst, WithFlatKind};

use mode::{ModeSteps, RunSteps};
use seam::Seam;

mod handler;
mod mode;
mod seam;

#[cfg(all(test, feature = "std"))]
mod tests;

/// Creates a flat expression driver over `table`, with `operand` as its operand parser.
///
/// The driver is one loop over the machine, with no recursion of its own, so it enters no level of
/// the input's recursion budget. A grammar that re-enters an expression, through its operand
/// parser or its handler, bounds that recursion itself.
#[inline]
pub fn flat<'inp, Expr, Power, Operand, L, O, Ctx, Lang>(
  table: PrattTable<Expr, Power>,
  operand: Operand,
) -> Flat<Expr, Power, Operand, L, O, Ctx, Lang>
where
  Expr: ?Sized,
  Lang: ?Sized,
  L: Lexer<'inp>,
  L::Token: PrattToken<'inp, Expr, Power>,
  Power: PrattPower,
  Operand: TryParseInput<'inp, L, O, Ctx, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  let _ = table;
  Flat {
    operand,
    cst: NoCst,
    mode: Checked::new(),
    _marker: PhantomData,
  }
}

/// A flat expression driver, made by [`flat`].
///
/// `Mode` is [`Checked`] until [`bare`](Self::bare) makes it [`Bare`], and `Cst` is [`NoCst`]
/// until [`with_cst_kind`](Self::with_cst_kind) makes it [`WithFlatKind`].
pub struct Flat<
  Expr: ?Sized,
  Power,
  Operand,
  L,
  O,
  Ctx,
  Lang: ?Sized = (),
  Cst = NoCst,
  Mode = Checked<Power>,
> {
  operand: Operand,
  cst: Cst,
  mode: Mode,
  _marker: PhantomData<fn() -> (*const Expr, Power, L, O, Ctx, *const Lang)>,
}

impl<Expr: ?Sized, Power, Operand, L, O, Ctx, Lang: ?Sized, Cst, B>
  Flat<Expr, Power, Operand, L, O, Ctx, Lang, Cst, Checked<Power, B>>
{
  /// Sets the minimum binding power, inclusive: an infix or postfix operator below it ends the
  /// expression, and is left on the input.
  ///
  /// The driver is then [`Bounded`], whatever the value, and has no [`bare`](Self::bare).
  #[must_use]
  #[inline]
  pub fn min_precedence(
    self,
    min_precedence: Power,
  ) -> Flat<Expr, Power, Operand, L, O, Ctx, Lang, Cst, Checked<Power, Bounded>> {
    Flat {
      operand: self.operand,
      cst: self.cst,
      mode: self.mode.with_floor(min_precedence),
      _marker: PhantomData,
    }
  }

  /// Caps the driver at `max_depth` pending operators: the operator past it is
  /// [`PrattStackFull`](crate::error::PrattStackFull), which reports the bound that refused.
  ///
  /// It is a cap and promises no room. Where the crate cannot allocate, the driver's stack holds 32
  /// pending operators and does not grow, and a cap above 32 does not raise that: the 33rd is
  /// refused, and the error reports 32.
  ///
  /// The driver is then [`Bounded`], whatever the value, and has no [`bare`](Self::bare).
  #[must_use]
  #[inline]
  pub fn max_depth(
    self,
    max_depth: usize,
  ) -> Flat<Expr, Power, Operand, L, O, Ctx, Lang, Cst, Checked<Power, Bounded>> {
    Flat {
      operand: self.operand,
      cst: self.cst,
      mode: self.mode.with_cap(max_depth),
      _marker: PhantomData,
    }
  }
}

impl<Expr: ?Sized, Power, Operand, L, O, Ctx, Lang: ?Sized, Cst>
  Flat<Expr, Power, Operand, L, O, Ctx, Lang, Cst, Checked<Power, Open>>
{
  /// Gives up the machine: [`parse_into`](Self::parse_into) then takes every operator the table
  /// names, and has no floor, no refusal of a non-associative chain and no cap.
  ///
  /// It exists only on a driver that has neither a floor nor a cap, so a chain of calls that sets
  /// a bound and then asks for no machine does not compile. With no bound set, it does:
  ///
  /// ```rust
  /// use tokora::{Lexer, ParseContext, TryParseInput, pratt::{Flat, PrattTable, flat}, token::PrattToken};
  ///
  /// struct Expr;
  ///
  /// fn driver<'inp, L, O, Ctx, Operand>(operand: Operand)
  /// where
  ///   L: Lexer<'inp>,
  ///   L::Token: PrattToken<'inp, Expr, u8>,
  ///   Operand: TryParseInput<'inp, L, O, Ctx>,
  ///   Ctx: ParseContext<'inp, L>,
  /// {
  ///   let driver: Flat<Expr, u8, Operand, L, O, Ctx> = flat(PrattTable::new(), operand);
  ///   let _bare = driver.bare();
  /// }
  /// ```
  ///
  /// After a floor, it does not:
  ///
  /// ```rust,compile_fail,E0599
  /// use tokora::{Lexer, ParseContext, TryParseInput, pratt::{Flat, PrattTable, flat}, token::PrattToken};
  ///
  /// struct Expr;
  ///
  /// fn driver<'inp, L, O, Ctx, Operand>(operand: Operand)
  /// where
  ///   L: Lexer<'inp>,
  ///   L::Token: PrattToken<'inp, Expr, u8>,
  ///   Operand: TryParseInput<'inp, L, O, Ctx>,
  ///   Ctx: ParseContext<'inp, L>,
  /// {
  ///   let driver: Flat<Expr, u8, Operand, L, O, Ctx> = flat(PrattTable::new(), operand);
  ///   let _bare = driver.min_precedence(1).bare();
  /// }
  /// ```
  ///
  /// Nor after a cap:
  ///
  /// ```rust,compile_fail,E0599
  /// use tokora::{Lexer, ParseContext, TryParseInput, pratt::{Flat, PrattTable, flat}, token::PrattToken};
  ///
  /// struct Expr;
  ///
  /// fn driver<'inp, L, O, Ctx, Operand>(operand: Operand)
  /// where
  ///   L: Lexer<'inp>,
  ///   L::Token: PrattToken<'inp, Expr, u8>,
  ///   Operand: TryParseInput<'inp, L, O, Ctx>,
  ///   Ctx: ParseContext<'inp, L>,
  /// {
  ///   let driver: Flat<Expr, u8, Operand, L, O, Ctx> = flat(PrattTable::new(), operand);
  ///   let _bare = driver.max_depth(8).bare();
  /// }
  /// ```
  ///
  /// The two setters themselves compile, and the mode's type records that a bound was asked for:
  /// `Checked<u8>` before, `Checked<u8, Bounded>` after.
  ///
  /// ```rust
  /// use tokora::{
  ///   Lexer, ParseContext, TryParseInput,
  ///   pratt::{Bounded, Checked, Flat, NoCst, PrattTable, flat},
  ///   token::PrattToken,
  /// };
  ///
  /// struct Expr;
  ///
  /// fn drivers<'inp, L, O, Ctx, Operand>(operand: Operand)
  /// where
  ///   L: Lexer<'inp>,
  ///   L::Token: PrattToken<'inp, Expr, u8>,
  ///   Operand: TryParseInput<'inp, L, O, Ctx>,
  ///   Ctx: ParseContext<'inp, L>,
  /// {
  ///   let open: Flat<Expr, u8, Operand, L, O, Ctx, (), NoCst, Checked<u8>> =
  ///     flat(PrattTable::new(), operand);
  ///   let floored: Flat<Expr, u8, Operand, L, O, Ctx, (), NoCst, Checked<u8, Bounded>> =
  ///     open.min_precedence(1);
  ///   let _capped: Flat<Expr, u8, Operand, L, O, Ctx, (), NoCst, Checked<u8, Bounded>> =
  ///     floored.max_depth(8);
  /// }
  /// ```
  #[must_use]
  #[inline]
  pub fn bare(self) -> Flat<Expr, Power, Operand, L, O, Ctx, Lang, Cst, Bare> {
    Flat {
      operand: self.operand,
      cst: self.cst,
      mode: Bare,
      _marker: PhantomData,
    }
  }
}

impl<Expr: ?Sized, Power, Operand, L, O, Ctx, Lang: ?Sized, Cst, Mode>
  Flat<Expr, Power, Operand, L, O, Ctx, Lang, Cst, Mode>
{
  /// Wraps every expression [`parse_into`](Self::parse_into) reads in one node of `kind`, a bare
  /// operand too. See [`WithFlatKind`] for where trivia lands.
  #[must_use]
  #[inline]
  pub fn with_cst_kind(
    self,
    kind: u16,
  ) -> Flat<Expr, Power, Operand, L, O, Ctx, Lang, WithFlatKind, Mode> {
    Flat {
      operand: self.operand,
      cst: WithFlatKind::new(kind),
      mode: self.mode,
      _marker: PhantomData,
    }
  }

  /// Parses one expression and tells `handler` each item in written order, resolving nothing.
  ///
  /// The operand parser is asked first at every term. Where it declines, the driver reads the
  /// head: a trivia token is committed and the operand parser asked again, and a prefix operator
  /// of the table is committed. After an operand, the head is read for an operator, and a trivia
  /// token there is committed and the head read again. The mode admits each operator the table
  /// names before it is committed, and an operator it does not take ends the expression, staged
  /// for the next reader.
  ///
  /// # Errors
  ///
  /// The driver returns its errors and emits none. Each offset is `span().end()` where the error is
  /// built.
  ///
  /// - [`UnexpectedEoLhs`](crate::error::UnexpectedEoLhs) where no term is found at the start or
  ///   after a prefix operator, and [`UnexpectedEoRhs`](crate::error::UnexpectedEoRhs) where none
  ///   is found directly after an infix operator. The token found instead is left on the input.
  /// - [`NonAssociativeChain`](crate::error::NonAssociativeChain), in [`Checked`] mode, where the
  ///   operator repeats a non-associative chain. It is left on the input.
  /// - [`PrattStackFull`](crate::error::PrattStackFull), in [`Checked`] mode, where an operator
  ///   finds the stack at its bound. It is terminal, and the trip is counted on the input session.
  /// - [`UnexpectedEoRhs`](crate::error::UnexpectedEoRhs) where the mode takes an
  ///   [`Adjacent`](super::PrattRHS::Adjacent) row: a zero-token operator has no token to tell.
  ///   The token is left on the input.
  /// - [`UnexpectedEot`](crate::error::UnexpectedEot) at a scanner stop, and whatever the operand
  ///   parser or the handler returns.
  ///
  /// What was consumed before an error is kept, and with a seam the expression's node is demoted:
  /// its tokens are the parent's.
  #[inline]
  pub fn parse_into<'inp, H, Cmpl>(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    handler: &mut H,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    L::Token: PrattToken<'inp, Expr, Power>,
    Power: PrattPower,
    Operand: TryParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
    H: PrattHandler<'inp, Power, L, O, Ctx, Lang, Cmpl>,
    Cst: FlatCst<'inp, L, Ctx, Lang, Cmpl>,
    Mode: FlatMode<Power>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: SurfaceIncomplete<'inp, L, Ctx, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<UnexpectedEoLhs<L::Offset, Lang>>
      + From<UnexpectedEoRhs<L::Offset, Lang>>
      + From<NonAssociativeChain<L::Offset, Lang>>
      + From<PrattStackFull<L::Offset, Lang>>,
  {
    let open = <Cst as Seam<'inp, L, Ctx, Lang, Cmpl>>::open(&self.cst, input)?;
    let mut run = <Mode as ModeSteps<Power>>::begin(&mut self.mode);
    let result = Self::parse_loop(&mut self.operand, &mut run, input, handler);
    run.end();
    match result {
      Ok(()) => {
        <Cst as Seam<'inp, L, Ctx, Lang, Cmpl>>::finish(&self.cst, open, input);
        Ok(())
      }
      Err(error) => {
        <Cst as Seam<'inp, L, Ctx, Lang, Cmpl>>::demote(&self.cst, open, input);
        Err(error)
      }
    }
  }

  /// The loop. Its place is its program counter: the outer loop is a term, and the inner one an
  /// operator, so it asks the mode for no phase.
  #[inline(always)]
  fn parse_loop<'inp, H, Cmpl, R>(
    operand: &mut Operand,
    run: &mut R,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    handler: &mut H,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    L::Token: PrattToken<'inp, Expr, Power>,
    Power: PrattPower,
    Operand: TryParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
    H: PrattHandler<'inp, Power, L, O, Ctx, Lang, Cmpl>,
    R: RunSteps<Power>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: SurfaceIncomplete<'inp, L, Ctx, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<UnexpectedEoLhs<L::Offset, Lang>>
      + From<UnexpectedEoRhs<L::Offset, Lang>>
      + From<NonAssociativeChain<L::Offset, Lang>>
      + From<PrattStackFull<L::Offset, Lang>>,
  {
    loop {
      // A term. The operand parser is asked first, and the head it inspects and declines is the
      // one the read below takes. A cache that can hold a token keeps whatever head the operand
      // parser's read lexed. One that can hold none keeps only a declined `try_expect`'s, and a
      // peek's is dropped with its window, so there the head is read first, and parked: the
      // operand parser then inspects it in place, by any read, and it is lexed once. A stop met
      // by that read is terminal, before the operand parser could take it for a decline.
      if input.cache().is_empty() && input.cache().remaining() == 0 && !input.ended_here() {
        input.try_expect_map_or_stop(|_| None::<()>)?;
      }
      if let ParseAttempt::Accept(value) = operand.try_parse_input(input)? {
        run.operand();
        handler.on_operand(input, value)?;
      } else {
        // A read that met the end of the input from here, after lexer errors, is not made again:
        // its answer stands, and a second read would lex the errors, and charge for them, again.
        // The missing operand is then reported where the input stands, the last committed offset.
        let head = if input.ended_here() {
          None
        } else {
          input.try_expect_map_or_stop(|tok| {
            if tok.is_trivia() {
              return Some(TermHead::Trivia);
            }
            match tok.try_pratt_lhs() {
              Some(PrattLHS::Prefix(prefix)) => Some(TermHead::Prefix(prefix.into_precedence())),
              Some(PrattLHS::Operand(())) | None => None,
            }
          })?
        };
        match head {
          // Committed: the operand parser is asked again.
          Some((TermHead::Trivia, _)) => {}
          Some((TermHead::Prefix(power), operator)) => {
            if let Err(bound) = run.prefix(&power) {
              return Err(stack_full(input, bound));
            }
            handler.on_prefix(input, Precedenced::new(operator, power))?;
          }
          // No term: the mode knows whether an infix operator waits for it.
          None => {
            let at = input.span().end();
            return Err(if run.missing() {
              UnexpectedEoRhs::<L::Offset, Lang>::eorhs_of(at).into()
            } else {
              UnexpectedEoLhs::<L::Offset, Lang>::eolhs_of(at).into()
            });
          }
        }
        continue;
      }

      // An operator, or the ending.
      loop {
        let mut ending = Ending::Here;
        let head = input.try_expect_map_or_stop(|tok| {
          if tok.is_trivia() {
            return Some(OperatorHead::Trivia);
          }
          let rhs = tok.try_pratt_rhs()?;
          if matches!(rhs, PrattRHS::End) {
            return None;
          }
          match (run.admit(&rhs), rhs) {
            (PrattOffer::Take, PrattRHS::Infix(infix)) => {
              let (associativity, power) = infix.into_components();
              Some(OperatorHead::Infix(associativity, power))
            }
            (PrattOffer::Take, PrattRHS::Postfix(postfix)) => {
              Some(OperatorHead::Postfix(postfix.into_precedence()))
            }
            (PrattOffer::Take, PrattRHS::Adjacent(_)) => {
              ending = Ending::Adjacent;
              None
            }
            (PrattOffer::Chain, _) => {
              ending = Ending::Chain;
              None
            }
            (PrattOffer::Decline, _) | (_, PrattRHS::End) => None,
            (PrattOffer::Reduce, _) => {
              unreachable!("a mode of the flat driver admitted an operator with `Reduce`")
            }
          }
        })?;
        match head {
          // Committed: the head is read again.
          Some((OperatorHead::Trivia, _)) => {}
          Some((OperatorHead::Infix(associativity, power), operator)) => {
            if let Err(bound) = run.infix(&associativity, &power) {
              return Err(stack_full(input, bound));
            }
            let operator = match associativity {
              PrattInfix::Left(()) => PrattInfix::Left(operator),
              PrattInfix::Right(()) => PrattInfix::Right(operator),
              PrattInfix::Neither(()) => PrattInfix::Neither(operator),
            };
            handler.on_infix(input, Precedenced::new(operator, power))?;
            break;
          }
          Some((OperatorHead::Postfix(power), operator)) => {
            run.postfix(&power);
            handler.on_postfix(input, Precedenced::new(operator, power))?;
          }
          // The token is staged, or there is none.
          None => {
            return match ending {
              Ending::Here => Ok(()),
              Ending::Chain => {
                Err(NonAssociativeChain::<L::Offset, Lang>::of(input.span().end()).into())
              }
              Ending::Adjacent => {
                Err(UnexpectedEoRhs::<L::Offset, Lang>::eorhs_of(input.span().end()).into())
              }
            };
          }
        }
      }
    }
  }
}

/// What the read at a term found, where the operand parser declined.
enum TermHead<Power> {
  /// A trivia token, committed.
  Trivia,
  /// A prefix operator of the table, committed, at its power.
  Prefix(Power),
}

/// What the read after an operand found, and the mode took.
enum OperatorHead<Power> {
  /// A trivia token, committed.
  Trivia,
  /// An infix operator, committed, with its associativity and power.
  Infix(PrattInfix<(), (), ()>, Power),
  /// A postfix operator, committed, at its power.
  Postfix(Power),
}

/// Why the read after an operand took nothing.
enum Ending {
  /// The expression ends here: the token is no operator of the table, the mode declined it, or
  /// the input is exhausted.
  Here,
  /// The operator repeats a non-associative chain.
  Chain,
  /// The mode took a zero-token operator, which has no token to tell.
  Adjacent,
}

/// A full stack. The trip is counted on the input session before the error is built, as a
/// descent's is, and the offset is the end of the operator that could not be stacked.
#[cold]
fn stack_full<'inp, L, Ctx, Lang, Cmpl>(
  input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  bound: usize,
) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  Cmpl: Completeness,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<PrattStackFull<L::Offset, Lang>>,
{
  input.count_resource_trip();
  PrattStackFull::<L::Offset, Lang>::of(input.span().end(), bound).into()
}
