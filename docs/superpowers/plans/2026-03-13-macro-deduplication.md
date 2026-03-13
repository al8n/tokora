# Macro-Based Deduplication of `parser/many/` Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace ~20,000 lines of near-identical impl blocks across 144 leaf files in `parser/many/` with macro invocations, preserving all behavior.

**Architecture:** Four `macro_rules!` macros (one per parser_type × delim_mode combination) that generate 4 impl blocks per leaf file. Each leaf file shrinks from ~140-160 lines to ~25-30 lines of macro invocation. The macros live in `tokit/src/parser/many/macros.rs`.

**Note on spec divergence:** The spec describes a two-layer macro with an inner `impl_collect_blocks!` and thin outer macros taking abstract `policy`/`cardinality` identifiers. This isn't feasible with `macro_rules!` because Rust macros cannot compose type expressions from identifier lists (e.g., you can't turn `[AllowTrailing, AtLeast]` into `AllowTrailing<AtLeast<Separated<...>>>` in type position). Instead, the plan uses four independent macros where each leaf file passes the full composed type expressions as token trees. This still achieves ~5x line reduction per file while remaining correct and debuggable.

**Tech Stack:** Rust, `macro_rules!`, `cargo test --all-features`

**Spec:** `docs/superpowers/specs/2026-03-13-macro-deduplication-design.md`

---

## Chunk 1: `sep/parse/` Macro and Conversion

### Task 1: Create the `impl_separated_parse!` macro and convert bare files

**Files:**
- Create: `tokit/src/parser/many/macros.rs`
- Modify: `tokit/src/parser/many/mod.rs` (add macro module)
- Modify: `tokit/src/parser/many/sep/parse/unbounded.rs`
- Modify: `tokit/src/parser/many/sep/parse/at_least.rs`
- Modify: `tokit/src/parser/many/sep/parse/at_most.rs`
- Modify: `tokit/src/parser/many/sep/parse/bounded.rs`

- [ ] **Step 1: Create `macros.rs` with the `impl_separated_parse!` macro**

Create `tokit/src/parser/many/macros.rs` with the following content:

```rust
/// Generates the 4 `ParseInput` impl blocks for `sep/parse/` leaf files.
///
/// Each leaf file in `sep/parse/` implements the same pattern with 4 impl blocks:
/// 1. `Collect<Outer, Container>` → `Container` (owned)
/// 2. `With<Collect<Outer, Container>, PhantomSpan>` → `Spanned<Container>` (owned + span)
/// 3. `Collect<&mut RefOuter, &mut Container>` → `L::Span` (borrowed, reconstructs parser)
/// 4. `Wrapper<Collect<WrapperOuter, &mut Container>>` → `L::Span` (core implementation)
///
/// Parameters:
/// - `owned_type`: The full type wrapping `Separated<F, Sep, O, L, Ctx, Lang>` for blocks 1,2
/// - `ref_type`: The type for block 3's `Collect<&'c mut $ref_type, ...>` (may have owned or &'c mut F)
/// - `wrapper_type`: The type for block 4's `Wrapper<Collect<$wrapper_type, ...>>` (always has &'c mut F)
/// - `map_self`: The map_parser chain for block 1
/// - `map_primary`: The map_parser chain for block 2
/// - `emitters`: Extra emitter trait bounds (e.g., `+ TooFewEmitter<'inp, L, Lang>`)
/// - `block3_inline`, `block4_inline`: Whether to add `#[cfg_attr(not(tarpaulin), inline(always))]`
/// - `block3_body`, `block4_body`: The function body for blocks 3 and 4
macro_rules! impl_separated_parse {
  (@inline true $($item:tt)*) => { #[cfg_attr(not(tarpaulin), inline(always))] $($item)* };
  (@inline false $($item:tt)*) => { $($item)* };
  (
    owned_type = [$($owned:tt)*],
    ref_type = [$($reft:tt)*],
    wrapper_type = [$($wt:tt)*],
    map_self = {$($map_self:tt)*},
    map_primary = {$($map_primary:tt)*},
    emitters = {$($emitters:tt)*},
    block3_inline = $b3i:ident,
    block3_body = {$($b3:tt)*},
    block4_inline = $b4i:ident,
    block4_body = {$($b4:tt)*} $(,)?
  ) => {
    // Block 1: owned -> Container
    impl<'inp, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, Container, Ctx, Lang>
      for Collect<$($owned)*, Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper($($map_self)*)
          .parse_input(inp)
          .map(|_| mem::take(&mut self.container))
      }
    }

    // Block 2: owned -> Spanned<Container>
    impl<'inp, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
      for With<Collect<$($owned)*, Container, Ctx, Lang>, PhantomSpan>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper($($map_primary)*)
          .parse_input(inp)
          .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
      }
    }

    // Block 3: &mut ref -> L::Span
    impl<'inp, 'c, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Collect<&'c mut $($reft)*, &'c mut Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L>,
    {
      impl_separated_parse!(@inline $b3i
        fn parse_input(
          &mut self,
          input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L: Lexer<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
        {
          $($b3)*
        }
      );
    }

    struct Wrapper<T>(T);

    // Block 4: Wrapper -> L::Span
    impl<'inp, 'c, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Wrapper<Collect<$($wt)*, &'c mut Container, Ctx, Lang>>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L>,
    {
      impl_separated_parse!(@inline $b4i
        fn parse_input(
          &mut self,
          inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
          $($b4)*
        }
      );
    }
  };
}
```

- [ ] **Step 2: Add `macros` module to `mod.rs`**

In `tokit/src/parser/many/mod.rs`, add `#[macro_use] mod macros;` **before** the existing `mod sep;` line (macros must be declared before use):

