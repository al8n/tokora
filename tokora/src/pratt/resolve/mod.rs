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
/// [`PrattRefusal`]. Nothing in the machine panics on a caller's slip, but for one a debug build
/// may panic on: going on after a caught unwind without the [`reset`](Self::reset) that the unwind
/// contract below asks for. Otherwise it panics only when its storage breaks `Stack`'s
/// obligations, with a message that names `Stack`.
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
/// - `Power`'s comparisons; its `Clone` where an infix reduction sets a latch, where a push keeps
///   the floor its operator gives its operand, and where a pop changes the innermost
///   sub-expression, whose floor and latch the machine keeps a copy of beside its frames so that
///   an offer reads no frame; and its `Default` in [`new`](Self::new) and
///   [`with_stack`](Self::with_stack). [`handle_prefix`](Self::handle_prefix) and
///   [`handle_infix`](Self::handle_infix) clone that floor once the storage has taken the frame.
///   The pushes the crate's own drivers make clone it before the storage takes the frame, and a
///   refused one drops that clone. So a push and a pop each clone at most one power, the floor
///   kept, a pop clones the latch it reads too while a latch is set, and a non-associative
///   reduction clones its latch twice, for where it lives and for the copy: a power whose `Clone`
///   costs more than a copy makes each of them cost that much, the price of an offer that reads no
///   frame;
/// - the methods of its storage;
/// - the destructors of `V`, of the operator payloads and of `Power`, when [`reset`](Self::reset),
///   [`with_stack`](Self::with_stack), [`into_stack`](Self::into_stack) or the machine's own drop
///   lets go of a frame or a value; and `Power`'s alone where a latch, the floor or a copy the
///   machine keeps is overwritten, and where a reduction or a [`retract`](Self::retract) lets go
///   of a frame's own latch;
/// - in its derived `Clone`, `PartialEq` and `Debug`, those traits of `S` and of each parameter.
///   Its `PartialEq` leaves out the copies it keeps beside its frames: read off the frames, they
///   would tell two machines apart where a storage that compares coarser than its frames calls
///   them equal.
///
/// An unwind out of any of them leaves the machine's count and its storage in agreement: a push
/// raises the count only once the storage has taken the frame, and a pop lowers it as soon as the
/// storage gives the frame up, before the frame or a value in it is dropped. It also leaves a phase
/// that claims no frame the machine has let go of: [`reset`](Self::reset) installs the new phase
/// before it drops anything, and a step whose value an unwind takes leaves the
/// [`Term`](PrattPoll::Term) phase, where that sub-expression starts over. A latch stays where it
/// lives, as it was, and a push that unwinds moves none, but what the machine keeps beside its
/// frames, copies of the innermost sub-expression's floor and latch and a count of the latches
/// set, may not be put back as they were, so a caller that goes on after an unwind it caught calls
/// [`reset`](Self::reset), which returns the machine to a known state. Which of that code has run
/// when an unwind leaves an event, the order one event runs it in, and how the machine compares
/// with another before that `reset`, are not promised. Until that `reset` an event may answer
/// wrongly, and in a debug build it may panic, at a check of the floor the machine keeps.
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
///
/// Its `PartialEq` is written out, over what the machine compared before it kept copies beside its
/// frames; `Debug` shows the copies too.
#[derive(Debug, Clone)]
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
  /// applied, and `None` when that operator was not non-associative. A frame holds the latch of its
  /// own operand, as this holds the root's, so the storage holds every latch below the root.
  root_latch: Option<Power>,
  /// A copy of the innermost sub-expression's latch, kept so that an offer reads no frame: the
  /// root's latch at the root, and otherwise the one the frame on top holds. Each push, each pop
  /// and each latch the rule writes keeps it so, and the latches themselves stay where they live.
  /// Equality leaves it out.
  latch: Option<Power>,
  /// How many of the latches the machine holds are set: the root's, and those of the frames below
  /// its count. A push gives its frame no latch, and each latch the rule writes, each pop, a `Done`
  /// and `reset` keep it exact on every state no unwind interrupted. No unwind leaves it below the
  /// true count: a write is counted with no caller code between the two, and a latch is uncounted
  /// only once it is out of the machine, its frame popped or the whole machine emptied. So at zero
  /// no latch is set: a pop reads no latch off the frame left on top, and the rule's `None` writes
  /// the copy alone. Equality leaves it out.
  latches: usize,
  /// The floor the frame on top gives its operand, kept so that an offer reads no frame: cloned out
  /// of the operator by each push the adapters feed, before the storage is asked, read back off
  /// the frame by each public push once the storage has taken it, and read off the frame each pop
  /// leaves on top. `None` at the root, whose floor is `floor` itself, so that `min_precedence` has
  /// nothing here to change. Equality leaves it out.
  frame_floor: Option<Floor<Power>>,
  /// The operator payloads of the frames, which only the storage holds.
  _frames: PhantomData<fn() -> (Pre, LeftAssoc, RightAssoc, NeitherAssoc)>,
}

