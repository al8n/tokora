//! `folding`: the flat driver's adapter that resolves precedence as it parses, and folds into the
//! caller's value with the typed driver's three fold traits.
//!
//! It runs the flat loop over a machine of the caller's values, and folds each reduction the
//! machine hands out, one for each poll, with the operator that asked for it left on the input.
//! After every fold the head is read again and offered afresh: whatever the fold consumed or left,
//! the read is of the input's current head. Its classification is reused only while the head is
//! still the token the last read classified, the same cache entry or the same parked token. Under
//! a cache with room the read takes the staged token and lexes nothing. A token is committed only
//! on the answer of that read's own offer.
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
  container::Stack,
  error::{NonAssociativeChain, PrattStackFull, UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot},
  input::{Complete, ScannerTripBaseline, StagedEntry, same_regime},
  span::{Span as _, Spanned},
  token::PrattToken,
};

#[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
use super::super::segment::Segment;
use super::{
  super::{
    PrattFoldInfix, PrattFoldPostfix, PrattFoldPrefix, PrattFrame, PrattInfix, PrattLHS,
    PrattOffer, PrattPending, PrattPoll, PrattPower, PrattRHS, PrattReduction, PrattResolver,
    Precedenced, resolve::ResolverState,
  },
  FlatCst, OperatorHead, TermHead,
  seam::Seam,
  stack_full, terminal_stop,
};

#[cfg(test)]
mod tests;

/// Where the crate cannot allocate, the cap of a storage that grows when none is set: the shipped
/// recursion budget, so that such a build refuses no depth the typed driver accepts by default.
#[cfg(not(any(feature = "std", feature = "alloc")))]
const GROWN_CAP: usize = 32;

// `GROWN_CAP` is the shipped recursion budget: the default depth cannot move without this line, and
// whoever moves it decides what the cap does.
#[cfg(not(any(feature = "std", feature = "alloc")))]
const _: () = assert!(GROWN_CAP == crate::state::RecursionLimiter::PARSE_DEFAULT_DEPTH);

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

/// A flat driver that resolves precedence as it parses, and folds into `O`: made by
/// [`folding`](super::Flat::folding).
///
/// `S` is the machine's storage, which the driver owns and lends to the machine for every parse.
/// It is emptied when a parse begins and when one ends, however it ends: an unwind out of the
/// operand parser or a fold empties it too.
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
  /// The driver [`folding`](super::Flat::folding) makes: its operand parser and folds, the seam,
  /// the floor and the cap set before it, and the default storage, which grows on the native stack
  /// where the crate cannot allocate.
  #[inline]
  pub(super) fn new(
    operand: Operand,
    (fold_prefix, fold_infix, fold_postfix): (FoldPrefix, FoldInfix, FoldPostfix),
    cst: Cst,
    (floor, cap): (Power, Option<usize>),
    stack: S,
  ) -> Self {
    Self {
      operand,
      fold_prefix,
      fold_infix,
      fold_postfix,
      cst,
      floor,
      cap,
      stack,
      #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
      grows: true,
      #[cfg(test)]
      growth: Growth::default(),
      _marker: PhantomData,
    }
  }

  /// Replaces the machine's storage. The driver owns it and lends it to the machine for every
  /// parse, emptying it when a parse begins and when one ends, however it ends: a storage lent as
  /// `&mut` keeps no frame, and so none of the operands they own, past an unwind out of user code,
  /// and keeps whatever it allocated.
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
  /// not take is left on the input while a reduction it asks for is folded, and the head is read
  /// again after each fold, so that what a fold consumed is never mistaken for the head. Its
  /// classification is reused while the head is the same cache entry or the same parked token, and
  /// an end of input the read confirmed is not read again. An operator below the floor, a token
  /// that is no operator, and the end of input end the expression: what is pending is folded, and
  /// the token is left on the input.
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
  /// - [`UnexpectedEot`](crate::error::UnexpectedEot), terminal, at a scanner stop. Once any
  ///   scanner trip has happened since the parse began, it is also what the expression's end or
  ///   refusal becomes, and what stands in for a fold that would follow, whatever followed the
  ///   trip: a rollback that took the stop off the input, or a new lexer regime. So is a decline
  ///   of the operand parser over a refused descent it caught. Recovering by re-keying the lexer is
  ///   the caller's, after the expression.
  /// - Whatever the operand parser or a fold returns.
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

/// What a read of the head decided where it committed nothing.
enum Answer {
  /// The read was handed no token at all: the end of input, a tail of lexer errors before it
  /// included. One sub-expression is closed. The read's closure never answers this: it is what
  /// the answer is while that closure has not run.
  End,
  /// No operator of the table, or `End`; an operator below the floor; or an operator that waits
  /// for a reduction. One sub-expression is closed.
  Close,
  /// The operator repeats a non-associative chain.
  Chain,
  /// The machine would take a zero-token operator.
  Adjacent,
}