```rust
#[macro_use]
mod macros;
```

- [ ] **Step 3: Convert `sep/parse/unbounded.rs`**

Replace the entire contents of `tokit/src/parser/many/sep/parse/unbounded.rs` with:

```rust
use crate::{
  container::Container as ContainerT,
  emitter::{UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter},
};

use super::*;

impl_separated_parse! {
  owned_type = [Separated<F, Sep, O, L, Ctx, Lang>],
  ref_type = [Separated<&'c mut F, Sep, O, L, Ctx, Lang>],
  wrapper_type = [Separated<&'c mut F, Sep, O, L, Ctx, Lang>],
  map_self = { self.as_mut().map_parser(|p| p.as_mut()) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.as_mut()) },
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = false,
  block3_body = {
    let (parser, container) = self.parts_mut();
    let f = parser.fn_mut();
    let parser = Collect::new(Separated::new::<Sep>(&mut **f), &mut **container);
    Wrapper(parser).parse_input(input)
  },
  block4_inline = false,
  block4_body = {
    const HANDLER: &Unbounded = &Unbounded;
    let (parser, container) = self.0.parts_mut();
    parser.parse(inp, container, HANDLER, HANDLER, HANDLER)
  },
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check --all-features -p tokit`
Expected: Compiles successfully.

- [ ] **Step 5: Convert `sep/parse/at_least.rs`**

Replace `tokit/src/parser/many/sep/parse/at_least.rs` with:

```rust
use crate::emitter::{
  TooFewEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  owned_type = [AtLeast<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  map_self = { self.as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block3_body = {
    let (parser, container) = self.parts_mut();
    let minimum = parser.minimum();
    let f = parser.parser_mut().fn_mut();
    let parser = AtLeast::new(Separated::new::<Sep>(&mut **f), minimum.get());
    Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
  },
  block4_inline = true,
  block4_body = {
    let (parser, container) = self.0.parts_mut();
    let minimum = parser.minimum();
    parser.parser_mut().parse(inp, container, &minimum, &minimum, &minimum)
  },
}
```

- [ ] **Step 6: Convert `sep/parse/at_most.rs`**

Replace `tokit/src/parser/many/sep/parse/at_most.rs` with:

```rust
use crate::emitter::{
  TooManyEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  owned_type = [AtMost<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  map_self = { self.as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block3_body = {
    let (parser, container) = self.parts_mut();
    let maximum = parser.maximum();
    let f = parser.parser_mut().fn_mut();
    let parser = AtMost::new(Separated::new::<Sep>(&mut **f), maximum.get());
    Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
  },
  block4_inline = true,
  block4_body = {
    let (parser, container) = self.0.parts_mut();
    let limitation = parser.maximum();
    parser.parser_mut().parse(inp, container, &limitation, &limitation, &limitation)
  },
}
```

- [ ] **Step 7: Convert `sep/parse/bounded.rs`**

Replace `tokit/src/parser/many/sep/parse/bounded.rs` with:

```rust
use crate::emitter::{
  TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
  UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  owned_type = [Bounded<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  map_self = { self.as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block3_body = {
    let (parser, container) = self.parts_mut();
    let maximum = parser.maximum();
    let minimum = parser.minimum();
    let f = parser.parser_mut().fn_mut();
    let parser = Bounded::new(Separated::new::<Sep>(&mut **f), maximum.get(), minimum.get());
    Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
  },
  block4_inline = true,
  block4_body = {
    let (parser, container) = self.0.parts_mut();
    let limitation = parser.to_with();
    parser.parser_mut().parse(inp, container, &limitation, &limitation, &limitation)
  },
}
```

- [ ] **Step 8: Verify compilation and tests**

Run: `cargo check --all-features -p tokit && cargo test --all-features -p tokit`
Expected: All compile and pass.

- [ ] **Step 9: Commit**