/// Equal over the phase, the count, the cap, the root's floor and the root's latch, which is what
/// the machine compared before it kept copies beside its frames. The copies of the innermost floor
/// and latch, and the count of latches set, are read off the frames, and a `Stack` may compare
/// coarser than its frames: compared here, they would tell apart two machines that the storage and
/// everything else call equal.
impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power> PartialEq
  for ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>
where
  V: PartialEq,
  Post: PartialEq,
  Power: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.phase == other.phase
      && self.depth == other.depth
      && self.max_depth == other.max_depth
      && self.floor == other.floor
      && self.root_latch == other.root_latch
      && self._frames == other._frames
  }
}

impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power> Eq
  for ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>
where
  V: Eq,
  Post: Eq,
  Power: Eq,
{
}

/// What a pending operator's operand admits: a power above `power`, and `power` itself too unless
/// `strict`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Floor<Power> {
  power: Power,
  strict: bool,
}

impl<Power: Ord> Floor<Power> {
  /// Whether the operand admits an operator at `power`.
  #[inline]
  fn admits(&self, power: &Power) -> bool {
    if self.strict {
      *power > self.power
    } else {
      *power >= self.power
    }
  }
}

impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power: Default>
  ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>
{
  /// Nothing pending, no latch, no cap, and the floor at `Power::default()`.
  #[inline]
  fn new() -> Self {
    Self::floored(Power::default())
  }
}

impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>
  ResolverState<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>
{
  /// Nothing pending, no latch, no cap, and the floor at `floor`: no `Power` is built here.
  #[inline]
  fn floored(floor: Power) -> Self {
    Self {
      phase: Phase::Term,
      depth: 0,
      max_depth: usize::MAX,
      floor,
      root_latch: None,
      latch: None,
      latches: 0,
      frame_floor: None,
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

impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>
  FrameOperator<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>
{
  /// The floor this operator gives its operand, as a power and whether it is strict. A prefix
  /// operator and a right-associative one admit their own power, and a left- or non-associative
  /// one only a power above it: the family's three floors, read off the operator, and spelled
  /// nowhere else in the machine.
  #[inline]
  fn floor_parts(&self) -> (&Power, bool) {
    match self {
      FrameOperator::Prefix(prefix) => (prefix.precedence(), false),
      FrameOperator::Infix(infix) => match infix.operator.token_ref() {
        PrattInfix::Right(_) => (infix.operator.precedence(), false),
        PrattInfix::Left(_) | PrattInfix::Neither(_) => (infix.operator.precedence(), true),
      },
    }
  }

  /// The floor this operator gives its operand, with its power cloned for the machine to keep.
  #[inline]
  fn floor(&self) -> Floor<Power>
  where
    Power: Clone,
  {
    let (power, strict) = self.floor_parts();
    Floor {
      power: power.clone(),
      strict,
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

/// The latch rule, for an infix reduction: the latch it leaves on the sub-expression its value
/// belongs to is the operator's power when the operator is non-associative, and none otherwise. A
/// prefix and a postfix reduction leave that latch as it is. The public path and the adapters' fast
/// close both apply it through this.
#[inline]
fn infix_latch<LeftAssoc, RightAssoc, NeitherAssoc, Power: Clone>(
  operator: &Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>,
) -> Option<Power> {
  match operator.token_ref() {
    PrattInfix::Neither(_) => Some(operator.precedence().clone()),
    PrattInfix::Left(_) | PrattInfix::Right(_) => None,
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
  pub fn with_stack(stack: S) -> Self {
    Self {
      stack: Self::emptied(stack),
      state: ResolverState::new(),
    }
  }

  /// [`with_stack`](Self::with_stack) with the root's floor at a clone of `floor`, inclusive: for
  /// an adapter whose caller has named the floor, which [`min_precedence`](Self::min_precedence)
  /// would set only once a `Power::default()` had been built and dropped. No default is built
  /// here, and the clone is taken once the storage is emptied, so the frames a parse that unwound
  /// left there are dropped before it.
  #[inline]
  pub(crate) fn with_stack_and_floor(stack: S, floor: &Power) -> Self {
    let stack = Self::emptied(stack);
    Self {
      stack,
      state: ResolverState::floored(floor.clone()),
    }
  }

  /// `stack`, emptied without trusting it: its length read once, and that many popped.
  #[inline]
  fn emptied(mut stack: S) -> S {
    for _ in 0..stack.len() {
      if stack.pop().is_none() {
        break;
      }
    }
    stack
  }

  /// Sets the root's floor, the minimum binding power, inclusive: an infix or postfix operator below
  /// it is not this expression's. No floor applies to a prefix operator.
  ///
  /// The floor is a value, as [`PrattPower`] asks of every power: the machine compares it with the
  /// powers it is offered, a clone of it compares as it does, and its order does not change while
  /// the machine holds it. When the machine drops it, and the powers it holds, is unspecified.
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
  /// The machine is as new before any caller code runs: the phase, the root's latch, and the copies
  /// of the innermost floor and latch it keeps beside its frames, are taken out first, and dropped
  /// after the frames. Then it pops its own count of frames, one at a time, lowering the count
  /// before each is dropped, and clears its count of latches set once they are all gone. So a
  /// destructor that unwinds leaves the count and the storage in agreement and no reduction owed,
  /// the value and what it took out are dropped on the way, and the next `reset` goes on from
  /// there.
  pub fn reset(&mut self) {
    let phase = core::mem::replace(&mut self.state.phase, Phase::Term);
    let root_latch = self.state.root_latch.take();
    let latch = self.state.latch.take();
    let frame_floor = self.state.frame_floor.take();
    while self.state.depth > 0 {
      drop(self.pop_raw());
    }
    // Every latch is gone, the root's taken out above and each frame's with its frame. Cleared only
    // now, so that a destructor that unwinds above leaves the count no lower than what is set.
    self.state.latches = 0;
    drop(frame_floor);
    drop(latch);
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
  /// nothing: it reads the machine's own state, which keeps a copy of the floor the operator on top
  /// gives its operand and of the innermost latch, and compares powers, and a debug build reads
  /// that frame back too, to check the floor kept. So it modifies nothing given a `Power` whose
  /// comparisons change nothing a caller observes.
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
        self.pushed();
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
        self.pushed();
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
        // `Done` resets, and the phase is `Term` already.
        self.clear_done();
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
        self.state.phase = Phase::Reducing(Resume {
          latch: Latch::Set(infix_latch(&operator)),
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
  /// chain, which only an infix operator can meet. It reads the floor and the latch the machine
  /// keeps, and no frame.
  #[inline]
  fn answer(&self, power: &Power, infix: bool) -> PrattOffer {
    debug_assert!(
      self.floor_is_kept(),
      "the floor kept for the operator on top is not the one it gives its operand: a defect of \
       the machine, a storage that broke the obligations of `tokora::container::Stack`, or a \
       caller that went on after a caught unwind without `reset`"
    );
    let admits = match &self.state.frame_floor {
      None => *power >= self.state.floor,
      Some(floor) => floor.admits(power),
    };
    if !admits {
      self.not_admitted()
    } else if infix && self.state.latch.as_ref() == Some(power) {
      PrattOffer::Chain
    } else {
      PrattOffer::Take
    }
  }

  /// Whether the floor kept is the one the frame on top gives its operand, read back off the
  /// storage, and none is kept at the root: what every push and every pop keep true.
  fn floor_is_kept(&self) -> bool {
    match (&self.state.frame_floor, self.top()) {
      (None, None) => true,
      (Some(kept), Some(frame)) => {
        let (power, strict) = frame.operator.floor_parts();
        kept.strict == strict && kept.power == *power
      }
      (None, Some(_)) | (Some(_), None) => false,
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

  /// Writes the latch of the innermost sub-expression where it lives, the root's or the one the
  /// frame on top holds for its operand, and the copy the machine keeps. With no latch set,
  /// `None` is written to the copy alone: every latch is `None` already. That is the inlined part,
  /// and the write where a latch is set or is being set is [`write_latch`](Self::write_latch), out
  /// of line, so that a reduction inlined into an adapter's loop stays small.
  #[inline]
  fn set_latch(&mut self, latch: Option<Power>) {
    if latch.is_none() && self.state.latches == 0 {
      // The copy is written still: after a pop it may be the latch of the frame popped.
      self.state.latch = None;
    } else {
      self.write_latch(latch);
    }
  }

  /// [`set_latch`](Self::set_latch) where a latch is set or is being set. The copy is cloned
  /// first, so an unwind out of that clone writes neither.
  #[cold]
  #[inline(never)]
  fn write_latch(&mut self, latch: Option<Power>) {
    let copy = latch.clone();
    let set = latch.is_some();
    let old = if self.state.depth == 0 {
      core::mem::replace(&mut self.state.root_latch, latch)
    } else {
      match self.stack.last_mut() {
        Some(frame) => core::mem::replace(&mut frame.latch, latch),
        None => broken_stack("`last_mut` answered `None` with a frame pending"),
      }
    };
    // Counted as the write happens, and before the latch it replaced is dropped.
    match (old.is_some(), set) {
      (false, true) => {
        self.state.latches = match self.state.latches.checked_add(1) {
          Some(latches) => latches,
          None => broken_stack("a frame came back twice, or lost a latch written on it"),
        };
      }
      (true, false) => {
        self.state.latches = match self.state.latches.checked_sub(1) {
          Some(latches) => latches,
          None => broken_stack("a latch the machine had already let go of came back on a frame"),
        };
      }
      _ => {}
    }
    self.state.latch = copy;
    drop(old);
  }

  /// The floor the frame on top gives its operand, cloned for the machine to keep, or `None` at
  /// the root, whose floor is `floor`.
  #[inline]
  fn floor_of_top(&self) -> Option<Floor<Power>> {
    self.top().map(|frame| frame.operator.floor())
  }

  /// A public push's last step, once the storage has taken the frame: the count is raised before
  /// any code of the power's runs, and the copies the machine keeps are of the new operand, which
  /// has no latch yet and the floor the frame gives, read back off it. So a refusal runs no code of
  /// the power's but its comparisons.
  #[inline]
  fn pushed(&mut self) {
    // It cannot overflow: the count was below `max_depth`, at most `usize::MAX`, a moment ago.
    self.state.depth += 1;
    self.state.latch = None;
    self.state.frame_floor = self.floor_of_top();
  }

  /// Keeps a copy of what the innermost sub-expression is, read where it lives: the floor the frame
  /// on top gives its operand and the latch that frame holds for it, or, at the root, no floor and
  /// the root's latch.
  #[inline]
  fn keep_innermost(&mut self) {
    self.state.frame_floor = self.floor_of_top();
    self.keep_latch();
  }

  /// Keeps a copy of the innermost sub-expression's latch, read where it lives. With no latch set,
  /// the latch is `None` wherever it lives, and no frame is read for it.
  #[inline(always)]
  fn keep_latch(&mut self) {
    self.state.latch = if self.state.latches == 0 {
      None
    } else {
      self.innermost_latch()
    };
  }

  /// What a `Done` clears, `poll`'s and the fast close's alike: nothing is pending, so no floor is
  /// kept, and the root's latch, the only one, its copy and the count of latches set are cleared,
  /// the count last, once the latch it counted is gone.
  #[inline(always)]
  fn clear_done(&mut self) {
    self.state.root_latch = None;
    self.state.latch = None;
    self.state.latches = 0;
  }

  /// The innermost sub-expression's latch, cloned where it lives: the one the frame on top holds,
  /// or the root's. Out of line, as [`write_latch`](Self::write_latch) is.
  #[cold]
  #[inline(never)]
  fn innermost_latch(&self) -> Option<Power> {
    match self.top() {
      None => self.state.root_latch.clone(),
      Some(frame) => frame.latch.clone(),
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

  /// The frame on top, popped, with the sub-expression below it innermost again, and the machine's
  /// copies of its floor and latch read where they live. The frame comes back with its own latch,
  /// for the caller to let go of.
  ///
  /// Called with a frame pending, by its two callers' own tests: `retract`'s, and the `Owed` phase
  /// `reduce_top` is reached from.
  #[inline]
  fn pop_frame(&mut self) -> PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
    let frame = self.pop_raw();
    self.forget_latch(&frame.latch);
    self.keep_innermost();
    frame
  }

  /// Uncounts the latch of a frame popped, if it is set: the frame is out of the storage, and the
  /// count is lowered before the frame can be dropped.
  #[inline]
  fn forget_latch(&mut self, latch: &Option<Power>) {
    if latch.is_some() {
      // Wrapping, so that this inlined path stays one subtraction, with no branch and no call,
      // and cannot panic in a debug build. Only a storage that hands a frame back twice uncounts
      // a latch twice. From zero the count wraps to `usize::MAX`, too high, which only turns its
      // shortcut off, so every latch is read where it lives, and the next latch counted overflows
      // into `broken_stack` in `write_latch`. From above zero it ends low, which can make an
      // answer wrong, as a storage that breaks `Stack` may, until `write_latch` meets a latch the
      // count no longer holds, and names `Stack` too.
      self.state.latches = self.state.latches.wrapping_sub(1);
    }
  }

  /// The frame on top, popped, and nothing else moved: `reset`'s pop, which lets go of the copies
  /// it keeps itself, and the first step of every other. The count is lowered as soon as the
  /// storage gives the frame up, and before the frame or a value in it can be dropped.
  ///
  /// Called with a frame pending: by `reset`'s loop and `pop_frame`'s callers, which test it, by
  /// `close_and_reduce` after its test of the depth, and by `reduce_pending`, where the offer's
  /// `Reduce` holds it, since an offer answers `Reduce` only under a pending operator. So the
  /// count cannot wrap.
  #[inline]
  fn pop_raw(&mut self) -> PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
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

/// What [`close_and_reduce`](PrattResolver::close_and_reduce) hands back: the one reduction the end
/// of the innermost sub-expression owes, or the expression's value.
pub(crate) enum Closed<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
  /// It was a prefix operator's operand: that operator's reduction.
  Prefix(PrattPrefixReduction<V, Pre, Power>),
  /// It was an infix operator's right operand: that operator's reduction.
  Infix(PrattInfixReduction<V, LeftAssoc, RightAssoc, NeitherAssoc, Power>),
  /// It was the root: the expression's value. The machine is as
  /// [`reset`](PrattResolver::reset) leaves it.
  Done(V),
}

/// The events an adapter of this crate feeds where its own loop already knows what the public
/// events test: the flat driver's two loops, which know where they stand, and offer inside the
/// read that commits an operator.
///
/// # The phase protocol
///
/// Each is a public event with the tests that the adapter's own structure makes true left out.
/// A push keeps the two tests the input can fail, the cap and the storage's room, and every other
/// test is a debug assertion. In a release build an adapter's slip goes untested, but for an event
/// that finds no value to move out, which panics.
///
/// - **`Term` and `Operator` alone.** Nothing an adapter feeds through these makes its machine owe
///   a reduction for `poll` to hand out: an event that reduces hands its reduction back. So the
///   adapter meets the `Term` and `Operator` phases, and `Owed` and `Reducing` stay the public
///   protocol's.
/// - **What each assumes.** `put_operand` and `push_prefix` assume `Term`, and leave `Operator` and
///   `Term`. `push_infix`, `take_postfix` and `close_and_reduce` assume `Operator`, and the first
///   two that the adapter's own offer answered `Take` for the operator with nothing fed since.
///   `push_infix` leaves `Term`. `reduce_pending` is `close_and_reduce` where that offer answered
///   `Reduce` instead, so an operator is pending.
/// - **A fold runs in `Term`.** `take_postfix` and `close_and_reduce` move the value out into the
///   reduction they hand back, which leaves `Term`. The adapter folds, and `put_reduced` puts the
///   value back, which leaves `Operator`.
/// - **The latch is applied at the pop.** Under a pending operator `close_and_reduce` pops its
///   frame and applies the latch rule there, before the fold, where the public path applies it in
///   `handle_reduced` once the value is back. It is the same rule, `infix_latch`, at an earlier
///   point: during the fold the machine holds one frame fewer, and the latch already written where
///   it lives, on the frame now on top or as the root's.
/// - **Depth 0 is the close's own case.** At the root nothing is pending: `close_and_reduce` pops
///   nothing, and hands the value out as `Done` with the root's latch cleared, as `poll`'s `Done`
///   does, so the machine is as `reset` leaves it. It never reaches `pop_raw` with no frame
///   pending.
/// - **Postfix.** `take_postfix` pops nothing either. Its operator is on no frame, so the depth is
///   as it was, and a postfix reduction leaves the latch as it is, which is the rule.
/// - **Why no input reaches an event in the wrong phase.** An adapter feeds each event only at the
///   point of its loop where the phase that event assumes holds: at a term, where the loop is when
///   it starts and after each push, or at an operator, where it is after each value it puts. What
///   the input decides is which event comes next at that point, a term, an operator or an ending,
///   and never the phase. The offer that admits an operator is asked in the read that commits it,
///   with nothing fed between the two. So a slip is a defect of the adapter's code, which a debug
///   test build catches, and no input can make one.
/// - **After an error or an unwind out of a fold** the frame is popped and the latch written,
///   where the public path still holds the latch back in `Reducing`. No adapter goes on with such
///   a machine: `folding`'s lives for one parse and is dropped with its error or its unwind, and
///   `parse_into`'s is reset when its parse ends, or dropped with an unwind out of it.
impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
  PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
where
  Power: PrattPower,
  S: Stack<PrattFrame<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power>>,
{
  /// [`handle_operand`](Self::handle_operand) in the `Term` phase, which the adapter stands in.
  #[inline]
  pub(crate) fn put_operand(&mut self, value: V) {
    debug_assert!(
      matches!(self.state.phase, Phase::Term),
      "an operand put outside the `Term` phase"
    );
    self.state.phase = Phase::Operator(value);
  }

  /// [`handle_prefix`](Self::handle_prefix) in the `Term` phase, which the adapter stands in. A
  /// refusal is `Full`, and hands the operator back: the stack is at its cap, or its storage
  /// refused the frame. The floor kept is cloned out of the operator before the storage is asked,
  /// so that no frame is read back, and a refusal by the storage drops that clone.
  #[inline]
  pub(crate) fn push_prefix(
    &mut self,
    operator: Precedenced<Pre, Power>,
  ) -> Result<(), Precedenced<Pre, Power>> {
    debug_assert!(
      matches!(self.state.phase, Phase::Term),
      "a prefix operator pushed outside the `Term` phase"
    );
    if self.state.depth >= self.state.max_depth {
      return Err(operator);
    }
    let frame = PrattFrame {
      operator: FrameOperator::Prefix(operator),
      latch: None,
    };
    let floor = frame.operator.floor();
    match self.stack.push(frame) {
      Ok(()) => {
        // Raised only once the storage has taken the frame, and bounded as in `handle_prefix`.
        self.state.depth += 1;
        self.state.latch = None;
        self.state.frame_floor = Some(floor);
        Ok(())
      }
      Err(PrattFrame {
        operator: FrameOperator::Prefix(operator),
        ..
      }) => Err(operator),
      Err(_) => broken_stack("`push` refused a frame, and handed back another"),
    }
  }

  /// [`handle_infix`](Self::handle_infix) for an operator the adapter's own
  /// [`offer`](Self::offer) has just answered [`Take`](PrattOffer::Take) for, in the `Operator`
  /// phase. A refusal is `Full`, hands the operator back, and leaves the value the machine's. The
  /// floor kept is cloned as in [`push_prefix`](Self::push_prefix).
  #[inline]
  pub(crate) fn push_infix(
    &mut self,
    operator: Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>,
  ) -> Result<(), Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>> {
    debug_assert!(
      matches!(self.state.phase, Phase::Operator(_))
        && self.answer(operator.precedence(), true) == PrattOffer::Take,
      "an infix operator pushed without its offer's `Take`"
    );
    if self.state.depth >= self.state.max_depth {
      return Err(operator);
    }
    let Some(left) = self.take_value() else {
      unreachable!("an infix operator pushed outside the `Operator` phase");
    };
    let frame = PrattFrame {
      operator: FrameOperator::Infix(InfixFrame { left, operator }),
      latch: None,
    };
    let floor = frame.operator.floor();
    match self.stack.push(frame) {
      Ok(()) => {
        // Raised only once the storage has taken the frame, and bounded as in `handle_prefix`.
        self.state.depth += 1;
        self.state.latch = None;
        self.state.frame_floor = Some(floor);
        Ok(())
      }
      Err(PrattFrame {
        operator: FrameOperator::Infix(InfixFrame { left, operator }),
        ..
      }) => {
        self.state.phase = Phase::Operator(left);
        Err(operator)
      }
      Err(_) => broken_stack("`push` refused a frame, and handed back another"),
    }
  }

  /// [`handle_postfix`](Self::handle_postfix) for an operator the adapter's own
  /// [`offer`](Self::offer) has just answered [`Take`](PrattOffer::Take) for, in the `Operator`
  /// phase, with its reduction handed straight back rather than owed to `poll`. The value is its
  /// operand, moved out, so the phase is `Term` until [`put_reduced`](Self::put_reduced).
  #[inline]
  pub(crate) fn take_postfix(
    &mut self,
    operator: Precedenced<Post, Power>,
  ) -> PrattPostfixReduction<V, Post, Power> {
    debug_assert!(
      matches!(self.state.phase, Phase::Operator(_))
        && self.answer(operator.precedence(), false) == PrattOffer::Take,
      "a postfix operator taken without its offer's `Take`"
    );
    let Some(operand) = self.take_value() else {
      unreachable!("a postfix operator taken outside the `Operator` phase");
    };
    PrattPostfixReduction { operand, operator }
  }

  /// [`close`](Self::close) in the `Operator` phase, which the adapter stands in, with what it
  /// owes handed straight back rather than to `poll`. Under a pending operator its frame is
  /// popped, the latch rule applied, and its reduction handed out: the phase is `Term`, one frame
  /// fewer, until [`put_reduced`](Self::put_reduced). At the root, the expression's value, and the
  /// machine is as [`reset`](Self::reset) leaves it.
  #[inline]
  pub(crate) fn close_and_reduce(
    &mut self,
  ) -> Closed<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
    let Some(right) = self.take_value() else {
      unreachable!("a close outside the `Operator` phase");
    };
    if self.state.depth == 0 {
      // `Done` resets, as `poll`'s does, and the phase is `Term` already.
      self.clear_done();
      return Closed::Done(right);
    }
    self.reduce_top_frame(right)
  }

  /// [`close_and_reduce`](Self::close_and_reduce) where an offer has just answered
  /// [`Reduce`](PrattOffer::Reduce), which it answers only under a pending operator: the
  /// operator's reduction, and never the root's value.
  ///
  /// Out of line, for `parse_into`'s loop, which reduces inside the read that commits an operator.
  /// Inlined there, a reduction makes that read too large to inline into the loop, and the unit
  /// machine then lives in memory for every operator the loop reads. Out of line, the read stays
  /// inlined, and a reduction alone pays a call.
  #[inline(never)]
  pub(crate) fn reduce_pending(
    &mut self,
  ) -> Closed<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
    debug_assert!(
      self.state.depth > 0,
      "a pending reduction with no operator pending"
    );
    let Some(right) = self.take_value() else {
      unreachable!("a reduction outside the `Operator` phase");
    };
    self.reduce_top_frame(right)
  }

  /// The reduction of the frame on top, whose right operand is `right`: popped, the floor read off
  /// the frame left on top, and the latch rule applied.
  #[inline(always)]
  fn reduce_top_frame(
    &mut self,
    right: V,
  ) -> Closed<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Power> {
    let PrattFrame { operator, latch } = self.pop_raw();
    self.forget_latch(&latch);
    drop(latch);
    // The floor is read off the frame left on top, once for either arm.
    self.state.frame_floor = self.floor_of_top();
    match operator {
      FrameOperator::Prefix(operator) => {
        // A prefix reduction leaves the latch as it is: its copy is read where it lives.
        self.keep_latch();
        Closed::Prefix(PrattPrefixReduction {
          operand: right,
          operator,
        })
      }
      FrameOperator::Infix(InfixFrame { left, operator }) => {
        // The latch the rule writes replaces the one the frame left on top holds, so it is not
        // read.
        self.set_latch(infix_latch(&operator));
        Closed::Infix(PrattInfixReduction {
          left,
          right,
          operator,
        })
      }
    }
  }

  /// The value of a reduction this impl's events handed back, put back in the `Term` phase its
  /// fold ran in, which this leaves `Operator`: what [`handle_reduced`](Self::handle_reduced) is
  /// on the public path, with nothing left to apply.
  #[inline]
  pub(crate) fn put_reduced(&mut self, value: V) {
    debug_assert!(
      matches!(self.state.phase, Phase::Term),
      "a reduction's value put back outside the `Term` phase its fold ran in"
    );
    self.state.phase = Phase::Operator(value);
  }
}

// The re-seat that lets a loop go on over another storage. The loops that grow a default stack on
// the native stack, a segment at a time, split the machine here and seat it on the next segment, or
// back on their own. Crate-private, since a public `from_parts` could seat a machine on a storage
// that does not hold its frames.
//
// `allow(dead_code)` outside tests: those loops are adapters', and a build that compiles none of
// them reaches these from the machine's tests alone.
#[cfg_attr(not(test), allow(dead_code))]
impl<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
  PrattResolver<V, Pre, LeftAssoc, RightAssoc, NeitherAssoc, Post, Power, S>
{
  /// The storage, to read: a loop over a segment asks it whether its own frames are gone.
  #[inline(always)]
  pub(crate) fn storage(&self) -> &S {
    &self.stack
  }

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
