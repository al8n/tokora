# Tokit Code & Design Review

## Project Overview

Tokit is a zero-copy, parse-while-lexing parser combinator library for Rust with `no_std` support, deterministic lookahead, and atomically composable error handling. It bridges Logos lexers directly to parser combinators without intermediate token buffering.

---

## Architecture: Strengths

**1. Parse-While-Lexing Pipeline** — The core architectural decision to stream tokens on-demand from the lexer (rather than buffering into `Vec<Token>`) is sound. It provides better cache locality, lower memory usage, and O(n) single-pass parsing with compile-time-bounded lookahead (`Window` trait with `U1`..`U32`).

**2. Atomically Composable Emitters** — Breaking error handling into fine-grained traits (`TooFewEmitter`, `SeparatedEmitter`, `UnexpectedLeadingSeparatorEmitter`, etc.) rather than one monolithic interface is elegant. The same parser code works with `Fatal` (fail-fast), `Verbose` (collect all), or `Silent` (suppress) emitters with zero runtime overhead for the zero-sized variants.

**3. Deterministic Parsing** — Explicit lookahead via `peek_then` with compile-time window capacity avoids the hidden backtracking problem of traditional parser combinators. `ParseInput` (must-succeed) vs `TryParseInput` (can-decline) makes the commit/backtrack distinction explicit in the type system.

**4. Comprehensive Separator Handling** — The 8 separator policies (allow/require leading/trailing/surrounded and combinations) with 4 count modifiers (unbounded, at_least, at_most, bounded) cover real-world grammar needs thoroughly.

---

## Architecture: Concerns

### 1. Massive Code Duplication in `parser/many/` (~85-90% boilerplate)

The `parser/many/` subsystem contains **257 files totaling ~35,000 lines** across `sep/parse/`, `sep/delim/`, `sep_while/parse/`, `sep_while/delim/`, and `handler/`. The four target directories have perfectly isomorphic structure (45 files each).

The difference between `sep/` and `sep_while/` counterpart files is purely mechanical: `Separated<F, Sep, O, ...>` becomes `SeparatedWhile<F, Sep, Condition, O, W, ...>`, `F: TryParseInput` becomes `F: ParseInput`, and one extra variable binding is added. The `parse/` vs `delim/` difference is equally mechanical (wrapping in `DelimitedBy`, adding `Delimiter` bounds).

Each file contains 4 `impl` blocks where only the type signatures vary — the bodies are character-for-character identical across all 128+ leaf files.

**Recommendation:** A `macro_rules!` macro parameterized on `(ParserType, extra_type_params, extra_bounds, constructor_call)` could reduce this from ~30,000 lines to ~5,000 lines. Adding a new separator policy currently requires creating 16+ new files; with macros it would be one table row.

### 2. High API Setup Cost

The examples reveal substantial ceremony before writing any parser:

- **Calculator (simplest):** ~175 lines of boilerplate before the first parser function
- **JSON (most complex):** ~360 lines before any parsing logic
- Requires: Token enum + TokenKind enum + Display impls + `From<&Token>` + `Token` trait impl + LexError newtype + error enum with 5-9 `From` impls

**Recommendation:** Consider `derive` macros for Token/TokenKind boilerplate and a standard error enum helper. A `prelude` module would reduce import lists from 27 lines to 1.

### 3. Type Parameter Proliferation

`ParseInput<'inp, L, O, Ctx, Lang>` has 5 type parameters. Every parser function signature repeats them plus associated type projections like `<Ctx::Emitter as Emitter<'inp, L, Lang>>::Error`. The `Lang` phantom parameter appears on nearly everything but defaults to `()` — most users never set it, yet it leaks into every trait bound and compiler error message.

**Recommendation:** Consider type aliases for common configurations (e.g., `type SimpleParser<'inp, L, O, Err> = ...`). Evaluate whether `Lang` can be hidden behind a feature flag.

### 4. HRTB Closure Limitation

Higher-ranked trait bound requirements force users to use named `fn` items instead of closures in many places. This is explicitly acknowledged in example comments but significantly reduces the ergonomic advantage of a combinator library compared to chumsky or winnow.

---

## Correctness Findings

### Cache Rewind Inconsistency (Low-Medium)