```bash
git add tokit/src/parser/many/macros.rs tokit/src/parser/many/mod.rs \
  tokit/src/parser/many/sep/parse/unbounded.rs \
  tokit/src/parser/many/sep/parse/at_least.rs \
  tokit/src/parser/many/sep/parse/at_most.rs \
  tokit/src/parser/many/sep/parse/bounded.rs
git commit -m "refactor: add impl_separated_parse macro and convert bare sep/parse files"
```

---

### Task 2: Convert all `sep/parse/` policy subdirectory files

This task converts the remaining 32 files in `sep/parse/` policy subdirectories. All use `impl_separated_parse!` with appropriate policy wrapping.

**Files:** All `unbounded.rs`, `at_least.rs`, `at_most.rs`, `bounded.rs` in each of the 8 policy subdirectories under `tokit/src/parser/many/sep/parse/`.

**Key patterns by policy:**

Each policy has specific emitter bounds and a specific wrapper type. Reference this table when converting files:

| Policy | Type wrapper | Emitter bounds (beyond `SeparatedEmitter + FullContainerEmitter`) |
|--------|-------------|------------------------------------------------------------------|
| `allow_trailing` | `AllowTrailing<...>` | `+ UnexpectedLeadingSeparatorEmitter` |
| `require_leading` | `RequireLeading<...>` | `+ MissingLeadingSeparatorEmitter + UnexpectedTrailingSeparatorEmitter` |
| `require_trailing` | `RequireTrailing<...>` | `+ MissingTrailingSeparatorEmitter + UnexpectedLeadingSeparatorEmitter` |
| `allow_leading` | `AllowLeading<...>` | `+ UnexpectedTrailingSeparatorEmitter` |
| `require_surrounded` | `RequireLeading<RequireTrailing<...>>` | `+ MissingLeadingSeparatorEmitter + MissingTrailingSeparatorEmitter` |
| `allow_surrounded` | `AllowLeading<AllowTrailing<...>>` | (none) |
| `require_leading_allow_trailing` | `RequireLeading<AllowTrailing<...>>` | `+ MissingLeadingSeparatorEmitter` |
| `allow_leading_require_trailing` | `AllowLeading<RequireTrailing<...>>` | `+ MissingTrailingSeparatorEmitter` |

Cardinality adds: `at_least` → `+ TooFewEmitter`, `at_most` → `+ TooManyEmitter`, `bounded` → both.

**Key structural rules for policy files:**

1. **Blocks 1,2 `map_chain` depth**: number of wrapper layers around `Separated`. Single-policy unbounded = 1 `map_parser_mut`. Single-policy + cardinality = 2 `map_parser_mut`. Double-policy (surrounded, etc.) unbounded = 2. Double-policy + cardinality = 3.

2. **Block 3 `ref_type`**: For policy-only unbounded files, F is plain `F` (NOT `&'c mut F`). For all files with cardinality wrappers, F is `&'c mut F`.

3. **Block 3 body**: Navigate through wrappers with `.parser_mut()`, extract cardinality values, reconstruct the full wrapper stack with `Policy::new(Cardinality::new(Separated::new::<Sep>(...)))`.

4. **Block 4 `wrapper_type`**: Always has `&'c mut F` inside `Separated`.

5. **Block 4 body for unbounded**: `const HANDLER: &Policy<Unbounded> = &Policy::new(Unbounded);` then `parser.parser_mut().parse(...)` (or `parser_mut().parser_mut()` for double-wrapped policies).

6. **Block 4 body for cardinality**: `let limitation = Policy::new(parser.parser.accessor());` then `parser.parser_mut().parser_mut().parse(...)`. For double-wrapped policies, add one more `.parser_mut()` and use `parser.parser.parser.accessor()`.

- [ ] **Step 1: Convert `allow_trailing/unbounded.rs`** (representative single-policy unbounded)

Replace `tokit/src/parser/many/sep/parse/allow_trailing/unbounded.rs` with:

```rust
use crate::emitter::UnexpectedLeadingSeparatorEmitter;

use super::*;

impl_separated_parse! {
  owned_type = [AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  map_self = { self.as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  emitters = { + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block3_body = {
    let (parser, container) = self.parts_mut();
    let f = parser.parser_mut().fn_mut();
    let parser = AllowTrailing::new(Separated::new::<Sep>(&mut *f));
    Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
  },
  block4_inline = false,
  block4_body = {
    const HANDLER: &AllowTrailing<Unbounded> = &AllowTrailing::new(Unbounded);
    let (parser, container) = self.0.parts_mut();
    parser.parser_mut().parse(inp, container, HANDLER, HANDLER, HANDLER)
  },
}
```

- [ ] **Step 2: Convert `allow_trailing/at_least.rs`** (representative single-policy + cardinality)

