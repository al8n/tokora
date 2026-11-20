//! The Sans-I/O core of the Pratt family: precedence resolution with an explicit stack, as a value
//! that is fed the items of one expression and asked what it wants next.
//!
//! The machine owns no input, no lexer and no position, and it calls no code of its caller's but
//! the kinds `PrattResolver`'s documentation lists. The adapters that read tokora's input run its
//! loop; a caller with no input at hand, or with operators the table cannot classify, runs the
//! loop itself.

use core::{convert::Infallible, marker::PhantomData};

use derive_more::IsVariant;

use super::{DefaultPrattStack, PrattInfix, PrattPower, PrattRHS, Precedenced};
use crate::container::Stack;

/// A Sans-I/O machine that resolves precedence with an explicit stack. With no allocator, its
/// default storage holds four pending operators and refuses the fifth, where the typed driver
/// holds 31 at most.
///
/// It is the whole protocol of one expression in one value. Feed it the expression's items through
/// the `handle_*` events, and [`poll`](Self::poll) says what it wants next: a term, an operator, a
/// reduction to perform, or the finished value. It applies the Pratt drivers' own rules: the
/// floors, the three associativities, the non-associative latch, prefix and postfix operators. It
/// hands the reductions out one for each `poll`, in exactly the order a recursive Pratt driver
/// folds, so the tree a caller builds from them is that driver's tree.
///
/// It owns nothing of a parse: no input, no lexer, no position, and no token type in its bounds.
/// The values and the operator payloads are the caller's, unbounded, and each comes back intact in
/// a reduction or from [`retract`](Self::retract). It borrows nothing between calls but a storage
/// it is lent, so a caller with no recursion of its own can park one in a work list while it
/// resolves an expression nested in an operand.
///
/// # The loop
///
/// | [`poll`](Self::poll) answers | It means | What the caller does next |
/// |---|---|---|
/// | [`Term`](PrattPoll::Term) | The innermost sub-expression has no value yet | [`handle_prefix`](Self::handle_prefix) or [`handle_operand`](Self::handle_operand). Where no term is to be had and an operator is pending, [`retract`](Self::retract) |
/// | [`Operator`](PrattPoll::Operator) | It has a value, and an operator or an ending is wanted | [`offer`](Self::offer), then [`handle_infix`](Self::handle_infix) or [`handle_postfix`](Self::handle_postfix). Or [`close`](Self::close), or [`finish`](Self::finish) |
/// | [`Reduce`](PrattPoll::Reduce) | One reduction is owed, and this is it | Perform it, and hand its value back with [`handle_reduced`](Self::handle_reduced) |
/// | [`Reducing`](PrattPoll::Reducing) | A reduction is out, and its value has not come back | [`handle_reduced`](Self::handle_reduced) |
/// | [`Done`](PrattPoll::Done) | The expression is complete | Keep the value. The machine is as [`reset`](Self::reset) leaves it |
///
/// The machine never reduces in a loop of its own. After `handle_reduced` the caller polls again,
/// and the answer is `Operator`, the next reduction a `finish` owes, or `Done`.
///
/// # Refusals
///
/// Every event either takes its item or refuses it, and **a refusal hands the item back and
/// modifies nothing**: the machine's own state is as it was when the call returns, and its storage
/// unchanged in everything `Stack`, `Container` and `PartialEq` observe. A refusal writes nothing
/// of the machine's own but what it puts back: when its storage refuses the frame,
/// [`handle_infix`](Self::handle_infix) has moved the value into it, and restores it in the
/// [`Operator`](PrattPoll::Operator) phase it was taken from. Of its storage it only reads, or
/// tries a push that the storage refuses, which by [`Stack`]'s obligations change nothing those
/// three observe. The item, and the values, payloads and powers the machine holds, are unchanged
/// given a `Power` whose comparisons change nothing a caller observes: of their code, a refusal
/// runs those comparisons alone, and a lawful power may still cache its rank in a `OnceCell` as it
/// compares. So, with such a `Power`, a clone taken before the call compares equal to the machine
/// after it whenever everything the machine holds equals both itself and its own clone (`x == x`,
/// `x.clone() == x`), which `f64::NAN` does not. [`PrattRefused`] carries the item and one
/// [`PrattRefusal`]. Nothing in the machine panics on a caller's slip; it panics only when its
/// storage breaks `Stack`'s obligations, with a message that names `Stack`.
///
/// # Its storage
///
/// The frames, one for each pending operator, live in `S`, and that is all the memory the machine
/// has: it allocates nothing itself. The default, [`DefaultPrattStack`], holds four frames inline,
/// and past four it grows on the heap where the crate can allocate. Where it cannot, the fifth
/// pending operator is refused with [`Full`](PrattRefusal::Full): name a larger
/// [`PrattStack`](super::PrattStack), such as `PrattStack<_, U16>`, or lend storage through
/// [`with_stack`](Self::with_stack). One crate anywhere in a build that turns `alloc` on makes the
/// same default spill instead, so a `no_std` library that drives a machine by hand is tested with
/// `--no-default-features` too. [`PrattStack`](super::PrattStack) says which expressions keep more
/// than four operators pending: `a = b = c = d = e = 0`, with `=` right-associative, keeps five.
///
/// The machine's depth is its own count, and never the storage's length.
/// [`max_depth`](Self::max_depth) caps it, and a push is refused when the count has reached the cap
/// or when the storage itself refuses the frame.
///
/// # What it relies on of its storage
///
/// Every guarantee on this page holds over a storage that keeps [`Stack`]'s obligations, which are
/// the whole of what the machine relies on of `S`. `Stack` gives the reason for each:
///
/// - last in, first out;
/// - only its own `push` and `pop` add or remove frames;
/// - `len` counts the frames, and [`with_stack`](Self::with_stack) reads it once;
/// - a refusal hands the frame back, and only a stack that cannot hold the frame refuses it;
/// - a `push` or a `pop` that unwinds leaves the frames as they were;
/// - a clone is an independent copy with the same items and the same `max_capacity`, equal to the
///   original whenever the original equals itself, which the machine's own `Clone` relies on;
/// - reads and a refused push change nothing that `Stack`, `Container` or `PartialEq` observes,
///   which every refusal, [`offer`](Self::offer) and the reads of [`poll`](Self::poll) rely on to
///   modify nothing;
/// - its `Clone`, `PartialEq` and `Debug` only read it. Its `PartialEq` and `Debug` never panic on
///   a stack that keeps these unless the frames' own code does, and its `Clone` only as growing
///   may, on a capacity overflow or an allocation failure, or where a frame's own `Clone` does.
///   The machine's own three inherit exactly that, with the code of the value, the payloads and
///   the powers it holds beside the frames'.
///
/// It relies on nothing of `S::default`, since [`new`](Self::new) empties what that returns. With
/// a storage that breaks one of them, an answer may be wrong, or a panic that names `Stack`, but it
/// is never undefined behaviour.
///
/// # The caller code it runs
///
/// It runs no reducer, no handler and no closure. The caller code it does run is this, and no more:
///
/// - `Power`'s comparisons, its `Clone` where an infix reduction sets a latch, and its `Default` in
///   [`new`](Self::new) and [`with_stack`](Self::with_stack);
/// - the methods of its storage;
/// - the destructors of `V`, of the operator payloads and of `Power`, when [`reset`](Self::reset),
///   [`with_stack`](Self::with_stack), [`into_stack`](Self::into_stack) or the machine's own drop
///   lets go of a frame or a value; and `Power`'s alone where a latch or the floor is overwritten,
///   and where a reduction or a [`retract`](Self::retract) lets go of a frame's own latch;
/// - in its derived `Clone`, `PartialEq` and `Debug`, those traits of `S` and of each parameter.
///
/// An unwind out of any of them leaves the machine's count and its storage in agreement: a push
/// raises the count only once the storage has taken the frame, and a pop lowers it as soon as the
/// storage gives the frame up, before the frame or a value in it is dropped. It also leaves a phase
/// that claims no frame the machine has let go of: [`reset`](Self::reset) installs the new phase
/// before it drops anything, and a step whose value an unwind takes leaves the
/// [`Term`](PrattPoll::Term) phase, where that sub-expression starts over.
///
/// Both rest on the storage's obligation for an unwind, among those above: a push or a pop that
/// unwinds leaves the frames as they were.
///
/// # Example
///
/// `1 + 2 * 3`, fed by hand, with each reduction evaluated as it is handed out:
///
/// ```rust
/// use tokora::pratt::{PrattInfix, PrattPoll, PrattReduction, PrattResolver, Precedenced};
///
/// let mut m: PrattResolver<i64, (), char, (), ()> = PrattResolver::new();
/// m.handle_operand(1).unwrap();
/// m.handle_infix(Precedenced::new(PrattInfix::Left('+'), 1)).unwrap();
/// m.handle_operand(2).unwrap();
/// m.handle_infix(Precedenced::new(PrattInfix::Left('*'), 2)).unwrap();
/// m.handle_operand(3).unwrap();
/// m.finish().unwrap();
///
/// let value = loop {
///   match m.poll() {
///     PrattPoll::Reduce(PrattReduction::Infix(infix)) => {
///       let (left, right, operator) = infix.into_components();
///       let value = match operator.into_data() {
///         PrattInfix::Left('+') => left + right,
///         PrattInfix::Left('*') => left * right,
///         _ => unreachable!("only `+` and `*` were fed"),
///       };
///       m.handle_reduced(value).unwrap();
///     }
///     PrattPoll::Done(value) => break value,
///     _ => unreachable!("after `finish` the machine reduces, and then it is done"),
///   }
/// };
/// assert_eq!(value, 7);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrattResolver<
  V,
  Pre,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  Post = Infallible,
  Power = i64,
  S = DefaultPrattStack<PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>>,
