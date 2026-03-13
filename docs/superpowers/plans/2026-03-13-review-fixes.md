# Review Fixes Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all actionable issues identified in the code review (REVIEW.md), prioritized by severity.

**Architecture:** Small, targeted fixes across cache, parser, documentation, and API modules. Each task is independent and can be committed separately.

**Tech Stack:** Rust, cargo test

---

## Chunk 1: Bug Fixes and Consistency

### Task 1: Fix Option cache rewind inconsistency

The `Option` cache treats `cursor == span.end` as "keep" while `GenericArrayDeque` treats `cursor >= span.end` as "clear". The Option behavior is wrong — when the cursor is at `span.end`, the token has been fully consumed and should be cleared.

**Files:**
- Modify: `tokit/src/cache/option.rs:59-61`

- [ ] **Step 1: Write a failing integration test exposing the bug**

`Checkpoint::new` is `pub(super)` (not accessible outside the `input` module), so this test must be written as an integration test that exercises the rewind behavior through the public API. Create an integration test in `tokit/tests/cache_rewind.rs` that:

1. Sets up a parser that saves a checkpoint, consumes a token (moving cursor to `span.end`), then restores the checkpoint
2. Verifies the cache is properly cleared after rewind

Alternatively, since the fix is straightforward and the existing test suite exercises rewind paths extensively, you may skip adding a dedicated test and verify correctness via the existing full test suite.

- [ ] **Step 2: Verify current tests pass before the fix**

Run: `cargo test --all-features`
Expected: All tests PASS (establishing baseline).

- [ ] **Step 3: Fix the rewind method**

In `tokit/src/cache/option.rs`, remove the early return when `off == span.end_ref()` (lines 59-61). This makes the code fall through to `*self = None` on line 64, matching GenericArrayDeque's behavior.

Change lines 59-61 from:
```rust
      if off == span.end_ref() {
        return;
      }
```
To: (delete these 3 lines entirely)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --all-features -p tokit cache::option`
Expected: All option cache tests PASS.

- [ ] **Step 5: Run full test suite**

Run: `cargo test --all-features`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tokit/src/cache/option.rs
git commit -m "fix(cache): clear Option cache when cursor is at token end

The Option cache rewind was keeping the cached token when cursor ==
span.end, but this position means the token was fully consumed.
GenericArrayDeque correctly clears in this case. Align the behavior."
```

---

### Task 2: Add missing inline(always) attributes

Two files are missing `#[cfg_attr(not(tarpaulin), inline(always))]` on their 3rd impl block, unlike every other file in the same directory structure.

**Files:**
- Modify: `tokit/src/parser/many/sep/parse/allow_trailing/unbounded.rs:81`
- Modify: `tokit/src/parser/many/sep_while/parse/allow_trailing/unbounded.rs:99`

- [ ] **Step 1: Add attribute to sep/parse/allow_trailing/unbounded.rs**

In `tokit/src/parser/many/sep/parse/allow_trailing/unbounded.rs`, add the attribute before `fn parse_input` on line 81:

```rust
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
```

- [ ] **Step 2: Add attribute to sep_while/parse/allow_trailing/unbounded.rs**

In `tokit/src/parser/many/sep_while/parse/allow_trailing/unbounded.rs`, add the attribute before `fn parse_input` on line 99:

```rust
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --all-features`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add tokit/src/parser/many/sep/parse/allow_trailing/unbounded.rs \
        tokit/src/parser/many/sep_while/parse/allow_trailing/unbounded.rs
git commit -m "fix: add missing inline(always) attributes on allow_trailing unbounded 3rd impl"
```

---

### Task 3: Normalize HANDLER vs UNBOUNDED constant naming

`sep/delim/unbounded.rs` and `sep_while/delim/unbounded.rs` name the constant `UNBOUNDED`, while `sep/parse/unbounded.rs` and `sep_while/parse/unbounded.rs` name it `HANDLER`. Normalize to `HANDLER` since that is the naming used by all other files (allow_trailing, require_leading, etc. all use `HANDLER`).

**Files:**
- Modify: `tokit/src/parser/many/sep/delim/unbounded.rs:139`
- Modify: `tokit/src/parser/many/sep_while/delim/unbounded.rs:158`

- [ ] **Step 1: Fix sep/delim/unbounded.rs**

In `tokit/src/parser/many/sep/delim/unbounded.rs`, change line 139:

From: `const UNBOUNDED: &Unbounded = &Unbounded;`
To: `const HANDLER: &Unbounded = &Unbounded;`

Also update the usage on line 146:
From: `DelimitedBy::<_, Delim>::new(Separated::new::<Sep>(&mut **f)).parse_separated(inp, container, UNBOUNDED, UNBOUNDED, UNBOUNDED)`
To: `DelimitedBy::<_, Delim>::new(Separated::new::<Sep>(&mut **f)).parse_separated(inp, container, HANDLER, HANDLER, HANDLER)`

- [ ] **Step 2: Fix sep_while/delim/unbounded.rs**

In `tokit/src/parser/many/sep_while/delim/unbounded.rs`, make the same change:

From: `const UNBOUNDED: &Unbounded = &Unbounded;`
To: `const HANDLER: &Unbounded = &Unbounded;`

And update the usage similarly.

- [ ] **Step 3: Verify compilation**

Run: `cargo check --all-features`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add tokit/src/parser/many/sep/delim/unbounded.rs \
        tokit/src/parser/many/sep_while/delim/unbounded.rs
git commit -m "style: normalize UNBOUNDED constant name to HANDLER for consistency"
```

