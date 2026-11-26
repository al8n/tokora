//! The token-level Pratt driver, `InputRef::pratt`: one loop over the precedence machine.
//!
//! The machine keeps one frame for each operator that waits for its operand, and the loop keeps
//! those frames in segments of `K` on the native stack: the first in the expression's own native
//! frame, and each further one in a deeper native frame, made when the one below is full and left
//! once its own frames are gone. Under `stacker` each of those native frames asks for room before
//! it is entered, the expression's own included. The recursion budget is the machine's cap. The
//! expression holds one level of it for itself, as the recursive driver's root frame did, and each
//! pending operator one more.

use hybrid_arraydeque::{ArraySize, typenum::U4};

use crate::{
  ErrorOf,
  container::Stack,
  emitter::PrattEmitter,
  error::{
    NonAssociativeChain, RecursionLimitReached, UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot,
  },
  pratt::{
    PrattFoldTokenInfix, PrattFoldTokenPostfix, PrattFoldTokenPrefix, PrattFrame, PrattInfix,
    PrattLHS, PrattOffer, PrattPending, PrattPoll, PrattPower, PrattRHS, PrattReduction,
    PrattResolver, Precedenced, resolve::ResolverState, segment::Segment,
  },
  token::PrattToken,
};

use super::*;

#[cfg(test)]
mod oracle;

#[cfg(test)]
mod tests;

/// What the crate's own tests read of a parse's native frames: how many segments it grew, and
/// where on the native stack each sat. Kept on the thread, since the driver has no value of its
/// own to keep a count in. The recursive oracle leaves the same marks, one for each frame.
#[cfg(all(test, feature = "std"))]
mod probe {
  use core::cell::{Cell, RefCell};

  std::thread_local! {
    static FRAMES: Cell<usize> = const { Cell::new(0) };
    static MARKS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
  }

  /// A segment was made, or an oracle frame entered, at `mark`: the address of a local of the
  /// native frame it lives in.
  pub(super) fn grew(mark: usize) {
    FRAMES.with(|frames| frames.set(frames.get() + 1));
    MARKS.with(|marks| marks.borrow_mut().push(mark));
  }

  /// An oracle frame was entered, at `mark`.
  pub(super) fn entered(mark: usize) {
    grew(mark);
  }

  /// The native frames counted since the last `take`, and their marks, innermost last. Both are
  /// zeroed.
  pub(super) fn take() -> (usize, Vec<usize>) {
    (
      FRAMES.with(|frames| frames.replace(0)),
      MARKS.with(|marks| core::mem::take(&mut *marks.borrow_mut())),
    )
  }
}

/// A committed token with its span: the value of every sub-expression, and the payload of every
/// operator the machine holds.
type Tok<'inp, L> = Spanned<<L as Lexer<'inp>>::Token, <L as Lexer<'inp>>::Span>;

/// One pending operator.
type Frame<'inp, L, Power> =
  PrattFrame<Tok<'inp, L>, Tok<'inp, L>, Tok<'inp, L>, Tok<'inp, L>, Tok<'inp, L>, Power>;

/// The machine, over the storage `St`.
type Machine<'inp, L, Power, St> = PrattResolver<
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Power,
  St,
>;

/// Everything of the machine but its storage: what moves between native frames, by value.
type State<'inp, L, Power> = ResolverState<
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Power,
>;

/// One reduction the machine hands out.
type Reduction<'inp, L, Power> = PrattReduction<
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Tok<'inp, L>,
  Power,
>;

/// The table's classification of a token at an operator's position.
type Rhs<Power> = PrattRHS<(), (), (), (), Power>;

/// An operator the read's own offer took: the read commits its token.
enum Taken<Power> {
  Infix(PrattInfix<(), (), ()>, Power),
  Postfix(Power),
}

