#![cfg(all(
  feature = "std",
  feature = "combinators",
  feature = "logos_0_16",
  not(feature = "trace")
))]

//! What the flat driver's two adapters allocate, by a counting allocator this suite has to itself.
//!
//! It is compiled without `trace`. That opt-in debugging feature's hooks fire at every
//! instrumented read of the input, with or without a `traced` wrapper, and each formats and writes
//! a line, so a parse under it allocates at every read by design. Without `trace` the suite runs in
//! the `test (valve-off)` job, `cargo test -p tokora --features logos,std`, and in the Miri shards.
//!
//! A checked driver, and a driver made by `folding`, keep their first four pending operators
//! inline, and allocate nothing while no more than four are pending. A bare driver keeps no stack
//! and never allocates. A kept driver keeps the storage it grew: its first parse past four
//! allocates, and the same parse again does not. A driver built afresh for each call grows afresh,
//! and allocates each time.
//!
//! Only the parse is counted. The operand parser, the handler and the folds allocate nothing, and
//! the lexer's log is given room before each parse.

mod flat_fixture;

use core::cell::Cell;
use std::alloc::{GlobalAlloc, Layout, System};

use tokora::{
  Emitter, InputRef, Parse, ParseAttempt, ParseContext, ParseInput, Parser, ParserContext,
  TryParseInput,
  emitter::Fatal,
  pratt::{PrattHandler, Precedenced, flat},
};

use flat_fixture::{Fail, FatalIr, InfixOp, Skip, TABLE, Tk, reserve_lexes};

// ── The counting allocator ────────────────────────────────────────────────────────────────────

thread_local! {
  /// Allocation events on this thread. `alloc`, `alloc_zeroed` and `realloc` all count.
  static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

struct Counting;

/// Counts every allocation event and forwards to the system allocator. The counter is updated
/// through `try_with`, so an allocation made while the thread's local storage is being set up or
/// torn down is not counted rather than re-entering it.
unsafe impl GlobalAlloc for Counting {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    bump();
    unsafe { System.alloc(layout) }
  }

  unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    bump();
    unsafe { System.alloc_zeroed(layout) }
  }

  unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    bump();
    unsafe { System.realloc(ptr, layout, new_size) }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe { System.dealloc(ptr, layout) }
  }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn bump() {
  let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
}

/// Runs `body` and returns how many allocation events it caused on this thread.
fn allocations(body: impl FnOnce()) -> u64 {
  let before = ALLOCATIONS.with(Cell::get);
  body();
  ALLOCATIONS.with(Cell::get) - before
}

// ── An operand parser and a handler that allocate nothing ─────────────────────────────────────

type ErrorIn<'inp, Ctx> =
  <<Ctx as ParseContext<'inp, Skip<'inp>>>::Emitter as Emitter<'inp, Skip<'inp>>>::Error;

/// A digit, as its byte.
#[derive(Debug, Clone, Copy)]
struct Digit;

impl<'inp, Ctx> TryParseInput<'inp, Skip<'inp>, u8, Ctx> for Digit
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
  ) -> Result<ParseAttempt<u8>, ErrorIn<'inp, Ctx>> {
    Ok(
      match input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
        Some(tok) => ParseAttempt::Accept(tok.into_data().0),
        None => ParseAttempt::Decline,
      },
    )
  }
}

/// Counts the items it is told.
#[derive(Debug, Default)]
struct Count(usize);

impl<'inp, Ctx> PrattHandler<'inp, u8, Skip<'inp>, u8, Ctx> for Count
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
{
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
    _operator: Precedenced<Tk, u8>,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 += 1;
    Ok(())
  }

  fn on_operand(
    &mut self,
    _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
    _operand: u8,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 += 1;
    Ok(())
  }

  fn on_infix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
    _operator: InfixOp,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 += 1;
    Ok(())
  }

  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
    _operator: Precedenced<Tk, u8>,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 += 1;
    Ok(())
  }
}

/// A prefix or a postfix operator's fold: its operand's value.
fn keep<'inp, Ctx>(
  _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
  operand: u8,
  _operator: Precedenced<Tk, u8>,
) -> Result<u8, ErrorIn<'inp, Ctx>>
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
{
  Ok(operand)
}

/// An infix operator's fold: the sum of its operands, wrapping.
fn add<'inp, Ctx>(
  _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
  left: u8,
  right: u8,
  _operator: InfixOp,
) -> Result<u8, ErrorIn<'inp, Ctx>>
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
{
  Ok(left.wrapping_add(right))
}

// ── The sources, and a measured parse ─────────────────────────────────────────────────────────

/// `1` and `n` times `operator 1`, then a terminator.
fn chain(operator: char, n: usize) -> String {
  let mut src = String::from("1");
  for _ in 0..n {
    src.push(operator);
    src.push('1');
  }
  src.push(';');
  src
}

/// `n` prefix operators before `1`, then a terminator.
fn prefixes(n: usize) -> String {
  let mut src = "-".repeat(n);
  src.push_str("1;");
  src
}

