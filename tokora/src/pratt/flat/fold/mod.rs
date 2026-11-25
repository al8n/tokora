//! `folding`: the flat driver's adapter that resolves precedence as it parses, and folds into the
//! caller's value with the typed driver's three fold traits.
//!
//! It runs the flat loop over a machine of the caller's values, and folds each reduction the
//! machine hands out, one for each poll, with the operator that asked for it left on the input. A
//! classification of the head stands across a fold while the input has not moved: offered again,
//! it reads nothing, and a token is committed only on the answer of a fresh read's own offer. After
//! a fold that moved the input, the head is read again.
//!
//! Where the crate cannot allocate, a default storage grows on the native stack, a segment at a
//! time. A loop that finds its storage full below the cap goes on in a deeper native frame over a
//! new segment, and returns to the frame below once that segment's own frames are gone, before it
//! next reads its stack. That path is compiled there and in the crate's own tests, so a build with
//! an allocator has one copy of the loop.

use core::marker::PhantomData;

use hybrid_arraydeque::{ArraySize, typenum::U4};

use crate::{
  Emitter, InputRef, Lexer, ParseAttempt, ParseContext, ParseInput, Token as _, TryParseInput,
  container::{Container as _, Stack},
  error::{NonAssociativeChain, PrattStackFull, UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot},
  input::{Complete, ScannerTripBaseline, same_regime},
  span::{Span as _, Spanned},
  token::PrattToken,
};

#[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
use super::super::segment::Segment;
use super::{
  super::{
    DefaultPrattStack, PrattFoldInfix, PrattFoldPostfix, PrattFoldPrefix, PrattFrame, PrattInfix,
    PrattLHS, PrattOffer, PrattPending, PrattPoll, PrattPower, PrattRHS, PrattReduction,
    PrattRefusal, PrattResolver, PrattStack, Precedenced, resolve::ResolverState,
  },
  Checked, Flat, FlatCst, OperatorHead, TermHead,
  seam::Seam,
  stack_full, terminal_stop,
};

#[cfg(test)]
mod tests;

/// Where the crate cannot allocate, the cap of a storage that grows when none is set: the shipped
/// recursion budget, so that such a build refuses no depth the typed driver accepts by default.
#[cfg(not(any(feature = "std", feature = "alloc")))]
const GROWN_CAP: usize = 32;

/// An operator a fold is handed: its committed token, with the token's span.
type Tk<'inp, L> = Spanned<<L as Lexer<'inp>>::Token, <L as Lexer<'inp>>::Span>;

/// One pending operator of the machine.
type Frame<'inp, L, O, Power> =
  PrattFrame<O, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, Power>;

/// The machine, over the storage `St`.
type Machine<'inp, L, O, Power, St> =
  PrattResolver<O, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, Power, St>;

/// Everything of the machine but its storage.
type State<'inp, L, O, Power> =
  ResolverState<O, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, Power>;