> {
  /// The frames, one for each pending operator.
  stack: S,
  /// Everything but the storage.
  state: ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>,
}

/// Everything of a machine but its storage. It holds no reference, so it moves between native
/// frames by value when a loop re-seats the machine on a storage of another type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power> {
  /// What `poll` answers next, with the value or the reduction that answer holds.
  phase: Phase<V, Post, Power>,
  /// How many frames are pending. It is the machine's own count: the cap, `depth` and every pop
  /// rest on it, and never on the storage's length.
  depth: usize,
  /// The cap on `depth`: `usize::MAX` when none is set.
  max_depth: usize,
  /// The root's floor, inclusive.
  floor: Power,
  /// The root's latch: the power of the non-associative operator the root's last infix reduction
  /// applied, and `None` when that operator was not non-associative.
  root_latch: Option<Power>,
  /// The operator payloads of the frames, which only the storage holds.
  _frames: PhantomData<fn() -> (Pre, LeftAssoc, RightAssoc, NeitherAssoc)>,
}

impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power: Default>
  ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>
{
  /// Nothing pending, no latch, no cap, and the floor at `Power::default()`.
  #[inline]
  fn new() -> Self {
    Self {
      phase: Phase::Term,
      depth: 0,
      max_depth: usize::MAX,
      floor: Power::default(),
      root_latch: None,
      _frames: PhantomData,
    }
  }
}

/// What a machine answers `poll` next.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Phase<V, Post, Power> {
  /// The innermost sub-expression has no value yet.
  Term,
  /// The innermost sub-expression has this value.
  Operator(V),
  /// A reduction or the finished value is owed, and `poll` hands it out.
  Owed(Owed<V, Post, Power>),
  /// A reduction is out, and this is what the machine does once its value comes back.
  Reducing(Resume<Power>),
}

