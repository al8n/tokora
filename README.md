> WIP: This project is still under active development and not ready for use.

<div align="center">
<h1>Tokit</h1>
</div>
<div align="center">

Blazing fast parser combinators with parse-while-lexing architecture (zero-copy), deterministic LALR-style parsing, and no hidden backtracking.

[<img alt="github" src="https://img.shields.io/badge/github-al8n/tokit-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
<img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Ftokit" height="22">
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/al8n/tokit/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/tokit?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-tokit-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">][doc-url]
[<img alt="crates.io" src="https://img.shields.io/crates/v/tokit?style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0yNTYsMEwzMS41MjgsMTEyLjIzNnYyODcuNTI4TDI1Niw1MTJsMjI0LjQ3Mi0xMTIuMjM2VjExMi4yMzZMMjU2LDB6IE0yMzQuMjc3LDQ1Mi41NjRMNzQuOTc0LDM3Mi45MTNWMTYwLjgxDQoJCQlsMTU5LjMwMyw3OS42NTFWNDUyLjU2NHogTTEwMS44MjYsMTI1LjY2MkwyNTYsNDguNTc2bDE1NC4xNzQsNzcuMDg3TDI1NiwyMDIuNzQ5TDEwMS44MjYsMTI1LjY2MnogTTQzNy4wMjYsMzcyLjkxMw0KCQkJbC0xNTkuMzAzLDc5LjY1MVYyNDAuNDYxbDE1OS4zMDMtNzkuNjUxVjM3Mi45MTN6IiBmaWxsPSIjRkZGIi8+DQoJPC9nPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPC9zdmc+DQo=" height="22">][crates-url]
[<img alt="crates.io" src="https://img.shields.io/crates/d/tokit?color=critical&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/PjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+PHN2ZyB0PSIxNjQ1MTE3MzMyOTU5IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjM0MjEiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkzIiB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIj48ZGVmcz48c3R5bGUgdHlwZT0idGV4dC9jc3MiPjwvc3R5bGU+PC9kZWZzPjxwYXRoIGQ9Ik00NjkuMzEyIDU3MC4yNHYtMjU2aDg1LjM3NnYyNTZoMTI4TDUxMiA3NTYuMjg4IDM0MS4zMTIgNTcwLjI0aDEyOHpNMTAyNCA2NDAuMTI4QzEwMjQgNzgyLjkxMiA5MTkuODcyIDg5NiA3ODcuNjQ4IDg5NmgtNTEyQzEyMy45MDQgODk2IDAgNzYxLjYgMCA1OTcuNTA0IDAgNDUxLjk2OCA5NC42NTYgMzMxLjUyIDIyNi40MzIgMzAyLjk3NiAyODQuMTYgMTk1LjQ1NiAzOTEuODA4IDEyOCA1MTIgMTI4YzE1Mi4zMiAwIDI4Mi4xMTIgMTA4LjQxNiAzMjMuMzkyIDI2MS4xMkM5NDEuODg4IDQxMy40NCAxMDI0IDUxOS4wNCAxMDI0IDY0MC4xOTJ6IG0tMjU5LjItMjA1LjMxMmMtMjQuNDQ4LTEyOS4wMjQtMTI4Ljg5Ni0yMjIuNzItMjUyLjgtMjIyLjcyLTk3LjI4IDAtMTgzLjA0IDU3LjM0NC0yMjQuNjQgMTQ3LjQ1NmwtOS4yOCAyMC4yMjQtMjAuOTI4IDIuOTQ0Yy0xMDMuMzYgMTQuNC0xNzguMzY4IDEwNC4zMi0xNzguMzY4IDIxNC43MiAwIDExNy45NTIgODguODMyIDIxNC40IDE5Ni45MjggMjE0LjRoNTEyYzg4LjMyIDAgMTU3LjUwNC03NS4xMzYgMTU3LjUwNC0xNzEuNzEyIDAtODguMDY0LTY1LjkyLTE2NC45MjgtMTQ0Ljk2LTE3MS43NzZsLTI5LjUwNC0yLjU2LTUuODg4LTMwLjk3NnoiIGZpbGw9IiNmZmZmZmYiIHAtaWQ9IjM0MjIiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkwIiBjbGFzcz0iIj48L3BhdGg+PC9zdmc+&style=for-the-badge" height="22">][crates-url]
<img alt="license" src="https://img.shields.io/badge/License-Apache%202.0/MIT-blue.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PCFET0NUWVBFIHN2ZyBQVUJMSUMgIi0vL1czQy8vRFREIFNWRyAxLjEvL0VOIiAiaHR0cDovL3d3dy53My5vcmcvR3JhcGhpY3MvU1ZHLzEuMS9EVEQvc3ZnMTEuZHRkIj4KDTwhLS0gVXBsb2FkZWQgdG86IFNWRyBSZXBvLCB3d3cuc3ZncmVwby5jb20sIFRyYW5zZm9ybWVkIGJ5OiBTVkcgUmVwbyBNaXhlciBUb29scyAtLT4KPHN2ZyBmaWxsPSIjZmZmZmZmIiBoZWlnaHQ9IjgwMHB4IiB3aWR0aD0iODAwcHgiIHZlcnNpb249IjEuMSIgaWQ9IkNhcGFfMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iMCAwIDI3Ni43MTUgMjc2LjcxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSIgc3Ryb2tlPSIjZmZmZmZmIj4KDTxnIGlkPSJTVkdSZXBvX2JnQ2FycmllciIgc3Ryb2tlLXdpZHRoPSIwIi8+Cg08ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz4KDTxnIGlkPSJTVkdSZXBvX2ljb25DYXJyaWVyIj4gPGc+IDxwYXRoIGQ9Ik0xMzguMzU3LDBDNjIuMDY2LDAsMCw2Mi4wNjYsMCwxMzguMzU3czYyLjA2NiwxMzguMzU3LDEzOC4zNTcsMTM4LjM1N3MxMzguMzU3LTYyLjA2NiwxMzguMzU3LTEzOC4zNTcgUzIxNC42NDgsMCwxMzguMzU3LDB6IE0xMzguMzU3LDI1OC43MTVDNzEuOTkyLDI1OC43MTUsMTgsMjA0LjcyMywxOCwxMzguMzU3UzcxLjk5MiwxOCwxMzguMzU3LDE4IHMxMjAuMzU3LDUzLjk5MiwxMjAuMzU3LDEyMC4zNTdTMjA0LjcyMywyNTguNzE1LDEzOC4zNTcsMjU4LjcxNXoiLz4gPHBhdGggZD0iTTE5NC43OTgsMTYwLjkwM2MtNC4xODgtMi42NzctOS43NTMtMS40NTQtMTIuNDMyLDIuNzMyYy04LjY5NCwxMy41OTMtMjMuNTAzLDIxLjcwOC0zOS42MTQsMjEuNzA4IGMtMjUuOTA4LDAtNDYuOTg1LTIxLjA3OC00Ni45ODUtNDYuOTg2czIxLjA3Ny00Ni45ODYsNDYuOTg1LTQ2Ljk4NmMxNS42MzMsMCwzMC4yLDcuNzQ3LDM4Ljk2OCwyMC43MjMgYzIuNzgyLDQuMTE3LDguMzc1LDUuMjAxLDEyLjQ5NiwyLjQxOGM0LjExOC0yLjc4Miw1LjIwMS04LjM3NywyLjQxOC0xMi40OTZjLTEyLjExOC0xNy45MzctMzIuMjYyLTI4LjY0NS01My44ODItMjguNjQ1IGMtMzUuODMzLDAtNjQuOTg1LDI5LjE1Mi02NC45ODUsNjQuOTg2czI5LjE1Miw2NC45ODYsNjQuOTg1LDY0Ljk4NmMyMi4yODEsMCw0Mi43NTktMTEuMjE4LDU0Ljc3OC0zMC4wMDkgQzIwMC4yMDgsMTY5LjE0NywxOTguOTg1LDE2My41ODIsMTk0Ljc5OCwxNjAuOTAzeiIvPiA8L2c+IDwvZz4KDTwvc3ZnPg==" height="22">

</div>

## Overview

**Tokit** is a blazing fast parser combinator library for Rust that uniquely combines:

- **Parse-While-Lexing Architecture**: Zero-copy streaming - parsers consume tokens directly from the lexer without buffering, eliminating allocation overhead
- **Deterministic LALR-Style Parsing**: Explicit lookahead with compile-time buffer capacity, no hidden backtracking
- **Flexible Error Handling**: Same parser code adapts for fail-fast runtime or greedy compiler diagnostics via the `Emitter` trait

Unlike traditional parser combinators that buffer tokens and rely on implicit backtracking, Tokit streams tokens on-demand with predictable, deterministic decisions. This makes it ideal for building high-performance language tooling, DSL parsers, compilers, and REPLs that need both speed and comprehensive error reporting.

### Key Features

- **Parse-While-Lexing**: Zero-copy streaming architecture - no token buffering, no extra allocations
- **No Hidden Backtracking**: Explicit, predictable parsing with lookahead-based decisions instead of implicit backtracking
- **Deterministic + Composable**: Combines the flexibility of parser combinators with LALR-style deterministic table parsing
- **Flexible Error Handling Architecture**: Designed to support both fail-fast parsing (runtime) and greedy parsing (compiler diagnostics) by swapping the `Emitter` type - same parser, different behavior
- **Token-Based Parsing**: Works directly on token streams from any lexer implementing the `Lexer<'inp>` trait
- **Composable Combinators**: Build complex parsers from simple, reusable building blocks
- **Flexible Error Handling**: Configurable error emission strategies (`Fatal`, `Silent`, `Ignored`)
- **Rich Error Recovery**: Built-in `recover()` and `inplace_recover()` combinators for resilient parsing with backtracking or synchronization
- **Zero-Cost Abstractions**: All configuration resolved at compile time
- **No-std Support**: Core functionality works without allocator
- **Multiple Source Types**: Support for `str`, `[u8]`, `Bytes`, `BStr`, `HipStr`
- **Logos Integration**: Optional `LogosLexer` adapter for seamless [logos](https://github.com/maciejhirsz/logos) integration
- **CST Support**: Optional Concrete Syntax Tree support via [rowan](https://github.com/rust-analyzer/rowan)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tokit = "0.0.0"
```

### Feature Flags

- `std` (default) - Enable standard library support
- `alloc` - Enable allocator support for no-std environments
- `logos` - Enable `LogosLexer` adapter for Logos integration
- `rowan` - Enable CST (Concrete Syntax Tree) support with rowan integration
- `bytes` - Support for `bytes::Bytes` as token source
- `bstr` - Support for `bstr::BStr` as token source
- `hipstr` - Support for `hipstr::HipStr` as token source
- `among` - Enable `Among<L, M, R>` parseable support
- `smallvec` - Enable small vector optimization utilities

## Core Components

### Lexer Layer

- **`Lexer<'inp>` Trait**

  Core trait for lexers that produce token streams. Implement this to use any lexer with Tokit.

- **`Token<'a>` Trait**

  Defines token types with:
  - `Kind`: Token kind discriminator
  - `Error`: Associated error type

- **`LogosLexer<'inp, T, L>`** (feature: `logos`)

  Ready-to-use adapter for integrating [Logos](https://github.com/maciejhirsz/logos) lexers.

### Error Handling

Tokit's flexible `Emitter` system allows the same parser to adapt to different use cases by simply changing the error handling strategy.

#### Atomically Composable Trait Design

Tokit's emitter system uses **atomically composable traits** - small, focused traits that each handle a specific parsing scenario. Instead of one monolithic interface, error handling is broken down into atomic building blocks:

- **Core**: `Emitter` - Base error handling (lexer errors, unexpected tokens)
- **Repetition**: `TooFewEmitter`, `TooManyEmitter`, `FullContainerEmitter`
- **Separation**: `SeparatedEmitter`, `UnexpectedLeadingSeparatorEmitter`, `UnexpectedTrailingSeparatorEmitter`
- **Delimiters**: `DelimitedEmitter`

This design provides:

- **Fine-grained control**: Implement only the traits you need
- **Composability**: Mix and match traits to build custom strategies
- **Extensibility**: Create specialized emitters for specific use cases

#### Built-in Emitter Strategies

Tokit provides complete implementations that implement all atomic traits with consistent behavior:

- `Fatal` - **Fail-fast parsing**: Stop on first error (default) - perfect for runtime parsing and REPLs
- `Verbose` - **Comprehensive error collection**: Collect all errors and continue parsing - perfect for compiler diagnostics and IDEs
- `Silent` - Silently ignore errors
- `Ignored` - Ignore errors completely

**Key Design**: Change the `Emitter` type to switch between fail-fast runtime parsing and comprehensive compiler diagnostics - **same parser code, different behavior**. This makes Tokit suitable for both:

- **Runtime/REPL**: Fast feedback with `Fatal` emitter
- **Compiler/IDE**: Comprehensive diagnostics with `Verbose` emitter

**The `Verbose` Emitter**: Unlike `Fatal` which stops at the first error, the `Verbose` emitter collects all errors during parsing and continues where possible. Errors are stored in a `BTreeMap` indexed by span, automatically sorted by their position in the source code. After parsing, retrieve all errors via the `errors()` method for display, analysis, or further processing. This is ideal for:

- Showing users all issues at once in compiler output
- Providing real-time diagnostics in IDE error panels
- Collecting comprehensive error information for debugging
- Generating detailed error reports for language servers

**Custom Emitters**: Thanks to the atomically composable trait design, you can create custom error handling strategies by implementing only the traits you need. You compose small, focused traits to build exactly the behavior you want. For example, you could build an emitter that:

- Implements only `Emitter` + `TooFewEmitter` for a parser that only needs those scenarios
- Limits the maximum number of errors before stopping
- Filters errors by severity level
- Sends errors to a logging system or telemetry service
- Implements domain-specific error recovery strategies for specific error types
- Wraps an existing emitter and adds custom behavior for certain atomic traits

- **Rich Error Types** (in `error/` module)
  - Token-level: `UnexpectedToken`, `MissingToken`, `UnexpectedEot`
  - Syntax-level: `Unclosed`, `Unterminated`, `Malformed`, `Invalid`
  - Escape sequences: `HexEscape`, `UnicodeEscape`
  - All errors include span tracking

### Error Recovery

Tokit provides built-in parser combinators for resilient parsing that can continue after errors:

#### Recovery Strategies

- **`recover(recovery_parser)`** - Error recovery with backtracking
  - If primary parser fails, **resets to starting position** and tries recovery parser
  - Use for: Alternative interpretations, fallback values, error nodes
  - Example: `parse_expr().recover(parse_error_node())`

- **`inplace_recover(recovery_parser)`** - Error recovery without backtracking
  - If primary parser fails, **continues from error position** with recovery parser
  - Use for: Panic mode recovery, resynchronization, skipping to safe points
  - Example: `parse_stmt().inplace_recover(skip_to_semicolon())`

#### Recovery Patterns

**Alternative Parsing** (with backtracking):

```rust,ignore
// Try parsing as function, fall back to error item
let parser = parse_function()
    .recover(parse_error_item());

// Input with error → recovers gracefully
```

**Synchronization Points** (without backtracking):

```rust,ignore
// Parse statement, skip to semicolon on error
let parser = parse_statement()
    .inplace_recover(
        skip_until(|tok| matches!(tok, Token::Semicolon))
            .then_ignore(any())
            .map(|_| Statement::Error)
    );

// Continues parsing from next statement
```

**Comprehensive Error Collection**:

```rust,ignore
// Use with Verbose emitter to collect all errors
let emitter = Verbose::new();
let items = many(
    parse_item().recover(parse_error_item())
);

// After parsing, retrieve all errors:
for (span, error) in emitter.errors() {
    eprintln!("Error at {:?}: {}", span, error);
}
```

Error recovery works seamlessly with the atomically composable emitter system - combine `Verbose` emitter with recovery combinators to build robust parsers that report all issues in a single pass.

### Utilities

- **Span Tracking**
  - `Span` - Lightweight span representation
  - `Spanned<T>` - Wrap value with span
  - `Located<T>` - Wrap value with span and source slice
  - `Sliced<T>` - Wrap value with source slice

- **Parser Configuration**
  - `Parser<F, L, O, Error, Context>` - Configurable parser
  - `ParseContext` - Context for emitter and cache
  - `Window` - Type-level peek buffer capacity for deterministic lookahead
  - **Note**: Lookahead windows support 1-32 token capacity via `typenum::{U1..U32}`

## Examples

Check out the examples directory:

```bash
# JSON token parsing with map combinators
cargo run --example json
```

## Architecture

Tokit's architecture follows a layered design:

1. **Lexer Layer** - Token production and source abstraction
2. **Parser Layer** - Composable parser combinators
3. **Error Layer** - Rich error types and emission strategies
4. **Utility Layer** - Spans, containers, and helpers

This separation enables:

- Use any lexer by implementing `Lexer<'inp>`
- Mix and match parser combinators
- Customize error handling per-parser or globally
- Zero-cost abstractions through compile-time configuration

## Design Philosophy

### Parse-While-Lexing: Zero-Copy Streaming

Tokit uses a **parse-while-lexing** architecture where parsers consume tokens directly from the lexer as needed, without intermediate buffering:

**Traditional Approach (Two-Phase):**

```text
Source → Lexer → [Token Buffer] → Parser
         ↓
    Allocate Vec<Token>  ← Extra allocation!
```

**Tokit Approach (Streaming):**

```text
Source → Lexer ←→ Parser
         ↑________↓
    Zero-copy streaming, no buffer
```

**Benefits:**

- ✅ **Zero Extra Allocations**: No token buffer, tokens consumed on-demand
- ✅ **Lower Memory Footprint**: Only lookahead window buffered on stack, not entire token stream
- ✅ **Better Cache Locality**: Tokens processed immediately after lexing
- ✅ **Predictable Performance**: No large allocations, deterministic memory usage

### No Hidden Backtracking

Unlike traditional parser combinators that rely on implicit backtracking (trying alternatives until one succeeds), Tokit uses **explicit lookahead-based decisions**. This design choice provides:

- **Predictable Performance**: No hidden exponential backtracking scenarios
- **Explicit Control**: Developers decide when and where to peek ahead via `peek_then()` and `peek_then_choice()`
- **Deterministic Parsing**: LALR-style table-driven decisions using fixed-capacity lookahead windows (`Window` trait)
- **Better Error Messages**: Failed alternatives don't hide earlier, more relevant errors

```rust,ignore
// Traditional parser combinator (hidden backtracking):
// try_parser1.or(try_parser2).or(try_parser3)  // May backtrack!

// Tokit approach (explicit lookahead, no backtracking):
let parser = any()
    .peek_then::<_, typenum::U2>(|peeked, _| {
        match peeked.front() {
            ...
        }
    });
```

### Parser Combinators + Deterministic Table Parsing

Tokit uniquely combines:

- **Parser Combinator Flexibility**: Compose small parsers into complex grammars
- **LALR-Style Determinism**: Fixed lookahead windows with deterministic decisions
- **Type-Level Capacity**: Lookahead buffer size known at compile time (`Window::CAPACITY`)

This hybrid approach gives you composable abstractions without sacrificing performance or predictability.

### Atomically Composable Error Handling

Tokit's error handling system breaks down error scenarios into small, focused traits. Each trait handles one specific parsing situation (like `TooFewEmitter` for "too few elements" or `UnexpectedLeadingSeparatorEmitter` for leading separators).

**Benefits of the Atomically Composable Trait Design:**

- ✅ **Implement only what you need**: Your parser only uses `TooFewEmitter`? Just implement that trait
- ✅ **Compose custom strategies**: Mix and match atomic traits to build specialized error handlers
- ✅ **Pre-built bundles**: `Fatal`, `Verbose`, and `Silent` implement all traits for convenience
- ✅ **Fine-grained control**: Small, reusable pieces that compose into complex behavior

This is fundamentally different from traditional monolithic error handler interfaces - you get both the simplicity of pre-built strategies and the flexibility to implement only what you need.

### Fail-Fast Runtime ↔ Comprehensive Compiler Diagnostics

Tokit's architecture decouples parsing logic from error handling strategy through the atomic `Emitter` trait system. This means:

**Same Parser, Different Contexts:**

- **Runtime/REPL Mode**: Use `Fatal` emitter → stop on first error for immediate feedback
- **Compiler/IDE Mode**: Use `Verbose` emitter → collect all errors for comprehensive diagnostics
- **Testing/Fuzzing**: Use `Ignored` emitter → parse through all errors for robustness testing

**Benefits:**

- ✅ Write parsers once, deploy everywhere
- ✅ No separate "error recovery mode" - it's just a different emitter
- ✅ Custom emitters can implement domain-specific error handling
- ✅ Zero-cost abstraction - emitter behavior resolved at compile time

### Inspirations

Tokit takes inspiration from:

- [**winnow**](https://github.com/winnow-rs/winnow) - For ergonomic parser API design
- [**chumsky**](https://github.com/zesterer/chumsky) - For composable parser combinator patterns
- [**logos**](https://github.com/maciejhirsz/logos) - For high-performance lexing
- [**rowan**](https://github.com/rust-analyzer/rowan) - For lossless syntax tree representation

### Core Priorities

1. **Performance** - Parse-while-lexing (zero-copy streaming), zero-cost abstractions, no hidden allocations
2. **Predictability** - No hidden backtracking, explicit control flow, deterministic decisions
3. **Composability** - Small parsers combine into complex ones; atomic emitter traits compose into custom strategies
4. **Versatility** - Same parser works for runtime (fail-fast) and compiler diagnostics (comprehensive) via atomic `Emitter` traits
5. **Flexibility** - Work with any lexer, atomic error handling traits, support both AST and CST
6. **Correctness** - Rich error types, span tracking, validation

## Who Uses Tokit?

- [`smear`](https://github.com/al8n/smear): Blazing fast, fully spec-compliant, reusable parser combinators for standard GraphQL and GraphQL-like DSLs

## License

`tokit` is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

You may choose either license for your purposes.

Copyright (c) 2026 Al Liu.

[Github-url]: https://github.com/al8n/tokit/
[CI-url]: https://github.com/al8n/tokit/actions/workflows/ci.yml
[doc-url]: https://docs.rs/tokit
[crates-url]: https://crates.io/crates/tokit
[codecov-url]: https://app.codecov.io/gh/al8n/tokit/