impl<Expr: ?Sized, Power, Operand, L, O, Ctx, Lang: ?Sized, Cst, B>
  Flat<Expr, Power, Operand, L, O, Ctx, Lang, Cst, Checked<Power, B>>
{
  /// Resolves precedence while parsing, and folds into `O`: the non-recursive counterpart of
  /// [`pratt`](super::super::pratt()), with its three fold traits.
  ///
  /// Each fold is handed the operator's committed token, with its span, so a grammar whose typed
  /// classifiers hand out `Spanned<Token, Span>` passes the same three folds to both drivers. The
  /// folds run in the typed driver's order, and each runs with the operator that asked for it left
  /// on the input. The floor, the cap and the seam set on this driver carry over.
  ///
  /// The machine's storage is a field of the driver, emptied when a parse begins and when one
  /// ends, so a driver that is kept reuses it. By default it holds four pending operators. Past
  /// them, it grows on the heap where the crate can allocate. Where it cannot, it grows on the
  /// native stack four at a time, up to [`max_depth`](Self::max_depth), or 32 when none is set;
  /// the operator past that is [`PrattStackFull`](crate::error::PrattStackFull).
  /// [`with_stack`](FlatFold::with_stack) supplies a storage that is not grown.
  #[must_use]
  #[inline]
  pub fn folding<'inp, FoldPrefix, FoldInfix, FoldPostfix>(
    self,
    fold_prefix: FoldPrefix,
    fold_infix: FoldInfix,
    fold_postfix: FoldPostfix,
  ) -> FlatFold<
    Expr,
    Power,
    Operand,
    FoldPrefix,
    FoldInfix,
    FoldPostfix,
    L,
    O,
    Ctx,
    Lang,
    Cst,
    DefaultPrattStack<
      PrattFrame<
        O,
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
        Spanned<L::Token, L::Span>,
        Power,
      >,
    >,
  >
  where
    L: Lexer<'inp>,
  {
    let (floor, cap) = self.mode.into_bounds();
    FlatFold {
      operand: self.operand,
      fold_prefix,
      fold_infix,
      fold_postfix,
      cst: self.cst,
      floor,
      cap,
      stack: PrattStack::new(),
      #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
      grows: true,
      #[cfg(test)]
      growth: Growth::default(),
      _marker: PhantomData,
    }
  }
}

/// A flat driver that resolves precedence as it parses, and folds into `O`: made by
/// [`folding`](super::Flat::folding).
///
/// `S` is the machine's storage, which the driver owns and lends to the machine for every parse.
/// It is emptied when a parse begins and when one ends.
pub struct FlatFold<
  Expr: ?Sized,
  Power,
  Operand,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  L,
  O,
  Ctx,
  Lang: ?Sized,
  Cst,
  S,
> {
  operand: Operand,
  fold_prefix: FoldPrefix,
  fold_infix: FoldInfix,
  fold_postfix: FoldPostfix,
  cst: Cst,
  /// The floor, inclusive.
  floor: Power,
  /// The cap on pending operators, once one is set.
  cap: Option<usize>,
  /// The machine's storage.
  stack: S,
  /// Whether the storage is the default, which grows on the native stack where the crate cannot
  /// allocate: `folding` sets it, and `with_stack` clears it.
  #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
  grows: bool,
  /// What the parses so far did on the native stack.
  #[cfg(test)]
  growth: Growth,
  _marker: PhantomData<fn() -> (*const Expr, L, O, Ctx, *const Lang)>,
}

impl<
  Expr: ?Sized,
  Power,
  Operand,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  L,
  O,
  Ctx,
  Lang: ?Sized,
  Cst,
  S,
> FlatFold<Expr, Power, Operand, FoldPrefix, FoldInfix, FoldPostfix, L, O, Ctx, Lang, Cst, S>
{
  /// Replaces the machine's storage. The driver owns it and lends it to the machine for every
  /// parse, emptying it when a parse begins and when one ends.
  ///
  /// It is not grown, in any tier: an operator it has no room for is
  /// [`PrattStackFull`](crate::error::PrattStackFull), which reports the lesser of its capacity
  /// and the cap.
  #[must_use]
  #[inline]
  pub fn with_stack<S2>(
    self,
    stack: S2,
  ) -> FlatFold<Expr, Power, Operand, FoldPrefix, FoldInfix, FoldPostfix, L, O, Ctx, Lang, Cst, S2>
  {
    FlatFold {
      operand: self.operand,
      fold_prefix: self.fold_prefix,
      fold_infix: self.fold_infix,
      fold_postfix: self.fold_postfix,
      cst: self.cst,
      floor: self.floor,
      cap: self.cap,
      stack,
      #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
      grows: false,
      #[cfg(test)]
      growth: self.growth,
      _marker: PhantomData,
    }
  }
}

impl<'inp, Expr, Power, Operand, FoldPrefix, FoldInfix, FoldPostfix, L, O, Ctx, Lang, Cst, S>
  ParseInput<'inp, L, O, Ctx, Lang>
  for FlatFold<Expr, Power, Operand, FoldPrefix, FoldInfix, FoldPostfix, L, O, Ctx, Lang, Cst, S>
