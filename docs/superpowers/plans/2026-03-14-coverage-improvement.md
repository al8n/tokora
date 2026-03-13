# Code Coverage Improvement Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve tarpaulin code coverage from ~85.5% (after excluding examples) to over 90% by writing targeted tests for uncovered library code paths.

**Architecture:** Three phases — (1) tarpaulin config to exclude examples, (2) integration tests for state machine and cache paths, (3) unit tests for trait defaults, error types, and utility code. Tests follow existing patterns in `tests/common/` and `tests/state_machine.rs`.

**Tech Stack:** Rust, cargo-tarpaulin, logos (for test lexers), generic-arraydeque (for cache types)

---

## Chunk 1: Configuration and Baseline

### Task 1: Add tarpaulin.toml to exclude examples

**Files:**
- Create: `tarpaulin.toml`

- [ ] **Step 1: Create tarpaulin.toml**

```toml
[default]
exclude-files = ["tokit/examples/*"]
```

- [ ] **Step 2: Verify existing tests still pass**

Run: `cd /Users/user/Develop/personal/tokit && cargo test --all-features -p tokit`
Expected: All tests pass (685+)

- [ ] **Step 3: Commit**

```bash
git add tarpaulin.toml
git commit -m "chore: add tarpaulin.toml to exclude examples from coverage"
```

---

## Chunk 2: Utility and Error Type Tests

These are small, independent, self-contained tests targeting trait defaults and construction methods. Each task produces a single test file.

### Task 2: MissingToken error struct tests

**Files:**
- Create: `tokit/tests/missing_token.rs`

This file tests the `MissingToken` error struct constructors, builder methods, accessors, and formatting in `tokit/src/error/token/missing_token/mod.rs`.

- [ ] **Step 1: Write tests**

```rust
#![cfg(feature = "std")]

use tokit::error::token::MissingToken;
use tokit::utils::CowStr;

// ── Constructors ────────────────────────────────────────────────────────────

#[test]
fn trailing_constructor() {
  let mt: MissingToken<'_, (), usize> = MissingToken::trailing(42);
  assert_eq!(*mt.offset_ref(), 42);
  assert!(mt.message().is_none());
  assert!(mt.expected().is_none());
}

#[test]
fn trailing_with_message() {
  let mt: MissingToken<'_, (), usize> =
    MissingToken::trailing_with_message(10, CowStr::from_static("expected comma"));
  assert_eq!(*mt.offset_ref(), 10);
  assert_eq!(mt.message().unwrap().as_str(), "expected comma");
}

#[test]
fn leading_constructor() {
  let mt: MissingToken<'_, (), usize> = MissingToken::leading(5);
  assert_eq!(*mt.offset_ref(), 5);
  assert!(mt.message().is_none());
}

#[test]
fn leading_with_message() {
  let mt: MissingToken<'_, (), usize> =
    MissingToken::leading_with_message(0, CowStr::from_static("need semicolon"));
  assert_eq!(*mt.offset_ref(), 0);
  assert_eq!(mt.message().unwrap().as_str(), "need semicolon");
}

#[test]
fn new_constructor() {
  let mt: MissingToken<'_, (), usize> = MissingToken::new(99);
  assert_eq!(*mt.offset_ref(), 99);
}

// ── Builder methods ─────────────────────────────────────────────────────────

#[test]
fn with_message_builder() {
  let mt: MissingToken<'_, (), usize> =
    MissingToken::new(0).with_message(CowStr::from_static("hello"));
  assert_eq!(mt.message().unwrap().as_str(), "hello");
}

// ── Accessors ───────────────────────────────────────────────────────────────

#[test]
fn offset_copy() {
  let mt: MissingToken<'_, (), usize> = MissingToken::new(42);
  assert_eq!(mt.offset(), 42);
}

#[test]
fn offset_mut() {
  let mut mt: MissingToken<'_, (), usize> = MissingToken::new(0);
  *mt.offset_mut() = 100;
  assert_eq!(mt.offset(), 100);
}

#[test]
fn message_mut() {
  let mut mt: MissingToken<'_, (), usize> =
    MissingToken::new(0).with_message(CowStr::from_static("old"));
  if let Some(m) = mt.message_mut() {
    *m = CowStr::from_static("new");
  }
  assert_eq!(mt.message().unwrap().as_str(), "new");
}

// ── into_components ─────────────────────────────────────────────────────────

#[test]
fn into_components() {
  let mt: MissingToken<'_, (), usize> =
    MissingToken::new(42).with_message(CowStr::from_static("msg"));
  let (off, exp, msg) = mt.into_components();
  assert_eq!(off, 42);
  assert!(exp.is_none());
  assert_eq!(msg.unwrap().as_str(), "msg");
}

// ── Display / Debug ─────────────────────────────────────────────────────────

#[test]
fn display_formatting() {
  let mt: MissingToken<'_, &str, usize> = MissingToken::new(0);
  // Just ensure it doesn't panic
  let _ = format!("{mt}");
}

#[test]
fn debug_formatting() {
  let mt: MissingToken<'_, &str, usize> = MissingToken::new(0);
  let _ = format!("{mt:?}");
}

// ── From<MissingToken> for () ───────────────────────────────────────────────

#[test]
fn from_missing_token_for_unit() {
  let mt: MissingToken<'_, (), usize> = MissingToken::new(0);
  let _: () = mt.into();
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --all-features -p tokit --test missing_token`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tokit/tests/missing_token.rs
git commit -m "test: add MissingToken error struct coverage"
```

---

### Task 3: CowStr and OneOf utility tests

**Files:**
- Create: `tokit/tests/utils_coverage.rs`

Tests for `utils/message.rs` (CowStr) and `utils/oneof.rs` (OneOf) feature-gated methods.

- [ ] **Step 1: Write tests**

```rust
#![cfg(feature = "std")]