/// A parse's hold on the machine's storage: it empties the storage by its own length when it is
/// dropped, at the end of the parse, however the parse ends. By its own length, as the machine's
/// `with_stack` empties a storage, because a parse that grew and then failed leaves the driver's
/// own frames there while the machine's count is gone with the native frames that carried it; and
/// on an unwind out of user code too, so that a storage the caller lent keeps none of the operands
/// its frames own once the parse is gone. It pops, so a storage that grew keeps its allocation for
/// the next parse.
struct Emptied<'s, S, T>
where
  S: Stack<T>,
{
  stack: &'s mut S,
  _frames: PhantomData<fn() -> T>,
}

impl<'s, S, T> Emptied<'s, S, T>
where
  S: Stack<T>,
{
  #[inline(always)]
  fn new(stack: &'s mut S) -> Self {
    Self {
      stack,
      _frames: PhantomData,
    }
  }

  /// The storage, to lend to the machine.
  #[inline(always)]
  fn get(&mut self) -> &mut S {
    &mut *self.stack
  }
}

impl<S, T> Drop for Emptied<'_, S, T>
where
  S: Stack<T>,
{
  fn drop(&mut self) {
    // A frame's destructor that unwinds leaves this drain part-way, and the unwind drops `rest`,
    // which drains what is left.
    let mut rest = Rest {
      stack: &mut *self.stack,
      _frames: PhantomData,
    };
    rest.drain();
  }
}

/// What is left of a storage being emptied, which drains it when it is dropped: the guard over the
/// drain in `Emptied`'s own drop. A frame's destructor that unwinds would otherwise stop that drain
/// with frames still in the storage, and a storage the caller lent would keep them. A second
/// destructor that unwinds while the guard drains aborts the process, as a panic during an unwind
/// does.
struct Rest<'s, S, T>
where
  S: Stack<T>,
{
  stack: &'s mut S,
  _frames: PhantomData<fn() -> T>,
}

impl<S, T> Rest<'_, S, T>
where
  S: Stack<T>,
{
  /// Pops and drops every frame, by the storage's length now. It pops, so the storage keeps its
  /// allocation.
  #[inline]
  fn drain(&mut self) {
    for _ in 0..self.stack.len() {
      match self.stack.pop() {
        Some(frame) => drop(frame),
        None => break,
      }
    }
  }
}

impl<S, T> Drop for Rest<'_, S, T>
where
  S: Stack<T>,
{
  fn drop(&mut self) {
    self.drain();
  }
}

/// How a loop over one storage ends without an error.
//
// With an allocator, outside tests, no loop runs over a segment, so nothing reads what a drained
// one hands back.
#[cfg_attr(
  not(any(test, not(any(feature = "std", feature = "alloc")))),
  allow(dead_code)
)]
enum Exit<V, State, Offset, Power> {
  /// The expression's value. Only the loop over the driver's own storage gets here: a segment
  /// exists only above full storage, so its loop returns before the count can reach zero.
  Done(V),
  /// The loop's segment is drained: the machine's state, and what the last read of the head found,
  /// for the loop below to go on with on its own segment.
  Drained(State, Option<Seen<Offset, Power>>),
}

/// What the last read of the head found, kept across the folds after it while it still answers for
/// the head, so that the read after a fold repeats no work it can prove would come out the same.
enum Seen<Offset, Power> {
  /// No token at all: the end of input, confirmed at a position and a regime.
  End(AtEnd<Offset>),
  /// A token the read classified and staged, by the identity the input gives the front: a cache
  /// entry's place in the push order, or the id the parked token took when it entered the slot. Its
  /// classification is reused while that token is the front, so a long token a table spells out
  /// is classified once, however many folds it waits for. A fold that took it, or left another
  /// token in its place, a token with the same span among them, leaves another identity.
  Staged {
    entry: StagedEntry,
    rhs: Option<PrattRHS<(), (), (), (), Power>>,
  },
}