where
  Expr: ?Sized,
  Lang: ?Sized,
  L: Lexer<'inp>,
  L::Token: PrattToken<'inp, Expr, Power>,
  Power: PrattPower,
  Operand: TryParseInput<'inp, L, O, Ctx, Lang>,
  FoldPrefix: PrattFoldPrefix<'inp, Power, Spanned<L::Token, L::Span>, L, O, Ctx, Lang>,
  FoldInfix: PrattFoldInfix<
      'inp,
      Power,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      L,
      O,
      Ctx,
      Lang,
    >,
  FoldPostfix: PrattFoldPostfix<'inp, Power, Spanned<L::Token, L::Span>, L, O, Ctx, Lang>,
  Cst: FlatCst<'inp, L, Ctx, Lang>,
  S: Stack<
    PrattFrame<
      O,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Spanned<L::Token, L::Span>,
      Power,
    >,
  >,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
    + From<UnexpectedEoLhs<L::Offset, Lang>>
    + From<UnexpectedEoRhs<L::Offset, Lang>>
    + From<NonAssociativeChain<L::Offset, Lang>>
    + From<PrattStackFull<L::Offset, Lang>>,
{
  /// Parses one expression, and folds it.
  ///
  /// The operand parser is asked first at every term, and where it declines the head is read:
  /// a trivia token is committed and the operand parser asked again, and a prefix operator of the
  /// table is committed. After an operand the head is read for an operator, and trivia there is
  /// committed and the head read again. An operator the machine takes is committed. One it does
  /// not take is left on the input while the reductions it asks for are folded, and is then
  /// offered again. An operator below the floor, a token that is no operator, and the end of input
  /// end the expression: what is pending is folded, and the token is left on the input.
  ///
  /// # Errors
  ///
  /// The driver returns its errors and emits none. Each offset is `span().end()` where the error
  /// is built.
  ///
  /// - [`UnexpectedEoLhs`](crate::error::UnexpectedEoLhs) where no term is found at the start or
  ///   after a prefix operator, and [`UnexpectedEoRhs`](crate::error::UnexpectedEoRhs) where none
  ///   is found directly after an infix operator. The token found instead is left on the input.
  /// - [`NonAssociativeChain`](crate::error::NonAssociativeChain) where the operator repeats a
  ///   non-associative chain, and [`UnexpectedEoRhs`](crate::error::UnexpectedEoRhs) where the
  ///   machine would take an [`Adjacent`](super::super::PrattRHS::Adjacent) row, which has no
  ///   token to fold. The token is left on the input.
  /// - [`PrattStackFull`](crate::error::PrattStackFull) where an operator finds the stack at its
  ///   bound. It is terminal, and the trip is counted on the input session.
  /// - [`UnexpectedEot`](crate::error::UnexpectedEot) at a scanner stop, and whatever the operand
  ///   parser or a fold returns.
  ///
  /// What was consumed before an error is kept, and with a seam the expression's node is demoted.
  /// Nothing is restored on an unwind out of the operand parser or a fold either: what was
  /// consumed and emitted before it stays.
  #[inline]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parse_at::<U4>(input)
  }
}

/// The classification of a head the loop left on the input. It stands while the input has not
/// moved, since the head is then the same token by the lexer's determinism.
enum Standing<Power> {
  /// No operator of the table, `End`, or the end of input: the expression ends there.
  Ending,
  /// An operator of the table, which waits for the reductions it asked for.
  Operator(PrattRHS<(), (), (), (), Power>),
}

/// What a fresh read decided where it committed nothing.
enum Answer<Power> {
  /// No operator of the table, `End`, or the end of input.
  Ending,
  /// The operator waits for a reduction.
  Reduce(PrattRHS<(), (), (), (), Power>),
  /// The operator is not this expression's.
  Decline,
  /// The operator repeats a non-associative chain.
  Chain,
  /// The machine would take a zero-token operator.
  Adjacent,
}

/// What a classification that stands asks of the loop.
enum Stood {
  /// Nothing stands: the head is read.
  Read,
  /// A reduction is owed, or the expression is over: one sub-expression is closed.
  Close,
  /// The operator repeats a non-associative chain.
  Chain,
  /// The machine would take a zero-token operator.
  Adjacent,
}

