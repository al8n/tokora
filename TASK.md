# Coverage Task: Reach 70% overall on `tokit/src/`, 75% on `parser/`

## Current State (measured with `cargo tarpaulin --all-features --run-types tests --run-types doctests --run-types examples --run-types lib --workspace`)

| Metric | Value |
|--------|-------|
| src/ total lines | 13,091 |
| src/ covered lines | 7,871 (60.1%) |
| **Overall Target** | **9,164 lines (70%)** |
| **Overall Gap** | **~1,293 lines** |
| parser/ total lines | 7,966 |
| parser/ covered lines | 3,661 (46.0%) |
| **Parser Target** | **5,975 lines (75%)** |
| **Parser Gap** | **~2,314 lines** |

### Coverage by module

| Covered/Total | Pct | Module | Status |
|---------------|-----|--------|--------|
| 3,661/7,966 | 46% | `parser/` | **needs work** |
| 103/155 | 66% | `token/` | needs work |
| 137/181 | 76% | `cache/` | close |
| 501/639 | 78% | `input/` | close |
| 163/201 | 81% | `emitter/` | ok |
| 402/459 | 88% | `utils/` | ok |
| 124/136 | 91% | `state/` | ok |
| 1,373/1,482 | 93% | `error/` | ok |
| 125/133 | 94% | `types/` | ok |
| 187/187 | 100% | `span.rs` | done |
| ~591/597 | 99% | others | done |

### Parser sub-module breakdown (largest uncovered first)

| Uncovered | Covered/Total | Pct | Parser sub-module |
|-----------|---------------|-----|-------------------|
| 1,115 | 532/1,647 | 32% | `many/sep_while/delim/` |
| 1,033 | 566/1,599 | 35% | `many/sep/delim/` |
| 848 | 511/1,359 | 38% | `many/sep_while/parse/` |
| 813 | 513/1,326 | 39% | `many/sep/parse/` |
| ~259 | ~418/677 | 62% | `many/handler/*` |
| ~237 | various | various | small files (expect, fail, map, accepted, peek, etc.) |

---

## What's Already Done (Steps 1–8 from previous plan)

- **Test harness**: `tests/common/` with shared logos lexer, emitter types, and helpers.
- **Basic combinators**: `tests/parser_basic.rs` — any, filter, map, validate, then, opt, etc.
- **Repetition/collection**: `tests/parser_many.rs`, `parser_repeated.rs`, `parser_repeated_while.rs`, `parser_repeated_delim.rs` — repeated and separated_by with parse variants.
- **Sep policies (parse)**: `tests/parser_sep_parse_policies.rs`, `parser_sep_while_parse_policies.rs` — 8 policies × 4 count variants for non-delimited parse paths.
- **Sep policies (delim)**: `tests/parser_sep_delim_policies.rs` — 8 policies × 4 count variants for delimited sep. `tests/parser_sep_while_delim.rs` — 8 policies × 3 count variants (missing unbounded).
- **Pratt parser**: `tests/parser_pratt.rs` — token-level and combinator pratt APIs (97% coverage).
- **Emitter**: `tests/emitter_tests.rs`, `emitter_coverage.rs` — Fatal/Silent/Ignored/Verbose.
- **Span**: `tests/span_tests.rs` — SimpleSpan (100% coverage).
- **Error types**: `tests/error_types.rs` — Display/From impls (93% coverage).
- **State/tracker**: `tests/tracker_tests.rs` — TokenTracker, RecursionTracker (91% coverage).
- **Misc**: `tests/utils_tests.rs`, `keyword_punct_coverage.rs`, `input_ref_tests.rs`, `handler_coverage.rs`, etc.

---

## Strategy to reach targets

The **parser** module is the sole bottleneck. Getting parser from 46% → 75% (+2,314 lines) will also push overall from 60% → 78%, exceeding the 70% target.

The 4 biggest gaps are all in `parser/many/`:
1. `sep_while/delim/` — 1,115 uncovered lines (8 policies × 4 count variants, each ~30-35% covered)
2. `sep/delim/` — 1,033 uncovered lines (same structure, ~35% covered)
3. `sep_while/parse/` — 848 uncovered lines (~38% covered)
4. `sep/parse/` — 813 uncovered lines (~39% covered)

Each policy+count file has 3–4 `impl` blocks (owned collect, spanned with, mutable ref, wrapper).
Current tests exercise the owned collect path but miss the **mutable-ref** and **wrapper** impls.

---

## Remaining Work Items (ordered by coverage impact)

### Step 1 — sep_while/delim unbounded tests (add to `tests/parser_sep_while_delim.rs`)

