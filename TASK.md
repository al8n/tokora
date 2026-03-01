# Coverage Task: Reach 70% on `tokit/src/`

## Baseline (measured with `cargo tarpaulin --run-types AllTargets --workspace --all-features`)

| Metric | Value |
|--------|-------|
| src/ total lines | 12,240 |
| src/ covered lines | 84 (0.69%) |
| **Target** | **8,568 lines (70%)** |
| **Gap** | **8,484 lines** |

### Lines by module (largest first)

| Lines | Module | Notes |
|-------|--------|-------|
| 7,268 | `parser/many/` | All 0% – the `sep_while` subtree (dozens of variant impls) |
| 698 | `parser/` (non-many) | Pratt 0/113, expect 0/58, then 0/42, map/filter/etc |
| 1,484 | `error/` | Rich error types, all 0% |
| 597 | `input/` | InputRef + helpers, all 0% |
| 459 | `utils/` | Display/parse helpers, all 0% |
| 201 | `emitter/` | Fatal/Silent/Ignored/Verbose impls |
| 192 | `span.rs` | Span arithmetic, all 0% |
| 181 | `cache/` | Cache impls, all 0% |
| 155 | `token/` | Token traits, all 0% |
| 136 | `state/` | Tracker, all 0% |
| ~869 | others | punct, types, source, slice, try_parse_input, … |

### Why nothing is covered today

The 4 examples (`json`, `calculator`, `s_expression`, `c_expression`) are compiled by tarpaulin
but **not run**: they only contain `fn main()`, and cargo's test harness reports "running 0
tests" without invoking `main()`. The 17 existing tests are compile-time trait assertions or
tiny container-error unit tests.

---

## Strategy

The most impactful lever is the `parser/many/` subtree (7,268 lines, 59% of all src lines).
Every call to `separated_by`, `repeated`, `fold`, etc. through the high-level `Parser` API
exercises those generics. So the plan is:

1. **Integration tests** in `tokit/tests/` using a real logos lexer (`--all-features` enables
   `logos_0_16`). Each test file calls `Parser::new().apply(fn).parse_str(src)` end-to-end,
   which instruments the full stack (lexer → cache → InputRef → combinators → emitter).
2. **Inline unit tests** (`#[cfg(test)] mod tests`) in the larger standalone modules
   (`span.rs`, `error/`, `state/`, etc.) where a logos integration test would be overkill.

---

## Work Items (ordered by coverage impact)

### Step 1 — Integration test harness (`tests/helpers.rs` + `tests/lexer.rs`)

Create a shared test lexer using logos_0_16 that all integration tests will import.
Token set: `Num(i64)`, `Ident(String)`, `Plus`, `Minus`, `Star`, `Slash`, `Comma`,
`Semi`, `LParen`, `RParen`, `LBracket`, `RBracket`, `Eq`.

Expected gain: ~50 lines (setup boilerplate, touches `lexer/logos.rs`).

### Step 2 — Basic parser combinator tests (`tests/parser_basic.rs`)

Test: `any`, `filter`, `filter_map`, `map`, `validate`, `then`, `then_ignore`, `ignore_then`,
`then_value`, `and_then`, `opt`, `expect`, `fail`, `empty`, `fold`, `peek_then`.

Both the happy path and error paths for each combinator (exercises emitter code too).

Expected gain: ~800 lines.

### Step 3 — Repetition / collection tests (`tests/parser_many.rs`)

Test `repeated` and `separated_by` with every separator policy variant:
- Plain (unbounded, at_least, at_most, bounded)
- `allow_leading` × 4 count variants
- `allow_trailing` × 4
- `allow_surrounded` × 4
- `require_leading` × 4
- `require_trailing` × 4
- `require_leading_allow_trailing` × 4
- `require_surrounded` × 4

This is the single highest-impact step: exercising all 33 files in
`parser/many/sep_while/parse/` (plus `mod.rs` with 104 lines of shared logic).

Expected gain: ~5,500 lines.

### Step 4 — Pratt parser tests (`tests/parser_pratt.rs`)

Test `InputRef::pratt` (token-level) and `pratt_of` (combinator) APIs.
Mirror the calculator and c_expression examples but inside proper `#[test]` functions.

Expected gain: ~113 lines (pratt module) + InputRef pratt.rs paths.

### Step 5 — Emitter tests (`tests/emitter.rs`)

Test `Fatal`, `Silent`, `Ignored`, and `Verbose` emitters by triggering error conditions
(unexpected token, unexpected EOT, too-few, too-many) and asserting the appropriate
behavior (propagate / default / ignore / collect).

Expected gain: ~200 lines.

### Step 6 — Span tests (inline in `src/span.rs`)

Test `SimpleSpan` construction, arithmetic, ordering, Display, and merging.

Expected gain: ~150 lines.

### Step 7 — Error type tests (inline in `src/error/`)

Test `Display` and `From` impls for: `UnexpectedToken`, `UnexpectedEot`, `MissingToken`,
`UnexpectedEof`, `HexEscape`, `UnicodeEscape`, `Malformed`, `Invalid`, `Unclosed`.

Expected gain: ~700 lines.

### Step 8 — State tracker tests (inline in `src/state/`)

Test `TokenTracker`, `RecursionTracker`, and `Tracker` (increment, overflow, reset, clone).

Expected gain: ~120 lines.

### Step 9 — Verify and gap-fill

Run `--all-features --run-types tests --run-types doctests --run-types examples --run-types lib --workspace`, inspect which src/
files are still below target, and add targeted tests until `src/` is ≥70%.

---

## Running coverage

```bash
--all-features --run-types tests --run-types doctests --run-types examples --run-types lib --workspace
```

Extract src-only line:
```bash
cargo tarpaulin ... 2>&1 | grep "^|| tokit/src/" | \
  grep -E "[0-9]+/[0-9]+" | sed 's/.*: //; s/ .*//' | \
  awk -F'/' '{c+=$1; t+=$2} END {printf "src: %d/%d = %.1f%%\n", c, t, c/t*100}'
```