use std::borrow::Cow;
use tokit::utils::{CowStr, OneOf};

// ── CowStr ──────────────────────────────────────────────────────────────────

#[test]
fn cowstr_from_static() {
  let s = CowStr::from_static("hello");
  assert_eq!(s.as_str(), "hello");
}

#[test]
fn cowstr_from_string() {
  let s = CowStr::from_string(String::from("dynamic"));
  assert_eq!(s.as_str(), "dynamic");
}

#[test]
fn cowstr_to_mut() {
  let mut s = CowStr::from_static("hello");
  let m = s.to_mut();
  m.push_str(" world");
  assert_eq!(s.as_str(), "hello world");
}

#[test]
fn cowstr_into_inner() {
  let s = CowStr::from_static("test");
  let inner = s.into_inner();
  assert_eq!(&*inner, "test");
}

#[test]
fn cowstr_as_inner() {
  let s = CowStr::from_static("test");
  let _ = s.as_inner();
}

#[test]
fn cowstr_from_string_impl() {
  let s: CowStr = String::from("owned").into();
  assert_eq!(s.as_str(), "owned");
}

#[test]
fn cowstr_from_cow() {
  let cow: Cow<'static, str> = Cow::Borrowed("borrowed");
  let s: CowStr = cow.into();
  assert_eq!(s.as_str(), "borrowed");
}

#[test]
fn cowstr_into_cow() {
  let s = CowStr::from_static("test");
  let cow: Cow<'static, str> = s.into();
  assert_eq!(&*cow, "test");
}

#[test]
fn cowstr_ref_into_cow() {
  let s = CowStr::from_static("test");
  let cow: Cow<'static, str> = (&s).into();
  assert_eq!(&*cow, "test");
}

#[test]
fn cowstr_as_ref() {
  let s = CowStr::from_static("test");
  let r: &str = s.as_ref();
  assert_eq!(r, "test");
}

#[test]
fn cowstr_borrow() {
  use std::borrow::Borrow;
  let s = CowStr::from_static("test");
  let r: &str = s.borrow();
  assert_eq!(r, "test");
}

#[test]
fn cowstr_to_mut_from_static() {
  // to_mut() on a Borrowed variant clones to Owned, then returns &mut String
  let mut s = CowStr::from_static("test");
  let m = s.to_mut();
  m.push_str("!");
  assert_eq!(s.as_str(), "test!");
}

#[test]
fn cowstr_display() {
  let s = CowStr::from_static("hello");
  assert_eq!(format!("{s}"), "hello");
}

#[test]
fn cowstr_debug() {
  let s = CowStr::from_static("hello");
  let _ = format!("{s:?}");
}

// ── OneOf ───────────────────────────────────────────────────────────────────

#[test]
fn oneof_from_slice() {
  let items: &[i32] = &[1, 2, 3];
  let o = OneOf::from_slice(items);
  assert_eq!(o.as_slice(), &[1, 2, 3]);
}

#[test]
fn oneof_from_vec() {
  let o = OneOf::from_vec(vec![1, 2, 3]);
  assert_eq!(o.as_slice(), &[1, 2, 3]);
}

#[test]
fn oneof_to_mut() {
  let items: &[i32] = &[1, 2];
  let mut o = OneOf::from_slice(items);
  let m = o.to_mut();
  assert_eq!(m, &[1, 2]);
}

#[test]
fn oneof_into_inner() {
  let o = OneOf::from_vec(vec![42]);
  let inner = o.into_inner();
  assert_eq!(&*inner, &[42]);
}

#[test]
fn oneof_as_inner() {
  let o = OneOf::from_vec(vec![1]);
  let _ = o.as_inner();
}

#[test]
fn oneof_from_vec_impl() {
  let o: OneOf<'_, i32> = vec![1, 2].into();
  assert_eq!(o.as_slice(), &[1, 2]);
}

#[test]
fn oneof_from_cow() {
  let cow: Cow<'_, [i32]> = Cow::Borrowed(&[1, 2]);
  let o: OneOf<'_, i32> = cow.into();
  assert_eq!(o.as_slice(), &[1, 2]);
}

#[test]
fn oneof_into_cow() {
  let o = OneOf::from_vec(vec![1]);
  let cow: Cow<'_, [i32]> = o.into();
  assert_eq!(&*cow, &[1]);
}

#[test]
fn oneof_ref_into_cow() {
  let o = OneOf::from_vec(vec![1]);
  let cow: Cow<'_, [i32]> = (&o).into();
  assert_eq!(&*cow, &[1]);
}

#[test]
fn oneof_as_ref() {
  let o = OneOf::from_vec(vec![1, 2]);
  let r: &[i32] = o.as_ref();
  assert_eq!(r, &[1, 2]);
}

#[test]
fn oneof_borrow() {
  use std::borrow::Borrow;
  let o = OneOf::from_vec(vec![1, 2]);
  let r: &[i32] = o.borrow();
  assert_eq!(r, &[1, 2]);
}

#[test]
fn oneof_display() {
  let o = OneOf::from_vec(vec![1, 2, 3]);
  let _ = format!("{o}");
}

