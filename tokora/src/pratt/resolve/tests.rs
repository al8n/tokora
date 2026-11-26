//! The machine alone, cell by cell, as its contract is written: every event in every phase, the
//! offer, `poll`, `close` and `finish`, `retract`, the cap, a storage that breaks `Stack`, the order
//! it keeps around a destructor that unwinds, chains of segments, and a random drive checked against
//! the tables.
//!
//! Every cell runs in each of the crate's three tiers but the three that pin one tier's storage.
//! They name `core` alone, but for the two that catch an unwind, which reach the `std` a test
//! harness always links through `host`.

extern crate std as host;

use core::{cell::Cell, fmt::Debug};

use hybrid_arraydeque::{
  ArrayDeque, ArraySize,
  typenum::{U1, U2, U4, U8, U64},
};

use super::*;
use crate::{container::Container, pratt::segment::Segment};

/// The frame the cells' machines hold: values are `u32`, every operator payload is a `u8` tag, and
/// a power is an `i8`, so that a floor can sit below every power.
type Frame = PrattFrame<u32, u8, u8, u8, u8, i8>;

/// The machine the cells drive, over a storage of their choosing.
type Machine<S> = PrattResolver<u32, u8, u8, u8, u8, u8, i8, S>;

/// A storage no cell fills: the random drives keep fewer than sixty operators pending.
type Wide = ArrayDeque<Frame, U64>;

fn machine() -> Machine<Wide> {
  PrattResolver::with_stack(Wide::new())
}

fn prefix(tag: u8, power: i8) -> Precedenced<u8, i8> {
  Precedenced::new(tag, power)
}

fn postfix(tag: u8, power: i8) -> Precedenced<u8, i8> {
  Precedenced::new(tag, power)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Assoc {
  Left,
  Right,
  Neither,
}

fn infix(assoc: Assoc, tag: u8, power: i8) -> Precedenced<PrattInfix<u8, u8, u8>, i8> {
  let operator = match assoc {
    Assoc::Left => PrattInfix::Left(tag),
    Assoc::Right => PrattInfix::Right(tag),
    Assoc::Neither => PrattInfix::Neither(tag),
  };
  Precedenced::new(operator, power)
}

fn left(tag: u8, power: i8) -> Precedenced<PrattInfix<u8, u8, u8>, i8> {
  infix(Assoc::Left, tag, power)
}

fn right(tag: u8, power: i8) -> Precedenced<PrattInfix<u8, u8, u8>, i8> {
  infix(Assoc::Right, tag, power)
}

fn neither(tag: u8, power: i8) -> Precedenced<PrattInfix<u8, u8, u8>, i8> {
  infix(Assoc::Neither, tag, power)
}

/// What an operator is offered as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rhs {
  Infix(i8),
  Adjacent(i8),
  Postfix(i8),
  End,
}

fn offered(rhs: Rhs) -> PrattRHS<(), (), (), (), i8> {
  match rhs {
    Rhs::Infix(power) => PrattRHS::Infix(Precedenced::new(PrattInfix::Left(()), power)),
    Rhs::Adjacent(power) => PrattRHS::Adjacent(Precedenced::new((), power)),
    Rhs::Postfix(power) => PrattRHS::Postfix(Precedenced::new((), power)),
    Rhs::End => PrattRHS::End,
  }
}

/// The five answers of `poll`, as a phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Answer {
  Term,
  Operator,
  Reduce,
  Reducing,
  Done,
}

/// The answer `poll` would give, read without asking it.
fn answer<S>(m: &Machine<S>) -> Answer {
  match &m.state.phase {
    Phase::Term => Answer::Term,
    Phase::Operator(_) => Answer::Operator,
    Phase::Owed(Owed::Done(_)) => Answer::Done,
    Phase::Owed(_) => Answer::Reduce,
    Phase::Reducing(_) => Answer::Reducing,
  }
}

fn answer_of(poll: &PrattPoll<u32, u8, u8, u8, u8, u8, i8>) -> Answer {
  match poll {
    PrattPoll::Term => Answer::Term,
    PrattPoll::Operator => Answer::Operator,
    PrattPoll::Reduce(_) => Answer::Reduce,
    PrattPoll::Reducing => Answer::Reducing,
    PrattPoll::Done(_) => Answer::Done,
  }
}

/// One call on a machine: one of the eight events, `poll`, or `offer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
  Prefix(u8, i8),
  Operand(u32),
  Infix(Assoc, u8, i8),
  Postfix(u8, i8),
  Reduced(u32),
  Close,
  Finish,
  Retract,
  Poll,
  Offer(Rhs),
}

/// What a refusal handed back.
#[derive(Debug, PartialEq, Eq)]
enum Item {
  Operator(Precedenced<u8, i8>),
  Infix(Precedenced<PrattInfix<u8, u8, u8>, i8>),
  Value(u32),
}

/// What one call answered.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
  Taken,
  Refused(PrattRefusal, Item),
  Ended(Result<(), PrattRefusal>),
  Retracted(Option<PrattPending<u8, u8, u8, u8, i8>>),
  Polled(PrattPoll<u32, u8, u8, u8, u8, u8, i8>),
  Offered(PrattOffer),
}

fn apply<S: Stack<Frame>>(m: &mut Machine<S>, event: Event) -> Outcome {
  fn outcome<T>(result: Result<(), PrattRefused<T>>, item: fn(T) -> Item) -> Outcome {
    match result {
      Ok(()) => Outcome::Taken,
      Err(refusal) => {
        let (why, fed) = refusal.into_components();
        Outcome::Refused(why, item(fed))
      }
    }
  }
  match event {
    Event::Prefix(tag, power) => outcome(m.handle_prefix(prefix(tag, power)), Item::Operator),
    Event::Operand(value) => outcome(m.handle_operand(value), Item::Value),
    Event::Infix(assoc, tag, power) => {
      outcome(m.handle_infix(infix(assoc, tag, power)), Item::Infix)
    }
    Event::Postfix(tag, power) => outcome(m.handle_postfix(postfix(tag, power)), Item::Operator),
    Event::Reduced(value) => outcome(m.handle_reduced(value), Item::Value),
    Event::Close => Outcome::Ended(m.close()),
    Event::Finish => Outcome::Ended(m.finish()),
    Event::Retract => Outcome::Retracted(m.retract()),
    Event::Poll => Outcome::Polled(m.poll()),
    Event::Offer(rhs) => Outcome::Offered(m.offer(&offered(rhs))),
  }
}

/// The outcome of `event` refused for `why`, with the item it fed handed back.
fn refused(event: Event, why: PrattRefusal) -> Outcome {
  match event {
    Event::Prefix(tag, power) | Event::Postfix(tag, power) => {
      Outcome::Refused(why, Item::Operator(Precedenced::new(tag, power)))
    }
    Event::Operand(value) | Event::Reduced(value) => Outcome::Refused(why, Item::Value(value)),
    Event::Infix(assoc, tag, power) => Outcome::Refused(why, Item::Infix(infix(assoc, tag, power))),
    Event::Close | Event::Finish => Outcome::Ended(Err(why)),
    Event::Retract => Outcome::Retracted(None),
    Event::Poll | Event::Offer(_) => unreachable!("{event:?} is never refused"),
  }
}

fn taken(outcome: &Outcome) -> bool {
  matches!(
    outcome,
    Outcome::Taken | Outcome::Ended(Ok(())) | Outcome::Retracted(Some(_))
  )
}