/// What `poll` hands out next.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Owed<V, Post, Power> {
  /// A postfix operator was taken, and its reduction is owed where it was read.
  Postfix(PrattPostfixReduction<V, Post, Power>),
  /// `close` ended the innermost sub-expression, whose value this is, with an operator pending:
  /// that operator's reduction is owed.
  Close(V),
  /// `finish` ended the expression with an operator pending: every pending operator's reduction is
  /// owed, innermost first, and this is the operand of the one on top.
  Finish(V),
  /// The expression is complete, and this is its value.
  Done(V),
}

/// What `handle_reduced` does with the value it is handed.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Resume<Power> {
  /// What the reduction does to the latch of the sub-expression the value belongs to.
  latch: Latch<Power>,
  /// Whether `finish` owed the reduction, so that the next pending operator's is owed after it.
  finishing: bool,
}

/// What a reduction does to the latch of the sub-expression its value belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Latch<Power> {
  /// A prefix and a postfix reduction leave it as it is.
  Keep,
  /// An infix reduction writes it: the operator's power when the operator was non-associative, and
  /// empty otherwise.
  Set(Option<Power>),
}

/// One pending operator on a machine's stack.
///
/// Opaque: it exists so that a machine's storage can be named, as in
/// `ArrayDeque<PrattFrame<..>, U16>`, and only a machine makes or reads one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power = i64> {
  /// The operator, with its left operand when it is infix.
  operator: FrameOperator<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>,
  /// The latch of the sub-expression this frame stands for, which is its operator's operand.
  latch: Option<Power>,
}

/// The operator a frame holds pending.
#[derive(Debug, Clone, PartialEq, Eq)]
enum FrameOperator<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
  /// A prefix operator, waiting for its operand.
  Prefix(Precedenced<Pre, Power>),
  /// An infix operator, waiting for its right operand.
  Infix(InfixFrame<V, LeftAssoc, RightAssoc, NeitherAssoc, Power>),
}

/// An infix operator held pending, with its left operand.
#[derive(Debug, Clone, PartialEq, Eq)]
struct InfixFrame<V, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
  left: V,
  operator: Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>,
}

impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power: Ord>
  PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>
{
  /// Whether the sub-expression this frame stands for admits an operator at `power`. A prefix
  /// operator and a right-associative one admit their own power, and a left- or non-associative
  /// one only a power above it: the family's three floors, read off the frame.
  #[inline]
  fn admits(&self, power: &Power) -> bool {
    match &self.operator {
      FrameOperator::Prefix(prefix) => power >= prefix.precedence(),
      FrameOperator::Infix(infix) => match infix.operator.token_ref() {
        PrattInfix::Right(_) => power >= infix.operator.precedence(),
        PrattInfix::Left(_) | PrattInfix::Neither(_) => power > infix.operator.precedence(),
      },
    }
  }
}

/// What a machine wants next: the answer of [`PrattResolver::poll`].
#[must_use = "a dropped `Reduce` leaves the machine waiting for its value, and a dropped `Done` \
              loses the expression's"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrattPoll<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post = Infallible, Power = i64> {
  /// The innermost sub-expression has no value yet: a prefix operator or an operand is wanted.
  Term,
  /// The innermost sub-expression has a value: an operator is wanted, or an ending.
  Operator,
  /// One reduction is owed, and this is it. Perform it, and hand its value back with
  /// [`handle_reduced`](PrattResolver::handle_reduced).
  Reduce(PrattReduction<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>),
  /// A reduction was handed out, and its value has not come back.
  Reducing,
  /// The expression is complete, and this is its value. The machine is as
  /// [`reset`](PrattResolver::reset) leaves it.
  Done(V),
}

/// What the innermost sub-expression answers an operator at its right-hand position: the answer of
/// [`PrattResolver::offer`].
///
/// The four are the closed set. The innermost sub-expression admits the power or it does not. If it
/// does, its latch is that power or it is not. If it does not, an operator is pending or the
/// sub-expression is the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
pub enum PrattOffer {
  /// It admits the power and, for an infix operator, its latch is not that power. Consume the
  /// operator, then hand it to [`handle_infix`](PrattResolver::handle_infix) or
  /// [`handle_postfix`](PrattResolver::handle_postfix).
  Take,
  /// It does not admit the power, and it is a pending operator's operand. Leave the operator where
  /// it is: [`close`](PrattResolver::close), perform the one reduction, and offer again.
  Reduce,
  /// It does not admit the power, and it is the root. The operator is not this expression's: leave
  /// it, and end the expression with [`close`](PrattResolver::close) or
  /// [`finish`](PrattResolver::finish).
  Decline,
  /// It admits the power, the operator is infix, and its latch is that power: the operator repeats
  /// a non-associative chain. Leave it.
  Chain,
}

/// One reduction: an operator with its operand or operands, handed out by
/// [`PrattResolver::poll`].
///
/// A grammar with no postfix operator leaves `Post` at [`Infallible`], and closes the `Postfix`
/// arm of its `match` with `match postfix.into_components().1.into_data() {}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrattReduction<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post = Infallible, Power = i64>
{
  /// A prefix operator and its operand.
  Prefix(PrattPrefixReduction<V, Pre, Power>),
  /// An infix operator and its two operands.
  Infix(PrattInfixReduction<V, LeftAssoc, RightAssoc, NeitherAssoc, Power>),
  /// A postfix operator and its operand.
  Postfix(PrattPostfixReduction<V, Post, Power>),
}