#[test]
fn oneof_debug() {
  let o = OneOf::from_vec(vec![1, 2]);
  let _ = format!("{o:?}");
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --all-features -p tokit --test utils_coverage`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tokit/tests/utils_coverage.rs
git commit -m "test: add CowStr and OneOf utility coverage"
```

---

### Task 4: Parser construction method tests

**Files:**
- Create: `tokit/tests/parser_construction.rs`

Tests for `tokit/src/parser/mod.rs` — construction methods, Deref, Default.

- [ ] **Step 1: Write tests**

```rust
#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, Parser, ParserContext,
  Token as TokenTrait,
  error::{UnexpectedEot, token::UnexpectedToken},
  input::Cursor,
  span::Spanned,
};

use common::{TestLexer, Token};

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug)]
struct E;

impl From<()> for E {
  fn from(_: ()) -> Self { E }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self { E }
}

impl From<UnexpectedEot> for E {
  fn from(_: UnexpectedEot) -> Self { E }
}

// ── Emitter ─────────────────────────────────────────────────────────────────

struct TestEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEmitter {
  type Error = E;

  fn emit_lexer_error(
    &mut self,
    _: Spanned<<<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(E) }

  fn emit_unexpected_token(
    &mut self,
    _: tokit::error::token::UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(E) }

  fn emit_error(
    &mut self,
    err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(err.into_data()) }

  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>) where TestLexer<'inp>: Lexer<'inp> {}
}

// ── Parser function ─────────────────────────────────────────────────────────

fn parse_first_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<i64, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
{
  match inp.next()? {
    Some(tok) => match tok.into_data() {
      Token::Num(n) => Ok(n),
      _ => Err(E),
    },
    None => Err(E),
  }
}

// ── Construction tests ──────────────────────────────────────────────────────

#[test]
fn parser_new_and_apply() {
  let r: Result<i64, _> = Parser::new()
    .apply(parse_first_num)
    .parse_str("42");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn parser_with_context() {
  let ctx = ParserContext::new(TestEmitter);
  let r: Result<i64, _> = Parser::with_context(ctx)
    .apply(parse_first_num)
    .parse_str("42");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn parser_with_parser() {
  let r: Result<i64, _> = Parser::with_parser(parse_first_num)
    .parse_str("42");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn parser_with_parser_and_context() {
  let ctx = ParserContext::new(TestEmitter);
  let r: Result<i64, _> = Parser::with_parser_and_context(parse_first_num, ctx)
    .parse_str("42");
  assert_eq!(r.unwrap(), 42);
}

// ── Deref ───────────────────────────────────────────────────────────────────

#[test]
fn parser_deref() {
  let p = Parser::with_parser(parse_first_num);
  // Deref gives access to the inner parser function
  let _: &_ = &*p;
}

#[test]
fn parser_deref_mut() {
  let mut p = Parser::with_parser(parse_first_num);
  let _: &mut _ = &mut *p;
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --all-features -p tokit --test parser_construction`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tokit/tests/parser_construction.rs
git commit -m "test: add Parser construction method coverage"
```

---

## Chunk 3: Cache and Input Tests

### Task 5: Cache rewind tests via InputRef save/restore

**Files:**
- Create: `tokit/tests/cache_rewind_coverage.rs`

Tests for cache rewind logic in `cache/generic_arraydeque.rs` and `cache/option.rs`. Since `Checkpoint` is `pub(super)` and cannot be constructed directly from integration tests, we exercise rewind through `InputRef::save()` + `InputRef::restore()` which internally calls `cache.rewind()`.

**Key insight:** `Checkpoint::new()` is `pub(super)`, so we cannot create checkpoints directly. Instead we:
1. `save()` a checkpoint at a known position
2. Advance the parser (consuming/peeking tokens to populate cache)
3. `restore()` to trigger `cache.rewind()` with various cursor positions

- [ ] **Step 1: Write rewind integration tests**

```rust
#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, Parser, ParserContext,
  Token as TokenTrait,
  error::{UnexpectedEot, token::{UnexpectedToken, UnexpectedTokenOf}},
  input::Cursor,
  span::Spanned,
};

use common::{TestLexer, Token};

#[derive(Debug)]
struct E;
impl From<()> for E { fn from(_: ()) -> Self { E } }
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self { E }
}
impl From<UnexpectedEot> for E { fn from(_: UnexpectedEot) -> Self { E } }

struct TestEm;
impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEm {
  type Error = E;
  fn emit_lexer_error(&mut self, _: Spanned<<<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(E) }
  fn emit_unexpected_token(&mut self, _: UnexpectedTokenOf<'inp, TestLexer<'inp>>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(E) }
  fn emit_error(&mut self, err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(err.into_data()) }
  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>) where TestLexer<'inp>: Lexer<'inp> {}
}

fn ctx() -> ParserContext<'static, TestLexer<'static>, TestEm> {
  ParserContext::new(TestEm)
}

// ── Rewind: save at start, advance, restore ─────────────────────────────────

#[test]
fn rewind_to_start_after_consuming() {
  // Save at start, consume tokens, restore → cache rewound, re-parse succeeds
  let r: Result<(i64, i64), _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let ckp = inp.save();
      let first = match inp.next()? {
        Some(tok) => match tok.into_data() { Token::Num(n) => n, _ => return Err(E) },
        None => return Err(E),
      };
      inp.restore(ckp);
      // Re-consume the same token after restore
      let again = match inp.next()? {
        Some(tok) => match tok.into_data() { Token::Num(n) => n, _ => return Err(E) },
        None => return Err(E),
      };
      Ok((first, again))
    })
    .parse_str("42");
  let (a, b) = r.unwrap();
  assert_eq!(a, b);
  assert_eq!(a, 42);
}

