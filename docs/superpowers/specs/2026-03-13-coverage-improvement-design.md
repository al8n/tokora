# Code Coverage Improvement to 90%

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan.

**Goal:** Improve tarpaulin code coverage from ~81.6% to over 90% by excluding example files and writing targeted tests for uncovered library code.

**Current state:** 81.57% (5,915/7,251 lines)
**After excluding examples:** ~85.5% (5,915/6,918 lines)
**Target:** 90% (~6,226 lines covered) — need ~311 additional lines covered

---

## Configuration

Add `tarpaulin.toml` at workspace root to exclude example files from coverage:

```toml
[default]
exclude-files = ["tokit/examples/*"]
```

Examples are demo code (333 lines, 0% covered) and are not library logic.

**Note:** The CI command (`cargo tarpaulin --all-features --run-types tests --run-types doctests --run-types lib --run-types examples --workspace --out xml`) should remain unchanged. `exclude-files` filters the coverage report output, not execution. No CI changes needed.

## Priority Order

| Priority | File(s) | Uncovered Lines | Category |
|----------|---------|-----------------|----------|
| 1 | `token/lit.rs` | 40 | Reference delegation + composite defaults |
| 2 | `input/input_ref/try_expect.rs` | 34 | Macro-generated punct methods |
| 3 | `parser/many/sep_while/parse/mod.rs` | 32 | State machine branches |
| 4 | `parser/many/sep/parse/mod.rs` | 30 | State machine branches |
| 5 | `parser/many/handler/mod.rs` | 21 | SeparatorHandler/DelimiterHandler impls on `()`, `PhantomData`, containers |
| 6 | `input/input_ref/peek.rs` | 25 | Cache overflow |
| 7 | `src/punct.rs` (not `parser/punct.rs` or `token/punct.rs`) | 20 | Trait defaults |
| 8 | `parser/mod.rs` | 16 | Construction methods |
| 9 | `cache/generic_arraydeque.rs` | 17 | Binary search rewind |
| 10 | `cache/option.rs` | 14 | Rewind boundary |
| 11 | `error/token/missing_token/mod.rs` | 17 | Error methods |
| 12 | `input/input_ref/sync_through.rs` | 17 | Sync logic |
| 13 | `input/input_ref/consume_cached.rs` | 14 | Consume logic |
| 14 | `utils/mod.rs` + `message.rs` + `oneof.rs` | 38 | Utility traits |

**Total addressable:** ~335 lines (over the ~311 needed)

**Note on feature-gated code:** All feature-gated container impls in `handler/mod.rs` (`smallvec_1`, `tinyvec_1`, `heapless_0_9`) are covered by `--all-features`. Tests exercising these impls must be compiled with the relevant features enabled (use `#[cfg(feature = "...")]` on test functions or use `cargo test --all-features`).

### Fallback Candidates

If the primary 335 lines prove insufficient (some lines may be unreachable), these additional files can be targeted:

| File | Uncovered Lines |
|------|-----------------|
| `parser/many/sep_while/delim/mod.rs` | ~15 |
| `input/input_ref/pratt.rs` | ~15 |
| `parser/many/delim/repeated.rs` | ~14 |
| `token/punct.rs` | ~6 |
| `parser/punct.rs` | ~2 |

These provide an additional ~52 lines of buffer.

## Test Strategy

### Phase 1: Scenario-Based Integration Tests

Tests in `tokit/tests/` that exercise multiple uncovered paths through realistic parsing scenarios.

#### 1. `tests/separated_parse_coverage.rs`
Targets: `parser/many/sep/parse/mod.rs` (30 lines), partially `handler/mod.rs`

Scenarios:
- **Leading separator** — Input starts with separator; tests `handle_separator()` unexpected-leading branch and emitter firing
- **Trailing separator** — Input ends with separator; tests `handle_end()` trailing-separator branch with `AllowTrailing` vs default policy
- **Too few items** — With `AtLeast` constraint; tests `handle_end()` too-few branch and `TooFewEmitter`
- **Empty input** — No tokens at all; tests early-exit path
- **Separator-only input** — Just a separator, no items; tests combined leading-separator + empty paths
- **Policy combinations** — `AllowLeading`, `AllowTrailing`, `AllowSurrounded`, `RequireLeading`, `RequireTrailing`, `RequireSurrounded` with various inputs