/// Why the read at an operator's position committed nothing: the far side of its `None`, which
/// alone cannot tell the endings apart. No offset rides in it: an offset is read off the input
/// itself, after the read, for the reason given where the errors are raised.
enum Parked<Power> {
  /// The read was handed no token: the end of input. Its closure never answers this; it is the
  /// answer while that closure has not run.
  End,
  /// The token is no operator of the table: it ends the expression, and stays staged.
  NoOperator,
  /// An operator the innermost sub-expression does not admit, below a pending one. Its
  /// classification stands while the reductions it waits for are performed.
  Reduce(Rhs<Power>),
  /// An operator the root does not admit: it is not this expression's.
  Decline,
  /// The operator repeats a non-associative chain.
  Chain,
  /// A zero-token continuation the machine would take, which this driver cannot serve: its
  /// termination rests on every operator being one committed token of nonzero width, and its infix
  /// fold is handed a real token, of which a zero-token operator has none. Diagnosed rather than
  /// declined, since ending the expression there would truncate it with an `Ok` and no diagnostic
  /// on any channel. The typed driver ([`Pratt`](crate::pratt::Pratt)) serves it.
  Adjacent,
}

/// What a loop does next. Kept here where the loop knows it, so that the machine is asked only for
/// a reduction it owes, or for the value.
enum Next<Power> {
  /// A term is wanted: a prefix operator or an operand is read.
  Term,
  /// An operator or an ending is wanted: the head is read, and offered.
  Operator,
  /// One reduction is owed, by `close` or by a postfix operator, and after it an operator.
  Reduce,
  /// The staged head waits for the reduction `close` owes, and its classification is offered
  /// again after it.
  Settle(Rhs<Power>),
  /// The staged head's classification, offered again with no read.
  Offer(Rhs<Power>),
  /// `finish` owes every pending reduction, one for each poll, and then the value.
  Finish,
}

/// How a loop over one segment ends without an error.
enum Exit<V, S, Power> {
  /// The expression's value, `None` where no term began it. Only the first segment's loop gets
  /// here: a segment is made only above a full one, so its loop returns before the count can reach
  /// zero.
  Done(Option<V>),
  /// The loop's segment is drained: the machine's state, and what the loop below does next, over
  /// its own segment.
  Drained(S, Next<Power>),
}

/// A push the full segment refused below the cap, fed again on the next one.
enum Refed<T, Power> {
  Prefix(Precedenced<T, Power>),
  Infix(Precedenced<PrattInfix<T, T, T>, Power>),
}

