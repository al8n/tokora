# Tokora

Tokora is a Rust parser-combinator library with on-demand lexing, explicit lookahead and
backtracking, configurable diagnostics, and optional Logos and Rowan integrations. Parsers work
over a `Lexer` and `Token` model, so the same grammar can use a fail-fast runtime emitter or a
collecting diagnostic emitter.

## Install

Most applications use the maintained Logos adapter:

```toml
[dependencies]
tokora = { version = "0.1", features = ["logos"] }
```

`logos` is the alias for the current `logos_0_16` integration. The default `std` feature remains
enabled unless you set `default-features = false`.

## Quick start

This complete program lexes one integer and asserts the parsed result.

```rust
use tokora::{
  Emitter, InputRef, Parse, ParseContext, Parser, Token as TokenT,
  error::token::UnexpectedTokenOf,
  logos::{self, Logos},
};

#[derive(Clone, Debug, Default, PartialEq)]
struct LexError;

impl From<()> for LexError {
  fn from(_: ()) -> Self { Self }
}

#[derive(Clone, Debug, Logos)]
#[logos(crate = logos, skip r"[ \t\r\n]+", error = LexError)]
enum Token {
  #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().map_err(|_| LexError))]
  Integer(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TokenKind {
  Integer,
}

impl core::fmt::Display for TokenKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("integer")
  }
}

impl core::fmt::Display for Token {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Integer(value) => write!(f, "{value}"),
    }
  }
}

impl TokenT<'_> for Token {
  type Kind = TokenKind;
  type Error = LexError;

  fn kind(&self) -> Self::Kind { TokenKind::Integer }
  fn is_trivia(&self) -> bool { false }
}

type IntegerLexer<'a> = tokora::lexer::LogosLexer<'a, Token>;

#[derive(Debug, PartialEq)]
enum ParseError {
  Lex,
  Unexpected,
}

impl From<LexError> for ParseError {
  fn from(_: LexError) -> Self { Self::Lex }
}

impl<'inp> From<UnexpectedTokenOf<'inp, IntegerLexer<'inp>>> for ParseError {
  fn from(_: UnexpectedTokenOf<'inp, IntegerLexer<'inp>>) -> Self { Self::Unexpected }
}

fn integer<'inp, Ctx>(
  input: &mut InputRef<'inp, '_, IntegerLexer<'inp>, Ctx>,
) -> Result<i64, ParseError>
where
  Ctx: ParseContext<'inp, IntegerLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, IntegerLexer<'inp>, Error = ParseError>,
{
  match input.next()? {
    Some(token) => match token.into_data() {
      Token::Integer(value) => Ok(value),
    },
    None => Err(ParseError::Unexpected),
  }
}

fn main() {
  assert_eq!(Parser::new().apply(integer).parse_str("42"), Ok(42));
}
```

## Capabilities

- On-demand token flow through `InputRef`, with explicit cache-backed lookahead and transactions.
- Plain parser functions plus composable sequencing, repetition, delimiters, and deterministic
  choice.
- Token-level and AST-level Pratt parsing.
- Configurable `Fatal`, `Verbose`, `Silent`, and `Ignored` diagnostics.
- Recovery, partial-input support, lexer conformance checks, tracing, and a public fuzz harness.
- Optional adapters for Logos, Rowan CSTs, source types, and container types.

## Error handling

Parsers are generic over their parse context, including the emitter. `Parser::new()` uses the
fail-fast `Fatal` emitter; `Verbose` records diagnostics and can continue when the grammar
recovers. The same parser functions can therefore serve a runtime parser, compiler front end, or
editor integration without a second grammar implementation.

Structured lexer, token, separator, container, and Pratt errors convert into the application's
error type through `From` implementations. Explicit transactions roll back input state and
recorded diagnostics, but application-owned side effects need their own transaction boundary.

## Guide and examples

