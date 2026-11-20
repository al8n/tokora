//! The operator table, and the two parsers through which the typed driver reads it.
//!
//! The two channels are the typed side of the comparison the flat driver is tested by, so they
//! share no helper with the flat loop. What both are given is the token's `PrattToken`
//! implementation, `InputRef::try_expect_map_or_stop`, and the errors' constructors.

use core::{fmt, marker::PhantomData};

use crate::{
  Emitter, InputRef, Lexer, ParseAttempt, ParseContext, ParseInput, Token as _, TryParseInput,
  error::{UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot},
  span::{Span as _, Spanned},
  token::PrattToken,
};

use super::{PrattInfix, PrattLHS, PrattRHS, Precedenced};

/// Names an operator table: the [`PrattToken`] implementation a lexer's token has for the marker
/// `Expr`, at the binding power `Power`.
///
/// It holds no value of either, so it is `Copy` and `Default` whatever they are, and a grammar
/// writes its two type arguments once, in a constant. A second marker over the same token is a
/// second table.
pub struct PrattTable<Expr: ?Sized, Power = i64> {
  _table: PhantomData<fn() -> (*const Expr, Power)>,
}

impl<Expr: ?Sized, Power> PrattTable<Expr, Power> {
  /// The table.
  #[inline]
  pub const fn new() -> Self {
    Self {
      _table: PhantomData,
    }
  }

  /// The table as the typed driver's left-hand-side channel, with `operand` as its operand
  /// parser.
  #[inline]
  pub fn lhs<Operand>(self, operand: Operand) -> PrattTableLHS<Expr, Power, Operand> {
    PrattTableLHS {
      operand,
      _table: PhantomData,
    }
  }

  /// The table as the typed driver's right-hand-side channel.
  #[inline]
  pub fn rhs(self) -> PrattTableRHS<Expr, Power> {
    PrattTableRHS {
      _table: PhantomData,
    }
  }
}

impl<Expr: ?Sized, Power> Clone for PrattTable<Expr, Power> {
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<Expr: ?Sized, Power> Copy for PrattTable<Expr, Power> {}

impl<Expr: ?Sized, Power> Default for PrattTable<Expr, Power> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<Expr: ?Sized, Power> fmt::Debug for PrattTable<Expr, Power> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("PrattTable")
  }
}

/// The typed driver's left-hand-side channel over a table, made by [`PrattTable::lhs`].
///
/// It asks its operand parser first, and reports the operand that parser accepts. Where the parser
/// declines, it reads the head: a trivia token is committed and the operand parser is asked again,
/// and a prefix operator of the table is committed and reported, with the committed token as the
/// operator. Anything else, the end of input among it, is
/// [`UnexpectedEoLhs`](crate::error::UnexpectedEoLhs) at `span().end()`, with the token left on the
/// input.
pub struct PrattTableLHS<Expr: ?Sized, Power, Operand> {
  operand: Operand,
  _table: PhantomData<fn() -> (*const Expr, Power)>,
}

impl<Expr: ?Sized, Power, Operand: Clone> Clone for PrattTableLHS<Expr, Power, Operand> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      operand: self.operand.clone(),
      _table: PhantomData,
    }
  }
}

impl<Expr: ?Sized, Power, Operand: Copy> Copy for PrattTableLHS<Expr, Power, Operand> {}

impl<Expr: ?Sized, Power, Operand: fmt::Debug> fmt::Debug for PrattTableLHS<Expr, Power, Operand> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("PrattTableLHS")
      .field("operand", &self.operand)
      .finish()
  }
}

/// The typed driver's right-hand-side channel over a table, made by [`PrattTable::rhs`].
///
/// It reads the head: a trivia token is committed and the head read again, and an infix or a
/// postfix operator of the table is committed and reported, with the committed token as the
/// operator. Any other token, and the end of input, is [`PrattRHS::End`], with the token left on
/// the input.
///
/// A row that says [`Adjacent`](PrattRHS::Adjacent) is not served: a zero-token operator has no
/// token to report. It is [`UnexpectedEoRhs`](crate::error::UnexpectedEoRhs) at `span().end()`,
/// with the token left on the input, wherever the channel reads one, below the typed driver's
/// floor too: the channel classifies before the driver compares a power.
pub struct PrattTableRHS<Expr: ?Sized, Power> {
  _table: PhantomData<fn() -> (*const Expr, Power)>,
}

impl<Expr: ?Sized, Power> Clone for PrattTableRHS<Expr, Power> {
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<Expr: ?Sized, Power> Copy for PrattTableRHS<Expr, Power> {}

impl<Expr: ?Sized, Power> fmt::Debug for PrattTableRHS<Expr, Power> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("PrattTableRHS")
  }
}

/// What the left-hand-side channel's read found at the head.
enum LhsHead<Power> {
  /// A trivia token, committed.
  Trivia,
  /// A prefix operator of the table, committed, at its power.
  Prefix(Power),
}