---

## Chunk 2: Documentation Improvements

### Task 4: Add module-level doc to json.rs example

The json example is the most complex example but has no module-level documentation.

**Files:**
- Modify: `tokit/examples/json.rs:1`

- [ ] **Step 1: Add module doc comment**

Add at the very top of `tokit/examples/json.rs`, before the `use` statements:

```rust
//! JSON parser example demonstrating advanced tokit features.
//!
//! This example parses JSON (objects, arrays, strings, numbers, booleans, null)
//! using `peek_then_choice` for deterministic dispatch, `separated_by` for
//! comma-separated lists, and `DelimitedBy` for bracket/brace-delimited containers.
//!
//! Run: `cargo run --example json --features logos`
```

- [ ] **Step 2: Verify the example still compiles and runs**

Run: `cargo run --example json --features logos`
Expected: Parses `sample.json` successfully.

- [ ] **Step 3: Commit**

```bash
git add tokit/examples/json.rs
git commit -m "docs: add module-level documentation to json example"
```

---

### Task 5: Document emitter non-rollback in recover.rs

The `Recover` combinator restores input position on failure but does NOT roll back errors already emitted to the `Emitter`. This is a subtle behavior that should be documented.

**Files:**
- Modify: `tokit/src/parser/recover.rs` (doc comments on `Recover` struct)

- [ ] **Step 1: Find the Recover struct doc comment**

Read `tokit/src/parser/recover.rs` and find the `/// ...` doc comment block on the `Recover` struct (around line 176-288 based on exploration).

- [ ] **Step 2: Add a "Caveats" section to the doc comment**

Add a `# Caveats` section to the existing doc comment on `Recover`, after the existing documentation:

```rust
/// # Caveats
///
/// When the primary parser fails, `Recover` restores the input position via
/// checkpoint but does **not** roll back errors already emitted to the
/// [`Emitter`](crate::Emitter). If the primary parser emitted errors before
/// returning `Err`, those errors remain in the emitter even if recovery
/// succeeds. This is important when using [`Verbose`](crate::emitter::Verbose)
/// or other stateful emitters — successful recovery may leave spurious errors
/// from the failed attempt.
```

- [ ] **Step 3: Verify doc builds**

Run: `cargo doc --all-features --no-deps 2>&1 | grep -i error || echo "docs OK"`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add tokit/src/parser/recover.rs
git commit -m "docs: document emitter non-rollback caveat in Recover"
```

---

### Task 6: Add doc warning to PrattPower::prev()

The `prev()` method can underflow if users implement it for numeric types without saturating arithmetic.

**Files:**
- Modify: `tokit/src/parser/pratt/mod.rs:11-12`

- [ ] **Step 1: Expand the prev() doc comment**

In `tokit/src/parser/pratt/mod.rs`, replace the doc comment on `prev()` (line 11):

From:
```rust
  /// Returns the previous lower power level.
  fn prev(&self) -> Self;
```

To:
```rust
  /// Returns the previous lower power level.
  ///
  /// # Important
  ///
  /// This is called for right-associative operators to compute the minimum
  /// precedence for the recursive parse. If your implementation uses numeric
  /// types, ensure `prev()` uses saturating subtraction to avoid
  /// underflow/panic when called on the minimum representable value.
  fn prev(&self) -> Self;
```

- [ ] **Step 2: Verify doc builds**

Run: `cargo doc --all-features --no-deps 2>&1 | grep -i error || echo "docs OK"`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add tokit/src/parser/pratt/mod.rs
git commit -m "docs: warn about underflow risk in PrattPower::prev()"
```

---

## Chunk 3: API Ergonomics

### Task 7: Add prelude module

Create a `prelude` module that re-exports the most commonly needed items, reducing the 15-27 line import blocks in examples and user code to a single `use tokit::prelude::*`.

**Files:**
- Create: `tokit/src/prelude.rs`
- Modify: `tokit/src/lib.rs` (add `pub mod prelude;`)

- [ ] **Step 1: Create the prelude module**

Create `tokit/src/prelude.rs`:

```rust
//! Convenience re-exports for common tokit usage.
//!
//! ```
//! use tokit::prelude::*;
//! ```
//!
//! This module re-exports the most commonly needed traits, types, and macros
//! for writing parsers with tokit.

// Core traits
pub use crate::{
  Emitter,
  Lexer,
  Parse,
  ParseContext,
  ParseInput,
  Token,
  TryParseInput,
};

// Core types
pub use crate::{
  FatalContext,
  InputRef,
  Parser,
  ParserContext,
  SimpleSpan,
  Span,
  Spanned,
};

// Error
pub use crate::error::UnexpectedEot;
pub use crate::error::token::UnexpectedTokenOf;
```

- [ ] **Step 2: Add the module to lib.rs**

In `tokit/src/lib.rs`, add after the existing public module declarations:

```rust
/// Convenience re-exports for common usage.
pub mod prelude;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --all-features`
Expected: Compiles successfully.

- [ ] **Step 4: Verify the prelude works by updating one example**

In one example (e.g., `tokit/examples/s_expression.rs`), try replacing the manual imports with `use tokit::prelude::*;` and verify it still compiles. Do NOT commit this change — revert after testing. This is just a smoke test.

- [ ] **Step 5: Run full test suite**

Run: `cargo test --all-features`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tokit/src/prelude.rs tokit/src/lib.rs
git commit -m "feat: add prelude module with common re-exports"
```