/// Parses `src` with `parse`, and hands back what it returned, how many items it told, and how
/// many allocations the parse made.
fn measured<'inp>(
  src: &'inp str,
  mut parse: impl FnMut(&mut FatalIr<'inp, '_, true>, &mut Count) -> Result<(), Fail>,
) -> (Result<(), Fail>, usize, u64) {
  reserve_lexes(1 << 12);
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(|input: &mut FatalIr<'inp, '_, true>| {
      let mut count = Count::default();
      let mut result = Ok(());
      let allocated = allocations(|| result = parse(input, &mut count));
      Ok((result, count.0, allocated))
    })
    .parse_str(src)
    .expect("the parse's own error is caught")
}

// ── The cells ─────────────────────────────────────────────────────────────────────────────────

/// While no more than four operators are pending, a checked driver allocates nothing: four
/// right-associative operators, four prefix operators, a ladder of rising powers, and a long
/// left-associative chain, which keeps one.
#[test]
fn a_checked_driver_allocates_nothing_while_four_are_pending() {
  for (src, items) in [
    (chain('^', 4), 9),
    (prefixes(4), 5),
    (String::from("1+2*3^4-5;"), 9),
    (chain('+', 100), 201),
  ] {
    let got = measured(&src, |input, count| {
      flat(TABLE, Digit).parse_into(input, count)
    });
    assert_eq!(got, (Ok(()), items, 0), "{src}");
  }
}

/// A bare driver never allocates, however many operators would be pending.
#[test]
fn a_bare_driver_never_allocates() {
  let n = if cfg!(miri) { 64 } else { 1000 };
  for (src, items) in [(chain('^', n), 2 * n + 1), (prefixes(n), n + 1)] {
    let got = measured(&src, |input, count| {
      flat(TABLE, Digit).bare().parse_into(input, count)
    });
    assert_eq!(got, (Ok(()), items, 0), "{src}");
  }
}

/// One kept checked driver parses a chain that keeps nine operators pending, twice. The first
/// parse grows its storage, and the second finds it grown.
#[test]
fn a_kept_checked_driver_allocates_on_its_first_parse_alone() {
  let nine = chain('^', 9);
  let mut driver = flat(TABLE, Digit);
  let (result, items, first) = measured(&nine, |input, count| driver.parse_into(input, count));
  assert_eq!((result, items), (Ok(()), 19));
  assert!(first > 0, "the first parse grows the storage");
  let second = measured(&nine, |input, count| driver.parse_into(input, count));
  assert_eq!(second, (Ok(()), 19, 0), "the second parse finds it grown");
}

/// A checked driver built afresh for each call grows afresh, and allocates each time.
#[test]
fn a_driver_built_afresh_allocates_each_time() {
  let nine = chain('^', 9);
  for call in 0..2 {
    let (result, items, allocated) = measured(&nine, |input, count| {
      flat(TABLE, Digit).parse_into(input, count)
    });
    assert_eq!((result, items), (Ok(()), 19), "call {call}");
    assert!(
      allocated > 0,
      "call {call}: a fresh driver grows its own storage"
    );
  }
}

/// While no more than four operators are pending, a driver made by `folding` allocates nothing,
/// over the same four sources.
#[test]
fn a_folding_driver_allocates_nothing_while_four_are_pending() {
  for src in [
    chain('^', 4),
    prefixes(4),
    String::from("1+2*3^4-5;"),
    chain('+', 100),
  ] {
    let (result, _, allocated) = measured(&src, |input, _| {
      flat(TABLE, Digit)
        .folding(keep, add, keep)
        .parse_input(input)
        .map(|_| ())
    });
    assert_eq!((result, allocated), (Ok(()), 0), "{src}");
  }
}

/// One kept driver made by `folding` parses a chain that keeps nine operators pending, twice. The
/// first parse grows its storage, and the second finds it grown.
#[test]
fn a_kept_folding_driver_allocates_on_its_first_parse_alone() {
  let nine = chain('^', 9);
  let mut driver = flat(TABLE, Digit).folding(keep, add, keep);
  let (result, _, first) = measured(&nine, |input, _| driver.parse_input(input).map(|_| ()));
  assert_eq!(result, Ok(()));
  assert!(first > 0, "the first parse grows the storage");
  let (result, _, second) = measured(&nine, |input, _| driver.parse_input(input).map(|_| ()));
  assert_eq!(
    (result, second),
    (Ok(()), 0),
    "the second parse finds it grown"
  );
}

/// A driver made by `folding` afresh for each call grows afresh, and allocates each time.
#[test]
fn a_folding_driver_built_afresh_allocates_each_time() {
  let nine = chain('^', 9);
  for call in 0..2 {
    let (result, _, allocated) = measured(&nine, |input, _| {
      flat(TABLE, Digit)
        .folding(keep, add, keep)
        .parse_input(input)
        .map(|_| ())
    });
    assert_eq!(result, Ok(()), "call {call}");
    assert!(
      allocated > 0,
      "call {call}: a fresh driver grows its own storage"
    );
  }
}
