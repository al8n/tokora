//! The two modes of the flat driver's `parse_into`, and the steps the flat loop asks of a mode.

use core::marker::PhantomData;

#[cfg(any(feature = "std", feature = "alloc"))]
use hybrid_arraydeque::typenum::U4;
#[cfg(not(any(feature = "std", feature = "alloc")))]
use hybrid_arraydeque::typenum::U32;

use super::super::{
  PrattFrame, PrattInfix, PrattOffer, PrattPending, PrattPower, PrattRHS, PrattResolver,
  PrattStack, Precedenced, resolve::Closed, sealed::Sealed,
};

/// One pending operator of `Checked`'s machine, which holds unit values.
type UnitFrame<Power> = PrattFrame<(), (), (), (), (), Power>;

/// `Checked`'s stack: four unit frames inline, and the heap past them.
#[cfg(any(feature = "std", feature = "alloc"))]
type UnitStack<Power> = PrattStack<UnitFrame<Power>, U4>;

/// `Checked`'s stack where the crate cannot allocate: 32 unit frames, and a refusal past them.
#[cfg(not(any(feature = "std", feature = "alloc")))]
type UnitStack<Power> = PrattStack<UnitFrame<Power>, U32>;

/// The default mode: [`parse_into`](super::Flat::parse_into) admits each operator through a
/// machine of unit values. It has a floor, it refuses a repeated non-associative operator where it
/// reads one, and it has a cap.
///
/// `B` records whether a bound was asked for: [`Open`] until
/// [`min_precedence`](super::Flat::min_precedence) or [`max_depth`](super::Flat::max_depth) is
/// called, and [`Bounded`] after, whatever the value. Only an `Open` driver can become [`Bare`].
///
/// The machine's stack is a field of the driver, so a driver that is kept reuses it, and it is
/// emptied when a parse begins and when one returns. It holds four unit frames inline, and grows
/// on the heap past them where the crate can allocate. Where it cannot, it holds 32 unit frames
/// and does not grow: the 33rd pending operator is
/// [`PrattStackFull`](crate::error::PrattStackFull). A unit frame holds a power, an optional power
/// and a tag, so the 32 take some 128 bytes at a `u8` power, and some 1 KiB at `i64`.
#[derive(Debug)]
pub struct Checked<Power, B = Open> {
  /// The floor, inclusive.
  floor: Power,
  /// The cap on pending operators, once one is set.
  cap: Option<usize>,
  /// The machine's stack, lent to it for each parse.
  stack: UnitStack<Power>,
  _bound: PhantomData<B>,
}

impl<Power: PrattPower> Checked<Power> {
  /// The mode with no floor above `Power::default()`, no cap, and an empty stack.
  #[inline]
  pub(super) fn new() -> Self {
    Self {
      floor: Power::default(),
      cap: None,
      stack: PrattStack::new(),
      _bound: PhantomData,
    }
  }
}

impl<Power, B> Checked<Power, B> {
  /// The same mode, with the floor at `floor`.
  #[inline]
  pub(super) fn with_floor(self, floor: Power) -> Checked<Power, Bounded> {
    Checked {
      floor,
      cap: self.cap,
      stack: self.stack,
      _bound: PhantomData,
    }
  }

  /// The same mode, with the cap at `cap`.
  #[inline]
  pub(super) fn with_cap(self, cap: usize) -> Checked<Power, Bounded> {
    Checked {
      floor: self.floor,
      cap: Some(cap),
      stack: self.stack,
      _bound: PhantomData,
    }
  }

  /// The floor, and the cap if one is set: what `folding` carries over. The unit stack is
  /// dropped.
  #[inline]
  pub(super) fn into_bounds(self) -> (Power, Option<usize>) {
    (self.floor, self.cap)
  }
}

#[cfg(all(test, feature = "std"))]
impl<Power, B> Checked<Power, B> {
  /// How many frames the stack holds.
  pub(super) fn held(&self) -> usize {
    crate::container::Container::len(&self.stack)
  }
}

/// [`Checked`] with neither a floor nor a cap set. Only this state can become [`Bare`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Open;

/// [`Checked`] once a floor or a cap is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bounded;

/// The other mode: [`parse_into`](super::Flat::parse_into) takes every operator its table names,
/// and no machine runs. It has no floor, no cap and no stack, and a repeated non-associative
/// operator is taken as any other is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bare;

/// What [`parse_into`](super::Flat::parse_into) asks of a mode.
///
/// Sealed: [`Checked`], in either state, and [`Bare`] implement it.
pub trait FlatMode<Power>: Sealed + ModeSteps<Power> {}

/// The steps a mode supplies the flat loop.
///
/// Its module is private, so it cannot be named outside the crate, and it is hidden so that it is
/// no part of the documented surface.
#[doc(hidden)]
pub trait ModeSteps<Power> {
  /// One parse's state.
  type Run<'a>: RunSteps<Power>
  where
    Self: 'a;

  /// Begins a parse.
  fn begin(&mut self) -> Self::Run<'_>;
}

/// The steps of one parse.
#[doc(hidden)]
pub trait RunSteps<Power> {
  /// Whether the expression takes the operator at the head, which the table classified, before it
  /// is committed. Never [`PrattOffer::Reduce`]: a mode performs the reductions it is owed here.
  fn admit(&mut self, rhs: &PrattRHS<(), (), (), (), Power>) -> PrattOffer;

  /// A prefix operator at `power` is committed. `Err` carries the bound that refused it.
  fn prefix(&mut self, power: &Power) -> Result<(), usize>;

  /// An infix operator is committed. `Err` carries the bound that refused it.
  fn infix(&mut self, associativity: &PrattInfix<(), (), ()>, power: &Power) -> Result<(), usize>;