#[test]
fn rewind_after_peek_populates_cache() {
  // Peek populates cache, save, consume, restore → exercises cache rewind path
  let r: Result<i64, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      // Peek to fill cache
      let _ = inp.peek_one()?;
      let ckp = inp.save();
      // Consume from cache
      let _ = inp.next()?;
      // Restore — triggers cache.rewind() with cached tokens
      inp.restore(ckp);
      // Should be able to consume again
      match inp.next()? {
        Some(tok) => match tok.into_data() { Token::Num(n) => Ok(n), _ => Err(E) },
        None => Err(E),
      }
    })
    .parse_str("42 99");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn rewind_mid_stream() {
  // Save after consuming first token, consume more, restore mid-stream
  let r: Result<Vec<i64>, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      // Consume first token
      let _ = inp.next()?;
      // Save after first token
      let ckp = inp.save();
      // Consume second and third
      let _ = inp.next()?;
      let _ = inp.next()?;
      // Restore to after first token
      inp.restore(ckp);
      // Re-consume second token
      let mut results = Vec::new();
      while let Some(tok) = inp.next()? {
        if let Token::Num(n) = tok.into_data() {
          results.push(n);
        }
      }
      Ok(results)
    })
    .parse_str("1 2 3");
  let nums = r.unwrap();
  assert_eq!(nums, vec![2, 3]);
}

#[test]
fn rewind_with_empty_remaining_input() {
  // Save, consume all tokens, restore
  let r: Result<i64, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let ckp = inp.save();
      while inp.next()?.is_some() {}
      inp.restore(ckp);
      match inp.next()? {
        Some(tok) => match tok.into_data() { Token::Num(n) => Ok(n), _ => Err(E) },
        None => Err(E),
      }
    })
    .parse_str("42");
  assert_eq!(r.unwrap(), 42);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --all-features -p tokit --test cache_rewind_coverage`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tokit/tests/cache_rewind_coverage.rs
git commit -m "test: add cache rewind coverage via save/restore"
```

---

### Task 6: InputRef consume_cached and sync_through tests

**Files:**
- Create: `tokit/tests/input_ref_coverage.rs`

Tests for `input/input_ref/consume_cached.rs` and `input/input_ref/sync_through.rs` — exercising the consume paths and sync logic that aren't covered by existing tests.

- [ ] **Step 1: Read existing test modules in these files**

Read: `tokit/src/input/input_ref/consume_cached.rs` (line 67+)
Read: `tokit/src/input/input_ref/sync_through.rs`

Understand what helpers exist and which paths are already tested.

- [ ] **Step 2: Write integration tests**

```rust
#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, Parser, ParserContext,
  Token as TokenTrait,
  error::{UnexpectedEot, token::{UnexpectedToken, UnexpectedTokenOf}},
  input::Cursor,
  span::Spanned,
};

use common::{TestLexer, Token, TokenKind};

#[derive(Debug)]
struct E;

impl From<()> for E { fn from(_: ()) -> Self { E } }
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self { E }
}
impl From<UnexpectedEot> for E { fn from(_: UnexpectedEot) -> Self { E } }

struct RecEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for RecEmitter {
  type Error = E;
  fn emit_lexer_error(&mut self, _: Spanned<<<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Ok(()) }
  fn emit_unexpected_token(&mut self, _: UnexpectedTokenOf<'inp, TestLexer<'inp>>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Ok(()) }
  fn emit_error(&mut self, _: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Ok(()) }
  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>) where TestLexer<'inp>: Lexer<'inp> {}
}

fn rec_ctx() -> ParserContext<'static, TestLexer<'static>, RecEmitter> {
  ParserContext::new(RecEmitter)
}

// ── consume_cached tests ────────────────────────────────────────────────────

#[test]
fn consume_cached_one_empty() {
  // When cache is empty, consume_cached_one returns None
  let r: Result<Option<i64>, _> = Parser::with_context(rec_ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let tok = inp.consume_cached_one();
      Ok(tok.map(|t| match t.into_data() {
        Token::Num(n) => n,
        _ => -1,
      }))
    })
    .parse_str("42");
  assert_eq!(r.unwrap(), None);
}

#[test]
fn consume_cached_one_after_peek() {
  // After peeking, cache is populated — consume_cached_one returns the token
  let r: Result<Option<i64>, _> = Parser::with_context(rec_ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      // Peek to populate cache
      let _ = inp.peek_one()?;
      let tok = inp.consume_cached_one();
      Ok(tok.map(|t| match t.into_data() {
        Token::Num(n) => n,
        _ => -1,
      }))
    })
    .parse_str("42");
  assert_eq!(r.unwrap(), Some(42));
}

#[test]
fn consume_all_cached_empty() {
  let r: Result<bool, _> = Parser::with_context(rec_ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.consume_all_cached().is_none())
    })
    .parse_str("1 2 3");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn consume_all_cached_after_peek() {
  let r: Result<bool, _> = Parser::with_context(rec_ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let _ = inp.peek_one()?;
      Ok(inp.consume_all_cached().is_some())
    })
    .parse_str("1 2 3");
  assert_eq!(r.unwrap(), true);
}

// ── sync_through tests ──────────────────────────────────────────────────────

#[test]
fn sync_through_finds_token() {
  // sync_through skips non-matching tokens and finds the matching one
  let r: Result<bool, _> = Parser::with_context(rec_ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let found = inp.sync_through(
        |t| matches!(t.data(), Token::Comma),
        || None,
      )?;
      Ok(found.is_some())
    })
    .parse_str("1 2 , 3");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn sync_through_no_match() {
  // sync_through exhausts input without finding match
  let r: Result<bool, _> = Parser::with_context(rec_ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let found = inp.sync_through(
        |t| matches!(t.data(), Token::Comma),
        || None,
      )?;
      Ok(found.is_none())
    })
    .parse_str("1 2 3");
  assert_eq!(r.unwrap(), true);
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --all-features -p tokit --test input_ref_coverage`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add tokit/tests/input_ref_coverage.rs
git commit -m "test: add InputRef consume_cached and sync_through coverage"
```

---

### Task 7: Peek cache overflow tests

**Files:**
- Modify: `tokit/src/input/input_ref/peek.rs` (add to existing `#[cfg(test)]` module at line 279)

