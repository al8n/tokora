<div align="center">
<h1>Tokora</h1>
</div>
<div align="center">

Deterministic parser combinators, with on-demand lexing, LALR-style dispatch, explicit backtracking, and configurable diagnostics.

<!-- <img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Ftokora" height="22"> -->

[<img alt="github" src="https://img.shields.io/badge/github-al8n/tokora-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/al8n/tokora/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/tokora?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-tokora-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">][doc-url]
[<img alt="crates.io" src="https://img.shields.io/crates/v/tokora?style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0yNTYsMEwzMS41MjgsMTEyLjIzNnYyODcuNTI4TDI1Niw1MTJsMjI0LjQ3Mi0xMTIuMjM2VjExMi4yMzZMMjU2LDB6IE0yMzQuMjc3LDQ1Mi41NjRMNzQuOTc0LDM3Mi45MTNWMTYwLjgxDQoJCQlsMTU5LjMwMyw3OS42NTFWNDUyLjU2NHogTTEwMS44MjYsMTI1LjY2MkwyNTYsNDguNTc2bDE1NC4xNzQsNzcuMDg3TDI1NiwyMDIuNzQ5TDEwMS44MjYsMTI1LjY2MnogTTQzNy4wMjYsMzcyLjkxMw0KCQkJbC0xNTkuMzAzLDc5LjY1MVYyNDAuNDYxbDE1OS4zMDMtNzkuNjUxVjM3Mi45MTN6IiBmaWxsPSIjRkZGIi8+DQoJPC9nPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPC9zdmc+DQo=" height="22">][crates-url]
[<img alt="crates.io" src="https://img.shields.io/crates/d/tokora?color=critical&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/PjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+PHN2ZyB0PSIxNjQ1MTE3MzMyOTU5IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjM0MjEiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkzIiB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIj48ZGVmcz48c3R5bGUgdHlwZT0idGV4dC9jc3MiPjwvc3R5bGU+PC9kZWZzPjxwYXRoIGQ9Ik00NjkuMzEyIDU3MC4yNHYtMjU2aDg1LjM3NnYyNTZoMTI4TDUxMiA3NTYuMjg4IDM0MS4zMTIgNTcwLjI0aDEyOHpNMTAyNCA2NDAuMTI4QzEwMjQgNzgyLjkxMiA5MTkuODcyIDg5NiA3ODcuNjQ4IDg5NmgtNTEyQzEyMy45MDQgODk2IDAgNzYxLjYgMCA1OTcuNTA0IDAgNDUxLjk2OCA5NC42NTYgMzMxLjUyIDIyNi40MzIgMzAyLjk3NiAyODQuMTYgMTk1LjQ1NiAzOTEuODA4IDEyOCA1MTIgMTI4YzE1Mi4zMiAwIDI4Mi4xMTIgMTA4LjQxNiAzMjMuMzkyIDI2MS4xMkM5NDEuODg4IDQxMy40NCAxMDI0IDUxOS4wNCAxMDI0IDY0MC4xOTJ6IG0tMjU5LjItMjA1LjMxMmMtMjQuNDQ4LTEyOS4wMjQtMTI4Ljg5Ni0yMjIuNzItMjUyLjgtMjIyLjcyLTk3LjI4IDAtMTgzLjA0IDU3LjM0NC0yMjQuNjQgMTQ3LjQ1NmwtOS4yOCAyMC4yMjQtMjAuOTI4IDIuOTQ0Yy0xMDMuMzYgMTQuNC0xNzguMzY4IDEwNC4zMi0xNzguMzY4IDIxNC43MiAwIDExNy45NTIgODguODMyIDIxNC40IDE5Ni45MjggMjE0LjRoNTEyYzg4LjMyIDAgMTU3LjUwNC03NS4xMzYgMTU3LjUwNC0xNzEuNzEyIDAtODguMDY0LTY1LjkyLTE2NC45MjgtMTQ0Ljk2LTE3MS43NzZsLTI5LjUwNC0yLjU2LTUuODg4LTMwLjk3NnoiIGZpbGw9IiNmZmZmZmYiIHAtaWQ9IjM0MjIiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkwIiBjbGFzcz0iIj48L3BhdGg+PC9zdmc+&style=for-the-badge" height="22">][crates-url]