/// What a parse lends its loop, at every depth: the three folds, and the cap.
struct Folds<'r, FoldPrefix, FoldInfix, FoldPostfix> {
  prefix: &'r mut FoldPrefix,
  infix: &'r mut FoldInfix,
  postfix: &'r mut FoldPostfix,
  /// The machine's cap: the levels the recursion budget has left once the expression's own is
  /// taken.
  cap: usize,
}

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Runs a token-level Pratt expression parse over this input.
  ///
  /// This is the low-level, token-centric Pratt API. It requires the token type to implement
  /// [`PrattToken`], which classifies each token as an operand, prefix, infix, or postfix
  /// operator. The fold closures receive raw [`Spanned`] tokens rather than typed AST nodes.
  ///
  /// Equivalent to calling [`pratt_with_min_precedence`](Self::pratt_with_min_precedence) with
  /// `Power::default()` as the minimum binding power.
  ///
  /// For a more ergonomic higher-level API that works with any AST node type, prefer
  /// the [`pratt`](fn@crate::pratt::pratt) free function instead.
  ///
  /// # CST-unsupported
  ///
  /// This token-level API folds expressions into **synthetic tokens** — spans covering
  /// already-folded regions with no node-kind seam to classify — so it carries no CST hook
  /// in this version. A parse that should build a syntax tree uses the typed driver and
  /// its [`with_cst_kinds`](crate::pratt::Pratt::with_cst_kinds) classifier instead; the
  /// committed tokens this API consumes still auto-flow to a recording sink, but no
  /// expression *nodes* are recorded around them.
  ///
  /// # Parameters
  ///
  /// - `fold_prefix` – called with `(operator_tok, operand_tok, emitter)` when a prefix
  ///   operator and its operand have been successfully parsed.
  /// - `fold_infix` – called with `(lhs_tok, rhs_tok, operator_tok, emitter)` when an infix
  ///   operator and both operands have been parsed.
  /// - `fold_postfix` – called with `(operand_tok, operator_tok, emitter)` when a postfix
  ///   operator has been applied.
  ///
  /// # Returns
  ///
  /// `Ok(Some(tok))` with the combined expression token on success, `Ok(None)` if the
  /// input cursor did not see an LHS token, or `Err(e)` on a fatal emitter error.
  ///
  /// # Two failures that are not the emitter's
  ///
  /// Both are **returned**, never emitted, so no recording emitter and no rewind can turn either
  /// into a truncated-but-successful parse:
  ///
  /// - [`RecursionLimitReached`] — the parse's shared depth budget (see
  ///   [`descend`](Self::descend)) ran out. The expression holds one level of it, and each
  ///   operator waiting for its operand holds one more, so the operator that would hold one past
  ///   the limit is refused, right after its token is committed. Terminal, and terminal
  ///   independently of the grammar's error type: the trip latches the input session, so a grammar
  ///   whose error discards the value on conversion (`()` included) loses the payload and still
  ///   gets the re-raise — see [`RecursionLimitReached`]'s own docs for what a discarding sink
  ///   costs and does not cost.
  /// - [`NonAssociativeChain`] — a second same-power [`PrattInfix::Neither`] operator appeared in
  ///   one chain. The operator is left on the input, unconsumed. Not terminal: recovery may
  ///   spend it.
  ///
  /// # Depth, and the native stack
  ///
  /// The driver is one loop over the precedence machine, [`PrattResolver`], and does not recurse
  /// for each operator. It keeps the operators that wait for an operand four to a native frame:
  /// the first four in its own, and each further four in a deeper one, which it leaves once they
  /// are reduced. So the native stack a parse takes grows with how many operators it keeps
  /// pending at once, and not with the expression's length, and the driver allocates nothing of
  /// its own. With the `stacker` feature, its own frame asks for room on the native stack before
  /// it is entered, and so does each of those deeper frames.
  pub fn pratt<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power>(
    &mut self,
    fold_prefix: FoldPrefix,
    fold_infix: FoldInfix,
    fold_postfix: FoldPostfix,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L::Token: PrattToken<'inp, Expr, Power>,
    Ctx::Emitter: PrattEmitter<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<RecursionLimitReached<L::Offset, Lang>>
      + From<NonAssociativeChain<L::Offset, Lang>>,
    Power: PrattPower,
    FoldPrefix: PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang>,
    FoldInfix: PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang>,
    FoldPostfix: PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang>,
  {
    self.pratt_with_min_precedence(fold_prefix, fold_infix, fold_postfix, Power::default())
  }

  /// Runs a token-level Pratt expression parse over this input starting at a given minimum
  /// binding power.
  ///
  /// This is the low-level, token-centric Pratt API. It requires the token type to implement
  /// [`PrattToken`], which classifies each token as an operand, prefix, infix, or postfix
  /// operator. The fold closures receive raw [`Spanned`] tokens rather than typed AST nodes.
  ///
  /// Only operators whose binding power is **greater than or equal to** `min_precedence` will be
  /// consumed. Operators below the threshold are left in the input for the surrounding
  /// context to handle. This is useful when embedding a Pratt expression inside a larger
  /// grammar — for example, parsing only the right-hand side of an infix operator at a
  /// specific precedence level.
  ///
  /// Use [`pratt`](Self::pratt) instead when you want to parse a full expression starting
  /// from `Power::default()`.
  ///
  /// # Parameters
  ///
  /// - `fold_prefix` – called with `(operator_tok, operand_tok, emitter)` when a prefix
  ///   operator and its operand have been successfully parsed.
  /// - `fold_infix` – called with `(lhs_tok, rhs_tok, operator_tok, emitter)` when an infix
  ///   operator and both operands have been parsed.
  /// - `fold_postfix` – called with `(operand_tok, operator_tok, emitter)` when a postfix
  ///   operator has been applied.
  /// - `min_precedence` – the minimum binding power; operators strictly below this level are not
  ///   consumed.
  ///
  /// # Returns
  ///
  /// `Ok(Some(tok))` with the combined expression token on success, `Ok(None)` if the
  /// input cursor did not see an LHS token, or `Err(e)` on a fatal emitter error.
  ///
  /// Plus the two returned-not-emitted failures [`pratt`](Self::pratt) documents:
  /// [`RecursionLimitReached`] and [`NonAssociativeChain`]. What the parse holds on the native
  /// stack is documented there too.
  pub fn pratt_with_min_precedence<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power>(
    &mut self,
    mut fold_prefix: FoldPrefix,
    mut fold_infix: FoldInfix,
    mut fold_postfix: FoldPostfix,
    min_precedence: Power,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L::Token: PrattToken<'inp, Expr, Power>,
    Ctx::Emitter: PrattEmitter<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<RecursionLimitReached<L::Offset, Lang>>
      + From<NonAssociativeChain<L::Offset, Lang>>,
    Power: PrattPower,
    FoldPrefix: PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang>,
    FoldInfix: PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang>,
    FoldPostfix: PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang>,
  {
    self.pratt_in::<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, U4>(
      min_precedence,
      &mut fold_prefix,
      &mut fold_infix,
      &mut fold_postfix,
    )
  }

  /// The expression's own level, the first segment, and the loop, over segments of `K` frames.
  /// The public methods name four.
  #[inline(always)]
  fn pratt_in<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, K>(
    &mut self,
    min_precedence: Power,
    fold_prefix: &mut FoldPrefix,
    fold_infix: &mut FoldInfix,
    fold_postfix: &mut FoldPostfix,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L::Token: PrattToken<'inp, Expr, Power>,
    Ctx::Emitter: PrattEmitter<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<RecursionLimitReached<L::Offset, Lang>>
      + From<NonAssociativeChain<L::Offset, Lang>>,
    Power: PrattPower,
    FoldPrefix: PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang>,
    FoldInfix: PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang>,
    FoldPostfix: PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang>,
    K: ArraySize,
  {
    // The expression's own level, taken before its first read as the recursive driver's root
    // frame took it: a trip here has read nothing, and the guard gives the level back on every
    // exit, an unwind included.
    let mut level = self.descend()?;
    let this = &mut *level;
    // Each operator waiting for its operand held a level of its own, the frame its operand was
    // parsed in. So the levels left are what the machine may hold pending, and its cap; the
    // push past it is the budget's trip. After a descent that held, the depth is at most the
    // limit.
    let cap = this
      .recursion()
      .limitation()
      .saturating_sub(this.recursion().depth());
    // The expression's own native frame asks for room on the native stack before it is entered,
    // as the recursive driver's root frame asked at its prologue: a parse entered with little
    // stack left runs on a new one under `stacker`, however few operators it keeps pending. Each
    // deeper frame `pratt_grow` makes asks again for its own.
    crate::native_stack::maybe_grow(
      move || -> Result<Option<Tok<'inp, L>>, ErrorOf<'inp, L, Ctx, Lang>> {
        // Four frames in this native frame, written only as frames are pushed, and nothing to drop
        // at the end but their own count. The floor is the caller's, and no `Power` is built from
        // `Default` for it.
        let mut first: Segment<'_, Frame<'inp, L, Power>, K> = Segment::root();
        let machine: Machine<'inp, L, Power, &mut Segment<'_, Frame<'inp, L, Power>, K>> =
          PrattResolver::with_stack_and_floor(&mut first, min_precedence).max_depth(cap);
        let mut folds = Folds {
          prefix: fold_prefix,
          infix: fold_infix,
          postfix: fold_postfix,
          cap,
        };
        match Self::pratt_run::<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, _, false, K>(
          machine,
          Next::Term,
          this,
          &mut folds,
        )? {
          Exit::Done(value) => Ok(value),
          Exit::Drained(..) => {
            unreachable!("the first segment has nothing below it, so its loop never drains")
          }
        }
      },
    )
  }

  /// The loop, over one segment: the first, in the expression's own native frame, or one
  /// `pratt_grow` made over a full one. `UPPER` says which. It returns once its own frames are
  /// gone with frames below, and hands the frame below the machine's state and what to do next.
  #[inline]
  fn pratt_run<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, St, const UPPER: bool, K>(
    mut machine: Machine<'inp, L, Power, St>,
    mut next: Next<Power>,
    this: &mut Self,
    folds: &mut Folds<'_, FoldPrefix, FoldInfix, FoldPostfix>,
  ) -> Result<Exit<Tok<'inp, L>, State<'inp, L, Power>, Power>, ErrorOf<'inp, L, Ctx, Lang>>
  where
    L::Token: PrattToken<'inp, Expr, Power>,
    Ctx::Emitter: PrattEmitter<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<RecursionLimitReached<L::Offset, Lang>>
      + From<NonAssociativeChain<L::Offset, Lang>>,
    Power: PrattPower,
    FoldPrefix: PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang>,
    FoldInfix: PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang>,
    FoldPostfix: PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang>,
    St: Stack<Frame<'inp, L, Power>>,
    K: ArraySize,
  {
    // No progress guard, and none is needed. Every step commits a token, performs one reduction,
    // or ends. A committed token has nonzero width by the lexer's contract, and an operator is
    // accepted only by the read that commits it, so no position can be admitted twice. Each
    // reduction pops a frame, each frame was pushed for a committed operator, and no fold is
    // handed the input, so none can move it.
    loop {
      next = match next {
        // A term. A terminal scanner stop is surfaced, so a tripped limit cannot pass for an
        // absent operand.
        Next::Term => match this.try_expect_map_or_stop(|tok| tok.try_pratt_lhs())? {
          Some((PrattLHS::Operand(_), tok)) => {
            if let Err(refused) = machine.handle_operand(tok) {
              unreachable!(
                "the token-level driver's machine refused an operand where it wanted a term: {:?}",
                refused.why()
              );
            }
            Next::Operator
          }
          Some((PrattLHS::Prefix(prefix), tok)) => {
            match machine.push_prefix(Precedenced::new(tok, prefix.into_precedence())) {
              Ok(()) => Next::Term,
              Err(operator) => {
                let (seated, after) =
                  Self::pratt_grow::<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, St, K>(
                    machine,
                    Refed::Prefix(operator),
                    this,
                    folds,
                  )?;
                machine = seated;
                after
              }
            }
          }
          // No term. With no operator pending there is no expression here, and nothing was
          // read but the declined head. Otherwise the operator that waited for it says which
          // recovery posture this is, and in both the sub-expression it began ends here, as the
          // recursive driver's frame returned: its diagnostic first, at the end of the last lexed
          // token, and then the reduction below it.
          None => {
            match machine.retract() {
              None => return Ok(Exit::Done(None)),
              // A prefix operator whose operand never arrived stands for the expression it could
              // not build: under a recording emitter the parse goes on with the bare operator.
              // The typed driver is the stricter posture, with a node to withhold.
              Some(PrattPending::Prefix(operator)) => {
                this
                  .session
                  .emitter
                  .emit_unexpected_end_of_lhs(UnexpectedEoLhs::eolhs_of(this.offset().clone()))?;
                if let Err(refused) = machine.handle_operand(operator.into_data()) {
                  unreachable!(
                    "the token-level driver's machine refused a retracted operator as a term: {:?}",
                    refused.why()
                  );
                }
              }
              // An infix operator whose right operand never arrived is dropped with its missing
              // side, and its left operand is the value again.
              Some(PrattPending::Infix(_)) => {
                this
                  .session
                  .emitter
                  .emit_unexpected_end_of_rhs(UnexpectedEoRhs::eorhs_of(this.offset().clone()))?;
              }
            }
            close(&mut machine);
            // The return test, before the loop next reads its stack: `retract` popped a frame.
            if UPPER && machine.storage().is_empty() {
              return Ok(Exit::Drained(machine.into_parts().0, Next::Reduce));
            }
            Next::Reduce
          }
        },

        // An operator, or the ending. One read: its closure classifies, offers, and accepts only
        // a `Take` that is no zero-token operator, and accepting is the commit. What it did not
        // accept, it says why, and the token stays staged. A terminal scanner stop is surfaced
        // first, ahead of every ending and every refusal.
        Next::Operator => {
          let mut parked = Parked::End;
          let head = this.try_expect_map_or_stop(|tok| {
            let rhs = match tok.try_pratt_rhs() {
              Some(PrattRHS::End) | None => {
                parked = Parked::NoOperator;
                return None;
              }
              Some(rhs) => rhs,
            };
            match (machine.offer(&rhs), rhs) {
              (PrattOffer::Take, PrattRHS::Infix(infix)) => {
                let (associativity, power) = infix.into_components();
                Some(Taken::Infix(associativity, power))
              }
              (PrattOffer::Take, PrattRHS::Postfix(postfix)) => {
                Some(Taken::Postfix(postfix.into_precedence()))
              }
              // The floor admits it and it repeats no chain: the zero-token operator this driver
              // has no token for.
              (PrattOffer::Take, PrattRHS::Adjacent(_)) => {
                parked = Parked::Adjacent;
                None
              }
              (PrattOffer::Reduce, rhs) => {
                parked = Parked::Reduce(rhs);
                None
              }
              (PrattOffer::Decline, _) => {
                parked = Parked::Decline;
                None
              }
              (PrattOffer::Chain, _) => {
                parked = Parked::Chain;
                None
              }
              // `End` was answered above, and nothing admits it.
              (PrattOffer::Take, PrattRHS::End) => {
                parked = Parked::NoOperator;
                None
              }
            }
          })?;
          match head {
            Some((Taken::Infix(associativity, power), tok)) => {
              match machine.push_infix(Precedenced::new(infix_of(associativity, tok), power)) {
                Ok(()) => Next::Term,
                Err(operator) => {
                  let (seated, after) =
                    Self::pratt_grow::<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, St, K>(
                      machine,
                      Refed::Infix(operator),
                      this,
                      folds,
                    )?;
                  machine = seated;
                  after
                }
              }
            }
            Some((Taken::Postfix(power), tok)) => {
              machine.take_postfix(Precedenced::new(tok, power));
              Next::Reduce
            }
            None => match parked {
              // Nothing is staged at the end of input, and the recursive driver's every returning
              // frame scanned again there, re-lexing whatever lexer errors stand before the end.
              // So one sub-expression ends, and the head is read again after its reduction.
              Parked::End => {
                close(&mut machine);
                Next::Reduce
              }
              // A staged token that ends the expression, which no fold can move: every pending
              // reduction, and no read between them.
              Parked::NoOperator | Parked::Decline => {
                finish(&mut machine);
                Next::Finish
              }
              Parked::Reduce(rhs) => {
                close(&mut machine);
                Next::Settle(rhs)
              }
              // The offset is read off the input after the handback, not off the parked token:
              // `NonAssociativeChain`'s offset is the position the input is handed back at, and
              // `span().end()` is that position, since a declined token is not committed. The
              // token's own start differs wherever the lexer skipped anything to reach it.
              Parked::Chain => return Err(NonAssociativeChain::of(this.span().end()).into()),
              // The same posture as a missing right operand: a diagnostic on the RHS channel,
              // read at the handback, and the sub-expression ends with its left side. The
              // sub-expression around it is offered the same token again, after the reduction.
              Parked::Adjacent => {
                this
                  .session
                  .emitter
                  .emit_unexpected_end_of_rhs(UnexpectedEoRhs::eorhs_of(this.span().end()))?;
                close(&mut machine);
                Next::Reduce
              }
            },
          }
        }

        // The staged head's classification stands: no fold is handed the input, so nothing can
        // have moved it, and it is offered again with no read. Where the machine now takes it, one
        // more read takes it as it was classified.
        Next::Offer(rhs) => match machine.offer(&rhs) {
          PrattOffer::Reduce => {
            close(&mut machine);
            Next::Settle(rhs)
          }
          PrattOffer::Decline => {
            finish(&mut machine);
            Next::Finish
          }
          PrattOffer::Chain => return Err(NonAssociativeChain::of(this.span().end()).into()),
          PrattOffer::Take => {
            if let PrattRHS::Adjacent(_) = rhs {
              this
                .session
                .emitter
                .emit_unexpected_end_of_rhs(UnexpectedEoRhs::eorhs_of(this.span().end()))?;
              close(&mut machine);
              Next::Reduce
            } else {
              let tok = Self::pratt_take_staged::<Expr, Power>(this, &rhs)?;
              match rhs {
                PrattRHS::Infix(infix) => {
                  let (associativity, power) = infix.into_components();
                  match machine.push_infix(Precedenced::new(infix_of(associativity, tok), power)) {
                    Ok(()) => Next::Term,
                    Err(operator) => {
                      let (seated, after) =
                        Self::pratt_grow::<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, St, K>(
                          machine,
                          Refed::Infix(operator),
                          this,
                          folds,
                        )?;
                      machine = seated;
                      after
                    }
                  }
                }
                PrattRHS::Postfix(postfix) => {
                  machine.take_postfix(Precedenced::new(tok, postfix.into_precedence()));
                  Next::Reduce
                }
                PrattRHS::Adjacent(_) | PrattRHS::End => unreachable!(
                  "only an infix or a postfix classification stands to be taken from the front"
                ),
              }
            }
          }
        },

        // A reduction is owed, or the value. One reduction, folded by today's three folds with
        // today's arguments, and its value handed back.
        waiting @ (Next::Reduce | Next::Settle(_) | Next::Finish) => match machine.poll() {
          PrattPoll::Reduce(reduction) => {
            let value = Self::pratt_fold(reduction, this, folds)?;
            if let Err(refused) = machine.handle_reduced(value) {
              unreachable!(
                "the token-level driver's machine refused the value of its own reduction: {:?}",
                refused.why()
              );
            }
            let after = match waiting {
              Next::Reduce => Next::Operator,
              Next::Settle(rhs) => Next::Offer(rhs),
              finishing => finishing,
            };
            // The return test, before the loop next reads its stack: a segment whose own frames
            // are gone hands the machine to the loop below, which goes on over its own segment,
            // full, with the classification that stands if one does.
            if UPPER && machine.storage().is_empty() {
              return Ok(Exit::Drained(machine.into_parts().0, after));
            }
            after
          }
          PrattPoll::Done(value) => return Ok(Exit::Done(Some(value))),
          PrattPoll::Term | PrattPoll::Operator | PrattPoll::Reducing => unreachable!(
            "the token-level driver polls its machine only where a reduction or the value is owed"
          ),
        },
      };
    }
  }

  /// A push the machine refused, which is `Full`: the one refusal a push here can meet. At the
  /// cap it is the budget's trip, raised where the recursive driver's child frame raised it:
  /// right after the operator's token is committed, and before the next read. Below the cap this
  /// segment's own frames are in use: a new segment is made over it, in a deeper native frame, the
  /// refused push is fed again there and taken, and the loop goes on over the new segment until it
  /// drains. Then the machine is seated here again, over this segment as it was left, full, and
  /// handed back with what the loop does next.
  ///
  /// Never inlined, so that the new segment lives in a native frame of its own, and the loop holds
  /// one copy of this cold path. Under `stacker` it asks for room on the native stack first, once
  /// for each segment.
  #[inline(never)]
  fn pratt_grow<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, St, K>(
    machine: Machine<'inp, L, Power, St>,
    refed: Refed<Tok<'inp, L>, Power>,
    this: &mut Self,
    folds: &mut Folds<'_, FoldPrefix, FoldInfix, FoldPostfix>,
  ) -> Result<(Machine<'inp, L, Power, St>, Next<Power>), ErrorOf<'inp, L, Ctx, Lang>>
  where
    L::Token: PrattToken<'inp, Expr, Power>,
    Ctx::Emitter: PrattEmitter<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<RecursionLimitReached<L::Offset, Lang>>
      + From<NonAssociativeChain<L::Offset, Lang>>,
    Power: PrattPower,
    FoldPrefix: PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang>,
    FoldInfix: PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang>,
    FoldPostfix: PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang>,
    St: Stack<Frame<'inp, L, Power>>,
    K: ArraySize,
  {
    if machine.depth() >= folds.cap {
      return Err(this.trip(machine.depth()).into());
    }
    let (state, mut below) = machine.into_parts();
    let exit = crate::native_stack::maybe_grow(|| {
      let mut segment: Segment<'_, Frame<'inp, L, Power>, K> = Segment::over(&mut below);
      #[cfg(all(test, feature = "std"))]
      probe::grew(core::ptr::from_ref(&segment).addr());
      let mut upper: Machine<'inp, L, Power, &mut Segment<'_, Frame<'inp, L, Power>, K>> =
        PrattResolver::from_parts(state, &mut segment);
      let fed = match refed {
        Refed::Prefix(operator) => upper.push_prefix(operator).is_ok(),
        Refed::Infix(operator) => upper.push_infix(operator).is_ok(),
      };
      if !fed {
        unreachable!("an empty segment below the cap refused the push the full one refused");
      }
      Self::pratt_run::<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power, _, true, K>(
        upper,
        Next::Term,
        this,
        folds,
      )
    })?;
    let (state, next) = drained(exit);
    Ok((PrattResolver::from_parts(state, below), next))
  }

  /// One reduction, folded by the driver's three folds with the arguments they take.
  #[inline(always)]
  fn pratt_fold<FoldPrefix, FoldInfix, FoldPostfix, Power>(
    reduction: Reduction<'inp, L, Power>,
    this: &mut Self,
    folds: &mut Folds<'_, FoldPrefix, FoldInfix, FoldPostfix>,
  ) -> Result<Tok<'inp, L>, ErrorOf<'inp, L, Ctx, Lang>>
  where
    FoldPrefix: PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang>,
    FoldInfix: PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang>,
    FoldPostfix: PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang>,
  {
    match reduction {
      PrattReduction::Prefix(reduction) => {
        let (operand, operator) = reduction.into_components();
        folds
          .prefix
          .fold_prefix(operator.into_data(), operand, this.emitter_view())
      }
      PrattReduction::Infix(reduction) => {
        let (left, right, operator) = reduction.into_components();
        let infix = match operator.into_data() {
          PrattInfix::Left(token) => {
            let (span, token) = token.into_components();
            Spanned::new(span, PrattInfix::Left(token))
          }
          PrattInfix::Right(token) => {
            let (span, token) = token.into_components();
            Spanned::new(span, PrattInfix::Right(token))
          }
          PrattInfix::Neither(token) => {
            let (span, token) = token.into_components();
            Spanned::new(span, PrattInfix::Neither(token))
          }
        };
        folds
          .infix
          .fold_infix(left, right, infix, this.emitter_view())
      }
      PrattReduction::Postfix(reduction) => {
        let (operand, operator) = reduction.into_components();
        folds
          .postfix
          .fold_postfix(operand, operator.into_data(), this.emitter_view())
      }
    }
  }

  /// Takes the staged head as it was classified: one more read, whose closure accepts and
  /// classifies nothing. A debug build classifies it again, and compares.
  #[inline(always)]
  fn pratt_take_staged<Expr, Power>(
    this: &mut Self,
    rhs: &Rhs<Power>,
  ) -> Result<Tok<'inp, L>, ErrorOf<'inp, L, Ctx, Lang>>
  where
    L::Token: PrattToken<'inp, Expr, Power>,
    Power: PrattPower,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    let taken = this.try_expect_map_or_stop(|tok| {
      debug_assert!(
        same_classification(tok.try_pratt_rhs(), rhs),
        "the staged head is not the token its standing classification is of"
      );
      Some(())
    })?;
    match taken {
      Some(((), tok)) => Ok(tok),
      None => unreachable!(
        "the head a standing classification is of stays staged until the read that takes it"
      ),
    }
  }
}

/// The committed token, as the infix operator its classification names.
#[inline(always)]
fn infix_of<T>(associativity: PrattInfix<(), (), ()>, token: T) -> PrattInfix<T, T, T> {
  match associativity {
    PrattInfix::Left(()) => PrattInfix::Left(token),
    PrattInfix::Right(()) => PrattInfix::Right(token),
    PrattInfix::Neither(()) => PrattInfix::Neither(token),
  }
}

/// Whether a token read again is classified as the standing classification it is taken by.
fn same_classification<Power: PartialEq>(read: Option<Rhs<Power>>, standing: &Rhs<Power>) -> bool {
  match (read, standing) {
    (Some(PrattRHS::Infix(read)), PrattRHS::Infix(standing)) => read == *standing,
    (Some(PrattRHS::Postfix(read)), PrattRHS::Postfix(standing)) => read == *standing,
    _ => false,
  }
}

/// One sub-expression closed, in the `Operator` phase every caller is in.
#[inline(always)]
fn close<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, St>(
  machine: &mut PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, St>,
) where
  Power: PrattPower,
  St: Stack<PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>>,
{
  if let Err(why) = machine.close() {
    unreachable!(
      "the token-level driver's machine refused to close in its `Operator` phase: {why:?}"
    );
  }
}

/// The expression ended, in the `Operator` phase every caller is in.
#[inline(always)]
fn finish<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, St>(
  machine: &mut PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, St>,
) where
  Power: PrattPower,
  St: Stack<PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>>,
{
  if let Err(why) = machine.finish() {
    unreachable!(
      "the token-level driver's machine refused to finish in its `Operator` phase: {why:?}"
    );
  }
}

/// The state, and what to do next, that a drained segment handed back.
#[inline(always)]
fn drained<V, S, Power>(exit: Exit<V, S, Power>) -> (S, Next<Power>) {
  match exit {
    Exit::Drained(state, next) => (state, next),
    Exit::Done(_) => unreachable!(
      "a deeper frame never answers `Done`: a segment is made only above a full one, so its loop \
       returns before the count can reach zero"
    ),
  }
}