The `Option` cache treats `cursor == span.end` as "keep the token," while `GenericArrayDeque` treats `cursor >= span.end` as "clear all tokens." This asymmetry could cause a consumed token to be re-read from the `Option` cache after a checkpoint restore.

**Location:** `src/cache/` — `Option` impl vs `GenericArrayDeque` impl rewind logic.

### Pratt Parser `prev()` Underflow Risk (Medium)

`PrattPower::prev()` is called for right-associative operators to get a lower minimum precedence. If a user implements `PrattPower` for a numeric type and assigns the minimum representable value as a precedence, `prev()` will underflow (panic in debug, wrap in release). The trait doesn't constrain `prev()` to be saturating.

**Location:** `src/parser/pratt/expr.rs` — the `rpower = lpower.prev()` path.

### Error Recovery Doesn't Roll Back Emitter (Design Limitation)

`Recover` saves a checkpoint and restores input on failure, but errors already emitted to the `Emitter` during the failed attempt are not un-emitted. If recovery succeeds, the parse returns `Ok` but the emitter may contain spurious errors from the failed branch. This is common in parser combinator libraries but worth documenting prominently.

### All `unsafe` Code Is Sound

5 uses of `unsafe`, all `#[repr(transparent)]` pointer casts or guarded `MaybeUninit` initialization. No issues found.

---

## Test Quality

**Strengths:**
- 2,647 test functions, ~47,000 lines, 84% coverage achieved
- Comprehensive combinatorial exhaustion of separator policies
- Strong behavioral tests in `parser_basic.rs`, `parser_pratt.rs`, examples

**Weaknesses:**
- ~30 coverage-focused test files primarily assert `r.is_ok()` or `!r.is_empty()` without verifying values
- Error type erasure (`struct E;`) in 18+ files makes it impossible to assert which error was produced
- No property-based testing (`proptest`/`quickcheck`), no fuzz targets
- No `alloc`-only (without `std`) test path — all tests gate on `feature = "std"`
- `parse_choice.rs` at 8.3% coverage and `parse_state.rs` at 27.3% are significant gaps

**Minor inconsistencies found:**
- Missing `#[cfg_attr(not(tarpaulin), inline(always))]` on the 3rd impl block in `sep/parse/allow_trailing/unbounded.rs` and `sep_while/parse/allow_trailing/unbounded.rs` (present in all other files)
- `Wrapper` impl constant named `HANDLER` in `parse/` files but `UNBOUNDED` in some `delim/` files

---

## Documentation

**Strong:**
- README is comprehensive with architecture overview, design philosophy, feature flags
- `#![deny(missing_docs)]` enforced — all public items have doc comments
- `parser/mod.rs` has excellent architecture diagrams and combinator reference
- 4 polished runnable examples with inline tests

**Missing:**
- No end-to-end "Hello, Parser" tutorial for new users
- All code snippets in module docs are `ignore`-tagged with placeholder types — none are copy-pasteable
- No example demonstrates `Verbose` emitter, error recovery, or custom emitters
- `json.rs` (most complex example) has no module-level doc
- `_with` combinator variants (`map_with`, `filter_with`, etc.) are undocumented relative to their non-`_with` counterparts
- No `prelude` module

---

## Prioritized Recommendations

| Priority | ID | Recommendation | Effort |
|----------|----|---------------|--------|
| **High** | R1 | Fix the `Option` vs `GenericArrayDeque` cache rewind inconsistency | Small |
| **High** | R2 | Add missing `inline(always)` attributes on 2 files | Tiny |
| **High** | R3 | Normalize `HANDLER` vs `UNBOUNDED` constant naming | Tiny |
| **High** | R4 | Add `prelude` module | Small |
| **Medium** | R5 | Document emitter non-rollback behavior in `Recover` | Small |
| **Medium** | R6 | Add module-level doc to `json.rs` example | Small |
| **Medium** | R7 | Add module-level doc to `emitter/mod.rs` | Small |
| **Medium** | R8 | Guard `PrattPower::prev()` with saturating arithmetic or doc warning | Small |
| **Low** | R9 | Reduce `parser/many/` duplication with macros (~30k lines to ~5k) | Large |
| **Low** | R10 | Add `derive` macros for Token/Error boilerplate | Large |
| **Low** | R11 | Add property-based / fuzz testing | Medium |
| **Low** | R12 | Add `alloc`-only test path (no `std`) | Medium |