The [Tokora Guide](https://al8n.github.io/tokora/) has three parts: ten Calc fundamentals, one
anatomy chapter plus four maintained-example walkthroughs (five applied-parser chapters), and an
optional Rowan/lossless-CST chapter. The examples below are canonical complete programs; the guide
links back to them instead of copying whole files into prose.

| Program | Focus | Canonical source | Run |
| --- | --- | --- | --- |
| `calculator` | Token-level Pratt evaluator | [`calculator.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/calculator.rs) | `cargo run -p tokora --example calculator --features logos` |
| `s_expression` | Recursive descent and evaluation | [`s_expression.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/s_expression.rs) | `cargo run -p tokora --example s_expression --features logos` |
| `json` | Borrowed values, delimiters, and tentative choice | [`json.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/json.rs) | `cargo run -p tokora --example json --features logos` |
| `c_expression` | AST-level Pratt parsing with postfix forms | [`c_expression.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/c_expression.rs) | `cargo run -p tokora --example c_expression --features logos` |

The book source lives under
[`tokora/src/guide`](https://github.com/al8n/tokora/tree/main/tokora/src/guide), and the examples
also compile together with `cargo test -p tokora --no-default-features --features std,logos --examples`.

## Features

| Feature | Effect |
| --- | --- |
| `default` | Enables `std`. |
| `std` | Enables standard-library support and default features of applicable dependencies. |
| `alloc` | Enables allocation-backed facilities in `no_std` builds. |
| `logos` | Alias for `logos_0_16`, the current Logos integration. |
| `logos_0_14` | Enables the optional `logos@0.14` adapter. |
| `logos_0_15` | Enables the optional `logos@0.15` adapter. |
| `logos_0_16` | Enables the optional `logos@0.16` adapter used by `logos`. |
| `trace` | Enables parser tracing; implies `std`. |
| `unstable-raw` | Exposes the unstable raw checkpoint API. |
| `conformance` | Enables the custom-lexer conformance test kit; implies `std`. |
| `fuzz` | Enables the deterministic public input/backtracking fuzz harness; implies `std`. |
| `rowan` | Enables Rowan CST utilities; implies `std`. Add `rowan = "0.16"` directly when implementing `rowan::Language`. |
| `bytes` | Alias for `bytes_1`. |
| `bytes_1` | Enables `bytes@1` source support. |
| `bstr` | Alias for `bstr_1`. |
| `bstr_1` | Enables `bstr@1` source support. |
| `hipstr` | Alias for `hipstr_0_8`. |
| `hipstr_0_8` | Enables `hipstr@0.8` source support. |
| `smallvec` | Alias for `smallvec_1`. |
| `smallvec_1` | Enables `smallvec@1` containers and implies `alloc`. |
| `heapless` | Alias for `heapless_0_9`. |
| `heapless_0_9` | Enables `heapless@0.9` containers. |
| `tinyvec` | Alias for `tinyvec_1`. |
| `tinyvec_1` | Enables `tinyvec@1` containers. |

Feature aliases select their versioned counterpart; versioned features make the corresponding
optional dependency available. One Logos version is normally sufficient, though multiple versioned
integrations may coexist. When several are enabled, the unversioned
`tokora::lexer::LogosLexer` selects 0.16, then 0.15, then 0.14. `tokora::logos` is available only
with `logos_0_16` and re-exports that version. `rowan` does not enable `logos`, and `smallvec_1`
is the versioned feature that adds `alloc`.

## Platform and development

Tokora's MSRV is Rust 1.87. Core functionality supports `no_std`: disable default features and
enable `alloc` only when your parser, cache, or selected integration needs allocation.

```toml
[dependencies]
tokora = { version = "0.1", default-features = false }
```

Useful repository checks:

```sh
cargo fmt --all --check
cargo test -p tokora --all-features
cargo test -p tokora --no-default-features --features std,logos --examples
RUSTDOCFLAGS="-D warnings" cargo test -p tokora --all-features --doc
(cd tokora && mdbook build)
```

The guide is validated both as rustdoc and as an mdBook so API links, local links, chapter order,
and Pages output stay aligned.

## License

Tokora is dual-licensed under [MIT](https://github.com/al8n/tokora/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/al8n/tokora/blob/main/LICENSE-APACHE), at your option.