Tests for the cache overflow handling in `peek_with_emitter_inner` (lines 76-126). The overflow path is triggered when the peek window requests more tokens than the cache can hold.

**Context:** The existing test module (lines 144-279) uses `parse_with()` with a `()` `ParseContext` which uses a default `GenericArrayDeque<CachedToken, U3>` cache (capacity 3). The default cache size depends on the `ParseContext::provide()` implementation. To trigger overflow, we need a window larger than the cache.

- [ ] **Step 1: Add overflow tests to the existing test module**

Add these tests after line 278 (before the closing `}` of the `tests` module) in `tokit/src/input/input_ref/peek.rs`:

```rust
  #[test]
  fn peek_window_exceeds_cache_capacity() {
    // U4 window on default U3 cache — triggers overflow path (lines 76-126)
    parse_with("abc 123 def ghi", |inp| {
      use generic_arraydeque::typenum::U4;
      let peeked = inp.peek::<U4>()?;
      // Should see all 4 tokens even though cache can only hold 3
      assert_eq!(peeked.len(), 4);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn peek_overflow_tokens_correct() {
    // Verify overflowed tokens have correct data
    parse_with("abc 123 def ghi jkl", |inp| {
      use generic_arraydeque::typenum::U4;
      let peeked = inp.peek::<U4>()?;
      assert_eq!(peeked.len(), 4);
      // Peek again — should get same result (tokens cached or re-lexed)
      let peeked2 = inp.peek::<U4>()?;
      assert_eq!(peeked2.len(), 4);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn peek_overflow_then_consume() {
    // Peek with overflow, then consume tokens normally
    parse_with("abc 123 def ghi", |inp| {
      use generic_arraydeque::typenum::U4;
      let peeked = inp.peek::<U4>()?;
      assert_eq!(peeked.len(), 4);
      // Consume should work correctly after overflow peek
      let tok = inp.next()?;
      assert!(tok.is_some());
      Ok(())
    })
    .unwrap();
  }
```

- [ ] **Step 2: Run tests**

Run: `cargo test --all-features -p tokit input_ref::peek`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tokit/src/input/input_ref/peek.rs
git commit -m "test: add peek cache overflow coverage"
```

---

## Chunk 4: Remaining Coverage Gaps

### Task 8: try_expect punctuator method tests

**Files:**
- Create: `tokit/tests/try_expect_coverage.rs`

Tests for the macro-generated `try_expect_<punct>()` and `expect_<punct>()` methods in `input/input_ref/try_expect.rs`, plus the empty-cache branches in `try_expect`, `try_expect_map`, and `try_expect_and_then`.

- [ ] **Step 1: Write tests**

```rust
#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, Parser, ParserContext,
  Token as TokenTrait,
  error::{UnexpectedEot, token::{UnexpectedToken, UnexpectedTokenOf}},
  input::Cursor,
  span::Spanned,
};

use common::{TestLexer, Token};

#[derive(Debug)]
struct E;
impl From<()> for E { fn from(_: ()) -> Self { E } }
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self { E }
}
impl From<UnexpectedEot> for E { fn from(_: UnexpectedEot) -> Self { E } }

struct TestEm;
impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEm {
  type Error = E;
  fn emit_lexer_error(&mut self, _: Spanned<<<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(E) }
  fn emit_unexpected_token(&mut self, _: UnexpectedTokenOf<'inp, TestLexer<'inp>>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(E) }
  fn emit_error(&mut self, err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(err.into_data()) }
  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>) where TestLexer<'inp>: Lexer<'inp> {}
}

fn ctx() -> ParserContext<'static, TestLexer<'static>, TestEm> {
  ParserContext::new(TestEm)
}

// ── try_expect_<punct> methods ──────────────────────────────────────────────

#[test]
fn try_expect_comma_success() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_comma()?.is_some())
    })
    .parse_str(",");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_comma_decline() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_comma()?.is_none())
    })
    .parse_str("42");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_semicolon() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_semicolon()?.is_some())
    })
    .parse_str(";");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_open_paren() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_open_paren()?.is_some())
    })
    .parse_str("(");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_close_paren() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_close_paren()?.is_some())
    })
    .parse_str(")");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_open_bracket() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_open_bracket()?.is_some())
    })
    .parse_str("[");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_close_bracket() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_close_bracket()?.is_some())
    })
    .parse_str("]");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_open_brace() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_open_brace()?.is_some())
    })
    .parse_str("{");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_close_brace() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      Ok(inp.try_expect_close_brace()?.is_some())
    })
    .parse_str("}");
  assert_eq!(r.unwrap(), true);
}