/// `close`, the one reduction it owes, and `value` handed back for it.
fn reduce_once<S: Stack<Frame>>(m: &mut Machine<S>, value: u32) {
  m.close().unwrap();
  assert!(matches!(m.poll(), PrattPoll::Reduce(_)));
  m.handle_reduced(value).unwrap();
}

/// A machine in each of the five phases `poll` can answer. The `Term` one has a prefix operator
/// pending, so that `retract` has one to take.
fn in_each_phase() -> [(Answer, Machine<Wide>); 5] {
  let mut term = machine();
  term.handle_prefix(prefix(1, 5)).unwrap();

  let mut operator = machine();
  operator.handle_operand(7).unwrap();

  let mut reduce = term.clone();
  reduce.handle_operand(8).unwrap();
  reduce.close().unwrap();

  let mut reducing = reduce.clone();
  assert!(matches!(reducing.poll(), PrattPoll::Reduce(_)));

  let mut done = operator.clone();
  done.close().unwrap();

  [
    (Answer::Term, term),
    (Answer::Operator, operator),
    (Answer::Reduce, reduce),
    (Answer::Reducing, reducing),
    (Answer::Done, done),
  ]
}

/// The eight events, at payloads each phase above takes when it takes the event at all.
const EVENTS: [Event; 8] = [
  Event::Prefix(9, 4),
  Event::Operand(11),
  Event::Infix(Assoc::Left, 9, 3),
  Event::Postfix(9, 3),
  Event::Reduced(12),
  Event::Close,
  Event::Finish,
  Event::Retract,
];

/// Eight events against five phases. Each is taken where the table takes it, and leaves the phase
/// the table names. Everywhere else it is refused as misplaced: the item handed back is the one
/// fed, and the machine equals the one before the call.
#[test]
fn every_event_in_every_phase_is_taken_or_refused_as_the_table_says() {
  use Answer::{Done, Operator, Reduce, Reducing, Term};
  let table: [(Answer, [Option<Answer>; 8]); 5] = [
    (
      Term,
      [
        Some(Term),
        Some(Operator),
        None,
        None,
        None,
        None,
        None,
        Some(Term),
      ],
    ),
    (
      Operator,
      [
        None,
        None,
        Some(Term),
        Some(Reduce),
        None,
        Some(Done),
        Some(Done),
        None,
      ],
    ),
    (Reduce, [None; 8]),
    (
      Reducing,
      [None, None, None, None, Some(Operator), None, None, None],
    ),
    (Done, [None; 8]),
  ];
  for ((phase, machine), (row, after)) in in_each_phase().into_iter().zip(table) {
    assert_eq!(phase, row);
    assert_eq!(answer(&machine), phase);
    for (event, after) in EVENTS.into_iter().zip(after) {
      let mut m = machine.clone();
      let outcome = apply(&mut m, event);
      match after {
        Some(after) => {
          assert!(taken(&outcome), "{event:?} in {phase:?}: {outcome:?}");
          assert_eq!(answer(&m), after, "{event:?} in {phase:?}");
        }
        None => {
          assert_eq!(
            outcome,
            refused(event, PrattRefusal::Misplaced),
            "{event:?} in {phase:?}"
          );
          assert_eq!(
            m, machine,
            "{event:?} in {phase:?}: a refusal changes nothing"
          );
        }
      }
    }
  }
}

/// The offer at `power` to an infix, an adjacent and a postfix operator, in that order. An
/// adjacency is an infix operator for the floor and the latch, so its answer is always the infix
/// one. The offer changes nothing.
fn assert_offers<S>(m: &Machine<S>, power: i8, answers: [PrattOffer; 3])
where
  S: Stack<Frame> + Clone + PartialEq + Debug,
{
  let before = m.clone();
  let got = [Rhs::Infix(power), Rhs::Adjacent(power), Rhs::Postfix(power)]
    .map(|rhs| m.offer(&offered(rhs)));
  assert_eq!(got, answers, "at power {power}");
  assert_eq!(got[0], got[1], "an adjacency answers as an infix operator");
  assert_eq!(*m, before);
}

#[test]
fn the_offer_tests_the_floor_then_the_chain_and_changes_nothing() {
  use PrattOffer::{Chain, Decline, Reduce, Take};

  // The root, at a floor of 3, inclusive.
  let mut m = machine().min_precedence(3);
  m.handle_operand(1).unwrap();
  assert_offers(&m, 3, [Take, Take, Take]);
  assert_offers(&m, 2, [Decline, Decline, Decline]);
  assert_eq!(m.offer(&offered(Rhs::End)), Decline);

  // The right operand of a left-associative operator at 5 admits only what is above 5.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(left(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  assert_offers(&m, 5, [Reduce, Reduce, Reduce]);
  assert_offers(&m, 6, [Take, Take, Take]);
  assert_eq!(m.offer(&offered(Rhs::End)), Reduce);

  // A non-associative one, the same.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(neither(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  assert_offers(&m, 5, [Reduce, Reduce, Reduce]);
  assert_offers(&m, 6, [Take, Take, Take]);

  // A right-associative one admits its own power.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(right(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  assert_offers(&m, 5, [Take, Take, Take]);
  assert_offers(&m, 4, [Reduce, Reduce, Reduce]);

  // A prefix operator's operand admits the prefix operator's own power, whatever the root's floor:
  // the frame's floor is read off the frame.
  let mut m = machine().min_precedence(7);
  m.handle_prefix(prefix(0, 5)).unwrap();
  m.handle_operand(1).unwrap();
  assert_offers(&m, 6, [Take, Take, Take]);
  assert_offers(&m, 5, [Take, Take, Take]);
  assert_offers(&m, 4, [Reduce, Reduce, Reduce]);

  // The root after a non-associative reduction at 5: an infix operator at 5 repeats the chain, and
  // a postfix operator at 5 is taken.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(neither(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  reduce_once(&mut m, 12);
  assert_offers(&m, 5, [Chain, Chain, Take]);
  assert_offers(&m, 6, [Take, Take, Take]);
  assert_offers(&m, 4, [Take, Take, Take]);

  // A pending operator's operand after the same reduction: `1 + (2 == 3)`, with `+` at 3.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(left(0, 3)).unwrap();
  m.handle_operand(2).unwrap();
  m.handle_infix(neither(1, 5)).unwrap();
  m.handle_operand(3).unwrap();
  reduce_once(&mut m, 23);
  assert_offers(&m, 5, [Chain, Chain, Take]);
  assert_offers(&m, 4, [Take, Take, Take]);
  assert_offers(&m, 3, [Reduce, Reduce, Reduce]);

  // Outside `Operator` no operator can be taken, and `End` is not an exception.
  for (phase, m) in in_each_phase() {
    if phase != Answer::Operator {
      assert_offers(&m, 5, [Decline, Decline, Decline]);
      assert_eq!(m.offer(&offered(Rhs::End)), Decline);
    }
  }
}