/// The end of input a read confirmed: no token was to be had from the committed position, under
/// the regime the read ran in, a tail of lexer errors before it included. It stands while the
/// position's end and the regime are as they were. At the end of input a fold has no token to
/// take, so anything a fold does to the input moves that end or re-keys.
///
/// Kept so that the read after each fold, which a staged token answers from the cache, does not
/// scan the input's tail again where there is no token to stage: each fold at the end of input
/// would otherwise rescan every space and every lexer error after the last operand.
struct AtEnd<Offset> {
  at: Offset,
  regime: u64,
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
    // Held before anything of the parse runs, the seam's entry included, so that the storage is
    // emptied however the parse ends, an unwind out of the seam, the operand parser or a fold
    // among the ways.
    let mut stack: Emptied<'_, S, Frame<'inp, L, O, Power>> = Emptied::new(&mut self.stack);
    let open = <Cst as Seam<'inp, L, Ctx, Lang, Complete>>::open(&self.cst, input)?;
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
    // The machine empties the storage too, by its own length, and finds it empty. It is built at
    // the driver's floor, so no default floor is built to be replaced.
    let machine =
      PrattResolver::with_stack_and_floor(stack.get(), self.floor.clone()).max_depth(cap);
    // The scanner's trip count as the expression begins, for every point at which the loop ends
    // or refuses, at every depth: a trip since, under user code or a read of the loop's own, makes
    // that point terminal whatever followed it. Recovering by re-keying is the caller's.
    let scans = input.scanner_trip_snapshot();
    let result = match Self::run::<_, false, K>(machine, None, &mut steps, input, scans) {
      Ok(Exit::Done(value)) => Ok(value),
      Ok(Exit::Drained(..)) => {
        unreachable!("the driver's own storage has nothing below it, so its loop never drains")
      }
      Err(error) => Err(error),
    };
    // Emptied before the seam's exit, as on an unwind it is before the unwind leaves.
    drop(stack);
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