Replace `tokit/src/parser/many/sep/parse/allow_trailing/at_least.rs` with:

```rust
use crate::emitter::{TooFewEmitter, UnexpectedLeadingSeparatorEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [AllowTrailing<AtLeast<Separated<F, Sep, O, L, Ctx, Lang>>>],
  ref_type = [AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  wrapper_type = [AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  map_self = { self.as_mut().map_parser(|p| p.map_parser_mut(|p| p.map_parser_mut(|p| p.as_mut()))) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.map_parser_mut(|p| p.map_parser_mut(|p| p.as_mut()))) },
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block3_body = {
    let (parser, container) = self.parts_mut();
    let inner = parser.parser_mut();
    let minimum = inner.minimum();
    let f = inner.parser_mut().fn_mut();
    let parser = AllowTrailing::new(AtLeast::new(Separated::new::<Sep>(&mut **f), minimum.get()));
    Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
  },
  block4_inline = true,
  block4_body = {
    let (parser, container) = self.0.parts_mut();
    let limitation = AllowTrailing::new(parser.parser.minimum());
    parser.parser_mut().parser_mut().parse(inp, container, &limitation, &limitation, &limitation)
  },
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --all-features -p tokit`
Expected: Compiles.

- [ ] **Step 4: Convert remaining `allow_trailing/` files** (`at_most.rs`, `bounded.rs`)

Follow the same pattern as steps 1-2, substituting:
- `at_most.rs`: `AtMost` instead of `AtLeast`, `TooManyEmitter` instead of `TooFewEmitter`, `.maximum()` instead of `.minimum()`, `AtMost::new(...)` with `maximum.get()`
- `bounded.rs`: `Bounded`, both `TooFewEmitter` + `TooManyEmitter`, block 3 extracts both `.maximum()` and `.minimum()`, block 4 uses `parser.parser.to_with()`

Read each original file before converting to match exactly.

- [ ] **Step 5: Convert all 7 remaining policy subdirectories**

For each policy subdirectory (`require_leading/`, `require_trailing/`, `allow_leading/`, `require_surrounded/`, `allow_surrounded/`, `require_leading_allow_trailing/`, `allow_leading_require_trailing/`), convert all 4 cardinality files (`unbounded.rs`, `at_least.rs`, `at_most.rs`, `bounded.rs`).

Read each original file first to get the exact:
- Policy wrapper type(s) and nesting
- Emitter bounds
- Block 3 accessor chain and reconstruction
- Block 4 handler constant or limitation expression
- Block 3/4 inline attribute presence

Critical details for double-wrapped policies (e.g., `allow_surrounded` = `AllowLeading<AllowTrailing<...>>`):
- Block 1,2 unbounded: 2 `map_parser_mut` levels
- Block 1,2 with cardinality: 3 `map_parser_mut` levels
- Block 3 unbounded `ref_type`: F is plain `F` (like single-policy unbounded)
- Block 4: `const HANDLER: &AllowLeading<AllowTrailing<Unbounded>> = &AllowLeading::new(AllowTrailing::new(Unbounded));`
- Block 4 with cardinality limitation: `let limitation = AllowLeading::new(AllowTrailing::new(parser.parser.parser.minimum()));`

- [ ] **Step 6: Verify full compilation and test suite**

Run: `cargo test --all-features -p tokit`
Expected: All tests pass.

- [ ] **Step 7: Commit**

```bash
git add tokit/src/parser/many/sep/parse/
git commit -m "refactor: convert all sep/parse/ leaf files to use impl_separated_parse macro"
```

---

## Chunk 2: `sep/delim/` Macro and Conversion

### Task 3: Add `impl_separated_delim!` macro and convert `sep/delim/` files

**Files:**
- Modify: `tokit/src/parser/many/macros.rs` (add new macro)
- Modify: All leaf files in `tokit/src/parser/many/sep/delim/`

The `sep/delim/` variant differs from `sep/parse/` in:
1. Extra generic: `Delim`
2. Extra bound: `Delim: Delimiter<'inp, L, Lang>`
3. Extra error bound: `<Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>`
4. Extra container trait: `+ DelimiterHandler<'inp, L>`
5. Block 3 uses `delim.parser` field access (not `.parts_mut()`)
6. Block 4 reconstructs parser with `DelimitedBy::<_, Delim>::new(...)` and calls `.parse_separated()`

- [ ] **Step 1: Add `impl_separated_delim!` macro to `macros.rs`**

Add to the end of `tokit/src/parser/many/macros.rs`:

```rust
/// Like `impl_separated_parse!` but for `sep/delim/` files.
/// Adds `Delim` generic, `Delimiter` bound, `DelimiterHandler` container trait,
/// and `From<UnexpectedEot>` error bound.
macro_rules! impl_separated_delim {
  (@inline true $($item:tt)*) => { #[cfg_attr(not(tarpaulin), inline(always))] $($item)* };
  (@inline false $($item:tt)*) => { $($item)* };
  (
    owned_type = [$($owned:tt)*],
    ref_type = [$($reft:tt)*],
    wrapper_type = [$($wt:tt)*],
    map_self = {$($map_self:tt)*},
    map_primary = {$($map_primary:tt)*},
    emitters = {$($emitters:tt)*},
    block3_inline = $b3i:ident,
    block3_body = {$($b3:tt)*},
    block4_inline = $b4i:ident,
    block4_body = {$($b4:tt)*} $(,)?
  ) => {
    // Block 1: owned -> Container
    impl<'inp, L, F, Sep, O, Delim, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, Container, Ctx, Lang>
      for Collect<$($owned)*, Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx: ParseContext<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
      Delim: Delimiter<'inp, L, Lang>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper($($map_self)*)
          .parse_input(inp)
          .map(|_| mem::take(&mut self.container))
      }
    }

    // Block 2: owned -> Spanned<Container>
    impl<'inp, L, F, Sep, O, Delim, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
      for With<Collect<$($owned)*, Container, Ctx, Lang>, PhantomSpan>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx: ParseContext<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
      Delim: Delimiter<'inp, L, Lang>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper($($map_primary)*)
          .parse_input(inp)
          .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
      }
    }

    // Block 3: &mut ref -> L::Span
    impl<'inp, 'c, L, F, Sep, O, Delim, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Collect<&'c mut $($reft)*, &'c mut Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx: ParseContext<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
      Delim: Delimiter<'inp, L, Lang>,
    {
      impl_separated_delim!(@inline $b3i
        fn parse_input(
          &mut self,
          input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L: Lexer<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
        {
          $($b3)*
        }
      );
    }

    struct Wrapper<T>(T);

    // Block 4: Wrapper -> L::Span
    impl<'inp, 'c, L, F, Sep, O, Delim, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Wrapper<Collect<$($wt)*, &'c mut Container, Ctx, Lang>>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx: ParseContext<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
      Delim: Delimiter<'inp, L, Lang>,
    {
      impl_separated_delim!(@inline $b4i
        fn parse_input(
          &mut self,
          inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
          $($b4)*
        }
      );
    }
  };
}
```

- [ ] **Step 2: Convert `sep/delim/unbounded.rs`** (representative bare delim)

Replace `tokit/src/parser/many/sep/delim/unbounded.rs` with:

```rust
use crate::emitter::{UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<Separated<F, Sep, O, L, Ctx, Lang>, Delim>],
  ref_type = [DelimitedBy<Separated<&'c mut F, Sep, O, L, Ctx, Lang>, Delim>],
  wrapper_type = [DelimitedBy<Separated<&'c mut F, Sep, O, L, Ctx, Lang>, Delim>],
  map_self = { self.as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block3_body = {
    let (delim, container) = self.parts_mut();
    let f = delim.parser.fn_mut();
    let parser = DelimitedBy::<_, Delim>::new(Separated::new::<Sep>(&mut **f));
    Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
  },
  block4_inline = true,
  block4_body = {
    const HANDLER: &Unbounded = &Unbounded;
    let (parser, container) = self.0.parts_mut();
    let f = parser.parser.fn_mut();
    DelimitedBy::<_, Delim>::new(Separated::new::<Sep>(&mut **f))
      .parse_separated(inp, container, HANDLER, HANDLER, HANDLER)
  },
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --all-features -p tokit`
Expected: Compiles.

- [ ] **Step 4: Convert remaining `sep/delim/` files**

Convert all remaining leaf files in `sep/delim/`. Note:
- `sep/delim/bounded.rs` has been populated (was previously empty) — convert it like the other cardinality files.
- Block 3 uses `delim.parser` field access to reach the inner parser (not `.parts_mut()`)
- Block 4 always reconstructs `DelimitedBy::<_, Delim>::new(...)` and calls `.parse_separated()`
- For cardinality files, block 4 accesses limitation via `parser.parser.minimum()` etc., then also reconstructs `DelimitedBy::<_, Delim>::new(Separated::new::<Sep>(...))` for the call
- For policy+cardinality in delim, block 4 uses `parser.parser.parser.minimum()` (extra `.parser` for policy layer)

- **`block4_inline` rule for delim mode:** ALL `sep/delim/` files use `block4_inline = true`, including unbounded files. This differs from `sep/parse/` where unbounded files use `block4_inline = false`.

Read each original file before converting to match field access patterns exactly.

- [ ] **Step 5: Verify full test suite**