/// A prefix operator's reduction: the operand and the operator fed, handed back intact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrattPrefixReduction<V, Pre, Power = i64> {
  operand: V,
  operator: Precedenced<Pre, Power>,
}

impl<V, Pre, Power> PrattPrefixReduction<V, Pre, Power> {
  /// The operand and the operator, in the order a prefix fold takes them.
  #[inline]
  pub fn into_components(self) -> (V, Precedenced<Pre, Power>) {
    (self.operand, self.operator)
  }
}

/// An infix operator's reduction: the two operands and the operator fed, handed back intact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrattInfixReduction<V, LeftAssoc, RightAssoc, NeitherAssoc, Power = i64> {
  left: V,
  right: V,
  operator: Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>,
}

impl<V, LeftAssoc, RightAssoc, NeitherAssoc, Power>
  PrattInfixReduction<V, LeftAssoc, RightAssoc, NeitherAssoc, Power>
{
  /// The left operand, the right operand and the operator, in the order an infix fold takes them.
  #[inline]
  pub fn into_components(
    self,
  ) -> (
    V,
    V,
    Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>,
  ) {
    (self.left, self.right, self.operator)
  }
}

/// A postfix operator's reduction: the operand and the operator fed, handed back intact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrattPostfixReduction<V, Post, Power = i64> {
  operand: V,
  operator: Precedenced<Post, Power>,
}

impl<V, Post, Power> PrattPostfixReduction<V, Post, Power> {
  /// The operand and the operator, in the order a postfix fold takes them.
  #[inline]
  pub fn into_components(self) -> (V, Precedenced<Post, Power>) {
    (self.operand, self.operator)
  }
}

/// The pending operator [`PrattResolver::retract`] abandons and hands back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrattPending<Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power = i64> {
  /// A prefix operator that never had its operand.
  Prefix(Precedenced<Pre, Power>),
  /// An infix operator that never had its right operand. Its left operand is the machine's value
  /// again.
  Infix(Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>),
}

/// An event a machine did not take: the item, handed back, and why.
///
/// A refusal modifies nothing: the machine's own state is as it was when the call returns, and
/// over a storage that keeps [`Stack`]'s obligations, its storage is unchanged in everything
/// `Stack`, `Container` and `PartialEq` observe. The item is the one that was fed. It, and what the
/// machine holds, are unchanged given a `Power` whose comparisons change nothing a caller
/// observes, and a clone of the machine taken before the call then compares equal to it afterwards
/// whenever everything the machine holds equals both itself and its own clone (`x == x`,
/// `x.clone() == x`), which `f64::NAN` does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrattRefused<T> {
  why: PrattRefusal,
  item: T,
}

impl<T> PrattRefused<T> {
  #[inline]
  const fn new(why: PrattRefusal, item: T) -> Self {
    Self { why, item }
  }

  /// Why the machine refused.
  #[inline]
  pub const fn why(&self) -> PrattRefusal {
    self.why
  }

  /// The item the machine handed back.
  #[inline]
  pub const fn item(&self) -> &T {
    &self.item
  }

  /// The item the machine handed back, by value.
  #[inline]
  pub fn into_item(self) -> T {
    self.item
  }

  /// Why, and the item.
  #[inline]
  pub fn into_components(self) -> (PrattRefusal, T) {
    (self.why, self.item)
  }
}

/// Why a machine refused an event.
///
/// Not exhaustive: unlike an offer and a poll, a refusal is not something a loop matches whole. It
/// acts on the refusal it expects, and treats the rest as a fault of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
#[non_exhaustive]
pub enum PrattRefusal {
  /// The event is not one that [`poll`](PrattResolver::poll) says is wanted.
  Misplaced,
  /// [`offer`](PrattResolver::offer) answers [`Reduce`](PrattOffer::Reduce) for the operator.
  Reduce,
  /// [`offer`](PrattResolver::offer) answers [`Decline`](PrattOffer::Decline) for the operator.
  Decline,
  /// [`offer`](PrattResolver::offer) answers [`Chain`](PrattOffer::Chain) for the operator.
  Chain,
  /// The stack is at its cap, or its storage refused the frame.
  Full,
}

/// The refusal of an operator the offer does not take, or `None` when it does.
#[inline]
const fn refusal_of(answer: PrattOffer) -> Option<PrattRefusal> {
  match answer {
    PrattOffer::Take => None,
    PrattOffer::Reduce => Some(PrattRefusal::Reduce),
    PrattOffer::Decline => Some(PrattRefusal::Decline),
    PrattOffer::Chain => Some(PrattRefusal::Chain),
  }
}

/// A storage broke [`Stack`]'s obligations.
#[cold]
#[inline(never)]
fn broken_stack(what: &str) -> ! {
  panic!("the machine's storage broke the obligations of `tokora::container::Stack`: {what}")
}

impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
  PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