/// How a loop over one storage ends without an error.
//
// With an allocator, outside tests, no loop runs over a segment, so nothing reads what a drained
// one hands back.
#[cfg_attr(
  not(any(test, not(any(feature = "std", feature = "alloc")))),
  allow(dead_code)
)]
enum Exit<V, State, Power> {
  /// The expression's value. Only the loop over the driver's own storage gets here: a segment
  /// exists only above full storage, so its loop returns before the count can reach zero.
  Done(V),
  /// The loop's segment is drained: the machine's state, and the classification that stands, for
  /// the loop below to go on with on its own segment.
  Drained(State, Option<Standing<Power>>),
}

/// A push a full storage refused below the cap, fed again on the next segment.
#[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
enum Refed<Tk, Power> {
  Prefix(Precedenced<Tk, Power>),
  Infix(Precedenced<PrattInfix<Tk, Tk, Tk>, Power>),
}

/// What a parse lends its loop, at every depth.
struct Steps<'r, Operand, FoldPrefix, FoldInfix, FoldPostfix, K> {
  operand: &'r mut Operand,
  fold_prefix: &'r mut FoldPrefix,
  fold_infix: &'r mut FoldInfix,
  fold_postfix: &'r mut FoldPostfix,
  /// The cap the parse runs under.
  cap: usize,
  /// Whether a storage that refuses below the cap grows.
  #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
  grows: bool,
  #[cfg(test)]
  growth: &'r mut Growth,
  /// The size of a segment, which only the growth path reads.
  _segment: PhantomData<fn() -> K>,
}

/// What the parses of one driver did on the native stack.
#[cfg(test)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Growth {
  /// Segments made.
  grown: usize,
  /// Returns to the frame below.
  returned: usize,
  /// Segments above the driver's own storage now, counted from zero when a parse begins.
  above: usize,
  /// The most segments above the driver's own storage at once.
  deepest: usize,
}

impl<'inp, Expr, Power, Operand, FoldPrefix, FoldInfix, FoldPostfix, L, O, Ctx, Lang, Cst, S>
  FlatFold<Expr, Power, Operand, FoldPrefix, FoldInfix, FoldPostfix, L, O, Ctx, Lang, Cst, S>