Run: `cargo test --all-features -p tokit`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add tokit/src/parser/many/macros.rs tokit/src/parser/many/sep/delim/
git commit -m "refactor: add impl_separated_delim macro and convert all sep/delim/ files"
```

---

## Chunk 3: `sep_while/` Macros and Conversion

### Task 4: Add `impl_separated_while_parse!` macro and convert `sep_while/parse/` files

**Files:**
- Modify: `tokit/src/parser/many/macros.rs` (add new macro)
- Modify: All leaf files in `tokit/src/parser/many/sep_while/parse/`

The `sep_while/parse/` variant differs from `sep/parse/` in:
1. Extra generics: `Condition, W` (plus `Condition` is `&'c mut Condition` in block 4's `SeparatedWhile` type)
2. Extra bounds: `Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>`, `W: Window`
3. `ParseInput` instead of `TryParseInput` for `F`
4. Base parser type is `SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>` (more type params)
5. Block 3 destructures `(f, condition) = parser.parts_mut()` and reconstructs with `SeparatedWhile::new::<Sep>(&mut **f, &mut *condition)`
6. Block 4's `SeparatedWhile` type has `&'c mut Condition` (not just `Condition`)

- [ ] **Step 1: Add `impl_separated_while_parse!` macro to `macros.rs`**

Add to `tokit/src/parser/many/macros.rs`. This macro is identical to `impl_separated_parse!` but with:
- `Condition, W` added to generic params (for all blocks) and `&'c mut Condition` in block 4 only
- `F: ParseInput` instead of `F: TryParseInput`
- `Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>` and `W: Window` bounds added

```rust
macro_rules! impl_separated_while_parse {
  (@inline true $($item:tt)*) => { #[cfg_attr(not(tarpaulin), inline(always))] $($item)* };
  (@inline false $($item:tt)*) => { $($item)* };
  (
    owned_type = [$($owned:tt)*],
    ref_type = [$($reft:tt)*],
    wrapper_type = [$($wt:tt)*],
    map_self = {$($map_self:tt)*},
    map_primary = {$($map_primary:tt)*},
    emitters = {$($emitters:tt)*},
    block3_inline = $b3i:ident,
    block3_body = {$($b3:tt)*},
    block4_inline = $b4i:ident,
    block4_body = {$($b4:tt)*} $(,)?
  ) => {
    // Block 1
    impl<'inp, L, F, Sep, Condition, O, Container, Ctx, Lang: ?Sized, W>
      ParseInput<'inp, L, Container, Ctx, Lang>
      for Collect<$($owned)*, Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: ParseInput<'inp, L, O, Ctx, Lang>,
      Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
      W: Window,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper($($map_self)*)
          .parse_input(inp)
          .map(|_| mem::take(&mut self.container))
      }
    }

    // Block 2
    impl<'inp, L, F, Sep, Condition, O, Container, Ctx, Lang: ?Sized, W>
      ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
      for With<Collect<$($owned)*, Container, Ctx, Lang>, PhantomSpan>
    where
      L: Lexer<'inp>,
      F: ParseInput<'inp, L, O, Ctx, Lang>,
      Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
      W: Window,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper($($map_primary)*)
          .parse_input(inp)
          .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
      }
    }

    // Block 3
    impl<'inp, 'c, L, F, Sep, Condition, O, Container, Ctx, Lang: ?Sized, W>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Collect<&'c mut $($reft)*, &'c mut Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: ParseInput<'inp, L, O, Ctx, Lang>,
      Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L>,
      W: Window,
    {
      impl_separated_while_parse!(@inline $b3i
        fn parse_input(
          &mut self,
          input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L: Lexer<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
        {
          $($b3)*
        }
      );
    }

    struct Wrapper<T>(T);

    // Block 4
    impl<'inp, 'c, L, F, Sep, Condition, O, Container, Ctx, Lang: ?Sized, W>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Wrapper<Collect<$($wt)*, &'c mut Container, Ctx, Lang>>
    where
      L: Lexer<'inp>,
      F: ParseInput<'inp, L, O, Ctx, Lang>,
      Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L>,
      W: Window,
    {
      impl_separated_while_parse!(@inline $b4i
        fn parse_input(
          &mut self,
          inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
          $($b4)*
        }
      );
    }
  };
}
```

- [ ] **Step 2: Convert `sep_while/parse/unbounded.rs`** (representative)

Replace `tokit/src/parser/many/sep_while/parse/unbounded.rs` with:

```rust
use crate::{
  container::Container as ContainerT,
  emitter::{UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter},
};

use super::*;

impl_separated_while_parse! {
  owned_type = [SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>],
  ref_type = [SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>],
  wrapper_type = [SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>],
  map_self = { self.as_mut().map_parser(|p| p.as_mut()) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.as_mut()) },
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = false,
  block3_body = {
    let (parser, container) = self.parts_mut();
    let (f, condition) = parser.parts_mut();
    let parser = Collect::new(
      SeparatedWhile::new::<Sep>(&mut **f, &mut *condition),
      &mut *container,
    );
    Wrapper(parser).parse_input(input)
  },
  block4_inline = false,
  block4_body = {
    const HANDLER: &Unbounded = &Unbounded;
    let (parser, container) = self.0.parts_mut();
    parser.parse(inp, container, HANDLER, HANDLER, HANDLER)
  },
}
```