/// The root's floor set on a live machine. At the root it answers the next offer at once. Under
/// pending operators their own floors answer, and the new floor answers once the reductions are
/// back at the root: an operator between the old floor and the new one is declined there.
#[test]
fn min_precedence_on_a_live_machine_is_the_root_s_floor_from_then_on() {
  use PrattOffer::{Decline, Reduce, Take};

  // At the root, with a value: the floor moves from 0 to 6.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  assert_offers(&m, 4, [Take, Take, Take]);
  let m = m.min_precedence(6);
  assert_offers(&m, 4, [Decline, Decline, Decline]);
  assert_offers(&m, 6, [Take, Take, Take]);

  // Under two pending operators, `1 + 2 * 3` with `+` at 2 and `*` at 3: the floor moves from 0 to
  // 6 at a depth of two.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(left(0, 2)).unwrap();
  m.handle_operand(2).unwrap();
  m.handle_infix(left(1, 3)).unwrap();
  m.handle_operand(3).unwrap();
  let mut m = m.min_precedence(6);
  assert_eq!(m.depth(), 2);
  assert_offers(&m, 4, [Take, Take, Take]);
  assert_offers(&m, 3, [Reduce, Reduce, Reduce]);
  reduce_once(&mut m, 23);
  assert_offers(&m, 4, [Take, Take, Take]);
  assert_offers(&m, 2, [Reduce, Reduce, Reduce]);
  reduce_once(&mut m, 123);
  assert_eq!(m.depth(), 0);
  assert_offers(&m, 4, [Decline, Decline, Decline]);
  assert_offers(&m, 6, [Take, Take, Take]);
}