/// What the right-hand-side channel's read found at the head.
enum RhsHead<Power> {
  /// A trivia token, committed.
  Trivia,
  /// An infix operator of the table, committed, with its associativity and power.
  Infix(PrattInfix<(), (), ()>, Power),
  /// A postfix operator of the table, committed, at its power.
  Postfix(Power),
}

// STAYS COMPLETE-ONLY, as the typed driver's own vocabulary impls are: the channels are read
// through `ParsePrattLHS` and `ParsePrattRHS`, whose blanket impls are written at `Complete`.
impl<'inp, Expr, Power, Operand, L, O, Ctx, Lang>
  ParseInput<'inp, L, PrattLHS<O, Spanned<L::Token, L::Span>, Power>, Ctx, Lang>
  for PrattTableLHS<Expr, Power, Operand>
where
  Expr: ?Sized,
  Lang: ?Sized,
  L: Lexer<'inp>,
  L::Token: PrattToken<'inp, Expr, Power>,
  Operand: TryParseInput<'inp, L, O, Ctx, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<UnexpectedEoLhs<L::Offset, Lang>>,
{
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    PrattLHS<O, Spanned<L::Token, L::Span>, Power>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    // The scanner's trip count as the call begins. The operand parser can meet a stop and still
    // accept or decline, and a rollback or a re-key inside it can take the stop off the input; the
    // trip counter, which neither reaches, still has it. The channel cannot know where the typed
    // driver will end, so each of its reports is a decision, and a trip since the call began makes
    // it the terminal stop instead. Its cost: a `u64` and the counter's address here, and one
    // nonce compare and one `u64` compare for each call.
    let scans = input.scanner_trip_snapshot();
    let reported = loop {
      if let ParseAttempt::Accept(operand) = self.operand.try_parse_input(input)? {
        break Ok(PrattLHS::Operand(operand));
      }
      let head = input.try_expect_map_or_stop(|tok| {
        if tok.is_trivia() {
          return Some(LhsHead::Trivia);
        }
        match tok.try_pratt_lhs() {
          Some(PrattLHS::Prefix(prefix)) => Some(LhsHead::Prefix(prefix.into_precedence())),
          Some(PrattLHS::Operand(())) | None => None,
        }
      })?;
      match head {
        Some((LhsHead::Trivia, _)) => {}
        Some((LhsHead::Prefix(power), operator)) => {
          break Ok(PrattLHS::Prefix(Precedenced::new(operator, power)));
        }
        None => {
          break Err(UnexpectedEoLhs::<L::Offset, Lang>::eolhs_of(input.span().end()).into());
        }
      }
    };
    if input.scanner_tripped_during_attempt(scans) {
      return Err(
        UnexpectedEot::<L::Offset, Lang>::eot_of(input.span().end())
          .into_terminal()
          .into(),
      );
    }
    reported
  }
}

impl<'inp, Expr, Power, L, Ctx, Lang>
  ParseInput<
    'inp,
    L,
    PrattRHS<
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Power,
    >,
    Ctx,
    Lang,
  > for PrattTableRHS<Expr, Power>
where
  Expr: ?Sized,
  Lang: ?Sized,
  L: Lexer<'inp>,
  L::Token: PrattToken<'inp, Expr, Power>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<UnexpectedEoRhs<L::Offset, Lang>>,
{
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    PrattRHS<
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Power,
    >,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    loop {
      let mut adjacent = false;
      let head = input.try_expect_map_or_stop(|tok| {
        if tok.is_trivia() {
          return Some(RhsHead::Trivia);
        }
        match tok.try_pratt_rhs()? {
          PrattRHS::Infix(infix) => {
            let (associativity, power) = infix.into_components();
            Some(RhsHead::Infix(associativity, power))
          }
          PrattRHS::Postfix(postfix) => Some(RhsHead::Postfix(postfix.into_precedence())),
          PrattRHS::Adjacent(_) => {
            adjacent = true;
            None
          }
          PrattRHS::End => None,
        }
      })?;
      match head {
        Some((RhsHead::Trivia, _)) => {}
        Some((RhsHead::Infix(associativity, power), operator)) => {
          let operator = match associativity {
            PrattInfix::Left(()) => PrattInfix::Left(operator),
            PrattInfix::Right(()) => PrattInfix::Right(operator),
            PrattInfix::Neither(()) => PrattInfix::Neither(operator),
          };
          return Ok(PrattRHS::Infix(Precedenced::new(operator, power)));
        }
        Some((RhsHead::Postfix(power), operator)) => {
          return Ok(PrattRHS::Postfix(Precedenced::new(operator, power)));
        }
        None if adjacent => {
          return Err(UnexpectedEoRhs::<L::Offset, Lang>::eorhs_of(input.span().end()).into());
        }
        None => return Ok(PrattRHS::End),
      }
    }
  }
}