The existing file only tests at_least, at_most, bounded. Add **8 unbounded test functions** (one per policy) with success + edge-case tests.

Policies: `allow_leading`, `allow_trailing`, `allow_surrounded`, `allow_leading_require_trailing`, `require_leading`, `require_trailing`, `require_leading_allow_trailing`, `require_surrounded`.

Expected gain: ~200 lines.

### Step 2 — Delimited mutable-ref and wrapper paths (`tests/parser_delim_mut_ref.rs`)

Current delim tests only call `.collect().parse_input(inp)` (owned path). Add tests that exercise:
1. **Mutable-ref collect path**: `&mut Collect<&mut DelimitedBy<...>>` — call via `.by_ref()` or manual mutable borrow patterns.
2. **Spanned wrapper path**: `With<Collect<DelimitedBy<...>>, PhantomSpan>` — call via `.with_span()` or `.spanned()`.

Cover all 8 policies × 4 count variants × 2 paths (delim for both `sep` and `sep_while`).

This is the **highest-impact step**: each file has ~15-20 lines in mut-ref/wrapper impls that are currently 0%.

Expected gain: ~1,500 lines (across sep/delim + sep_while/delim).

### Step 3 — Parse mutable-ref and wrapper paths (`tests/parser_parse_mut_ref.rs`)

Same as Step 2 but for the non-delimited `sep/parse/` and `sep_while/parse/` directories.
Exercise `.by_ref()` and `.spanned()` paths for all 8 policies × 4 count variants.

Expected gain: ~1,000 lines.

### Step 4 — Handler coverage (`tests/handler_coverage.rs` expansion)

The handler module (`many/handler/`) is at 62% overall, with `mod.rs` at only 21%.
Add tests that trigger:
- `EndStateHandler` state transitions (start → leading → separator → element)
- `ContinueStateHandler` stop conditions
- `SeparatorStateHandler` invalid-initial-separator paths
- Edge cases: empty delimited input `[]`, single element `[1]`, exactly-at-limit, exceed-limit

Focus on `allow_leading_require_trailing` (54%) and `require_surrounded` (55%) which have the most complex state machines.

Expected gain: ~200 lines.

### Step 5 — Small parser file gap-fill

| File | Current | Notes |
|------|---------|-------|
| `accepted.rs` | 0/11 | Test `Accepted` wrapper |
| `ident.rs` | 0/7 | Test ident parser |
| `ident_list.rs` | 0/9 | Test ident list parser |
| `todo.rs` | 0/3 | Test todo! parser (should panic) |
| `unwrapped.rs` | 0/3 | Test unwrapped parser |
| `peek/peek_then.rs` | 4/14 | More peek_then edge cases |
| `expect.rs` | 33/58 | Error paths |
| `fail.rs` | 8/20 | More fail variants |
| `map.rs` | 10/21 | map error paths |
| `then/and_then_with.rs` | 7/13 | and_then_with edge cases |
| `then/then_value.rs` | 3/8 | then_value error path |
| `fold/fold_while.rs` | 28/34 | fold_while edge cases |
| `with.rs` | 30/40 | with combinator paths |
| `mod.rs` | 22/45 | Parser mod paths |

Expected gain: ~200 lines.

### Step 6 — Token module (`token/lit.rs`)

`token/lit.rs` is 50/90 (55%). Add tests for uncovered `LitToken` trait impls and Display paths.

Expected gain: ~30 lines.

### Step 7 — Verify and gap-fill

Run full tarpaulin, inspect per-file results, add targeted tests for any remaining gaps below 75% in parser or 70% overall.

---

## Running coverage

When calculating coverage, also consider the ignore section of `.codecov.yml` (ignores `tokit/examples/` and `tokit/tests/`).

```bash
cargo tarpaulin --all-features --run-types tests --run-types doctests --run-types examples --run-types lib --workspace
```

Extract src-only totals:
```bash
cargo tarpaulin ... 2>&1 | grep "^|| tokit/src/" | \
  grep -E "[0-9]+/[0-9]+" | sed 's/.*: //; s/ .*//' | \
  awk -F'/' '{c+=$1; t+=$2} END {printf "src: %d/%d = %.1f%%\n", c, t, c/t*100}'
```

Extract parser-only totals:
```bash
cargo tarpaulin ... 2>&1 | grep "^|| tokit/src/parser/" | \
  grep -E "[0-9]+/[0-9]+" | while IFS= read -r line; do \
  nums=$(echo "$line" | grep -oE '[0-9]+/[0-9]+'); \
  echo "$nums"; done | awk -F'/' '{c+=$1; t+=$2} END {printf "parser: %d/%d = %.1f%%\n", c, t, c/t*100}'
```
