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
- `constructor_args` — how to destructure `parts_mut()` and call `Parser::new::<Sep>(...)`. For `sep`: `let f = parser.fn_mut()` then `Separated::new::<Sep>(&mut **f)`. For `sep_while`: `let (f, condition) = parser.parts_mut()` then `SeparatedWhile::new::<Sep>(&mut **f, &mut *condition)`.

**Delim axis** (set by outer macro):
- Whether to wrap in `DelimitedBy<..., Delim>` and add `Delim: Delimiter` bound
- `.parse()` vs `.parse_separated()` call
- Extra container trait `DelimiterHandler<'inp, L>`
- Extra error bound `From<UnexpectedEot>`

**Policy axis** (set by leaf file):
- `policy_wrapper` — identity / `AllowTrailing` / `RequireLeading` / `Surrounded` / etc.
- `handler_expr` — the `const HANDLER` initializer expression

**Cardinality axis** (set by leaf file):
- `cardinality_wrapper` — identity / `AtLeast` / `AtMost` / `Bounded`
- `extra_emitters` — additional emitter trait bounds (`TooFewEmitter`, `TooManyEmitter`, or both)
- `block3_constructor` — how to reconstruct the cardinality in block 3 (e.g., `AtLeast::new(..., minimum.get())`, `Bounded::new(..., maximum.get(), minimum.get())`)
- `block4_limitation` — how to compute the limitation in block 4: nothing for unbounded (uses const), `.minimum()` for at_least, `.maximum()` for at_most, `.to_with()` for bounded. The limitation value is wrapped in the policy wrapper before being passed as handler args.

### Impl Block Pattern

Each leaf file generates 4 impl blocks:

1. `Collect<Policy<Parser>, Container> → Container` — Owned, returns collected container. Always has `#[cfg_attr(not(tarpaulin), inline(always))]`.

2. `With<Collect<Policy<Parser>, Container>, PhantomSpan> → Spanned<Container>` — Owned with span, returns `Spanned<Container>`. Always has `#[cfg_attr(not(tarpaulin), inline(always))]`.

3. `Collect<&mut Policy<Parser>, &mut Container> → L::Span` — Borrowed, returns span only. Destructures via `parts_mut()`, constructs a fresh parser, wraps in `Wrapper`. Has `#[cfg_attr(not(tarpaulin), inline(always))]` **except** for the bare unbounded case (no policy wrapper, unbounded cardinality).

4. `Wrapper<Collect<Policy<Parser<&mut F, ...>>, &mut Container>> → L::Span` — The core implementation. Has `#[cfg_attr(not(tarpaulin), inline(always))]` **only** for non-unbounded cardinalities (`AtLeast`, `AtMost`, `Bounded`). Unbounded cardinality files have no inline on this block.

The macro also generates `struct Wrapper<T>(T);` once per file.

### Block 3 Constructor Patterns

Block 3 reconstructs a fresh parser from borrowed parts. The exact pattern varies by axis:

**parse mode (sep and sep_while):**
```rust
let (parser, container) = self.parts_mut();
let f = parser.fn_mut();  // or parser.parser_mut().fn_mut() for policy+cardinality
let parser = Policy::new(Cardinality::new(Separated::new::<Sep>(&mut **f), ...));
Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
```

**delim mode:** Same pattern but wraps in `DelimitedBy`:
```rust
let (delim, container) = self.parts_mut();
let f = delim.parser.fn_mut();  // navigates through DelimitedBy to inner parser
let parser = DelimitedBy::<_, Delim>::new(Policy::new(Cardinality::new(Separated::new::<Sep>(&mut **f), ...)));
Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
```

**sep_while** adds: `let (f, condition) = parser.parts_mut();` and passes `condition` to the constructor.

### Block 4 (Wrapper) Patterns

Block 4 has two distinct patterns based on cardinality:

**Unbounded cardinality** (uses const handler):
```rust
const HANDLER: &PolicyWrapper<Unbounded> = &PolicyWrapper::new(Unbounded);
let (parser, container) = self.0.parts_mut();
parser.parse(inp, container, HANDLER, HANDLER, HANDLER)  // or .parse_separated() for delim
```

**Non-unbounded cardinality** (`AtLeast`/`AtMost`/`Bounded` — uses dynamic limitation):

For `AtLeast` (no policy wrapper):
```rust
let (parser, container) = self.0.parts_mut();
let minimum = parser.minimum();
parser.parser_mut().parse(inp, container, &minimum, &minimum, &minimum)
```

For `AtLeast` with policy wrapper (e.g., `AllowTrailing`):
```rust
let (parser, container) = self.0.parts_mut();
let limitation = AllowTrailing::new(parser.parser.minimum());
parser.parser_mut().parser_mut().parse(inp, container, &limitation, &limitation, &limitation)
```

For `Bounded` with policy wrapper:
```rust
let (parser, container) = self.0.parts_mut();
let limitation = AllowTrailing::new(parser.parser.to_with());
parser.parser_mut().parser_mut().parse(inp, container, &limitation, &limitation, &limitation)
```

Note: `parser.parser` is a direct field access on the policy wrapper struct, not a method call. The number of `.parser_mut()` calls equals the nesting depth (1 for cardinality-only, 2 for policy+cardinality). `to_with()` returns a value combining both minimum and maximum.

**delim mode** additionally reconstructs `DelimitedBy::new(Separated::new::<Sep>(...))` in the Wrapper block and calls `.parse_separated()` instead of `.parse()`.

### Policy Wrapping

For the "unbounded" (no policy) case, the policy wrapper is identity — no wrapping applied. For all other policies (`AllowTrailing`, `RequireLeading`, `RequireTrailing`, `Surrounded`, `AllowLeading`, `AllowSurrounded`, `RequireLeadingAllowTrailing`, `AllowLeadingRequireTrailing`), the policy wraps the parser type in all 4 blocks uniformly.

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