[<img alt="book" src="https://img.shields.io/badge/book-tokora-e5928d?style=for-the-badge&logo=mdbook" height="22">][tutorial-url]
[<img alt="Discord" src="https://img.shields.io/discord/835936528140206122?style=for-the-badge&logo=discord&logoColor=white&label=Discord&color=7289da" height="22">][discord]
<img alt="license" src="https://img.shields.io/badge/License-Apache%202.0/MIT-blue.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PCFET0NUWVBFIHN2ZyBQVUJMSUMgIi0vL1czQy8vRFREIFNWRyAxLjEvL0VOIiAiaHR0cDovL3d3dy53My5vcmcvR3JhcGhpY3MvU1ZHLzEuMS9EVEQvc3ZnMTEuZHRkIj4KDTwhLS0gVXBsb2FkZWQgdG86IFNWRyBSZXBvLCB3d3cuc3ZncmVwby5jb20sIFRyYW5zZm9ybWVkIGJ5OiBTVkcgUmVwbyBNaXhlciBUb29scyAtLT4KPHN2ZyBmaWxsPSIjZmZmZmZmIiBoZWlnaHQ9IjgwMHB4IiB3aWR0aD0iODAwcHgiIHZlcnNpb249IjEuMSIgaWQ9IkNhcGFfMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iMCAwIDI3Ni43MTUgMjc2LjcxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSIgc3Ryb2tlPSIjZmZmZmZmIj4KDTxnIGlkPSJTVkdSZXBvX2JnQ2FycmllciIgc3Ryb2tlLXdpZHRoPSIwIi8+Cg08ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz4KDTxnIGlkPSJTVkdSZXBvX2ljb25DYXJyaWVyIj4gPGc+IDxwYXRoIGQ9Ik0xMzguMzU3LDBDNjIuMDY2LDAsMCw2Mi4wNjYsMCwxMzguMzU3czYyLjA2NiwxMzguMzU3LDEzOC4zNTcsMTM4LjM1N3MxMzguMzU3LTYyLjA2NiwxMzguMzU3LTEzOC4zNTcgUzIxNC42NDgsMCwxMzguMzU3LDB6IE0xMzguMzU3LDI1OC43MTVDNzEuOTkyLDI1OC43MTUsMTgsMjA0LjcyMywxOCwxMzguMzU3UzcxLjk5MiwxOCwxMzguMzU3LDE4IHMxMjAuMzU3LDUzLjk5MiwxMjAuMzU3LDEyMC4zNTdTMjA0LjcyMywyNTguNzE1LDEzOC4zNTcsMjU4LjcxNXoiLz4gPHBhdGggZD0iTTE5NC43OTgsMTYwLjkwM2MtNC4xODgtMi42NzctOS43NTMtMS40NTQtMTIuNDMyLDIuNzMyYy04LjY5NCwxMy41OTMtMjMuNTAzLDIxLjcwOC0zOS42MTQsMjEuNzA4IGMtMjUuOTA4LDAtNDYuOTg1LTIxLjA3OC00Ni45ODUtNDYuOTg2czIxLjA3Ny00Ni45ODYsNDYuOTg1LTQ2Ljk4NmMxNS42MzMsMCwzMC4yLDcuNzQ3LDM4Ljk2OCwyMC43MjMgYzIuNzgyLDQuMTE3LDguMzc1LDUuMjAxLDEyLjQ5NiwyLjQxOGM0LjExOC0yLjc4Miw1LjIwMS04LjM3NywyLjQxOC0xMi40OTZjLTEyLjExOC0xNy45MzctMzIuMjYyLTI4LjY0NS01My44ODItMjguNjQ1IGMtMzUuODMzLDAtNjQuOTg1LDI5LjE1Mi02NC45ODUsNjQuOTg2czI5LjE1Miw2NC45ODYsNjQuOTg1LDY0Ljk4NmMyMi4yODEsMCw0Mi43NTktMTEuMjE4LDU0Ljc3OC0zMC4wMDkgQzIwMC4yMDgsMTY5LjE0NywxOTguOTg1LDE2My41ODIsMTk0Ljc5OCwxNjAuOTAzeiIvPiA8L2c+IDwvZz4KDTwvc3ZnPg==" height="22">

</div>

## Introduction

Tokora is a parser-combinator library for Rust that lexes while it parses. A grammar is ordinary
Rust: parser functions and combinators over an input handle that pulls each token from your lexer
only when the parser asks for it. Choices are deterministic: a fixed lookahead window or a table of
token kinds selects exactly one branch. Backtracking is explicit and transactional, so an abandoned
branch leaves neither consumed input nor diagnostics behind. The same parser functions fail fast in
a runtime or collect every diagnostic for a compiler or an editor, depending on the emitter they
run with. Tokora builds for `no_std` without an allocator, lexes through a bundled Logos adapter or
any lexer you write, and can build lossless syntax trees on Rowan.