**Critical difference from `sep/parse/`:** Block 4's `wrapper_type` has `&'c mut Condition` (not just `Condition`). This is because the Wrapper impl's `SeparatedWhile` type in block 4 uses `&'c mut Condition` while blocks 1-3 use plain `Condition`.

- [ ] **Step 3: Verify compilation**

Run: `cargo check --all-features -p tokit`
Expected: Compiles.

- [ ] **Step 4: Convert remaining `sep_while/parse/` files**

Convert all 36 leaf files following the patterns from Task 2 (policy table and structural rules), adapted for `SeparatedWhile`:
- All policies and cardinalities follow the same patterns as `sep/parse/`
- The key differences: `SeparatedWhile` type params, `ParseInput` trait, `(f, condition)` destructuring in block 3, `&'c mut Condition` in block 4 wrapper_type
- **Block 3 container deref:** `sep_while/parse/` files use `&mut *container` (single deref) in block 3, NOT `&mut **container` (double deref) as in `sep/parse/`. Match the actual file.
- **Block 4 inline rule (parse mode):** `block4_inline = false` for all unbounded files (bare and policy), `block4_inline = true` for cardinality files (`at_least`, `at_most`, `bounded`).

Read each original file before converting.

- [ ] **Step 5: Verify test suite**

Run: `cargo test --all-features -p tokit`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add tokit/src/parser/many/macros.rs tokit/src/parser/many/sep_while/parse/
git commit -m "refactor: add impl_separated_while_parse macro and convert all sep_while/parse/ files"
```

---

### Task 5: Add `impl_separated_while_delim!` macro and convert `sep_while/delim/` files

**Files:**
- Modify: `tokit/src/parser/many/macros.rs` (add new macro)
- Modify: All leaf files in `tokit/src/parser/many/sep_while/delim/`

This combines `sep_while` differences (extra `Condition, W` generics, `ParseInput`, `Decision` bound) with `delim` differences (extra `Delim` generic, `Delimiter` bound, `DelimiterHandler`, `From<UnexpectedEot>`, `parse_separated`).

- [ ] **Step 1: Add `impl_separated_while_delim!` macro to `macros.rs`**

Add to `tokit/src/parser/many/macros.rs`:

```rust
/// Like `impl_separated_while_parse!` but for `sep_while/delim/` files.
/// Combines sep_while generics (Condition, W, Decision, Window) with delim
/// additions (Delim, Delimiter, DelimiterHandler, From<UnexpectedEot>).
macro_rules! impl_separated_while_delim {
  (@inline true $($item:tt)*) => { #[cfg_attr(not(tarpaulin), inline(always))] $($item)* };
  (@inline false $($item:tt)*) => { $($item)* };
  (
    owned_type = [$($owned:tt)*],
    ref_type = [$($reft:tt)*],
    wrapper_type = [$($wt:tt)*],
    map_self = {$($map_self:tt)*},
    map_primary = {$($map_primary:tt)*},
    emitters = {$($emitters:tt)*},
    block3_inline = $b3i:ident,
    block3_body = {$($b3:tt)*},
    block4_inline = $b4i:ident,
    block4_body = {$($b4:tt)*} $(,)?
  ) => {
    // Block 1: owned -> Container
    impl<'inp, L, F, Sep, Condition, O, Delim, Container, Ctx, Lang: ?Sized, W>
      ParseInput<'inp, L, Container, Ctx, Lang>
      for Collect<$($owned)*, Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: ParseInput<'inp, L, O, Ctx, Lang>,
      Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx: ParseContext<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
      W: Window,
      Delim: Delimiter<'inp, L, Lang>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper($($map_self)*)
          .parse_input(inp)
          .map(|_| mem::take(&mut self.container))
      }
    }

    // Block 2: owned -> Spanned<Container>
    impl<'inp, L, F, Sep, Condition, O, Delim, Container, Ctx, Lang: ?Sized, W>
      ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
      for With<Collect<$($owned)*, Container, Ctx, Lang>, PhantomSpan>
    where
      L: Lexer<'inp>,
      F: ParseInput<'inp, L, O, Ctx, Lang>,
      Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx: ParseContext<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
      W: Window,
      Delim: Delimiter<'inp, L, Lang>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper($($map_primary)*)
          .parse_input(inp)
          .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
      }
    }

    // Block 3: &mut ref -> L::Span
    impl<'inp, 'c, L, F, Sep, Condition, O, Delim, Container, Ctx, Lang: ?Sized, W>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Collect<&'c mut $($reft)*, &'c mut Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: ParseInput<'inp, L, O, Ctx, Lang>,
      Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx: ParseContext<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
      W: Window,
      Delim: Delimiter<'inp, L, Lang>,
    {
      impl_separated_while_delim!(@inline $b3i
        fn parse_input(
          &mut self,
          input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L: Lexer<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
        {
          $($b3)*
        }
      );
    }

    struct Wrapper<T>(T);

    // Block 4: Wrapper -> L::Span
    impl<'inp, 'c, L, F, Sep, Condition, O, Delim, Container, Ctx, Lang: ?Sized, W>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Wrapper<Collect<$($wt)*, &'c mut Container, Ctx, Lang>>
    where
      L: Lexer<'inp>,
      F: ParseInput<'inp, L, O, Ctx, Lang>,
      Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx: ParseContext<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
      W: Window,
      Delim: Delimiter<'inp, L, Lang>,
    {
      impl_separated_while_delim!(@inline $b4i
        fn parse_input(
          &mut self,
          inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
          $($b4)*
        }
      );
    }
  };
}
```

- [ ] **Step 2: Convert `sep_while/delim/unbounded.rs`** (representative)

Replace `tokit/src/parser/many/sep_while/delim/unbounded.rs` with:

```rust
use crate::emitter::{UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>, Delim>],
  ref_type = [DelimitedBy<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>, Delim>],
  wrapper_type = [DelimitedBy<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>, Delim>],
  map_self = { self.as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  map_primary = { self.primary_mut().as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut())) },
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block3_body = {
    let (delim, container) = self.parts_mut();
    let (f, condition) = delim.parser.parts_mut();
    let parser =
      DelimitedBy::<_, Delim>::new(SeparatedWhile::new::<Sep>(&mut **f, &mut *condition));
    Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
  },
  block4_inline = true,
  block4_body = {
    const HANDLER: &Unbounded = &Unbounded;
    let (parser, container) = self.0.parts_mut();
    let (f, condition) = parser.parser.parts_mut();
    DelimitedBy::<_, Delim>::new(SeparatedWhile::new::<Sep>(&mut **f, &mut **condition))
      .parse_separated(inp, container, HANDLER, HANDLER, HANDLER)
  },
}
```

Key differences from `sep_while/parse/unbounded.rs`:
- `owned_type`, `ref_type`, `wrapper_type` all wrap in `DelimitedBy<..., Delim>`
- Block 3: `let (delim, container) = self.parts_mut();` then `let (f, condition) = delim.parser.parts_mut();`
- Block 4: reconstructs `DelimitedBy::<_, Delim>::new(SeparatedWhile::new::<Sep>(...))` and calls `.parse_separated()`
- Block 4 condition deref: `&mut **condition` (double deref through `&'c mut`)