/// One reduction for each `poll`, innermost first. Between a reduction and its value `poll` answers
/// `Reducing` and changes nothing, and `Done` resets: the next answer is `Term`, and the machine is
/// as new, with the same floor and cap.
#[test]
fn poll_hands_out_one_reduction_at_a_time_innermost_first() {
  let new = machine().min_precedence(-2).max_depth(9);
  let mut m = new.clone();
  // 1 + 2 * 3 ^ 4, with all three pending when `finish` is asked.
  m.handle_operand(1).unwrap();
  m.handle_infix(left(0, 1)).unwrap();
  m.handle_operand(2).unwrap();
  m.handle_infix(left(1, 2)).unwrap();
  m.handle_operand(3).unwrap();
  m.handle_infix(right(2, 3)).unwrap();
  m.handle_operand(4).unwrap();
  m.finish().unwrap();
  assert_eq!(m.depth(), 3);

  let mut seen = [(0, 0, 0); 3];
  let mut reductions = 0;
  let value = loop {
    match m.poll() {
      PrattPoll::Reduce(PrattReduction::Infix(reduction)) => {
        let before = m.clone();
        assert_eq!(m.poll(), PrattPoll::Reducing);
        assert_eq!(m.poll(), PrattPoll::Reducing);
        assert_eq!(m, before);
        let (l, r, operator) = reduction.into_components();
        let tag = match operator.into_data() {
          PrattInfix::Left(tag) | PrattInfix::Right(tag) | PrattInfix::Neither(tag) => tag,
        };
        seen[reductions] = (l, r, tag);
        reductions += 1;
        m.handle_reduced(l * 100 + r).unwrap();
      }
      PrattPoll::Done(value) => break value,
      other => panic!("after `finish` only reductions and the value: {other:?}"),
    }
  };
  assert_eq!(seen, [(3, 4, 2), (2, 304, 1), (1, 504, 0)]);
  assert_eq!(value, 604);
  assert_eq!(m.poll(), PrattPoll::Term);
  assert_eq!(m, new);

  // `Done` lets go of the root's latch too.
  m.handle_operand(1).unwrap();
  m.handle_infix(neither(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  reduce_once(&mut m, 12);
  assert_eq!(m.offer(&offered(Rhs::Infix(5))), PrattOffer::Chain);
  m.finish().unwrap();
  assert_eq!(m.poll(), PrattPoll::Done(12));
  assert_eq!(m, new);
}

#[test]
fn close_ends_the_innermost_sub_expression_and_finish_the_expression() {
  // `close` at the root is `Done`.
  let mut m = machine();
  m.handle_operand(5).unwrap();
  m.close().unwrap();
  assert_eq!(m.poll(), PrattPoll::Done(5));

  // `close` under two pending operators owes one reduction, and then answers `Operator`.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(left(0, 1)).unwrap();
  m.handle_operand(2).unwrap();
  m.handle_infix(left(1, 2)).unwrap();
  m.handle_operand(3).unwrap();
  m.close().unwrap();
  let PrattPoll::Reduce(PrattReduction::Infix(reduction)) = m.poll() else {
    panic!("`close` owes the reduction of the operator on top");
  };
  assert_eq!(reduction.into_components(), (2, 3, left(1, 2)));
  m.handle_reduced(23).unwrap();
  assert_eq!(m.poll(), PrattPoll::Operator);
  assert_eq!(m.depth(), 1);

  // `finish` under none, one and many: every pending operator's reduction, innermost first.
  for pending in [0, 1, 5] {
    let mut m = machine();
    for tag in 0..pending {
      m.handle_prefix(prefix(tag, 1)).unwrap();
    }
    m.handle_operand(100).unwrap();
    m.finish().unwrap();
    let mut reductions = 0;
    let value = loop {
      match m.poll() {
        PrattPoll::Reduce(PrattReduction::Prefix(reduction)) => {
          let (operand, operator) = reduction.into_components();
          assert_eq!(operator, prefix(pending - 1 - reductions, 1));
          reductions += 1;
          m.handle_reduced(operand + 1).unwrap();
        }
        PrattPoll::Done(value) => break value,
        other => panic!("after `finish` only reductions and the value: {other:?}"),
      }
    };
    assert_eq!(reductions, pending);
    assert_eq!(value, 100 + u32::from(pending));
  }
}

#[test]
fn retract_hands_back_the_pending_operator() {
  // A prefix operator: the phase is still `Term`, one sub-expression out.
  let mut m = machine();
  m.handle_prefix(prefix(1, 4)).unwrap();
  m.handle_prefix(prefix(2, 4)).unwrap();
  assert_eq!(m.retract(), Some(PrattPending::Prefix(prefix(2, 4))));
  assert_eq!(m.poll(), PrattPoll::Term);
  assert_eq!(m.depth(), 1);

  // An infix operator: its left operand is the value again, and the latch is what it was before
  // the operator was taken, so the chain is still refused.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(neither(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  reduce_once(&mut m, 12);
  let before = m.clone();
  m.handle_infix(left(1, 7)).unwrap();
  assert_eq!(m.retract(), Some(PrattPending::Infix(left(1, 7))));
  assert_eq!(m, before);
  assert_eq!(m.offer(&offered(Rhs::Infix(5))), PrattOffer::Chain);

  // So after `1 + )`, `retract` and `finish` give `1`.
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(left(0, 3)).unwrap();
  assert_eq!(m.retract(), Some(PrattPending::Infix(left(0, 3))));
  m.finish().unwrap();
  assert_eq!(m.poll(), PrattPoll::Done(1));

  // `None` with nothing pending, and nothing changed.
  let mut m = machine();
  let before = m.clone();
  assert_eq!(m.retract(), None);
  assert_eq!(m, before);
}

/// The fifth pending operator over four frames of storage, and over the default stack capped at
/// four: `Full`, the item handed back, the left operand still the machine's, and the operator taken
/// once a reduction has made room.
#[test]
fn the_fifth_pending_operator_is_full_until_a_reduction_makes_room() {
  fn fill_and_refuse<S: Stack<Frame> + Clone + PartialEq + Debug>(mut m: Machine<S>) {
    for tag in 0..4 {
      m.handle_prefix(prefix(tag, 5)).unwrap();
    }
    m.handle_operand(1).unwrap();
    let before = m.clone();
    let operator = left(9, 6);
    assert_eq!(m.offer(&offered(Rhs::Infix(6))), PrattOffer::Take);
    assert_eq!(
      m.handle_infix(operator),
      Err(PrattRefused::new(PrattRefusal::Full, operator))
    );
    assert_eq!(m, before);
    assert_eq!(m.poll(), PrattPoll::Operator);
    reduce_once(&mut m, 2);
    assert_eq!(m.depth(), 3);
    assert_eq!(m.handle_infix(operator), Ok(()));
    assert_eq!(m.depth(), 4);
  }
  fill_and_refuse(PrattResolver::with_stack(ArrayDeque::<Frame, U4>::new()));
  fill_and_refuse(PrattResolver::with_stack(DefaultPrattStack::<Frame>::new()).max_depth(4));
}

#[test]
fn capacity_is_the_lesser_of_the_cap_and_the_storage() {
  let m: Machine<ArrayDeque<Frame, U4>> = PrattResolver::with_stack(ArrayDeque::new());
  assert_eq!(m.capacity(), 4);
  assert_eq!(m.clone().max_depth(2).capacity(), 2);
  assert_eq!(m.max_depth(9).capacity(), 4);
  assert_eq!(machine().capacity(), 64);
}

/// The latch rule: a non-associative operator's reduction latches its power, any other infix
/// reduction clears the latch, and a postfix reduction leaves it alone. So `a == b < c` is refused at
/// the `<` and `a < b == c` is not, as in the family's table, and a postfix operator at the latched
/// power is taken.
#[test]
fn the_latch_refuses_a_repeated_non_associative_power_and_nothing_else() {
  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(neither(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  assert_eq!(m.offer(&offered(Rhs::Infix(5))), PrattOffer::Reduce);
  reduce_once(&mut m, 12);
  let before = m.clone();
  assert_eq!(
    m.handle_infix(left(1, 5)),
    Err(PrattRefused::new(PrattRefusal::Chain, left(1, 5)))
  );
  assert_eq!(m, before);

  assert_eq!(m.handle_postfix(postfix(2, 5)), Ok(()));
  let PrattPoll::Reduce(PrattReduction::Postfix(reduction)) = m.poll() else {
    panic!("a postfix reduction is owed where the operator is read");
  };
  assert_eq!(reduction.into_components(), (12, postfix(2, 5)));
  m.handle_reduced(13).unwrap();
  assert_eq!(m.offer(&offered(Rhs::Infix(5))), PrattOffer::Chain);

  let mut m = machine();
  m.handle_operand(1).unwrap();
  m.handle_infix(left(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  assert_eq!(m.offer(&offered(Rhs::Infix(5))), PrattOffer::Reduce);
  reduce_once(&mut m, 12);
  assert_eq!(m.handle_infix(neither(1, 5)), Ok(()));
}

/// `Adjacent` is a left-associative infix operator at its power: the offer applies the floor to it,
/// and fed as `PrattInfix::Left` its reduction never arms the latch.
#[test]
fn an_adjacency_is_a_left_associative_infix_operator_at_its_power() {
  let mut m = machine().min_precedence(2);
  m.handle_operand(1).unwrap();
  assert_eq!(m.offer(&offered(Rhs::Adjacent(1))), PrattOffer::Decline);
  assert_eq!(m.offer(&offered(Rhs::Adjacent(5))), PrattOffer::Take);
  m.handle_infix(left(0, 5)).unwrap();
  m.handle_operand(2).unwrap();
  assert_eq!(m.offer(&offered(Rhs::Adjacent(5))), PrattOffer::Reduce);
  reduce_once(&mut m, 12);
  assert_eq!(m.offer(&offered(Rhs::Adjacent(5))), PrattOffer::Take);
}

/// The caller's payloads: no bound on any of them, and each back intact, by value.
#[test]
fn the_payloads_need_no_bound_and_come_back_intact() {
  // No `Clone`, no `Debug`, no `PartialEq`: nothing at all.
  struct Opaque(u32);
  type Bare = PrattFrame<Opaque, Opaque, Opaque, Opaque, Opaque, u8>;
  let mut m: PrattResolver<
    Opaque,
    Opaque,
    Opaque,
    Opaque,
    Opaque,
    Opaque,
    u8,
    ArrayDeque<Bare, U4>,
  > = PrattResolver::with_stack(ArrayDeque::new());

  // `-2!`, with the prefix and the postfix operator both at 9, and then a `+` whose right operand
  // never comes.
  assert!(m.handle_prefix(Precedenced::new(Opaque(1), 9)).is_ok());
  assert!(m.handle_operand(Opaque(2)).is_ok());
  assert!(m.handle_postfix(Precedenced::new(Opaque(3), 9)).is_ok());
  let PrattPoll::Reduce(PrattReduction::Postfix(reduction)) = m.poll() else {
    panic!("the postfix reduction is owed");
  };
  let (operand, operator) = reduction.into_components();
  assert_eq!((operand.0, operator.into_data().0), (2, 3));
  assert!(m.handle_reduced(Opaque(23)).is_ok());

  assert!(m.close().is_ok());
  let PrattPoll::Reduce(PrattReduction::Prefix(reduction)) = m.poll() else {
    panic!("the prefix reduction is owed");
  };
  let (operand, operator) = reduction.into_components();
  assert_eq!((operand.0, operator.into_data().0), (23, 1));
  assert!(m.handle_reduced(Opaque(123)).is_ok());

  assert!(
    m.handle_infix(Precedenced::new(PrattInfix::Left(Opaque(4)), 1))
      .is_ok()
  );
  let Some(PrattPending::Infix(operator)) = m.retract() else {
    panic!("`retract` is taken only in `Term`");
  };
  assert!(matches!(operator.into_data(), PrattInfix::Left(Opaque(4))));
  assert!(m.finish().is_ok());
  let PrattPoll::Done(value) = m.poll() else {
    panic!("nothing is pending");
  };
  assert_eq!(value.0, 123);
}

/// A storage whose `pop` loses its item: it takes the item off, and answers `None`.
#[derive(Default)]
struct Lossy(ArrayDeque<Frame, U8>);

impl Container<Frame> for Lossy {
  fn push(&mut self, item: Frame) -> Result<(), Frame> {
    Container::push(&mut self.0, item)
  }

  fn first(&self) -> Option<&Frame> {
    self.0.front()
  }

  fn last(&self) -> Option<&Frame> {
    self.0.back()
  }

  fn len(&self) -> usize {
    self.0.len()
  }

  fn max_capacity(&self) -> usize {
    8
  }
}

impl Stack<Frame> for Lossy {
  fn pop(&mut self) -> Option<Frame> {
    let _ = self.0.pop_back();
    None
  }

  fn last_mut(&mut self) -> Option<&mut Frame> {
    self.0.back_mut()
  }
}

#[test]
#[should_panic(expected = "Stack")]
fn a_storage_whose_pop_loses_an_item_is_a_panic_that_names_stack() {
  let mut m: Machine<Lossy> = PrattResolver::with_stack(Lossy::default());
  m.handle_prefix(prefix(0, 1)).unwrap();
  m.handle_operand(1).unwrap();
  m.finish().unwrap();
  let _ = m.poll();
}

/// A storage that reports three items, and whose `pop` never answers `None`.
#[derive(Default)]
struct Endless {
  pops: usize,
}

impl Container<Frame> for Endless {
  fn push(&mut self, item: Frame) -> Result<(), Frame> {
    Err(item)
  }

  fn first(&self) -> Option<&Frame> {
    None
  }

  fn last(&self) -> Option<&Frame> {
    None
  }

  fn len(&self) -> usize {
    3
  }

  fn max_capacity(&self) -> usize {
    0
  }
}

impl Stack<Frame> for Endless {
  fn pop(&mut self) -> Option<Frame> {
    self.pops += 1;
    Some(PrattFrame {
      operator: FrameOperator::Prefix(prefix(0, 0)),
      latch: None,
    })
  }

  fn last_mut(&mut self) -> Option<&mut Frame> {
    None
  }
}

/// `with_stack` empties its storage by the length it reads once: a `pop` that never answers `None`
/// cannot hang it. Frames left in a storage that was lent stay there for the next machine, which
/// empties them.
#[test]
fn with_stack_empties_its_storage_by_a_length_read_once() {
  let m: Machine<Endless> = PrattResolver::with_stack(Endless::default());
  assert_eq!(m.depth(), 0);
  assert_eq!(m.into_stack().pops, 3);

  let mut kept: ArrayDeque<Frame, U8> = ArrayDeque::new();
  let mut m: Machine<&mut ArrayDeque<Frame, U8>> = PrattResolver::with_stack(&mut kept);
  m.handle_prefix(prefix(0, 1)).unwrap();
  m.handle_prefix(prefix(1, 1)).unwrap();
  assert_eq!(m.depth(), 2);
  let lent = m.into_stack();
  assert_eq!(lent.len(), 2);
  let m: Machine<&mut ArrayDeque<Frame, U8>> = PrattResolver::with_stack(lent);
  assert_eq!(m.depth(), 0);
  assert!(m.into_stack().is_empty());
}

/// A value whose destructor unwinds once it is armed.
#[derive(Debug, PartialEq)]
struct Bomb(bool);

impl Drop for Bomb {
  fn drop(&mut self) {
    if self.0 {
      panic!("a destructor unwinds");
    }
  }
}

type BombFrame = PrattFrame<Bomb, u8, u8, u8, u8, i8>;

type BombMachine<S> = PrattResolver<Bomb, u8, u8, u8, u8, u8, i8, S>;

fn bomb_frame(armed: bool) -> BombFrame {
  PrattFrame {
    operator: FrameOperator::Infix(InfixFrame {
      left: Bomb(armed),
      operator: right(0, 1),
    }),
    latch: None,
  }
}

/// `reset` lowers the count as each frame is given up, before the frame is dropped: when a
/// destructor unwinds, the count is what the storage still holds, and the next `reset` goes on.
#[test]
fn a_destructor_that_unwinds_in_reset_leaves_the_count_and_the_storage_in_agreement() {
  let mut m: BombMachine<ArrayDeque<BombFrame, U8>> = PrattResolver::with_stack(ArrayDeque::new());
  for armed in [false, true, false] {
    m.handle_operand(Bomb(armed)).unwrap();
    m.handle_infix(right(0, 1)).unwrap();
  }
  assert_eq!(m.depth(), 3);

  let caught = host::panic::catch_unwind(host::panic::AssertUnwindSafe(|| m.reset()));
  assert!(caught.is_err());
  assert_eq!(m.depth(), 1);
  assert_eq!(m.stack.len(), 1);

  m.reset();
  assert_eq!(m.depth(), 0);
  assert!(m.stack.is_empty());
}

/// A value that counts its drops.
#[derive(Debug, PartialEq)]
struct Tally<'d>(&'d Cell<u32>);

impl Drop for Tally<'_> {
  fn drop(&mut self) {
    self.0.set(self.0.get() + 1);
  }
}

/// A prefix operator whose payload's destructor unwinds, pending under `close` and under `finish`
/// when `reset` drops it. Once the unwind is caught no reduction is owed: the owed value was dropped
/// on the way, and the machine answers as a fresh one does.
#[test]
fn a_reset_that_unwinds_leaves_no_reduction_owed() {
  type Owing<'d> = PrattResolver<
    Tally<'d>,
    Bomb,
    u8,
    u8,
    u8,
    u8,
    i8,
    ArrayDeque<PrattFrame<Tally<'d>, Bomb, u8, u8, u8, i8>, U8>,
  >;
  for finishing in [false, true] {
    let drops = Cell::new(0);
    let fresh: Owing<'_> = PrattResolver::with_stack(ArrayDeque::new());
    let mut m: Owing<'_> = PrattResolver::with_stack(ArrayDeque::new());
    m.handle_prefix(Precedenced::new(Bomb(true), 5)).unwrap();
    m.handle_operand(Tally(&drops)).unwrap();
    if finishing {
      m.finish().unwrap();
    } else {
      m.close().unwrap();
    }

    let caught = host::panic::catch_unwind(host::panic::AssertUnwindSafe(|| m.reset()));
    assert!(caught.is_err());
    assert_eq!(
      drops.get(),
      1,
      "the owed value is dropped, and not left owed"
    );
    assert_eq!(m.poll(), PrattPoll::Term);
    assert_eq!(m, fresh);
  }
}

/// `with_stack` gives each item up before it is dropped: when a destructor unwinds, the storage
/// holds what was not popped, and the next machine over it empties that.
#[test]
fn a_destructor_that_unwinds_in_with_stack_leaves_the_storage_holding_what_was_not_popped() {
  let mut kept: ArrayDeque<BombFrame, U8> = ArrayDeque::new();
  for armed in [false, true, false] {
    assert!(kept.push_back(bomb_frame(armed)).is_none());
  }

  let caught = host::panic::catch_unwind(host::panic::AssertUnwindSafe(|| {
    let _: BombMachine<&mut ArrayDeque<BombFrame, U8>> = PrattResolver::with_stack(&mut kept);
  }));
  assert!(caught.is_err());
  assert_eq!(kept.len(), 1);

  let m: BombMachine<&mut ArrayDeque<BombFrame, U8>> = PrattResolver::with_stack(&mut kept);
  assert_eq!(m.depth(), 0);
  assert!(m.into_stack().is_empty());
}

/// A storage that keeps `Stack`'s obligation for an unwind: once armed, its next `push` or `pop`
/// panics before it touches what it holds.
#[derive(Default)]
struct Flaky {
  frames: ArrayDeque<Frame, U8>,
  push_unwinds: bool,
  pop_unwinds: bool,
}

impl Container<Frame> for Flaky {
  fn push(&mut self, item: Frame) -> Result<(), Frame> {
    if core::mem::take(&mut self.push_unwinds) {
      panic!("a push unwinds before it takes its frame");
    }
    Container::push(&mut self.frames, item)
  }

  fn first(&self) -> Option<&Frame> {
    self.frames.front()
  }

  fn last(&self) -> Option<&Frame> {
    self.frames.back()
  }

  fn len(&self) -> usize {
    self.frames.len()
  }

  fn max_capacity(&self) -> usize {
    8
  }
}

impl Stack<Frame> for Flaky {
  fn pop(&mut self) -> Option<Frame> {
    if core::mem::take(&mut self.pop_unwinds) {
      panic!("a pop unwinds before it gives a frame up");
    }
    self.frames.pop_back()
  }

  fn last_mut(&mut self) -> Option<&mut Frame> {
    self.frames.back_mut()
  }
}

/// Over a storage whose `push` and `pop` unwind before they change what it holds, as `Stack`
/// requires, the count stays what the storage holds through every unwind: out of `handle_prefix`,
/// `retract`, a reduction's `poll` and `reset`. The next `reset` then empties both.
#[test]
fn a_storage_that_unwinds_before_it_changes_keeps_the_count_true() {
  use host::panic::{AssertUnwindSafe, catch_unwind};

  let mut m: Machine<Flaky> = PrattResolver::with_stack(Flaky::default());
  m.handle_prefix(prefix(0, 1)).unwrap();
  m.handle_prefix(prefix(1, 1)).unwrap();

  m.stack.push_unwinds = true;
  assert!(catch_unwind(AssertUnwindSafe(|| m.handle_prefix(prefix(2, 1)))).is_err());
  assert_eq!((m.depth(), m.stack.frames.len()), (2, 2));

  m.stack.pop_unwinds = true;
  assert!(catch_unwind(AssertUnwindSafe(|| m.retract())).is_err());
  assert_eq!((m.depth(), m.stack.frames.len()), (2, 2));

  m.handle_operand(5).unwrap();
  m.close().unwrap();
  m.stack.pop_unwinds = true;
  assert!(catch_unwind(AssertUnwindSafe(|| m.poll())).is_err());
  assert_eq!((m.depth(), m.stack.frames.len()), (2, 2));

  m.stack.pop_unwinds = true;
  assert!(catch_unwind(AssertUnwindSafe(|| m.reset())).is_err());
  assert_eq!((m.depth(), m.stack.frames.len()), (2, 2));

  m.reset();
  assert_eq!((m.depth(), m.stack.frames.len()), (0, 0));
  assert_eq!(m.poll(), PrattPoll::Term);
}

#[test]
fn into_parts_and_from_parts_give_the_same_machine_back_in_every_phase() {
  for (phase, m) in in_each_phase() {
    let (state, stack) = m.clone().into_parts();
    assert_eq!(PrattResolver::from_parts(state, stack), m, "{phase:?}");
  }
}

/// A non-associative operator that is a segment's only frame, reduced, latches the sub-expression
/// below it, the right operand of the frame below the link, and nothing is written through the
/// link: the machine keeps that latch, and the frame below still holds the root's.
#[test]
fn a_reduction_that_empties_a_segment_latches_the_operand_below_it() {
  let mut root: Segment<'_, Frame, U1> = Segment::root();
  let mut m: Machine<&mut Segment<'_, Frame, U1>> = PrattResolver::with_stack(&mut root);
  m.handle_operand(1).unwrap();
  m.handle_infix(left(0, 3)).unwrap();
  m.handle_operand(2).unwrap();
  // The first segment is full, and its refusal is the storage's, below the cap.
  let refusal = m.handle_infix(neither(1, 5)).unwrap_err();
  assert_eq!(refusal.why(), PrattRefusal::Full);
  assert_eq!(m.poll(), PrattPoll::Operator);

  let (state, segment) = m.into_parts();
  let state = {
    let mut upper: Segment<'_, Frame, U1> = Segment::over(&mut *segment);
    let mut m = PrattResolver::from_parts(state, &mut upper);
    m.handle_infix(refusal.into_item()).unwrap();
    m.handle_operand(3).unwrap();
    m.close().unwrap();
    let PrattPoll::Reduce(PrattReduction::Infix(reduction)) = m.poll() else {
      panic!("`close` owes the reduction of the `==`");
    };
    assert_eq!(reduction.into_components(), (2, 3, neither(1, 5)));
    assert!(m.stack.drained());
    m.handle_reduced(23).unwrap();
    m.into_parts().0
  };

  let m = PrattResolver::from_parts(state, segment);
  assert_eq!(m.depth(), 1);
  assert_eq!(m.state.latch, Some(5));
  assert_eq!(m.stack.last().map(|frame| frame.latch), Some(None));
  assert_eq!(m.offer(&offered(Rhs::Infix(5))), PrattOffer::Chain);
  assert_eq!(m.offer(&offered(Rhs::Infix(4))), PrattOffer::Take);
}

/// A small deterministic PRNG, xorshift64, so that every drive replays from its seed.
struct Rng(u64);

impl Rng {
  fn bits(&mut self) -> u64 {
    let mut x = self.0;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    self.0 = x;
    x
  }

  fn below(&mut self, n: u64) -> u64 {
    self.bits() % n
  }
}

const SEED: u64 = 0x2545_F491_4F6C_DD1D;

/// How many drives a random cell runs: fewer under Miri, which interprets every step.
const DRIVES: usize = if cfg!(miri) { 12 } else { 3000 };

/// The floor, the cap and the number of events of the next drive.
fn next_drive(rng: &mut Rng) -> (i8, usize, usize) {
  let floor = [0, 3, -5][rng.below(3) as usize];
  let cap = [usize::MAX, usize::MAX, 2, 4][rng.below(4) as usize];
  let events = 1 + rng.below(60) as usize;
  (floor, cap, events)
}

/// One event of a drive: most of them what `poll` asks for, and the rest anything at all.
fn next_event(rng: &mut Rng, phase: Answer) -> Event {
  let power = rng.below(10) as i8;
  let tag = rng.below(4) as u8;
  let value = rng.below(1000) as u32;
  let assoc = [Assoc::Left, Assoc::Right, Assoc::Neither][rng.below(3) as usize];
  let rhs = [
    Rhs::Infix(power),
    Rhs::Adjacent(power),
    Rhs::Postfix(power),
    Rhs::End,
  ][rng.below(4) as usize];
  let kind = if rng.below(10) < 7 {
    let fits: &[u64] = match phase {
      Answer::Term => &[0, 0, 1],
      Answer::Operator => &[2, 2, 3, 5, 6],
      Answer::Reduce | Answer::Done => &[8],
      Answer::Reducing => &[4],
    };
    fits[rng.below(fits.len() as u64) as usize]
  } else {
    rng.below(10)
  };
  match kind {
    0 => Event::Operand(value),
    1 => Event::Prefix(tag, power),
    2 => Event::Infix(assoc, tag, power),
    3 => Event::Postfix(tag, power),
    4 => Event::Reduced(value),
    5 => Event::Close,
    6 => Event::Finish,
    7 => Event::Retract,
    8 => Event::Poll,
    _ => Event::Offer(rhs),
  }
}

/// What a set of drives met.
#[derive(Debug, Default)]
struct Counts {
  /// Refusals, by kind: misplaced, reduce, decline, chain, full.
  refusals: [usize; 5],
  /// Segments made, segments drained, and the most in use at once.
  grown: usize,
  drained: usize,
  deepest: usize,
}

impl Counts {
  fn met(&mut self, why: PrattRefusal) {
    let kind = match why {
      PrattRefusal::Misplaced => 0,
      PrattRefusal::Reduce => 1,
      PrattRefusal::Decline => 2,
      PrattRefusal::Chain => 3,
      PrattRefusal::Full => 4,
    };
    self.refusals[kind] += 1;
  }
}

/// The refusal the machine's contract gives `event` on `before`, or `None` where it takes the event.
fn expected_refusal<S: Stack<Frame>>(before: &Machine<S>, event: Event) -> Option<PrattRefusal> {
  let phase = answer(before);
  let misplaced = |wanted: Answer| (phase != wanted).then_some(PrattRefusal::Misplaced);
  let by_offer = |rhs: Rhs| match before.offer(&offered(rhs)) {
    PrattOffer::Take => None,
    PrattOffer::Reduce => Some(PrattRefusal::Reduce),
    PrattOffer::Decline => Some(PrattRefusal::Decline),
    PrattOffer::Chain => Some(PrattRefusal::Chain),
  };
  let full = (before.depth() >= before.capacity()).then_some(PrattRefusal::Full);
  match event {
    Event::Operand(_) => misplaced(Answer::Term),
    Event::Prefix(..) => misplaced(Answer::Term).or(full),
    Event::Infix(_, _, power) => {
      misplaced(Answer::Operator).or_else(|| by_offer(Rhs::Infix(power)).or(full))
    }
    Event::Postfix(_, power) => {
      misplaced(Answer::Operator).or_else(|| by_offer(Rhs::Postfix(power)))
    }
    Event::Close | Event::Finish => misplaced(Answer::Operator),
    Event::Reduced(_) => misplaced(Answer::Reducing),
    Event::Retract | Event::Poll | Event::Offer(_) => {
      unreachable!("{event:?} is checked by an arm of its own")
    }
  }
}

/// The phase the machine's contract leaves after `event` is taken on `before`.
fn expected_after<S>(before: &Machine<S>, event: Event) -> Answer {
  let pending = before.state.depth > 0;
  match event {
    Event::Prefix(..) | Event::Infix(..) => Answer::Term,
    Event::Operand(_) => Answer::Operator,
    Event::Postfix(..) => Answer::Reduce,
    Event::Close | Event::Finish if pending => Answer::Reduce,
    Event::Close | Event::Finish => Answer::Done,
    Event::Reduced(_) => match &before.state.phase {
      Phase::Reducing(Resume {
        finishing: true, ..
      }) if pending => Answer::Reduce,
      Phase::Reducing(Resume {
        finishing: true, ..
      }) => Answer::Done,
      _ => Answer::Operator,
    },
    Event::Retract | Event::Poll | Event::Offer(_) => {
      unreachable!("{event:?} is checked by an arm of its own")
    }
  }
}

/// One step of a drive on `m`, checked against the machine's contract, and every refusal against the
/// machine as it was before the call. Returns the step's outcome.
fn checked_step<S>(
  m: &mut Machine<S>,
  event: Event,
  fresh: &Machine<S>,
  counts: &mut Counts,
) -> Outcome
where
  S: Stack<Frame> + Clone + PartialEq + Debug,
{
  let before = m.clone();
  let phase = answer(&before);
  let outcome = apply(m, event);
  // The floor an offer reads is the one the frames give, read back off the storage: the drive's
  // own expectations come from the machine's offer, so they cannot catch a floor kept wrong.
  assert!(
    m.floor_is_kept(),
    "{event:?} in {phase:?}: the floor kept is the frame on top's"
  );
  match (event, &outcome) {
    (Event::Poll, Outcome::Polled(poll)) => {
      assert_eq!(answer_of(poll), phase, "`poll` answers the phase");
      match phase {
        Answer::Term | Answer::Operator | Answer::Reducing => {
          assert_eq!(*m, before, "a read changes nothing");
        }
        Answer::Done => assert_eq!(*m, *fresh, "after `Done` the machine is as new"),
        Answer::Reduce => {
          let popped = !matches!(poll, PrattPoll::Reduce(PrattReduction::Postfix(_)));
          assert_eq!(m.depth() + usize::from(popped), before.depth());
          assert_eq!(answer(m), Answer::Reducing);
        }
      }
    }
    (Event::Offer(rhs), Outcome::Offered(offer)) => {
      let offer = *offer;
      assert_eq!(*m, before, "the offer changes nothing");
      if phase != Answer::Operator {
        assert_eq!(
          offer,
          PrattOffer::Decline,
          "no operator is taken outside `Operator`"
        );
      } else if rhs == Rhs::End {
        let end = if before.depth() > 0 {
          PrattOffer::Reduce
        } else {
          PrattOffer::Decline
        };
        assert_eq!(offer, end, "nothing admits `End`");
      }
      if offer == PrattOffer::Reduce {
        assert!(before.depth() > 0, "`Reduce` only under a pending operator");
      }
      if matches!(rhs, Rhs::Postfix(_)) {
        assert_ne!(
          offer,
          PrattOffer::Chain,
          "a postfix operator never meets the chain"
        );
      }
    }
    (Event::Retract, Outcome::Retracted(pending)) => {
      if phase != Answer::Term || before.depth() == 0 {
        assert!(pending.is_none());
        assert_eq!(*m, before, "a `retract` that takes nothing changes nothing");
      } else {
        // One frame fewer, and nothing else moved: the frames below stay, and the latch is the one
        // the popped frame kept, the latch the sub-expression had before the operator was taken.
        let mut below = before.stack.clone();
        let top = below.pop().unwrap();
        assert_eq!(m.stack, below);
        assert_eq!(m.depth() + 1, before.depth());
        assert_eq!(m.state.latch, top.latch);
        match top.operator {
          FrameOperator::Prefix(operator) => {
            assert_eq!(*pending, Some(PrattPending::Prefix(operator)));
            assert_eq!(m.state.phase, Phase::Term);
          }
          FrameOperator::Infix(InfixFrame { left, operator }) => {
            assert_eq!(*pending, Some(PrattPending::Infix(operator)));
            assert_eq!(m.state.phase, Phase::Operator(left));
          }
        }
      }
    }
    (_, outcome) => match expected_refusal(&before, event) {
      Some(why) => {
        assert_eq!(*outcome, refused(event, why), "{event:?} in {phase:?}");
        assert_eq!(
          *m, before,
          "{event:?} in {phase:?}: a refusal changes nothing"
        );
        counts.met(why);
      }
      None => {
        assert!(
          taken(outcome),
          "{event:?} in {phase:?} is taken: {outcome:?}"
        );
        let pushed = matches!(event, Event::Prefix(..) | Event::Infix(..));
        assert_eq!(m.depth(), before.depth() + usize::from(pushed));
        assert_eq!(
          answer(m),
          expected_after(&before, event),
          "{event:?} in {phase:?}"
        );
      }
    },
  }
  outcome
}

/// `DRIVES` random drives through a machine over a fresh storage from `stack`, each step checked
/// against the tables. Returns what they met.
fn drive_plain<S>(stack: impl Fn() -> S) -> Counts
where
  S: Stack<Frame> + Clone + PartialEq + Debug,
{
  let mut rng = Rng(SEED);
  let mut counts = Counts::default();
  for _ in 0..DRIVES {
    let (floor, cap, events) = next_drive(&mut rng);
    let fresh: Machine<S> = PrattResolver::with_stack(stack())
      .min_precedence(floor)
      .max_depth(cap);
    let mut m = fresh.clone();
    for _ in 0..events {
      let event = next_event(&mut rng, answer(&m));
      checked_step(&mut m, event, &fresh, &mut counts);
    }
  }
  counts
}

/// The reference machine and a second one in step: the same state, and the same frame on top.
fn assert_same<S: Stack<Frame>>(reference: &Machine<Wide>, other: &Machine<S>) {
  assert_eq!(reference.state, other.state);
  assert_eq!(reference.top(), other.top());
}

/// The random drive: every call is checked against the tables, and every refusal against the
/// machine before it. Over the default stack too, which refuses the fifth pending operator where
/// the crate cannot allocate.
#[test]
fn a_random_drive_is_total_and_every_refusal_changes_nothing() {
  let counts = drive_plain(Wide::new);
  if !cfg!(miri) {
    assert!(
      counts.refusals.iter().all(|&met| met > 0),
      "every refusal is met: {counts:?}"
    );
  }
  drive_plain(DefaultPrattStack::<Frame>::new);
}

/// The same drives over a `Vec`, in step with the reference machine over a fixed array.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn a_random_drive_over_a_vec_answers_as_over_a_fixed_array() {
  let mut rng = Rng(SEED);
  let mut counts = Counts::default();
  for _ in 0..DRIVES {
    let (floor, cap, events) = next_drive(&mut rng);
    let fresh = machine().min_precedence(floor).max_depth(cap);
    let mut reference = fresh.clone();
    let mut m: Machine<std::vec::Vec<Frame>> = PrattResolver::with_stack(std::vec::Vec::new())
      .min_precedence(floor)
      .max_depth(cap);
    for _ in 0..events {
      let event = next_event(&mut rng, answer(&reference));
      let want = checked_step(&mut reference, event, &fresh, &mut counts);
      assert_eq!(apply(&mut m, event), want, "{event:?}");
      assert_same(&reference, &m);
    }
  }
}

/// What a segment's loop hands back to the frame below it.
enum Exit {
  /// The drive's events ran out.
  Finished,
  /// Its segment drained: the machine's state, to be seated again on the segment below.
  Drained(ResolverState<u32, u8, u8, u8, u8, u8, i8>),
}

/// One drive over chains of segments, beside the reference machine over one wide storage.
struct Drive {
  rng: Rng,
  reference: Machine<Wide>,
  fresh: Machine<Wide>,
  events: usize,
  segments: usize,
  counts: Counts,
}

/// The loop of a drive, over one segment of `K` frames. It grows in a deeper native frame when the
/// segment refuses a frame below the cap, and returns to the frame below once its own frames are
/// gone: the protocol of a loop that grows by segments, with the reference machine as its oracle.
fn run<K: ArraySize>(mut m: Machine<&mut Segment<'_, Frame, K>>, drive: &mut Drive) -> Exit {
  loop {
    // The return test, at every point but between a reduction and its value, where the adapters
    // put it: they return once the value is back.
    if !matches!(m.state.phase, Phase::Reducing(_)) {
      if m.stack.drained() {
        drive.counts.drained += 1;
        return Exit::Drained(m.into_parts().0);
      }
      // What is reserved is what is in use, rounded up to a segment.
      assert_eq!(drive.segments, m.depth().div_ceil(K::USIZE).max(1));
    }
    if drive.events == 0 {
      return Exit::Finished;
    }
    drive.events -= 1;
    let event = next_event(&mut drive.rng, answer(&drive.reference));
    let want = checked_step(&mut drive.reference, event, &drive.fresh, &mut drive.counts);
    let got = apply(&mut m, event);
    if got == want {
      assert_same(&drive.reference, &m);
      continue;
    }
    // The one difference allowed: the segment refused a frame below the cap.
    assert!(
      matches!(got, Outcome::Refused(PrattRefusal::Full, _)) && m.depth() < m.state.max_depth,
      "{event:?}: {got:?}, where the reference answers {want:?}",
    );
    let (state, segment) = m.into_parts();
    match grow(state, &mut *segment, event, &want, drive) {
      Exit::Drained(state) => m = PrattResolver::from_parts(state, segment),
      Exit::Finished => return Exit::Finished,
    }
  }
}

/// One more segment, in a deeper native frame, over the full one: the event it refused is fed
/// again, and taken, and the loop goes on over the new segment until it drains.
#[inline(never)]
fn grow<K: ArraySize>(
  state: ResolverState<u32, u8, u8, u8, u8, u8, i8>,
  below: &mut Segment<'_, Frame, K>,
  event: Event,
  want: &Outcome,
  drive: &mut Drive,
) -> Exit {
  let mut next: Segment<'_, Frame, K> = Segment::over(below);
  let mut m = PrattResolver::from_parts(state, &mut next);
  assert_eq!(
    apply(&mut m, event),
    *want,
    "an empty segment takes what the full one refused"
  );
  assert_same(&drive.reference, &m);
  drive.counts.grown += 1;
  drive.segments += 1;
  drive.counts.deepest = drive.counts.deepest.max(drive.segments);
  let exit = run(m, drive);
  drive.segments -= 1;
  exit
}

/// `DRIVES` drives over chains of segments of `K` frames, beside the reference machine.
fn drive_segmented<K: ArraySize>() -> Counts {
  let mut drive = Drive {
    rng: Rng(SEED),
    reference: machine(),
    fresh: machine(),
    events: 0,
    segments: 1,
    counts: Counts::default(),
  };
  for _ in 0..DRIVES {
    let (floor, cap, events) = next_drive(&mut drive.rng);
    drive.fresh = machine().min_precedence(floor).max_depth(cap);
    drive.reference.clone_from(&drive.fresh);
    drive.events = events;
    let mut root: Segment<'_, Frame, K> = Segment::root();
    let m = PrattResolver::with_stack(&mut root)
      .min_precedence(floor)
      .max_depth(cap);
    match run(m, &mut drive) {
      Exit::Finished => {}
      Exit::Drained(_) => panic!("the first segment has nothing below it"),
    }
    assert_eq!(drive.segments, 1);
  }
  drive.counts
}

/// The random drive over chains of segments of one, two and four frames: every answer and every
/// refusal is the reference machine's, wherever the loop grows and returns.
#[test]
fn a_random_drive_over_chains_of_segments_answers_as_over_one_wide_storage() {
  let sizes = [
    drive_segmented::<U1>(),
    drive_segmented::<U2>(),
    drive_segmented::<U4>(),
  ];
  if !cfg!(miri) {
    for counts in &sizes {
      assert!(
        counts.grown > 0 && counts.drained > 0,
        "it grew and returned: {counts:?}"
      );
    }
    assert!(
      sizes[0].deepest > sizes[1].deepest && sizes[1].deepest > sizes[2].deepest,
      "{sizes:?}"
    );
  }
}

/// With no allocator, a machine driven by hand holds four pending operators by default and refuses
/// the fifth; one that names `PrattStack<_, U16>` refuses the seventeenth.
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[test]
fn without_an_allocator_the_default_storage_refuses_the_fifth_pending_operator() {
  use crate::pratt::PrattStack;
  use hybrid_arraydeque::typenum::U16;

  let mut m: Machine<DefaultPrattStack<Frame>> = PrattResolver::new();
  for tag in 0..4 {
    assert_eq!(m.handle_prefix(prefix(tag, 1)), Ok(()));
  }
  assert_eq!(m.capacity(), 4);
  let before = m.clone();
  assert_eq!(
    m.handle_prefix(prefix(4, 1)),
    Err(PrattRefused::new(PrattRefusal::Full, prefix(4, 1)))
  );
  assert_eq!(m, before);

  let mut m: Machine<PrattStack<Frame, U16>> = PrattResolver::new();
  for tag in 0..16 {
    assert_eq!(m.handle_prefix(prefix(tag, 1)), Ok(()));
  }
  assert_eq!(
    m.handle_prefix(prefix(16, 1)),
    Err(PrattRefused::new(PrattRefusal::Full, prefix(16, 1)))
  );
}

/// With an allocator, the same default spills past four and holds them all.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn with_an_allocator_the_default_storage_spills_past_four() {
  let mut m: Machine<DefaultPrattStack<Frame>> = PrattResolver::new();
  for tag in 0..100 {
    assert_eq!(m.handle_prefix(prefix(tag, 1)), Ok(()));
  }
  assert_eq!(m.depth(), 100);
  assert_eq!(m.capacity(), usize::MAX);
  m.handle_operand(0).unwrap();
  m.finish().unwrap();
  let value = loop {
    match m.poll() {
      PrattPoll::Reduce(PrattReduction::Prefix(reduction)) => {
        m.handle_reduced(reduction.into_components().0 + 1).unwrap();
      }
      PrattPoll::Done(value) => break value,
      other => panic!("after `finish` only reductions and the value: {other:?}"),
    }
  };
  assert_eq!(value, 100);
}