// ── try_expect with empty cache (goes through try_expect_on_input) ──────────

#[test]
fn try_expect_empty_cache_match() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      // No peek first — cache is empty
      let tok = inp.try_expect(|t| matches!(t.data(), Token::Num(_)))?;
      Ok(tok.is_some())
    })
    .parse_str("42");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_empty_cache_no_match() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let tok = inp.try_expect(|t| matches!(t.data(), Token::Comma))?;
      Ok(tok.is_none())
    })
    .parse_str("42");
  assert_eq!(r.unwrap(), true);
}

// ── try_expect_map with empty cache ─────────────────────────────────────────

#[test]
fn try_expect_map_success() {
  let r: Result<Option<i64>, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let result = inp.try_expect_map(|t| match t.data() {
        Token::Num(n) => Some(*n),
        _ => None,
      })?;
      Ok(result.map(|(n, _)| n))
    })
    .parse_str("42");
  assert_eq!(r.unwrap(), Some(42));
}

#[test]
fn try_expect_map_decline() {
  let r: Result<bool, _> = Parser::with_context(ctx())
    .apply(|inp: &mut InputRef<'_, '_, TestLexer<'_>, _>| {
      let result = inp.try_expect_map::<i64, _>(|t| match t.data() {
        Token::Num(n) => Some(*n),
        _ => None,
      })?;
      Ok(result.is_none())
    })
    .parse_str(",");
  assert_eq!(r.unwrap(), true);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --all-features -p tokit --test try_expect_coverage`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tokit/tests/try_expect_coverage.rs
git commit -m "test: add try_expect punctuator and empty-cache coverage"
```

---

### Task 9: Punctuator trait default and reference delegation tests

**Files:**
- Create: `tokit/tests/punct_trait_coverage.rs`

Tests for `src/punct.rs` — trait default methods (`description()`, `eval()`, `unexpected_token()`) and reference delegation.

- [ ] **Step 1: Write tests**

```rust
#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  Lexer,
  punct::{Comma, Punctuator},
};

use common::TestLexer;

// ── Trait default methods ───────────────────────────────────────────────────

#[test]
fn punctuator_description_is_some() {
  // Comma overrides the trait default to return a description string
  let desc = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::description();
  assert!(desc.is_some());
  assert!(!desc.unwrap().as_str().is_empty());
}

#[test]
fn punctuator_name_is_populated() {
  let name = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::name();
  assert!(!name.as_str().is_empty());
}

#[test]
fn punctuator_eval_matches_kind() {
  use common::TokenKind;
  let kind = TokenKind::Comma;
  let matches = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::eval(&kind);
  assert!(matches);
}

#[test]
fn punctuator_eval_rejects_wrong_kind() {
  use common::TokenKind;
  let kind = TokenKind::Num;
  let matches = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::eval(&kind);
  assert!(!matches);
}

// ── Reference delegation ────────────────────────────────────────────────────

#[test]
fn ref_punctuator_name() {
  let name = <&Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::name();
  let orig = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::name();
  assert_eq!(name.as_str(), orig.as_str());
}

#[test]
fn ref_punctuator_eval() {
  use common::TokenKind;
  let kind = TokenKind::Comma;
  let matches = <&Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::eval(&kind);
  assert!(matches);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --all-features -p tokit --test punct_trait_coverage`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tokit/tests/punct_trait_coverage.rs
git commit -m "test: add Punctuator trait default and ref delegation coverage"
```

---

### Task 10: LitToken reference delegation and composite default tests

**Files:**
- Create: `tokit/tests/lit_token_coverage.rs`

Tests for `token/lit.rs` — reference delegation (`&T where T: LitToken`) and composite default methods (`is_integer_literal()`, `is_float_literal()`, etc.). Priority 1 in spec (40 uncovered lines).

- [ ] **Step 1: Write tests**

```rust
#![cfg(feature = "std")]

use tokit::Token;
use tokit::token::LitToken;

// ── Minimal token types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Kind;

impl core::fmt::Display for Kind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "kind")
  }
}

macro_rules! define_lit_tok {
  ($name:ident, $method:ident) => {
    #[derive(Debug, Clone)]
    struct $name;

    impl core::fmt::Display for $name {
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, stringify!($name))
      }
    }

    impl Token<'_> for $name {
      type Kind = Kind;
      type Error = ();
      fn kind(&self) -> Kind { Kind }
      fn is_trivia(&self) -> bool { false }
    }

    impl LitToken<'_> for $name {
      fn $method(&self) -> bool { true }
    }
  };
}

define_lit_tok!(DecTok, is_decimal_literal);
define_lit_tok!(HexTok, is_hex_literal);
define_lit_tok!(OctTok, is_octal_literal);
define_lit_tok!(BinTok, is_binary_literal);
define_lit_tok!(FloatTok, is_float_decimal_literal);
define_lit_tok!(HexFloatTok, is_float_hex_literal);
define_lit_tok!(InlineStrTok, is_inline_string_literal);
define_lit_tok!(MultilineStrTok, is_multiline_string_literal);
define_lit_tok!(RawStrTok, is_raw_string_literal);
define_lit_tok!(CharTok, is_char_literal);
define_lit_tok!(ByteTok, is_byte_literal);
define_lit_tok!(ByteStrTok, is_byte_string_literal);
define_lit_tok!(TrueTok, is_true_literal);
define_lit_tok!(FalseTok, is_false_literal);
define_lit_tok!(NullTok, is_null_literal);