- [ ] **Step 3: Verify compilation**

Run: `cargo check --all-features -p tokit`

- [ ] **Step 4: Convert remaining `sep_while/delim/` files**

Convert all 36 leaf files. Read each original first. Same policy/cardinality patterns as previous tasks, adapted for both sep_while and delim differences.

**Important `block4_inline` rule for delim mode:** In `sep_while/delim/` (and `sep/delim/`), ALL files use `block4_inline = true` — including unbounded files. This differs from parse mode (`sep/parse/`, `sep_while/parse/`) where unbounded files use `block4_inline = false`. Verify by reading each actual file's block 4 for the `#[cfg_attr(not(tarpaulin), inline(always))]` attribute.

- [ ] **Step 5: Verify full test suite**

Run: `cargo test --all-features -p tokit`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add tokit/src/parser/many/macros.rs tokit/src/parser/many/sep_while/delim/
git commit -m "refactor: add impl_separated_while_delim macro and convert all sep_while/delim/ files"
```

---

### Task 6: Final verification and cleanup

- [ ] **Step 1: Run full test suite**

Run: `cargo test --all-features`
Expected: All tests pass with zero behavioral change.

- [ ] **Step 2: Verify line count reduction**

Run: `find tokit/src/parser/many/{sep,sep_while}/{parse,delim} -name '*.rs' -not -name 'mod.rs' | xargs wc -l | tail -1`
Expected: Significant reduction from the original ~20,000+ lines.

- [ ] **Step 3: Verify no API changes**

Run: `cargo doc --all-features --no-deps 2>&1 | grep -i error || echo "docs OK"`
Expected: No documentation errors.

- [ ] **Step 4: Commit any cleanup**

If any cleanup is needed (e.g., removing unused imports from macros.rs), commit it:

```bash
git add -A
git commit -m "refactor: final cleanup after macro deduplication"
```