## Install

```toml
[dependencies]
tokora = { version = "0.12", features = ["logos"] }
```

The default features are `std` and the `combinators` umbrella. `logos` adds `LogosLexer`, the
adapter for [Logos](https://github.com/maciejhirsz/logos)-derived token enums, and re-exports Logos
as `tokora::logos` (pinned: `tokora::logos_0_16`), so a derive can name it with
`#[logos(crate = tokora::logos)]`. [Features](#features) lists every flag, and
[Platform support](#platform-support) covers `no_std`.

## Quick start

A complete program: a Logos token enum, the `Token` impl that connects it to Tokora, and a parser
written as a plain function. It needs the `logos` feature.

```rust,ignore
use tokora::{
  Emitter, InputRef, ParseContext, Token,
  lexer::LogosLexer,
  logos::{self, Logos},
  parse,
};

// The token vocabulary. It carries no payloads, so it serves as its own kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Logos)]
#[logos(crate = logos, skip r"[ \t\r\n]+")]
enum Tok {
  #[regex(r"[0-9]+")]
  Int,
  #[token("+")]
  Plus,
}

// Diagnostics print a kind through `Display`.
impl core::fmt::Display for Tok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(match self {
      Tok::Int => "integer",
      Tok::Plus => "`+`",
    })
  }
}

impl Token<'_> for Tok {
  type Kind = Tok;
  type Error = (); // the Logos lexer's default error
  // A claim about how far past a token the lexer reads; only partial input consults it.
  const SCAN_LOOKAHEAD: tokora::ScanLookahead = tokora::ScanLookahead::Unbounded;

  fn kind(&self) -> Tok {
    *self
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

type SumLexer<'a> = LogosLexer<'a, Tok>;

// A parser is a function over `InputRef`. It is generic over the parse context, so the caller
// picks the emitter; `()` is the error type to keep the example short.
fn sum<'inp, Ctx>(inp: &mut InputRef<'inp, '_, SumLexer<'inp>, Ctx>) -> Result<u64, ()>
where
  Ctx: ParseContext<'inp, SumLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, SumLexer<'inp>, Error = ()>,
{
  let mut total = 0;
  loop {
    // `try_expect` consumes the next token only if it matches; `slice` borrows its text.
    if inp.try_expect(|tok| matches!(tok.data(), Tok::Int))?.is_none() {
      return Err(());
    }
    total += inp.slice().parse::<u64>().map_err(|_| ())?;
    if inp.try_expect(|tok| matches!(tok.data(), Tok::Plus))?.is_none() {
      break;
    }
  }
  // `next` pulls one more token: anything after the sum is an error.
  match inp.next()? {
    None => Ok(total),
    Some(_) => Err(()),
  }
}

let total: Result<u64, ()> = parse(sum, "1 + 2 + 39");
assert_eq!(total, Ok(42));

let unfinished: Result<u64, ()> = parse(sum, "1 +");
assert_eq!(unfinished, Err(()));
```

`parse` runs a parser with the fail-fast `Fatal` emitter. `parse_with` takes a context you supply,
such as a collecting emitter or a configured cache, and `Parser` configures one before the source
exists. A real grammar defines its own error type and implements `From` for the structured errors
it keeps; [chapter 2 of the guide](https://al8n.github.io/tokora/ch02_parsers.html) lists them, and
the guide's tutorial builds a complete calculator from the same pieces.

## What makes Tokora different

- **Parse while lexing.** There is no token vector: tokens are lexed as the parser asks for them,
  and a bounded cache holds only what lookahead or backtracking needs, with a lookahead window of
  1 to 32 tokens fixed at compile time.
- **Deterministic choice with exact expectations.** A kind dispatch that matches nothing reports
  its whole table as "expected one of …".
- **Transactional backtracking.** Speculation is opt-in and visible in the code, and a rollback
  undoes both the input the abandoned branch consumed and the diagnostics it emitted.
- **One grammar, any error policy.** Parsers are generic over their parse context, and so over its
  emitter: fail fast, collect everything, stay silent, or implement only the emitter capabilities a
  grammar uses.
- **Recovery and partial input.** Recovery is explicit, with nesting-aware resynchronization, and a
  parse over a buffer that is still growing reports `Incomplete` for an unfinished construct rather
  than an error.
- **Bounded recursion.** Nesting past a configurable budget fails with a catchable error instead of
  overflowing the stack.
- **Pratt expressions** through two drivers, token-level and AST-level, that share one precedence
  vocabulary.
- **Lossless syntax trees** (feature `rowan`), built over the same backtracking, so a rollback also
  rewinds the half-built tree.
- **`no_std` without an allocator**, down to core-only parsing over bounded caches and containers.
- **Tooling**: a conformance kit for custom lexers, a deterministic fuzz harness for the
  backtracking machinery, and parse tracing that compiles away when it is off.

## How Tokora parses

A grammar is ordinary Rust. Parser functions and combinators read through `InputRef`, which drives
the `Lexer` and stages tokens in its cache when lookahead or backtracking needs them.
`peek_then_choice` decides from a fixed lookahead window; `dispatch_on_kind` and
`fused_dispatch_on_kind` route the next token's `Token::Kind` to exactly one branch, and the fused
form hands the token it classified straight to that branch instead of round-tripping it through
the cache.

That token-kind dispatch is what "LALR-style" means here, and it is local to a hand-written
combinator grammar: Tokora does not accept an LALR grammar, generate LALR parse tables, or act as a
parser generator.

Speculation is explicit. `InputRef::attempt` and `InputRef::try_attempt` keep a successful branch
and roll back a declined or failed one, and `input::Transaction` exposes commit and rollback
directly. A rollback restores the input position, span, lexer state, token cache, and the
diagnostics emitted since the checkpoint. Side effects in your own code need their own transaction
boundary.

## Diagnostics and recovery

Parsers are generic over their parse context, which carries the emitter. `parse` and
`Parser::new()` use `Fatal`, which turns the first diagnostic into the parse's error.
`emitter::Verbose` (`std` or `alloc`) records every diagnostic together with its open `labelled`
context, and lets a recovering grammar continue; `Silent` and `Ignored` discard diagnostics,
keeping or erasing the error type. Structured lexer, token, separator, container, and Pratt errors
convert into the application's error type through `From` implementations.

Recovery is explicit. `recover` rewinds to where the failed parser started and runs a recovery
parser; `inplace_recover` continues from the failure position. `sync_balanced` and
`skip_then_retry` skip to a synchronization point without crossing unbalanced delimiters, and
`Verbose` records each non-empty skipped region once, alongside the other diagnostics. An
`Incomplete` error is re-raised rather than recovered, so unfinished partial input is never
discarded.

## Recursion budget

Every `InputRef::descend` or `InputRef::descending` a grammar takes, and every frame of Tokora's
two Pratt engines, draws on one shared budget. Input nested past it fails the parse with a
catchable `error::RecursionLimitReached` instead of exhausting the native stack.

The default, `RecursionLimiter::PARSE_DEFAULT_DEPTH`, is **32 in every build profile**. A release
frame is roughly an order of magnitude cheaper than a debug one, but Tokora cannot see how the
frames it bounds were compiled: `debug_assertions` is not `opt-level`, and it is per crate, while
the recursion is spent in your own productions. Both profiles are therefore priced at the debug
frame cost, and no profile combination can abort. The depth a fully optimised parse supports is
published as `RecursionLimiter::OPTIMIZED_PARSE_DEPTH`; nothing installs it, and taking it is one
call:

```rust,ignore
use tokora::state::RecursionLimiter;

// The default budget, the same in every build profile.
assert_eq!(RecursionLimiter::PARSE_DEFAULT_DEPTH, 32);

// The depth a fully optimised parse supports. Nothing installs it; a caller does.
assert_eq!(RecursionLimiter::OPTIMIZED_PARSE_DEPTH, 256);

let limiter = RecursionLimiter::with_limitation(RecursionLimiter::OPTIMIZED_PARSE_DEPTH);
assert_eq!(limiter.limitation(), 256);
```

Hand the limiter to `ParserContext::with_recursion_limiter`, or to
`input::InputContext::with_recursion_limiter` for a driver that builds its own input context. Pass
256 only when every frame the budget bounds, Tokora's and your own productions alike, is compiled
at `opt-level = 3`; under a per-package profile override, or a debug consumer against a release
Tokora, the number that applies is the default. `RecursionLimiter::unlimited()` opts out for a
grammar whose depth is bounded by something other than its input. With the `stacker` feature,
`RecursionLimiter::SEGMENTED_PRATT_DEPTH` (**1024**) is the budget for a descent made entirely of
Pratt frames; [What `stacker` segments](#what-stacker-segments) explains why it is a separate
constant.

## Module map

The crate root holds exactly the names a production needs in scope: the parsing vocabulary
(`ParseInput`, `TryParseInput`, `ParseAttempt`, `Accumulator`, `ParseChoice`, `ParseTokenChoice`,
`Action`), the parse context (`ParseContext`, `ComposableParseContext`, `ParserContext`,
`FatalContext`, `ErrorOf`), the drivers (`Parse`, `Parser`, `parse`, `parse_with`,
`parse_with_state`, `parse_partial`), the input and lexer vocabulary (`InputRef`, `Lexer`,
`ScanLookahead`, `ReadFrontier`, `Token`, `Source`, `Slice`, `Span`, `SimpleSpan`), and the emitter
(`Emitter`, `EmitterView`). Beside them sit the `attempt!`, `keyword!`, `punctuator!`, `select!`,
`syntax_kinds!` and `try_select!` macros, and the `logos` / `logos_0_16` re-exports. Each of those
names also lives in the module of its concept, and everything else is reached through its module.
There is no prelude: a file imports what it uses.

| Module | Holds |
| --- | --- |
| [`parse`](https://docs.rs/tokora/latest/tokora/parse/) | The parsing traits and the types their contracts name, the parse-context vocabulary, and the drivers. |
| [`combinator`](https://docs.rs/tokora/latest/tokora/combinator/) | Combinator constructors, the parser types they return, and the values they take: `expect`, `delimited`, `node`, `labelled`, `Any`, `Recover`, and the rest. No trait lives here. |
| [`pratt`](https://docs.rs/tokora/latest/tokora/pratt/) | Pratt expression parsing: the typed `pratt` driver, binding powers, fold traits, and CST hooks. Feature `pratt`. |
| [`input`](https://docs.rs/tokora/latest/tokora/input/) | `InputRef`, transactions and savepoints, the `Complete` / `Partial` typestate, and partial-input sessions. |
| [`lexer`](https://docs.rs/tokora/latest/tokora/lexer/) | The `Lexer` contract, `Lexed`, and the `LogosLexer` adapter (feature `logos`). |
| [`token`](https://docs.rs/tokora/latest/tokora/token/) | The `Token` trait and token capability traits such as `KeywordToken` and `PrattToken`. |
| [`emitter`](https://docs.rs/tokora/latest/tokora/emitter/) | `Emitter`, its atomic capability traits, and the `Fatal`, `Verbose`, `Silent` and `Ignored` policies. |
| [`error`](https://docs.rs/tokora/latest/tokora/error/) | Span-carrying structured errors: unexpected tokens and ends, missing syntax, delimiters, malformed lexemes, recursion limits. |
| [`diagnostic`](https://docs.rs/tokora/latest/tokora/diagnostic/) | `Diagnose`, a renderer-neutral contract (code, severity, locations, labels, help) for an application's own errors. |
| [`cache`](https://docs.rs/tokora/latest/tokora/cache/) | Bounded token caches for lookahead and backtracking. |
| [`state`](https://docs.rs/tokora/latest/tokora/state/) | Lexer state and limits: `State`, `RecursionLimiter`, `TokenLimiter`. |
| [`span`](https://docs.rs/tokora/latest/tokora/span/) | `Span`, `SimpleSpan`, `Spanned`, `Located`. |
| [`source`](https://docs.rs/tokora/latest/tokora/source/) | `Source`, the input a lexer reads: `str`, `[u8]`, and the optional backends. |
| [`slice`](https://docs.rs/tokora/latest/tokora/slice/) | `Slice`, a zero-copy view of a source. |
| [`container`](https://docs.rs/tokora/latest/tokora/container/) | `Container`, the collection a repetition fills: `Vec`, `VecDeque`, `ArrayDeque`, `Option`, and the optional backends. |
| [`delimiter`](https://docs.rs/tokora/latest/tokora/delimiter/) | `Delimiter` and `TypedDelimiter` for open/close pairs, and `DelimiterKind`. |
| [`punct`](https://docs.rs/tokora/latest/tokora/punct/) | Punctuator markers such as `Comma` and `Semicolon`, and the `Punctuator` trait. |
| [`dialect`](https://docs.rs/tokora/latest/tokora/dialect/) | `Dialect`, one anchor type per language that names its lexer, so productions keep short `where` clauses. |
| [`types`](https://docs.rs/tokora/latest/tokora/types/) | Reusable AST building blocks: identifiers, keywords, literals, and recovery status. |
| [`syntax`](https://docs.rs/tokora/latest/tokora/syntax/) | `Syntax`, for syntax elements with a known set of components so every missing part can be reported, and the `AstNode` and `Language` traits. |
| [`cst`](https://docs.rs/tokora/latest/tokora/cst/) | Concrete syntax trees: the event vocabulary in every build, and the recording `Sink` and typed lossless trees with `rowan`. |
| [`utils`](https://docs.rs/tokora/latest/tokora/utils/) | Helpers: array deques and type-level numbers, display adapters, expected sets and messages. |
| [`conformance`](https://docs.rs/tokora/latest/tokora/conformance/) | Test kits for custom lexers, caches and emitters. Feature `conformance`. |
| [`fuzz`](https://docs.rs/tokora/latest/tokora/fuzz/) | A deterministic fuzz harness for the input and backtracking machinery. Feature `fuzz`. |
| [`guide`](https://docs.rs/tokora/latest/tokora/guide/) | The Tokora Guide, rendered by rustdoc. Features `std`, `logos` and `combinators`. |

## Guide and examples

The [Tokora Guide](https://al8n.github.io/tokora/) is a book in five parts, also published as the
`guide` module on docs.rs:

- **Part I** introduces Tokora and Calc, the small calculator language the tutorial builds.
- **Part II** is the tutorial, ten chapters that build Calc end to end: tokens and the lexer, first
  parsers, composition, deterministic choice, Pratt expressions, backtracking, diagnostics,
  recovery, partial input, and testing.
- **Part III** is the internals, for a reader who wants to know why an API has its shape: the
  parse-while-lexing engine, checkpoints and the LIFO rewind contract, the atomic emitter, the
  event-stream CST engine, and the `Source` / `Slice` storage backends.
- **Part IV** applies it: the anatomy of a real parser, a custom-lexer recipe, walkthroughs of the
  four examples below, and lossless CSTs with Rowan.
- **Part V** is the reference: combinators and atoms; errors, emitters and context; vocabulary,
  macros and feature flags; Pratt; types and syntax building blocks.

The examples are canonical complete programs, and the guide links to them rather than copying them
into prose.

| Program | Focus | Canonical source | Run |
| --- | --- | --- | --- |
| `calculator` | Token-level Pratt evaluator | [`calculator.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/calculator.rs) | `cargo run -p tokora --example calculator --features logos` |
| `s_expression` | Recursive descent and evaluation | [`s_expression.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/s_expression.rs) | `cargo run -p tokora --example s_expression --features logos` |
| `json` | Borrowed values, delimiters, and tentative choice | [`json.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/json.rs) | `cargo run -p tokora --example json --features logos` |
| `c_expression` | AST-level Pratt parsing with postfix forms | [`c_expression.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/c_expression.rs) | `cargo run -p tokora --example c_expression --features logos` |

Each has a lossless-CST twin (`calculator_cst`, `s_expression_cst`, `json_cst`,
`c_expression_cst`), and `expr_recovery` parses malformed input to completion, producing an AST and
a lossless tree whose holes are explicit error nodes. Those five need `--features logos,rowan`.
The book source lives under
[`tokora/src/guide`](https://github.com/al8n/tokora/tree/main/tokora/src/guide), and all nine
examples compile and run their tests with
`cargo test -p tokora --no-default-features --features std,logos,rowan,combinators --examples`.

## Features

An integration feature is the crate's own name plus its version, so a hyphenated crate keeps its
hyphen (`smol-bytes_0_2`). The unversioned name is an alias that selects the supported version, or
the newest where two are supported, and a versioned feature enables the matching optional
dependency.

### Library tiers

| Feature | Effect |
| --- | --- |
| `default` | Enables `std` and `combinators`. |
| `std` | Enables standard-library support and the default features of applicable dependencies. |
| `alloc` | Enables allocator-backed facilities without `std`, such as the `Verbose` emitter, `Vec`-collecting repetition, and stacked savepoints. Token caches stay bounded. |

### Combinator families

`combinators` turns on all thirteen families below. Each family is also its own feature, so a
`default-features = false` build compiles only the combinators its grammar calls. What the families
build on is unconditional: `Parser`, `Parse` and the `parse` drivers, the `ParseInput`,
`TryParseInput` and `ParseChoice` traits, and the substrate combinators (`expect`, `delimited`,
`recover`, `select`, `opt`, `padded`, `node`, `labelled`, and the rest).

| Feature | Effect |
| --- | --- |
| `combinators` | Umbrella for every combinator family below. On by default. |
| `any` | `Any`: accept one token of any kind, with its spanned, sliced and located shapes. |
| `fail` | `fail` / `fail_with`. |
| `filter` | `filter`, `filter_with`, `filter_map`, `filter_map_with`. |
| `fold` | The fold drivers (`fold_while`, `try_fold*`, `rfold*`); implies `many`. |
| `ident` | `Ident::parse` / `try_parse` and their `_except` twins. |
| `keyword` | `Keyword::parse` / `try_parse` and their `_exact` / `_sliced` twins. |
| `many` | The repetition family: `repeated*`, `separated*`, `delim*`, the delimiter handlers, the cardinality bounds, `list` and `separated1`. |
| `map` | `map` / `map_with`. |
| `peek` | `peek_then*`, `peek_then_choice`, `peek_kind`, `dispatch_on_kind` and its fused twin. |
| `pratt` | Pratt expressions: the `tokora::pratt` module with the typed `pratt` driver, the token-level `InputRef::pratt*`, `PrattToken`, and the `PrattEmitter` channel. |
| `punct` | The punctuator parsers (`Comma::parse`, …) and the `parens` / `braces` / `brackets` / `angles` delimited shapes built on them. |
| `then` | `then`, `then_ignore`, `ignore_then`, `then_value`, `and_then`, `and_then_with`. |
| `validate` | `validate` / `validate_with`. |

### Integrations

| Feature | Effect |
| --- | --- |
| `logos` | Alias for `logos_0_16`. |
| `logos_0_16` | Enables the `LogosLexer` adapter over `logos@0.16`, the only supported Logos major, and the `tokora::logos` / `tokora::logos_0_16` re-exports. |
| `rowan` | Enables the recording CST sink and the typed lossless tree over `rowan@0.17`; implies `std`, not `logos`. **Read the [safety disclosure](#rowan-and-known-upstream-undefined-behaviour) before enabling it.** |
| `bytes` | Alias for `bytes_1`. |
| `bytes_1` | Enables `bytes::Bytes` as a `Source` and `Slice`. |
| `bstr` | Alias for `bstr_1`. |
| `bstr_1` | Enables `bstr::BStr` as a `Source` and `Slice`. |
| `hipstr` | Alias for `hipstr_0_8`. |
| `hipstr_0_8` | Enables `HipStr` and `HipByt` from `hipstr@0.8` as sources and slices. |
| `smol-bytes` | Alias for `smol-bytes_0_2`. |
| `smol-bytes_0_1` | Enables the `smol-bytes@0.1` types (`shared::Bytes`, `compact::Bytes`, `Utf8Bytes`, `compact::Utf8Bytes`) as sources and slices; requires smol-bytes ≥ 0.1.2. |
| `smol-bytes_0_2` | Enables the same four types from `smol-bytes@0.2`. Both versions can be enabled together. |
| `smallvec` | Alias for `smallvec_1`. |
| `smallvec_1` | Enables `SmallVec` as a `Container`; implies `alloc`. |
| `heapless` | Alias for `heapless_0_9`. |
| `heapless_0_9` | Enables `heapless::Vec` and `heapless::Deque` as containers. |
| `tinyvec` | Alias for `tinyvec_1`. |
| `tinyvec_1` | Enables `ArrayVec` and `SliceVec` as containers, and `TinyVec` with `alloc`. |

### Tooling

| Feature | Effect |
| --- | --- |
| `stacker` | Runs each **Pratt frame prologue** on a fresh heap stack segment when the native stack is nearly exhausted; implies `std` and `pratt`. It does not raise the recursion budget; see [What `stacker` segments](#what-stacker-segments). |
| `trace` | Turns `combinator::traced` into a wrapper that prints an indented enter, exit, and rollback transcript to stderr, and instruments Tokora's own combinators; implies `std`. Without it, `traced` is the identity. |
| `unstable-raw` | Exposes the unstable raw checkpoint API: `InputRef::save`, `restore` and `commit`, and `Checkpoint` construction. |
| `conformance` | Enables the `tokora::conformance` test kits for custom lexers, caches and emitters; implies `alloc`. |
| `fuzz` | Enables the `tokora::fuzz` deterministic operation-script harness for the input and backtracking machinery; implies `std`. |

### What `stacker` segments

`stacker` puts a fresh heap stack segment under the frame prologue of Tokora's **two Pratt
engines**, and under nothing else. Your own `descend` / `descending` frames are ordinary native
frames and are untouched, so the feature does **not** move `RecursionLimiter::PARSE_DEFAULT_DEPTH`,
the [budget they share](#recursion-budget). `RecursionLimiter::SEGMENTED_PRATT_DEPTH` is the
larger figure it does justify, for a caller whose whole descent is Pratt frames to opt into.

It is not a substitute for the recursion budget: a segment is an `mmap`, so a deep enough input
still ends the process with nothing on any `Result` channel.

### `rowan` and known upstream undefined behaviour

A lossless sink requires a lexer that surfaces trivia (`Lexer::SURFACES_TRIVIA`). Tokora does not
re-export Rowan; add `rowan = "0.17"` directly when implementing `rowan::Language`.

## Platform support

Tokora's MSRV is Rust 1.95. The core supports allocator-free `no_std` (`no_std` without `alloc`)
and allocation-enabled `no_std` (`no_std` with `alloc`). Disable default features for core-only
parsing over bounded caches and containers, and enable `alloc` when a parser, container or selected
facility needs allocation. `rowan`, `stacker`, `trace` and `fuzz` require `std`; `conformance` and
`smallvec` require `alloc`.

Allocator-free `no_std`:

```toml
[dependencies]
tokora = { version = "0.12", default-features = false }
```

`no_std` with `alloc`:

```toml
[dependencies]
tokora = { version = "0.12", default-features = false, features = ["alloc"] }
```

Neither line enables a combinator family, because `combinators` is a default feature and both turn
the defaults off. Add `"combinators"` for the umbrella, or list the families the grammar calls, as
in `features = ["alloc", "many", "map"]`.

CI checks both `no_std` builds on `thumbv6m-none-eabi`, and builds the default features for Linux,
Android, Windows (GNU), and WebAssembly targets. `bytes`, `hipstr` and `smol-bytes` share buffers
through atomic compare-and-swap, so they do not build on a target without it, such as
`thumbv6m-none-eabi`; `bstr` does, and CI checks `alloc,bstr_1` there.

## Design notes

### Priorities

1. **Performance**: pull tokens from the lexer on demand, and offer fused dispatch where avoiding a
   peek-and-cache round trip matters.
2. **Predictability**: prefer deterministic lookahead and token-kind dispatch; make speculation
   explicit and transactional.
3. **Composability**: combine small parser functions and combinators; compose focused emitter
   traits into custom diagnostic strategies.
4. **Versatility**: run the same parser functions with fail-fast, collecting, silent, or custom
   emitters.
5. **Flexibility**: work through the generic `Lexer` and `Token` traits, with optional Logos input
   and Rowan CST integrations.
6. **Correctness**: track spans and structured errors, rewind emitted diagnostics with parser
   rollbacks, and provide conformance and fuzz test kits.

### Inspirations

Tokora takes inspiration from:

- [**winnow**](https://github.com/winnow-rs/winnow), for ergonomic parser API design
- [**chumsky**](https://github.com/zesterer/chumsky), for composable parser-combinator patterns
- [**logos**](https://github.com/maciejhirsz/logos), for high-performance lexing
- [**rowan**](https://github.com/rust-analyzer/rowan), for lossless syntax tree representation

## Development

Useful repository checks:

```sh
cargo fmt --all --check
cargo test -p tokora --all-features
cargo test -p tokora --no-default-features --features std,logos,rowan,combinators --examples
RUSTDOCFLAGS="-D warnings" cargo test -p tokora --all-features --doc
python3 tokora/tools/validate_docs.py --source
(cd tokora && mdbook build)
python3 tokora/tools/validate_docs.py --book target/book
```

This README is the crate-root documentation, so its Rust blocks are doctests as well. The guide is
validated both as rustdoc and as an mdBook, built with the mdBook version `pages.yml` pins, so API
links, local links, chapter order, and the published pages stay aligned. The full pre-push gate
set, and what each line in it protects, is the header of
[`.github/workflows/ci.yml`](https://github.com/al8n/tokora/blob/main/.github/workflows/ci.yml).

## License

`tokora` is under the terms of both the MIT license and the
Apache License (Version 2.0).

See the [Apache License, Version 2.0](https://github.com/al8n/tokora/blob/main/LICENSE-APACHE) and
the [MIT license](https://github.com/al8n/tokora/blob/main/LICENSE-MIT) text for details.

Copyright (c) 2026 Al Liu.

[Github-url]: https://github.com/al8n/tokora/
[CI-url]: https://github.com/al8n/tokora/actions/workflows/ci.yml
[doc-url]: https://docs.rs/tokora
[crates-url]: https://crates.io/crates/tokora
[codecov-url]: https://app.codecov.io/gh/al8n/tokora/
[discord]: https://discord.gg/FTuwh4d4N7
[tutorial-url]: https://al8n.github.io/tokora