where
  Expr: ?Sized,
  Lang: ?Sized,
  L: Lexer<'inp>,
  L::Token: PrattToken<'inp, Expr, Power>,
  Power: PrattPower,
  Operand: TryParseInput<'inp, L, O, Ctx, Lang>,
  FoldPrefix: PrattFoldPrefix<'inp, Power, Tk<'inp, L>, L, O, Ctx, Lang>,
  FoldInfix: PrattFoldInfix<'inp, Power, Tk<'inp, L>, Tk<'inp, L>, Tk<'inp, L>, L, O, Ctx, Lang>,
  FoldPostfix: PrattFoldPostfix<'inp, Power, Tk<'inp, L>, L, O, Ctx, Lang>,
  Cst: FlatCst<'inp, L, Ctx, Lang>,
  S: Stack<Frame<'inp, L, O, Power>>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
    + From<UnexpectedEoLhs<L::Offset, Lang>>
    + From<UnexpectedEoRhs<L::Offset, Lang>>
    + From<NonAssociativeChain<L::Offset, Lang>>
    + From<PrattStackFull<L::Offset, Lang>>,
{
  /// A parse, whose storage grows by segments of `K` frames where it grows at all. The public
  /// path names four.
  #[inline]
  fn parse_at<K: ArraySize>(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let open = <Cst as Seam<'inp, L, Ctx, Lang, Complete>>::open(&self.cst, input)?;
    let result = self.resolve::<K>(input);
    // By its own length, as `with_stack` empties it when the next parse begins: a parse that grew
    // and failed left the driver's own frames here, and the machine's count went with the native
    // frames that carried it.
    for _ in 0..self.stack.len() {
      if self.stack.pop().is_none() {
        break;
      }
    }
    match result {
      Ok(value) => {
        <Cst as Seam<'inp, L, Ctx, Lang, Complete>>::finish(&self.cst, open, input);
        Ok(value)
      }
      Err(error) => {
        <Cst as Seam<'inp, L, Ctx, Lang, Complete>>::demote(&self.cst, open, input);
        Err(error)
      }
    }
  }

  /// The machine, seated on the driver's storage with the driver's floor and cap, and its loop.
  #[inline]
  fn resolve<K: ArraySize>(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    let unset = if self.grows { GROWN_CAP } else { usize::MAX };
    #[cfg(any(feature = "std", feature = "alloc"))]
    let unset = usize::MAX;
    let cap = self.cap.unwrap_or(unset);
    #[cfg(test)]
    {
      self.growth.above = 0;
    }
    let mut steps = Steps {
      operand: &mut self.operand,
      fold_prefix: &mut self.fold_prefix,
      fold_infix: &mut self.fold_infix,
      fold_postfix: &mut self.fold_postfix,
      cap,
      #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
      grows: self.grows,
      #[cfg(test)]
      growth: &mut self.growth,
      _segment: PhantomData,
    };
    // `with_stack` empties the storage first: a parse that unwound left its frames there.
    let machine = PrattResolver::with_stack(&mut self.stack)
      .min_precedence(self.floor.clone())
      .max_depth(cap);
    // The scanner's trip count as the expression begins, for every point at which the loop ends
    // or refuses, at every depth: a trip since, under user code or a read of the loop's own, makes
    // that point terminal whatever followed it. Recovering by re-keying is the caller's.
    let scans = input.scanner_trip_snapshot();
    match Self::run::<_, false, K>(machine, None, &mut steps, input, scans)? {
      Exit::Done(value) => Ok(value),
      Exit::Drained(..) => {
        unreachable!("the driver's own storage has nothing below it, so its loop never drains")
      }
    }
  }

  /// The loop, over one storage: the driver's own, or a segment over the storage below. `UPPER`
  /// says which.
  #[inline]
  fn run<'closure, St, const UPPER: bool, K: ArraySize>(
    mut machine: Machine<'inp, L, O, Power, St>,
    mut standing: Option<Standing<Power>>,
    steps: &mut Steps<'_, Operand, FoldPrefix, FoldInfix, FoldPostfix, K>,
    input: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    scans: ScannerTripBaseline<'closure>,
  ) -> Result<
    Exit<O, State<'inp, L, O, Power>, Power>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    St: Stack<Frame<'inp, L, O, Power>>,
  {
    loop {
      match machine.poll() {
        // A term. The operand parser is asked first, and a token it declines is staged.
        PrattPoll::Term => loop {
          // The descent-trip count for this attempt alone: a refused descent the operand parser
          // caught and declined over is no absence, and the head is not classified after it.
          let trips = input.trip_snapshot();
          if let ParseAttempt::Accept(value) = steps.operand.try_parse_input(input)? {
            if let Err(refused) = machine.handle_operand(value) {
              unreachable!(
                "the flat driver's machine refused an operand where it wanted a term: {:?}",
                refused.why()
              );
            }
            break;
          }
          if input.tripped_during_attempt(trips) {
            return Err(terminal_stop(input));
          }
          let head = input.try_expect_map_or_stop(|tok| {
            if tok.is_trivia() {
              return Some(TermHead::Trivia);
            }
            match tok.try_pratt_lhs() {
              Some(PrattLHS::Prefix(prefix)) => Some(TermHead::Prefix(prefix.into_precedence())),
              Some(PrattLHS::Operand(())) | None => None,
            }
          })?;
          match head {
            // Committed: the operand parser is asked again.
            Some((TermHead::Trivia, _)) => {}
            Some((TermHead::Prefix(power), operator)) => {
              if let Err(refused) = machine.handle_prefix(Precedenced::new(operator, power)) {
                Self::past_cap(refused.why(), machine.depth(), steps.cap, input)?;
                #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
                {
                  if steps.grows {
                    let (state, mut storage) = machine.into_parts();
                    let exit = Self::grow(
                      state,
                      &mut storage,
                      Refed::Prefix(refused.into_item()),
                      steps,
                      input,
                      scans,
                    )?;
                    let (state, back) = drained(exit);
                    machine = PrattResolver::from_parts(state, storage);
                    standing = back;
                    break;
                  }
                }
                return Err(stack_full(input, machine.capacity()));
              }
              break;
            }
            // No term. A trip since the expression began answers for it, since the operand
            // parser may have declined only because the input was cut short. Otherwise the
            // operator that waited for it says which error it is.
            None => {
              if input.scanner_tripped_during_attempt(scans) {
                return Err(terminal_stop(input));
              }
              let infix_waits = matches!(machine.retract(), Some(PrattPending::Infix(_)));
              let at = input.span().end();
              return Err(if infix_waits {
                UnexpectedEoRhs::<L::Offset, Lang>::eorhs_of(at).into()
              } else {
                UnexpectedEoLhs::<L::Offset, Lang>::eolhs_of(at).into()
              });
            }
          }
        },

        // An operator, or the ending.
        PrattPoll::Operator => {
          // A classification that stands is offered again, and nothing is read. On `Take` the
          // head is read afresh, and the read's own offer decides what is committed.
          let stood = match &standing {
            None => Stood::Read,
            Some(Standing::Ending) => Stood::Close,
            Some(Standing::Operator(rhs)) => match machine.offer(rhs) {
              PrattOffer::Reduce | PrattOffer::Decline => Stood::Close,
              PrattOffer::Chain => Stood::Chain,
              PrattOffer::Take if matches!(rhs, PrattRHS::Adjacent(_)) => Stood::Adjacent,
              PrattOffer::Take => Stood::Read,
            },
          };
          match stood {
            Stood::Read => standing = None,
            Stood::Close => {
              close(&mut machine);
              continue;
            }
            Stood::Chain => return Err(chain(input, scans)),
            Stood::Adjacent => return Err(adjacent(input, scans)),
          }

          let mut answer = Answer::Ending;
          let head = input.try_expect_map_or_stop(|tok| {
            if tok.is_trivia() {
              return Some(OperatorHead::Trivia);
            }
            let rhs = match tok.try_pratt_rhs() {
              Some(PrattRHS::End) | None => {
                answer = Answer::Ending;
                return None;
              }
              Some(rhs) => rhs,
            };
            match (machine.offer(&rhs), rhs) {
              (PrattOffer::Take, PrattRHS::Infix(infix)) => {
                let (associativity, power) = infix.into_components();
                Some(OperatorHead::Infix(associativity, power))
              }
              (PrattOffer::Take, PrattRHS::Postfix(postfix)) => {
                Some(OperatorHead::Postfix(postfix.into_precedence()))
              }
              (PrattOffer::Take, PrattRHS::Adjacent(_)) => {
                answer = Answer::Adjacent;
                None
              }
              (PrattOffer::Take, PrattRHS::End) => {
                answer = Answer::Ending;
                None
              }
              (PrattOffer::Reduce, rhs) => {
                answer = Answer::Reduce(rhs);
                None
              }
              (PrattOffer::Decline, _) => {
                answer = Answer::Decline;
                None
              }
              (PrattOffer::Chain, _) => {
                answer = Answer::Chain;
                None
              }
            }
          })?;
          match head {
            // Committed: the head is read again.
            Some((OperatorHead::Trivia, _)) => {}
            Some((OperatorHead::Infix(associativity, power), token)) => {
              let operator = match associativity {
                PrattInfix::Left(()) => PrattInfix::Left(token),
                PrattInfix::Right(()) => PrattInfix::Right(token),
                PrattInfix::Neither(()) => PrattInfix::Neither(token),
              };
              if let Err(refused) = machine.handle_infix(Precedenced::new(operator, power)) {
                Self::past_cap(refused.why(), machine.depth(), steps.cap, input)?;
                #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
                {
                  if steps.grows {
                    let (state, mut storage) = machine.into_parts();
                    let exit = Self::grow(
                      state,
                      &mut storage,
                      Refed::Infix(refused.into_item()),
                      steps,
                      input,
                      scans,
                    )?;
                    let (state, back) = drained(exit);
                    machine = PrattResolver::from_parts(state, storage);
                    standing = back;
                    continue;
                  }
                }
                return Err(stack_full(input, machine.capacity()));
              }
            }
            Some((OperatorHead::Postfix(power), token)) => {
              if let Err(refused) = machine.handle_postfix(Precedenced::new(token, power)) {
                unreachable!(
                  "the flat driver's machine refused a postfix operator its offer took: {:?}",
                  refused.why()
                );
              }
            }
            // The token is staged, or there is none. A trip since the expression began answers
            // for an ending and for a refusal: an operand parser or a fold that read on may have
            // left the head it stopped behind.
            None => match answer {
              Answer::Ending => {
                if input.scanner_tripped_during_attempt(scans) {
                  return Err(terminal_stop(input));
                }
                standing = Some(Standing::Ending);
                close(&mut machine);
              }
              Answer::Reduce(rhs) => {
                standing = Some(Standing::Operator(rhs));
                close(&mut machine);
              }
              Answer::Decline => {
                if input.scanner_tripped_during_attempt(scans) {
                  return Err(terminal_stop(input));
                }
                close(&mut machine);
              }
              Answer::Chain => return Err(chain(input, scans)),
              Answer::Adjacent => return Err(adjacent(input, scans)),
            },
          }
        }

        // One reduction, folded with the operator that asked for it still on the input.
        PrattPoll::Reduce(reduction) => {
          // The moved pair, read only while a classification stands, which is after every
          // reduction but a postfix operator's.
          let before = standing
            .as_ref()
            .map(|_| (input.span().end_ref().clone(), input.regime_generation()));
          let value = match reduction {
            PrattReduction::Prefix(reduction) => {
              let (operand, operator) = reduction.into_components();
              steps.fold_prefix.fold_prefix(input, operand, operator)?
            }
            PrattReduction::Infix(reduction) => {
              let (left, right, operator) = reduction.into_components();
              steps.fold_infix.fold_infix(input, left, right, operator)?
            }
            PrattReduction::Postfix(reduction) => {
              let (operand, operator) = reduction.into_components();
              steps.fold_postfix.fold_postfix(input, operand, operator)?
            }
          };
          if let Err(refused) = machine.handle_reduced(value) {
            unreachable!(
              "the flat driver's machine refused the value of its own reduction: {:?}",
              refused.why()
            );
          }
          // The one key: the offsets compared for equality, and the regimes by `same_regime`, as
          // the input compares them. Moved, the head is read again.
          if let Some((at, regime)) = before
            && (*input.span().end_ref() != at || !same_regime(input.regime_generation(), regime))
          {
            standing = None;
          }
          // The return test, before the loop next reads its stack: a segment whose own frames are
          // gone hands the machine to the loop below, which goes on over its own segment, full.
          if UPPER && machine.storage().is_empty() {
            #[cfg(test)]
            {
              steps.growth.returned += 1;
              steps.growth.above -= 1;
            }
            return Ok(Exit::Drained(machine.into_parts().0, standing));
          }
        }

        PrattPoll::Reducing => {
          unreachable!("the flat driver hands each reduction's value back before it polls again")
        }

        // The expression is complete. A trip since it began makes its end terminal.
        PrattPoll::Done(value) => {
          if input.scanner_tripped_during_attempt(scans) {
            return Err(terminal_stop(input));
          }
          return Ok(Exit::Done(value));
        }
      }
    }
  }

  /// A push the machine refused. `Full` at the cap is the error, and reports the cap. Below it,
  /// the storage refused, and the caller grows it or reports its bound. Any other refusal of a
  /// push in the phase that wanted it, with the offer's `Take`, is the adapter's own slip.
  #[inline]
  fn past_cap(
    why: PrattRefusal,
    depth: usize,
    cap: usize,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    match why {
      PrattRefusal::Full if depth >= cap => Err(stack_full(input, cap)),
      PrattRefusal::Full => Ok(()),
      PrattRefusal::Misplaced
      | PrattRefusal::Reduce
      | PrattRefusal::Decline
      | PrattRefusal::Chain => {
        unreachable!("the flat driver's machine refused a push it had taken: {why:?}")
      }
    }
  }

  /// One more segment, in a deeper native frame, over the full storage `below`: the refused push
  /// is fed again, and taken, and the loop goes on over the new segment until it drains.
  #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
  #[inline(never)]
  fn grow<'closure, St, K: ArraySize>(
    state: State<'inp, L, O, Power>,
    below: &mut St,
    refed: Refed<Tk<'inp, L>, Power>,
    steps: &mut Steps<'_, Operand, FoldPrefix, FoldInfix, FoldPostfix, K>,
    input: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    scans: ScannerTripBaseline<'closure>,
  ) -> Result<
    Exit<O, State<'inp, L, O, Power>, Power>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    St: Stack<Frame<'inp, L, O, Power>>,
  {
    let mut next: Segment<'_, Frame<'inp, L, O, Power>, K> = Segment::over(below);
    let mut machine = PrattResolver::from_parts(state, &mut next);
    let fed = match refed {
      Refed::Prefix(operator) => machine
        .handle_prefix(operator)
        .map_err(|refused| refused.why()),
      Refed::Infix(operator) => machine
        .handle_infix(operator)
        .map_err(|refused| refused.why()),
    };
    if let Err(why) = fed {
      unreachable!(
        "an empty segment below the cap refused the push the full storage refused: {why:?}"
      );
    }
    #[cfg(test)]
    {
      steps.growth.grown += 1;
      steps.growth.above += 1;
      steps.growth.deepest = steps.growth.deepest.max(steps.growth.above);
    }
    // Nothing stands: the push followed a fresh read, or a term.
    Self::run::<_, true, K>(machine, None, steps, input, scans)
  }
}