  /// The loop, over one storage: the driver's own, or a segment over the storage below. `UPPER`
  /// says which.
  #[inline]
  fn run<'closure, St, const UPPER: bool, K: ArraySize>(
    mut machine: Machine<'inp, L, O, Power, St>,
    mut seen: Option<Seen<L::Offset, Power>>,
    steps: &mut Steps<'_, Operand, FoldPrefix, FoldInfix, FoldPostfix, K>,
    input: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    scans: ScannerTripBaseline<'closure>,
  ) -> Result<
    Exit<O, State<'inp, L, O, Power>, L::Offset, Power>,
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
            machine.put_operand(value);
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
              if let Err(operator) = machine.push_prefix(Precedenced::new(operator, power)) {
                Self::past_cap(machine.depth(), steps.cap, input)?;
                #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
                {
                  if steps.grows {
                    let (state, mut storage) = machine.into_parts();
                    let exit = Self::grow(
                      state,
                      &mut storage,
                      Refed::Prefix(operator),
                      steps,
                      input,
                      scans,
                    )?;
                    let (state, back) = drained(exit);
                    machine = PrattResolver::from_parts(state, storage);
                    seen = back;
                    break;
                  }
                }
                // Not grown: the error reports the storage's bound, and the operator is dropped.
                drop(operator);
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

        // An operator, or the ending. The head is read every time, after a fold too: a fold may
        // have consumed the token that was staged, or left one in its place that ends where it
        // ended, and only a read sees which. The read's own offer decides what is committed, and a
        // token the machine does not take stays staged, so under a cache with room the next read
        // lexes nothing.
        PrattPoll::Operator => {
          // What the last read found spares this one its work while it stands. An end of input
          // confirmed at this position and regime is not read again: the expression ends there,
          // and the close meets the trip check as a read's would. A token the read staged keeps
          // its classification while it is still the front.
          let mut reuse = None;
          match seen.take() {
            Some(Seen::End(at_end)) => {
              if *input.span().end_ref() == at_end.at
                && same_regime(input.regime_generation(), at_end.regime)
              {
                seen = Some(Seen::End(at_end));
                close(&mut machine, input, scans)?;
                continue;
              }
            }
            Some(Seen::Staged { entry, rhs }) if input.staged_entry() == Some(entry) => {
              reuse = Some(rhs);
            }
            // Nothing kept, or a staged token that is no longer the front.
            _ => {}
          }
          let mut answer = Answer::End;
          let mut classified = None;
          let head = input.try_expect_map_or_stop(|tok| {
            // The front is the entry checked above when its classification is reused, and a staged
            // entry is never trivia.
            let rhs = match reuse.take() {
              Some(rhs) => rhs,
              None => {
                if tok.is_trivia() {
                  return Some(OperatorHead::Trivia);
                }
                tok.try_pratt_rhs()
              }
            };
            let rhs = match rhs {
              Some(PrattRHS::End) | None => {
                answer = Answer::Close;
                classified = Some(None);
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
                answer = Answer::Close;
                classified = Some(None);
                None
              }
              (PrattOffer::Reduce | PrattOffer::Decline, rhs) => {
                answer = Answer::Close;
                classified = Some(Some(rhs));
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
              if let Err(operator) = machine.push_infix(Precedenced::new(operator, power)) {
                Self::past_cap(machine.depth(), steps.cap, input)?;
                #[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
                {
                  if steps.grows {
                    let (state, mut storage) = machine.into_parts();
                    let exit = Self::grow(
                      state,
                      &mut storage,
                      Refed::Infix(operator),
                      steps,
                      input,
                      scans,
                    )?;
                    let (state, back) = drained(exit);
                    machine = PrattResolver::from_parts(state, storage);
                    seen = back;
                    continue;
                  }
                }
                // Not grown: the error reports the storage's bound, and the operator is dropped.
                drop(operator);
                return Err(stack_full(input, machine.capacity()));
              }
            }
            // A postfix operator's reduction is folded where the operator is read, with the
            // operator just committed. It pops nothing, so no return test follows it: a segment
            // holds a frame of its own wherever its loop is at an operator.
            Some((OperatorHead::Postfix(power), token)) => {
              let (operand, operator) = machine
                .take_postfix(Precedenced::new(token, power))
                .into_components();
              let value = steps.fold_postfix.fold_postfix(input, operand, operator)?;
              machine.put_reduced(value);
            }
            // The token is staged, or there is none. A trip since the expression began answers
            // for an ending, a reduction and a refusal alike: an operand parser or a fold that
            // read on may have left the head it stopped behind.
            None => match answer {
              Answer::End => {
                seen = Some(Seen::End(AtEnd {
                  at: input.span().end_ref().clone(),
                  regime: input.regime_generation(),
                }));
                close(&mut machine, input, scans)?;
              }
              Answer::Close => {
                if let (Some(rhs), Some(entry)) = (classified, input.staged_entry()) {
                  seen = Some(Seen::Staged { entry, rhs });
                }
                close(&mut machine, input, scans)?;
              }
              Answer::Chain => return Err(chain(input, scans)),
              Answer::Adjacent => return Err(adjacent(input, scans)),
            },
          }
        }

        // One reduction, folded with the operator that asked for it still on the input.
        PrattPoll::Reduce(reduction) => {
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
          // The return test, before the loop next reads its stack: a segment whose own frames are
          // gone hands the machine to the loop below, which goes on over its own segment, full.
          if UPPER && machine.storage().is_empty() {
            #[cfg(test)]
            {
              steps.growth.returned += 1;
              steps.growth.above -= 1;
            }
            return Ok(Exit::Drained(machine.into_parts().0, seen));
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

  /// A push the machine refused, which is `Full`. At the cap it is the error, and reports the cap.
  /// Below it, the storage refused, and the caller grows it or reports its bound.
  #[inline]
  fn past_cap(
    depth: usize,
    cap: usize,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    if depth >= cap {
      Err(stack_full(input, cap))
    } else {
      Ok(())
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
    Exit<O, State<'inp, L, O, Power>, L::Offset, Power>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    St: Stack<Frame<'inp, L, O, Power>>,
  {
    let mut next: Segment<'_, Frame<'inp, L, O, Power>, K> = Segment::over(below);
    let mut machine = PrattResolver::from_parts(state, &mut next);
    let fed = match refed {
      Refed::Prefix(operator) => machine.push_prefix(operator).is_ok(),
      Refed::Infix(operator) => machine.push_infix(operator).is_ok(),
    };
    if !fed {
      unreachable!("an empty segment below the cap refused the push the full storage refused");
    }
    #[cfg(test)]
    {
      steps.growth.grown += 1;
      steps.growth.above += 1;
      steps.growth.deepest = steps.growth.deepest.max(steps.growth.above);
    }
    // Nothing is staged and no end of input was confirmed: the push followed a read that committed
    // a token, or a term.
    Self::run::<_, true, K>(machine, None, steps, input, scans)
  }
}

/// The state, and what the last read of the head found, that a drained segment handed back.
#[cfg(any(test, not(any(feature = "std", feature = "alloc"))))]
#[inline]
fn drained<V, State, Offset, Power>(
  exit: Exit<V, State, Offset, Power>,
) -> (State, Option<Seen<Offset, Power>>) {
  match exit {
    Exit::Drained(state, seen) => (state, seen),
    Exit::Done(_) => unreachable!(
      "a deeper frame never answers `Done`: a segment exists only above full storage, so its \
       loop returns before the count can reach zero"
    ),
  }
}

/// One sub-expression closed, in the `Operator` phase, which every caller is in. What follows is
/// a fold, unless the expression is over: so a trip since the expression began makes this point
/// terminal first, whether the classification behind it was read just now or stood, and no fold
/// runs over a stop it could move past.
#[inline]
fn close<
  'inp,
  'closure,
  L,
  Ctx,
  Lang,
  V,
  Pre,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  Post,
  Power,
  St,
>(
  machine: &mut PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, St>,
  input: &InputRef<'inp, 'closure, L, Ctx, Lang>,
  scans: ScannerTripBaseline<'closure>,
) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  Power: PrattPower,
  St: Stack<PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
{
  if input.scanner_tripped_during_attempt(scans) {
    return Err(terminal_stop(input));
  }
  if let Err(why) = machine.close() {
    unreachable!("the flat driver's machine refused to close in its `Operator` phase: {why:?}");
  }
  Ok(())
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