// ── Composite default tests ─────────────────────────────────────────────────

#[test]
fn is_integer_literal_decimal() {
  assert!(DecTok.is_integer_literal());
  assert!(HexTok.is_integer_literal());
  assert!(OctTok.is_integer_literal());
  assert!(BinTok.is_integer_literal());
}

#[test]
fn is_float_literal_both() {
  assert!(FloatTok.is_float_literal());
  assert!(HexFloatTok.is_float_literal());
}

#[test]
fn is_numeric_literal_covers_all() {
  assert!(DecTok.is_numeric_literal());
  assert!(FloatTok.is_numeric_literal());
}

#[test]
fn is_string_literal_all_variants() {
  assert!(InlineStrTok.is_string_literal());
  assert!(MultilineStrTok.is_string_literal());
  assert!(RawStrTok.is_string_literal());
}

#[test]
fn is_boolean_literal() {
  assert!(TrueTok.is_boolean_literal());
  assert!(FalseTok.is_boolean_literal());
}

#[test]
fn is_literal_covers_all() {
  assert!(DecTok.is_literal());
  assert!(FloatTok.is_literal());
  assert!(InlineStrTok.is_literal());
  assert!(CharTok.is_literal());
  assert!(ByteTok.is_literal());
  assert!(ByteStrTok.is_literal());
  assert!(TrueTok.is_literal());
  assert!(NullTok.is_literal());
}

// ── Reference delegation ────────────────────────────────────────────────────

#[test]
fn ref_delegates_is_decimal() {
  let tok = DecTok;
  let r: &DecTok = &tok;
  assert!(LitToken::is_decimal_literal(r));
  assert!(LitToken::is_integer_literal(r));
  assert!(LitToken::is_literal(r));
}

#[test]
fn ref_delegates_is_float() {
  let tok = FloatTok;
  let r: &FloatTok = &tok;
  assert!(LitToken::is_float_decimal_literal(r));
  assert!(LitToken::is_float_literal(r));
}

#[test]
fn ref_delegates_is_string() {
  let tok = InlineStrTok;
  let r: &InlineStrTok = &tok;
  assert!(LitToken::is_inline_string_literal(r));
  assert!(LitToken::is_string_literal(r));
}

#[test]
fn ref_delegates_is_char() {
  let tok = CharTok;
  assert!(LitToken::is_char_literal(&tok));
}

#[test]
fn ref_delegates_is_byte() {
  let tok = ByteTok;
  assert!(LitToken::is_byte_literal(&tok));
}

#[test]
fn ref_delegates_is_byte_string() {
  let tok = ByteStrTok;
  assert!(LitToken::is_byte_string_literal(&tok));
}

#[test]
fn ref_delegates_is_true() {
  let tok = TrueTok;
  assert!(LitToken::is_true_literal(&tok));
}

#[test]
fn ref_delegates_is_false() {
  let tok = FalseTok;
  assert!(LitToken::is_false_literal(&tok));
}

#[test]
fn ref_delegates_is_null() {
  let tok = NullTok;
  assert!(LitToken::is_null_literal(&tok));
}

// ── Default false tests ─────────────────────────────────────────────────────