#### 2. `tests/separated_while_parse_coverage.rs`
Targets: `parser/many/sep_while/parse/mod.rs` (32 lines)

Scenarios:
- **While-condition terminates mid-parse** — Parser stops before separator due to while-condition
- **While-condition fails on separator** — Separator present but while-condition rejects continuation
- **Mixed termination** — While-condition and delimiter interactions
- Same policy combinations as separated_parse but with while-condition variants

#### 3. `tests/cache_coverage.rs`
Targets: `input/input_ref/peek.rs` (25 lines), `cache/generic_arraydeque.rs` (17 lines), `cache/option.rs` (14 lines)

Scenarios:
- **Fill cache past capacity** — Peek enough tokens to trigger overflow path in `peek.rs`
- **Rewind to boundary** — Trigger binary search path in `generic_arraydeque.rs`
- **Option cache rewind** — Trigger boundary checks in `option.rs`

### Phase 2: Targeted Unit Tests

Inline `#[cfg(test)]` modules in source files for testing private/internal APIs.

#### 4. `token/lit.rs` — add to existing test module
- Test reference delegation: `&T where T: LitToken` delegates all methods correctly
- Test composite default methods: `is_integer_literal()`, `is_float_literal()`, `is_string_literal()`, `is_char_literal()`, `is_numeric_literal()`

#### 5. `input/input_ref/try_expect.rs` — add test module
- Test representative sample of `try_expect_punct!` methods (5-10 methods covering different punctuator types)
- Tests exercise the macro expansion: successful expect, failed expect, and error paths

#### 6. `src/punct.rs` — add to existing tests or new test module
- Test trait default methods on `Punctuator`
- Test reference impl (`&T where T: Punctuator`)

#### 7. `parser/many/handler/mod.rs` — add test module
- Test `SeparatorHandler` and `DelimiterHandler` impls on `()` and `PhantomData` (no-op/discard handlers)
- Test one feature-gated container impl per feature flag (`Vec`, `SmallVec` with `#[cfg(feature = "smallvec_1")]`, `TinyVec` with `#[cfg(feature = "tinyvec_1")]`, `HeaplessVec` with `#[cfg(feature = "heapless_0_9")]`)

#### 8. `parser/mod.rs` — add test module
- Test parser construction methods (builder pattern methods)

#### 9. `error/token/missing_token/mod.rs` — add test module
- Test error struct `Display` impl, accessors, construction

#### 10. `utils/mod.rs`, `message.rs`, `oneof.rs` — add test modules
- Test trait default impls
- Test message formatting
- Test `OneOf` type behavior

#### 11. `input/input_ref/sync_through.rs` + `consume_cached.rs` — add test modules or integration test
- Test sync-through behavior: syncing input position through cached tokens
- Test consume-cached behavior: consuming tokens from cache

### Phase 3: Coverage Verification

1. Run `cargo tarpaulin --all-features` after Phase 1 to measure progress
2. Run again after Phase 2
3. If under 90%, consult the Fallback Candidates table above and target those files next. Use `cargo tarpaulin --all-features` output to identify specific uncovered line ranges.
4. Final verification run to confirm 90%+ achieved

## Test Patterns

Follow existing project conventions:
- Integration tests use minimal token/language types defined per test file
- Use `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]` for test token types
- Implement `Token<'_>` and relevant traits for test types
- Unit tests use `#[cfg(test)] mod tests { use super::*; ... }`

## Success Criteria

- `cargo tarpaulin --all-features` reports >= 90% line coverage (with examples excluded)
- All existing tests continue to pass (`cargo test --all-features`)
- No test relies on implementation details that would break on refactoring