  /// The operand parser accepted.
  fn operand(&mut self);

  /// A postfix operator at `power` is committed.
  fn postfix(&mut self, power: &Power);

  /// No term is to be had: whether an infix operator waits for it, rather than a prefix operator
  /// or nothing.
  fn missing(&mut self) -> bool;

  /// The parse returns, with `Ok` or `Err`.
  fn end(self);
}

impl<Power, B> Sealed for Checked<Power, B> {}

impl<Power: PrattPower, B> FlatMode<Power> for Checked<Power, B> {}

impl<Power: PrattPower, B> ModeSteps<Power> for Checked<Power, B> {
  type Run<'a>
    = CheckedRun<'a, Power>
  where
    Self: 'a;

  /// Lends the stack to a machine with the driver's floor and cap, and builds no default floor
  /// to replace. The machine empties the stack first, before it clones the floor: a parse that
  /// unwound left its frames there.
  #[inline]
  fn begin(&mut self) -> CheckedRun<'_, Power> {
    CheckedRun {
      machine: PrattResolver::with_stack_and_floor(&mut self.stack, &self.floor)
        .max_depth(self.cap.unwrap_or(usize::MAX)),
    }
  }
}

/// One parse of a [`Checked`] driver: a machine of unit values over the driver's stack.
#[doc(hidden)]
pub struct CheckedRun<'a, Power> {
  machine: PrattResolver<(), (), (), (), (), (), Power, &'a mut UnitStack<Power>>,
}

impl<Power: PrattPower> CheckedRun<'_, Power> {
  /// The bound a refused push met, which is `Full`. The capacity is the lesser of the cap and the
  /// stack's own, and the stack is never a segment here. So it is the bound that refused: the cap
  /// where the count reached it, and the stack's where the stack refused below the cap.
  #[inline]
  fn bound(&self) -> usize {
    self.machine.capacity()
  }
}

impl<Power: PrattPower> RunSteps<Power> for CheckedRun<'_, Power> {
  /// The offer. On `Reduce` the one reduction is performed where the head is classified, and the
  /// operator is offered again, until the answer changes. The close hands the reduction straight
  /// back, and a unit reduction builds nothing and needs no input: it is dropped, and the unit is
  /// its value. `Reduce` is answered only under a pending operator, so the close never ends the
  /// root here.
  #[inline]
  fn admit(&mut self, rhs: &PrattRHS<(), (), (), (), Power>) -> PrattOffer {
    loop {
      match self.machine.offer(rhs) {
        PrattOffer::Reduce => match self.machine.close_and_reduce() {
          Closed::Prefix(_) | Closed::Infix(_) => self.machine.put_reduced(()),
          Closed::Done(()) => {
            unreachable!(
              "the flat driver's unit machine ended its root where its offer said reduce"
            )
          }
        },
        answer => return answer,
      }
    }
  }

  #[inline]
  fn prefix(&mut self, power: &Power) -> Result<(), usize> {
    match self
      .machine
      .push_prefix(Precedenced::new((), power.clone()))
    {
      Ok(()) => Ok(()),
      Err(_) => Err(self.bound()),
    }
  }

  #[inline]
  fn infix(&mut self, associativity: &PrattInfix<(), (), ()>, power: &Power) -> Result<(), usize> {
    match self
      .machine
      .push_infix(Precedenced::new(*associativity, power.clone()))
    {
      Ok(()) => Ok(()),
      Err(_) => Err(self.bound()),
    }
  }

  #[inline]
  fn operand(&mut self) {
    self.machine.put_operand(());
  }

  /// The postfix reduction is handed straight back. A unit reduction builds nothing, so it is
  /// dropped, and the unit is its value.
  #[inline]
  fn postfix(&mut self, power: &Power) {
    drop(
      self
        .machine
        .take_postfix(Precedenced::new((), power.clone())),
    );
    self.machine.put_reduced(());
  }

  /// `retract` hands the waiting operator back, if there is one.
  #[inline]
  fn missing(&mut self) -> bool {
    matches!(self.machine.retract(), Some(PrattPending::Infix(_)))
  }

  /// At an ending nothing is owed: a unit reduction builds nothing, so the frames still pending
  /// are dropped by the reset, which empties the driver's stack.
  #[inline]
  fn end(mut self) {
    self.machine.reset();
  }
}

impl Sealed for Bare {}

impl<Power> FlatMode<Power> for Bare {}

impl<Power> ModeSteps<Power> for Bare {
  type Run<'a>
    = BareRun
  where
    Self: 'a;

  #[inline(always)]
  fn begin(&mut self) -> BareRun {
    BareRun { infix_waits: false }
  }
}

/// One parse of a [`Bare`] driver: one bit, for which error a missing operand is.
#[doc(hidden)]
pub struct BareRun {
  /// Whether the operator read last is an infix operator, which then waits for its right operand.
  infix_waits: bool,
}

impl<Power> RunSteps<Power> for BareRun {
  #[inline(always)]
  fn admit(&mut self, _rhs: &PrattRHS<(), (), (), (), Power>) -> PrattOffer {
    PrattOffer::Take
  }

  #[inline(always)]
  fn prefix(&mut self, _power: &Power) -> Result<(), usize> {
    self.infix_waits = false;
    Ok(())
  }

  #[inline(always)]
  fn infix(
    &mut self,
    _associativity: &PrattInfix<(), (), ()>,
    _power: &Power,
  ) -> Result<(), usize> {
    self.infix_waits = true;
    Ok(())
  }

  #[inline(always)]
  fn operand(&mut self) {}

  #[inline(always)]
  fn postfix(&mut self, _power: &Power) {}

  #[inline(always)]
  fn missing(&mut self) -> bool {
    self.infix_waits
  }

  #[inline(always)]
  fn end(self) {}
}