#[test]
fn all_defaults_false() {
  // DecTok only returns true for is_decimal_literal
  let tok = DecTok;
  assert!(!tok.is_hex_literal());
  assert!(!tok.is_octal_literal());
  assert!(!tok.is_binary_literal());
  assert!(!tok.is_float_decimal_literal());
  assert!(!tok.is_float_hex_literal());
  assert!(!tok.is_float_literal());
  assert!(!tok.is_inline_string_literal());
  assert!(!tok.is_multiline_string_literal());
  assert!(!tok.is_raw_string_literal());
  assert!(!tok.is_string_literal());
  assert!(!tok.is_char_literal());
  assert!(!tok.is_byte_literal());
  assert!(!tok.is_byte_string_literal());
  assert!(!tok.is_true_literal());
  assert!(!tok.is_false_literal());
  assert!(!tok.is_boolean_literal());
  assert!(!tok.is_null_literal());
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --all-features -p tokit --test lit_token_coverage`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tokit/tests/lit_token_coverage.rs
git commit -m "test: add LitToken reference delegation and composite default coverage"
```

---

### Task 11: Additional state machine coverage (sep/parse and sep_while/parse)

**Files:**
- Modify: `tokit/tests/state_machine.rs`

The existing `state_machine.rs` already covers many branches with the recovering emitter pattern. This task adds any remaining uncovered paths, particularly:
- `require_surrounded` policy combinations
- `allow_surrounded` policy combinations
- Additional edge cases where the existing tests don't trigger specific handler branches

**Note:** The existing `state_machine.rs` already has 1520 lines of tests covering most state machine branches. Read the current tarpaulin output for `sep/parse/mod.rs` and `sep_while/parse/mod.rs` to identify which specific lines still need coverage, then add targeted tests following the existing pattern.

- [ ] **Step 1: Run tarpaulin to identify specific uncovered lines**

Run: `cargo tarpaulin --all-features --workspace 2>&1 | grep -A2 'sep/parse/mod.rs\|sep_while/parse/mod.rs'`

- [ ] **Step 2: Add targeted tests for remaining uncovered branches**

Follow the existing pattern in `state_machine.rs`:
- Use `recovering_ctx()` to exercise error-recovery paths
- Use `fatal_ctx()` to verify error paths do error
- For sep_while tests, always end input with `+` sentinel

Add tests for `require_surrounded` and `allow_surrounded` policies:

```rust
// ── require_surrounded with recovering ────────────────────────────────────

fn parse_sep_require_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_surrounded()
    .collect()
    .parse_input(inp)
}

#[test]
fn sep_require_surrounded_ok() {
  let r: Result<Vec<i64>, _> = Parser::with_context(recovering_ctx())
    .apply(parse_sep_require_surrounded)
    .parse_str(",1,2,");
  assert!(r.is_ok());
}

#[test]
fn sep_require_surrounded_missing_both() {
  let r: Result<Vec<i64>, _> = Parser::with_context(recovering_ctx())
    .apply(parse_sep_require_surrounded)
    .parse_str("1,2");
  assert!(r.is_ok());
}

// ── allow_surrounded with recovering ──────────────────────────────────────

fn parse_sep_allow_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_surrounded()
    .collect()
    .parse_input(inp)
}

#[test]
fn sep_allow_surrounded_ok() {
  let r: Result<Vec<i64>, _> = Parser::with_context(recovering_ctx())
    .apply(parse_sep_allow_surrounded)
    .parse_str(",1,2,");
  assert!(r.is_ok());
}

#[test]
fn sep_allow_surrounded_no_separators() {
  let r: Result<Vec<i64>, _> = Parser::with_context(recovering_ctx())
    .apply(parse_sep_allow_surrounded)
    .parse_str("1,2");
  assert!(r.is_ok());
}
```

Add equivalent `sep_while` variants:

```rust
fn parse_sw_require_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_surrounded()
    .collect()
    .parse_input(inp)
}

#[test]
fn sw_require_surrounded_ok() {
  let r: Result<Vec<i64>, _> = Parser::with_context(recovering_ctx())
    .apply(parse_sw_require_surrounded)
    .parse_str(",1,2,+");
  assert!(r.is_ok());
}

#[test]
fn sw_require_surrounded_missing_both() {
  let r: Result<Vec<i64>, _> = Parser::with_context(recovering_ctx())
    .apply(parse_sw_require_surrounded)
    .parse_str("1,2+");
  assert!(r.is_ok());
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --all-features -p tokit --test state_machine`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add tokit/tests/state_machine.rs
git commit -m "test: add require_surrounded and allow_surrounded state machine coverage"
```

---

### Task 12: Handler trait impl coverage

**Files:**
- Create: `tokit/tests/handler_coverage.rs`

Tests for `parser/many/handler/mod.rs` — `SeparatorHandler` and `DelimiterHandler` impls on `()`, `PhantomData`, and feature-gated containers. These are exercised indirectly by the state machine tests, but direct tests ensure the trait impls themselves are covered.

**Note:** Most handler coverage comes from the state machine tests (Task 11) and existing `tests/handler.rs`. Read the current tarpaulin output for `handler/mod.rs` to identify which specific impls still need coverage. The existing `tests/handler.rs` (2229 lines) likely covers most paths. Focus only on lines the tarpaulin report shows as uncovered after Tasks 1-11 are complete. This task may be unnecessary if the state machine tests already cover the handler impls.

- [ ] **Step 1: Check if handler coverage is sufficient after previous tasks**

Run: `cargo tarpaulin --all-features --workspace 2>&1 | grep handler/mod.rs`

If the uncovered count has dropped significantly, skip this task. Otherwise, read the tarpaulin output to identify specific uncovered impls and write targeted tests.

- [ ] **Step 2: Write handler tests if needed**

Follow patterns from `tests/handler.rs`. Focus on:
- `()` impl (no-op separator/delimiter handler)
- `PhantomData` impl
- Feature-gated container impls (`#[cfg(feature = "smallvec_1")]`, etc.)

- [ ] **Step 3: Run tests**

Run: `cargo test --all-features -p tokit --test handler_coverage`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add tokit/tests/handler_coverage.rs
git commit -m "test: add handler trait impl coverage"
```

---

### Task 13: Run coverage and fill remaining gaps

**Files:**
- Possibly modify: any test file from previous tasks
- Possibly create: additional test files if needed

- [ ] **Step 1: Run tarpaulin to measure current coverage**

Run: `cd /Users/user/Develop/personal/tokit && cargo tarpaulin --all-features --workspace 2>&1 | tail -20`
Expected: Coverage percentage (target: >= 90%)

- [ ] **Step 2: Identify remaining gaps**

If coverage is below 90%, examine the tarpaulin output to find files with the most remaining uncovered lines. Prioritize:
1. Files from the Fallback Candidates table in the spec
2. Any files from the original priority list that still have significant gaps

- [ ] **Step 3: Write additional targeted tests for remaining gaps**

Based on the coverage report, write focused tests to close the gap. Likely candidates:
- `parser/many/sep_while/delim/mod.rs` (~15 uncovered lines)
- `input/input_ref/pratt.rs` (~15 uncovered lines)
- Additional `handler/mod.rs` paths not exercised by state_machine.rs

- [ ] **Step 4: Run coverage again to verify >= 90%**

Run: `cargo tarpaulin --all-features --workspace 2>&1 | tail -20`
Expected: >= 90% coverage

- [ ] **Step 5: Run full test suite**

Run: `cargo test --all-features -p tokit`
Expected: All tests pass

- [ ] **Step 6: Commit remaining test additions**

Stage specific files that were added/modified:

```bash
git add tokit/tests/*.rs tokit/src/**/*.rs
git commit -m "test: fill remaining coverage gaps to reach 90%+"
```