/// The state and the classification a drained segment handed back.
#[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
#[inline]
fn drained<V, State, Power>(exit: Exit<V, State, Power>) -> (State, Option<Standing<Power>>) {
  match exit {
    Exit::Drained(state, standing) => (state, standing),
    Exit::Done(_) => unreachable!(
      "a deeper frame never answers `Done`: a segment exists only above full storage, so its \
       loop returns before the count can reach zero"
    ),
  }
}

/// One sub-expression closed: in the `Operator` phase, which every caller is in.
#[inline]
fn close<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, St>(
  machine: &mut PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, St>,
) where
  Power: PrattPower,
  St: Stack<PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>>,
{
  if let Err(why) = machine.close() {
    unreachable!("the flat driver's machine refused to close in its `Operator` phase: {why:?}");
  }
}

/// The operator repeats a non-associative chain, unless a trip since the expression began makes
/// the refusal terminal. The operator is left on the input.
#[inline]
fn chain<'inp, 'closure, L, Ctx, Lang>(
  input: &InputRef<'inp, 'closure, L, Ctx, Lang>,
  scans: ScannerTripBaseline<'closure>,
) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<NonAssociativeChain<L::Offset, Lang>>,
{
  if input.scanner_tripped_during_attempt(scans) {
    return terminal_stop(input);
  }
  NonAssociativeChain::<L::Offset, Lang>::of(input.span().end()).into()
}

/// The machine would take a zero-token operator, which has no token to fold: a missing right
/// operand, unless a trip since the expression began makes the refusal terminal. The token is
/// left on the input.
#[inline]
fn adjacent<'inp, 'closure, L, Ctx, Lang>(
  input: &InputRef<'inp, 'closure, L, Ctx, Lang>,
  scans: ScannerTripBaseline<'closure>,
) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<UnexpectedEoRhs<L::Offset, Lang>>,
{
  if input.scanner_tripped_during_attempt(scans) {
    return terminal_stop(input);
  }
  UnexpectedEoRhs::<L::Offset, Lang>::eorhs_of(input.span().end()).into()
}
