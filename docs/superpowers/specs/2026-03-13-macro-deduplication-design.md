# Macro-Based Deduplication of `parser/many/` — Design Spec

## Goal

Reduce ~20,000+ lines of near-identical code across 144 leaf files in `parser/many/` to macro invocations, using a layered macro approach that preserves the existing module structure and all behavior.

## Scope

**In scope:** `sep/parse/`, `sep/delim/`, `sep_while/parse/`, `sep_while/delim/` — 144 leaf files (36 per directory) plus the macro definitions.

**Out of scope:** `handler/` directory, core `mod.rs` files containing parse logic, policy subdirectory `mod.rs` files (pure module declarations).

## Architecture

### Layered Macro Design

Two layers of macros defined in `tokit/src/parser/many/macros.rs`:

**Inner macro: `impl_collect_blocks!`**

Generates all 4 impl blocks for a leaf file plus the `struct Wrapper<T>(T);` declaration. Parameterized on every axis of variation.

**4 outer macros:**
- `impl_separated!` — bakes in `Separated`, `TryParseInput`, no extra generics, `.parse()` method
- `impl_separated_while!` — bakes in `SeparatedWhile`, `ParseInput`, `Condition`/`W` generics, `.parse()` method
- `impl_separated_delim!` — bakes in `Separated`, `TryParseInput`, `DelimitedBy` wrapping, `.parse_separated()` method
- `impl_separated_while_delim!` — bakes in `SeparatedWhile`, `ParseInput`, `Condition`/`W` generics, `DelimitedBy` wrapping, `.parse_separated()` method

Each outer macro calls `impl_collect_blocks!` with the parser-type and delim-mode parameters pre-filled, leaving only policy and cardinality parameters for the leaf file to specify.

### Parameters

**Parser type axis** (set by outer macro):
- `parser_type` — `Separated` or `SeparatedWhile`
- `parser_trait` — `TryParseInput` or `ParseInput`
- `extra_generics` — empty for sep, `Condition, W` for sep_while
- `extra_bounds` — empty for sep, `Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>, W: Window` for sep_while
- `constructor_args` — how to call `Parser::new::<Sep>(...)` and destructure `parts_mut()`

**Delim axis** (set by outer macro):
- Whether to wrap in `DelimitedBy<..., Delim>` and add `Delim: Delimiter` bound
- `.parse()` vs `.parse_separated()` call
- Extra container trait `DelimiterHandler<'inp, L>`
- Extra error bound `From<UnexpectedEot>`

**Policy axis** (set by leaf file):
- `policy_wrapper` — identity / `AllowTrailing` / `RequireLeading` / `Surrounded` / etc.
- `handler_expr` — the `const HANDLER` initializer expression

**Cardinality axis** (set by leaf file):
- `extra_emitters` — additional emitter trait bounds (`TooFewEmitter`, `TooManyEmitter`, or both)
- `limitation` — method chain on the parse result (`.minimum()`, `.maximum()`, `.to_with()`, or nothing)

### Impl Block Pattern

Each leaf file generates 4 impl blocks:

1. `Collect<Policy<Parser>, Container> → Container` — Owned, returns collected container. Has `#[cfg_attr(not(tarpaulin), inline(always))]`.

2. `With<Collect<Policy<Parser>, Container>, PhantomSpan> → Spanned<Container>` — Owned with span, returns `Spanned<Container>`. Has `#[cfg_attr(not(tarpaulin), inline(always))]`.

3. `Collect<&mut Policy<Parser>, &mut Container> → L::Span` — Borrowed, returns span only. Has `#[cfg_attr(not(tarpaulin), inline(always))]`. Destructures via `parts_mut()`, constructs a fresh parser, wraps in `Wrapper`.

4. `Wrapper<Collect<Policy<Parser<&mut F, ...>>, &mut Container>> → L::Span` — The core implementation. No inline attribute. Defines `const HANDLER`, calls the parse method with handler and limitation chain.

The macro also generates `struct Wrapper<T>(T);` once per file.

### Policy Wrapping

For the "unbounded" (no policy) case, the policy wrapper is identity — no wrapping applied. For all other policies (`AllowTrailing`, `RequireLeading`, `RequireTrailing`, `Surrounded`, `AllowLeading`, `AllowSurrounded`, `RequireLeadingAllowTrailing`, `RequireTrailingAllowLeading`), the policy wraps the parser type in all 4 blocks uniformly.

### Leaf File Example

```rust
// sep/parse/allow_trailing/at_least.rs
use super::*;
impl_separated!(
    policy = AllowTrailing,
    cardinality = AtLeast,
    handler_expr = &AllowTrailing::new(AtLeast::new(Unbounded)),
    extra_emitters = (TooFewEmitter),
    limitation = { .minimum() },
);
```

For unbounded with no policy:
```rust
// sep/parse/unbounded.rs
use super::*;
impl_separated!(
    handler_expr = &Unbounded,
    extra_emitters = (),
    limitation = {},
);
```

## File Changes

**New:**
- `tokit/src/parser/many/macros.rs` — macro definitions (~600 lines)

**Modified:**
- `tokit/src/parser/many/mod.rs` — add `#[macro_use] mod macros;`
- 144 leaf files — replace ~140-160 lines each with ~5-15 line macro invocations

**Untouched:**
- Policy subdirectory `mod.rs` files (pure `mod` declarations)
- Core `mod.rs` files (`sep/parse/mod.rs`, `sep/delim/mod.rs`, `sep_while/parse/mod.rs`, `sep_while/delim/mod.rs`)
- `handler/` directory

## Verification

- `cargo test --all-features` must pass with zero behavioral change
- `cargo check --all-features` must compile cleanly
- This is a pure refactor — no API changes, no new features, no behavioral differences