where
  Power: PrattPower,
  S: Stack<PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>>,
{
  /// A machine over a fresh default of its storage, with the floor at `Power::default()` and no
  /// cap.
  #[inline]
  pub fn new() -> Self
  where
    S: Default,
  {
    Self::with_stack(S::default())
  }

  /// A machine over storage the caller supplies, with the floor at `Power::default()` and no cap.
  ///
  /// It empties the storage first, without trusting it: it reads
  /// [`len`](crate::container::Container::len) once, and pops that many. So a storage whose `pop`
  /// never answers `None` cannot hang it, and one that reports fewer items than it holds keeps the
  /// rest below everything the machine pushes, where the machine never pops.
  ///
  /// A storage the caller supplies follows its own growth policy: the machine answers
  /// [`Full`](PrattRefusal::Full) when a fixed one is full, and one that grows, a lent `Vec` among
  /// them, grows past what was reserved. `&mut S` is a storage too, so a caller that wants no
  /// allocation after the first reserves one, reads its capacity back, and lends it to each
  /// machine with [`max_depth`](Self::max_depth) set to that capacity.
  pub fn with_stack(mut stack: S) -> Self {
    for _ in 0..stack.len() {
      if stack.pop().is_none() {
        break;
      }
    }
    Self {
      stack,
      state: ResolverState::new(),
    }
  }

  /// Sets the root's floor, the minimum binding power, inclusive: an infix or postfix operator below
  /// it is not this expression's. No floor applies to a prefix operator.
  #[must_use]
  #[inline]
  pub fn min_precedence(mut self, min_precedence: Power) -> Self {
    self.state.floor = min_precedence;
    self
  }

  /// Caps the stack at `max_depth` pending operators, whatever the storage could hold: the push
  /// past it is refused with [`Full`](PrattRefusal::Full).
  ///
  /// The cap governs the pushes after it is set. Lowered below a live machine's depth, it drops
  /// nothing, and refuses every push until the depth falls below it; until then
  /// [`depth`](Self::depth) may exceed [`capacity`](Self::capacity), so the room left is
  /// `capacity().saturating_sub(depth())`.
  ///
  /// It bounds the stack's memory, and not the depth of the value the reductions build: a
  /// left-associative chain keeps one operator pending however long it is, and reduces to a value
  /// as deep as the chain.
  #[must_use]
  #[inline]
  pub fn max_depth(mut self, max_depth: usize) -> Self {
    self.state.max_depth = max_depth;
    self
  }

  /// Hands the storage back, with any frames still pending in it.
  #[inline]
  pub fn into_stack(self) -> S {
    self.stack
  }

  /// Drops everything pending, and the value and the latches with it. The floor, the cap and the
  /// storage stay, so a kept machine serves the next expression with the storage it has.
  ///
  /// The machine is as new before any caller code runs: the phase and the root's latch are taken
  /// out first, and dropped after the frames. Then it pops its own count of frames, one at a time,
  /// lowering the count before each is dropped. So a destructor that unwinds leaves the count and
  /// the storage in agreement and no reduction owed, the value and the latch it took out are
  /// dropped on the way, and the next `reset` goes on from there.
  pub fn reset(&mut self) {
    let phase = core::mem::replace(&mut self.state.phase, Phase::Term);
    let root_latch = self.state.root_latch.take();
    while self.state.depth > 0 {
      drop(self.pop_frame());
    }
    drop(root_latch);
    drop(phase);
  }

  /// How many operators are waiting for an operand: the machine's own count.
  #[inline]
  pub const fn depth(&self) -> usize {
    self.state.depth
  }

  /// How many pending operators it can hold: the lesser of the cap and the storage's
  /// [`max_capacity`](crate::container::Container::max_capacity).
  ///
  /// An accessor, and nothing more: no step of the machine reads it. A push is refused when the
  /// count has reached the cap, or when the storage itself refuses the frame.
  #[inline]
  pub fn capacity(&self) -> usize {
    core::cmp::min(self.state.max_depth, self.stack.max_capacity())
  }

  /// What is wanted next.
  ///
  /// [`Term`](PrattPoll::Term), [`Operator`](PrattPoll::Operator) and
  /// [`Reducing`](PrattPoll::Reducing) are reads, and modify nothing. [`Reduce`](PrattPoll::Reduce)
  /// hands out the one reduction that is owed, innermost first, and the next answer is `Reducing`
  /// until its value comes back. [`Done`](PrattPoll::Done) hands out the value and resets: the
  /// machine is then as [`reset`](Self::reset) leaves it, nothing pending and no latch, with the
  /// same floor, cap and storage.
  #[inline]
  pub fn poll(&mut self) -> PrattPoll<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power> {
    match self.state.phase {
      Phase::Term => PrattPoll::Term,
      Phase::Operator(_) => PrattPoll::Operator,
      Phase::Reducing(_) => PrattPoll::Reducing,
      Phase::Owed(_) => self.hand_out(),
    }
  }

  /// Whether the innermost sub-expression takes an operator at its right-hand position. It writes
  /// nothing: it reads the machine's own state and the frame on top of its storage, and compares
  /// powers. So it modifies nothing given a `Power` whose comparisons change nothing a caller
  /// observes.
  ///
  /// `rhs` is what [`PrattToken::try_pratt_rhs`](crate::token::PrattToken::try_pratt_rhs) answers.
  /// The innermost sub-expression admits an operator at `q` when it is:
  ///
  /// | The innermost sub-expression | Admits `q` when |
  /// |---|---|
  /// | the root | `q >= floor` |
  /// | the operand of a prefix operator at `p` | `q >= p` |
  /// | the right operand of a right-associative operator at `p` | `q >= p` |
  /// | the right operand of a left- or non-associative operator at `p` | `q > p` |
  ///
  /// The floor is tested first, then the chain, as the family tests them. Only an infix operator
  /// can meet the chain, and an [`Adjacent`](PrattRHS::Adjacent) one is an infix operator for the
  /// floor and for the latch. [`End`](PrattRHS::End) is no power, so nothing admits it: it is
  /// [`Reduce`](PrattOffer::Reduce) under a pending operator and [`Decline`](PrattOffer::Decline) at
  /// the root. Outside the [`Operator`](PrattPoll::Operator) phase no operator can be taken, and
  /// the answer is `Decline`.
  #[inline]
  pub fn offer(&self, rhs: &PrattRHS<(), (), (), (), Power>) -> PrattOffer {
    if !matches!(self.state.phase, Phase::Operator(_)) {
      return PrattOffer::Decline;
    }
    match rhs {
      PrattRHS::Infix(infix) => self.answer(infix.precedence(), true),
      PrattRHS::Adjacent(adjacent) => self.answer(adjacent.precedence(), true),
      PrattRHS::Postfix(postfix) => self.answer(postfix.precedence(), false),
      PrattRHS::End => self.not_admitted(),
    }
  }

  /// A prefix operator, taken in the [`Term`](PrattPoll::Term) phase, which it leaves `Term`. No
  /// floor applies to it: its frame admits what binds at least as tightly as it does, whatever the
  /// frames below would admit.
  ///
  /// Refused with [`Full`](PrattRefusal::Full) at the cap, or when the storage refuses the frame,
  /// and with [`Misplaced`](PrattRefusal::Misplaced) in any other phase.
  #[inline]
  pub fn handle_prefix(
    &mut self,
    operator: Precedenced<Pre, Power>,
  ) -> Result<(), PrattRefused<Precedenced<Pre, Power>>> {
    if !matches!(self.state.phase, Phase::Term) {
      return Err(PrattRefused::new(PrattRefusal::Misplaced, operator));
    }
    if self.state.depth >= self.state.max_depth {
      return Err(PrattRefused::new(PrattRefusal::Full, operator));
    }
    let frame = PrattFrame {
      operator: FrameOperator::Prefix(operator),
      latch: None,
    };
    match self.stack.push(frame) {
      Ok(()) => {
        // Raised only once the storage has taken the frame. It cannot overflow: it was below
        // `max_depth`, at most `usize::MAX`, a moment ago.
        self.state.depth += 1;
        Ok(())
      }
      Err(PrattFrame {
        operator: FrameOperator::Prefix(operator),
        ..
      }) => Err(PrattRefused::new(PrattRefusal::Full, operator)),
      Err(_) => broken_stack("`push` refused a frame, and handed back another"),
    }
  }

  /// The innermost sub-expression's value, taken in the [`Term`](PrattPoll::Term) phase, which it
  /// leaves [`Operator`](PrattPoll::Operator). Refused with
  /// [`Misplaced`](PrattRefusal::Misplaced) in any other phase.
  #[inline]
  pub fn handle_operand(&mut self, value: V) -> Result<(), PrattRefused<V>> {
    match self.state.phase {
      Phase::Term => {
        self.state.phase = Phase::Operator(value);
        Ok(())
      }
      _ => Err(PrattRefused::new(PrattRefusal::Misplaced, value)),
    }
  }

  /// An infix operator, taken in the [`Operator`](PrattPoll::Operator) phase when
  /// [`offer`](Self::offer) answers [`Take`](PrattOffer::Take) for it: it is pushed, with the value
  /// as its left operand, and the phase is [`Term`](PrattPoll::Term).
  ///
  /// The event checks the offer itself, so the machine cannot be handed an operator its innermost
  /// sub-expression does not admit: an answer other than `Take` is the refusal, as
  /// [`Reduce`](PrattRefusal::Reduce), [`Decline`](PrattRefusal::Decline) or
  /// [`Chain`](PrattRefusal::Chain). It is refused with [`Full`](PrattRefusal::Full) at the cap or
  /// when the storage refuses the frame, and the value stays the machine's, so the operator may be
  /// offered again once a reduction has made room. In any other phase it is
  /// [`Misplaced`](PrattRefusal::Misplaced).
  ///
  /// A zero-token operator, [`Adjacent`](PrattRHS::Adjacent), is fed here as
  /// [`PrattInfix::Left`]: so it never arms the latch.
  #[inline]
  pub fn handle_infix(
    &mut self,
    operator: Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>,
  ) -> Result<(), PrattRefused<Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>>>
  {
    if !matches!(self.state.phase, Phase::Operator(_)) {
      return Err(PrattRefused::new(PrattRefusal::Misplaced, operator));
    }
    if let Some(why) = refusal_of(self.answer(operator.precedence(), true)) {
      return Err(PrattRefused::new(why, operator));
    }
    if self.state.depth >= self.state.max_depth {
      return Err(PrattRefused::new(PrattRefusal::Full, operator));
    }
    // The phase was tested above, and every check that can refuse ran before this move.
    let Some(left) = self.take_value() else {
      return Err(PrattRefused::new(PrattRefusal::Misplaced, operator));
    };
    let frame = PrattFrame {
      operator: FrameOperator::Infix(InfixFrame { left, operator }),
      latch: None,
    };
    match self.stack.push(frame) {
      Ok(()) => {
        // Raised only once the storage has taken the frame, and bounded as in `handle_prefix`.
        self.state.depth += 1;
        Ok(())
      }
      Err(PrattFrame {
        operator: FrameOperator::Infix(InfixFrame { left, operator }),
        ..
      }) => {
        self.state.phase = Phase::Operator(left);
        Err(PrattRefused::new(PrattRefusal::Full, operator))
      }
      Err(_) => broken_stack("`push` refused a frame, and handed back another"),
    }
  }

  /// A postfix operator, taken in the [`Operator`](PrattPoll::Operator) phase when
  /// [`offer`](Self::offer) answers [`Take`](PrattOffer::Take) for it. It is on no frame: its
  /// reduction is owed where it is read, and `poll` hands it out next.
  ///
  /// A postfix operator never meets the chain, so one at the latched power is taken. It is refused
  /// with the offer's own answer, [`Reduce`](PrattRefusal::Reduce) or
  /// [`Decline`](PrattRefusal::Decline), and with [`Misplaced`](PrattRefusal::Misplaced) in any
  /// other phase.
  #[inline]
  pub fn handle_postfix(
    &mut self,
    operator: Precedenced<Post, Power>,
  ) -> Result<(), PrattRefused<Precedenced<Post, Power>>> {
    if !matches!(self.state.phase, Phase::Operator(_)) {
      return Err(PrattRefused::new(PrattRefusal::Misplaced, operator));
    }
    if let Some(why) = refusal_of(self.answer(operator.precedence(), false)) {
      return Err(PrattRefused::new(why, operator));
    }
    // The phase was tested above.
    let Some(operand) = self.take_value() else {
      return Err(PrattRefused::new(PrattRefusal::Misplaced, operator));
    };
    self.state.phase = Phase::Owed(Owed::Postfix(PrattPostfixReduction { operand, operator }));
    Ok(())
  }

  /// The value of the reduction `poll` handed out, taken in the [`Reducing`](PrattPoll::Reducing)
  /// phase: it is the innermost sub-expression's value. Refused with
  /// [`Misplaced`](PrattRefusal::Misplaced) in any other phase.
  ///
  /// It applies the latch rule. After an infix reduction, the latch of the sub-expression the value
  /// belongs to is the operator's power when the operator was non-associative, and empty
  /// otherwise. A prefix and a postfix reduction leave it alone. So `a == b < c` is refused at the
  /// `<` when both are at one power and `==` is non-associative, and `a < b == c` is not.
  #[inline]
  pub fn handle_reduced(&mut self, value: V) -> Result<(), PrattRefused<V>> {
    if !matches!(self.state.phase, Phase::Reducing(_)) {
      return Err(PrattRefused::new(PrattRefusal::Misplaced, value));
    }
    // The phase was tested above.
    let Resume { latch, finishing } = match core::mem::replace(&mut self.state.phase, Phase::Term) {
      Phase::Reducing(resume) => resume,
      phase => {
        self.state.phase = phase;
        return Err(PrattRefused::new(PrattRefusal::Misplaced, value));
      }
    };
    if let Latch::Set(latch) = latch {
      self.set_latch(latch);
    }
    self.state.phase = if !finishing {
      Phase::Operator(value)
    } else if self.state.depth > 0 {
      Phase::Owed(Owed::Finish(value))
    } else {
      Phase::Owed(Owed::Done(value))
    };
    Ok(())
  }

  /// The innermost sub-expression ends here. Taken in the [`Operator`](PrattPoll::Operator) phase,
  /// and refused with [`Misplaced`](PrattRefusal::Misplaced) in any other.
  ///
  /// With an operator pending, exactly one reduction is owed, that operator's, and after its value
  /// comes back the phase is `Operator` again, one sub-expression out. With none pending, the
  /// innermost sub-expression is the root, and `poll` answers [`Done`](PrattPoll::Done).
  ///
  /// It is also how a caller acts on [`PrattOffer::Reduce`]: close, perform the one reduction, and
  /// offer again.
  #[inline]
  pub fn close(&mut self) -> Result<(), PrattRefusal> {
    self.end(false)
  }

  /// The expression ends here. Taken in the [`Operator`](PrattPoll::Operator) phase, and refused
  /// with [`Misplaced`](PrattRefusal::Misplaced) in any other.
  ///
  /// Every pending operator's reduction is owed, innermost first, one for each `poll`, and then
  /// `poll` answers [`Done`](PrattPoll::Done).
  #[inline]
  pub fn finish(&mut self) -> Result<(), PrattRefusal> {
    self.end(true)
  }

  /// Abandons the pending operator, and hands it back: the machine's half of recovery, for when an
  /// operator's operand never comes. Taken in the [`Term`](PrattPoll::Term) phase with an operator
  /// pending; otherwise it answers `None` and modifies nothing.
  ///
  /// A prefix operator comes back as [`PrattPending::Prefix`], and the phase is still `Term`, one
  /// sub-expression out. An infix operator comes back as [`PrattPending::Infix`]: its left operand
  /// is the value again, the phase is [`Operator`](PrattPoll::Operator), and the sub-expression's
  /// latch is what it was before the operator was taken. So after `1 + )`, `retract` and `finish`
  /// give `1`.
  pub fn retract(
    &mut self,
  ) -> Option<PrattPending<Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>> {
    if !matches!(self.state.phase, Phase::Term) || self.state.depth == 0 {
      return None;
    }
    let PrattFrame { operator, latch: _ } = self.pop_frame();
    match operator {
      FrameOperator::Prefix(operator) => Some(PrattPending::Prefix(operator)),
      FrameOperator::Infix(InfixFrame { left, operator }) => {
        self.state.phase = Phase::Operator(left);
        Some(PrattPending::Infix(operator))
      }
    }
  }

  /// `close` and `finish`: one reduction owed under a pending operator, or every one, and the value
  /// at the root.
  #[inline]
  fn end(&mut self, finishing: bool) -> Result<(), PrattRefusal> {
    let Some(value) = self.take_value() else {
      return Err(PrattRefusal::Misplaced);
    };
    self.state.phase = Phase::Owed(if self.state.depth == 0 {
      Owed::Done(value)
    } else if finishing {
      Owed::Finish(value)
    } else {
      Owed::Close(value)
    });
    Ok(())
  }

  /// What the `Owed` phase holds, moved out.
  #[inline]
  fn hand_out(&mut self) -> PrattPoll<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power> {
    match core::mem::replace(&mut self.state.phase, Phase::Term) {
      Phase::Owed(Owed::Postfix(reduction)) => {
        self.state.phase = Phase::Reducing(Resume {
          latch: Latch::Keep,
          finishing: false,
        });
        PrattPoll::Reduce(PrattReduction::Postfix(reduction))
      }
      Phase::Owed(Owed::Close(right)) => self.reduce_top(right, false),
      Phase::Owed(Owed::Finish(right)) => self.reduce_top(right, true),
      Phase::Owed(Owed::Done(value)) => {
        // `Done` resets: nothing is pending, and the phase is `Term` already.
        self.state.root_latch = None;
        PrattPoll::Done(value)
      }
      // `poll` calls this in the `Owed` phase alone. Any other phase is put back as it was, and
      // answered as `poll` answers it.
      Phase::Term => PrattPoll::Term,
      phase @ Phase::Operator(_) => {
        self.state.phase = phase;
        PrattPoll::Operator
      }
      phase @ Phase::Reducing(_) => {
        self.state.phase = phase;
        PrattPoll::Reducing
      }
    }
  }

  /// The reduction of the operator on top, whose operand is `right`.
  ///
  /// Reached from an `Owed::Close` or `Owed::Finish` alone, and each is made only with an operator
  /// pending, by `end` and by `handle_reduced`. Nothing pops while one is owed: every event but
  /// `poll` is refused in that phase, and `reset` takes the phase out before it pops, so not even an
  /// unwind out of it leaves one owed over frames it has let go of.
  #[inline]
  fn reduce_top(
    &mut self,
    right: V,
    finishing: bool,
  ) -> PrattPoll<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power> {
    let PrattFrame { operator, latch: _ } = self.pop_frame();
    match operator {
      FrameOperator::Prefix(operator) => {
        self.state.phase = Phase::Reducing(Resume {
          latch: Latch::Keep,
          finishing,
        });
        PrattPoll::Reduce(PrattReduction::Prefix(PrattPrefixReduction {
          operand: right,
          operator,
        }))
      }
      FrameOperator::Infix(InfixFrame { left, operator }) => {
        let latch = match operator.token_ref() {
          PrattInfix::Neither(_) => Some(operator.precedence().clone()),
          PrattInfix::Left(_) | PrattInfix::Right(_) => None,
        };
        self.state.phase = Phase::Reducing(Resume {
          latch: Latch::Set(latch),
          finishing,
        });
        PrattPoll::Reduce(PrattReduction::Infix(PrattInfixReduction {
          left,
          right,
          operator,
        }))
      }
    }
  }

  /// What the innermost sub-expression answers an operator at `power`: the floor first, then the
  /// chain, which only an infix operator can meet.
  #[inline]
  fn answer(&self, power: &Power, infix: bool) -> PrattOffer {
    let (admits, latch) = match self.top() {
      None => (*power >= self.state.floor, self.state.root_latch.as_ref()),
      Some(frame) => (frame.admits(power), frame.latch.as_ref()),
    };
    if !admits {
      self.not_admitted()
    } else if infix && latch == Some(power) {
      PrattOffer::Chain
    } else {
      PrattOffer::Take
    }
  }

  /// The answer to an operator the innermost sub-expression does not admit: under a pending
  /// operator it is that operator's to reduce, and at the root it is not this expression's.
  #[inline]
  fn not_admitted(&self) -> PrattOffer {
    if self.state.depth > 0 {
      PrattOffer::Reduce
    } else {
      PrattOffer::Decline
    }
  }

  /// The frame whose operand is the innermost sub-expression, or `None` at the root. Read by the
  /// machine's own count, and never by the storage's length.
  #[inline]
  fn top(&self) -> Option<&PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>> {
    if self.state.depth == 0 {
      return None;
    }
    match self.stack.last() {
      Some(frame) => Some(frame),
      None => broken_stack("`last` answered `None` with a frame pending"),
    }
  }

  /// Writes the latch of the innermost sub-expression: the root's, or the frame's on top.
  #[inline]
  fn set_latch(&mut self, latch: Option<Power>) {
    if self.state.depth == 0 {
      self.state.root_latch = latch;
      return;
    }
    match self.stack.last_mut() {
      Some(frame) => frame.latch = latch,
      None => broken_stack("`last_mut` answered `None` with a frame pending"),
    }
  }

  /// The value of the innermost sub-expression, moved out of the `Operator` phase, which this
  /// leaves `Term`. In any other phase it changes nothing, and answers `None`.
  #[inline]
  fn take_value(&mut self) -> Option<V> {
    match core::mem::replace(&mut self.state.phase, Phase::Term) {
      Phase::Operator(value) => Some(value),
      phase => {
        self.state.phase = phase;
        None
      }
    }
  }

  /// The frame on top, popped. The count is lowered as soon as the storage gives the frame up, and
  /// before the frame or a value in it can be dropped.
  ///
  /// Called with a frame pending, by its three callers' own tests: `reset`'s loop, `retract`'s,
  /// and the `Owed` phase `reduce_top` is reached from. So the count cannot wrap.
  #[inline]
  fn pop_frame(&mut self) -> PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
    debug_assert!(self.state.depth > 0, "a frame popped with none pending");
    match self.stack.pop() {
      Some(frame) => {
        self.state.depth -= 1;
        frame
      }
      None => broken_stack("`pop` answered `None` with a frame pending"),
    }
  }
}

// The re-seat that lets a loop go on over another storage. The loops that grow a default stack on
// the native stack, a segment at a time, split the machine here and seat it on the next segment, or
// back on their own. Crate-private, since a public `from_parts` could seat a machine on a storage
// that does not hold its frames.
//
// `allow(dead_code)` outside tests: those loops are adapters', and a build that compiles none of
// them reaches these two from the machine's tests alone.
#[cfg_attr(not(test), allow(dead_code))]
impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
  PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
{
  /// Everything but the storage, and the storage.
  #[inline]
  pub(crate) fn into_parts(
    self,
  ) -> (
    ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>,
    S,
  ) {
    (self.state, self.stack)
  }

  /// The machine `into_parts` split, seated on `stack`. The frames `state` counts must be in
  /// `stack`, or reachable through it.
  #[inline]
  pub(crate) fn from_parts(
    state: ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>,
    stack: S,
  ) -> Self {
    Self { stack, state }
  }
}

impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S> Default
  for PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
where
  Power: PrattPower,
  S: Stack<PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>> + Default,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests;
