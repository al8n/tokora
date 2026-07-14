# The Tokora Guide

A guided tour of tokora: build a small language end-to-end.

The crate's reference documentation explains each API in isolation; this guide tells the
story in order. The first ten chapters build **Calc**, a tiny calculator language with
variables. One anatomy chapter plus four maintained-example walkthroughs make up the applied-parser
section, and a final optional tooling chapter introduces lossless CSTs with Rowan.

```text
program := stmt+
stmt    := "let" ident "=" expr ";"        bind a variable
         | "print" expr ("," expr)* ";"    print one or more values
         | expr ";"                        evaluate and discard
expr    := integers, variables, + - * / ^, unary -, ( ) grouping
```

## How to read this guide

Every non-ignored Rust fence is a **doctest** — the suite compiles and runs it, so the examples
cannot quietly drift from the API. Later chapters may hide reduced token and error definitions
to keep the visible code focused (expand an example in the HTML docs to see them). The chapters
build on each other, but each states what it teaches up front, so you can jump in anywhere.

## Chapters

1. [`ch01_tokens`](crate::guide::ch01_tokens) — tokens and the lexer: the [`Token`](crate::Token) split into data and
   [`Kind`](crate::Token::Kind), the logos adapter, and the [`Lexer`](crate::Lexer) contract
   in brief.
2. [`ch02_parsers`](crate::guide::ch02_parsers) — first parsers: [`InputRef`](crate::InputRef),
   [`next`](crate::InputRef::next) / [`try_expect`](crate::InputRef::try_expect), typed
   errors, and the fluent [`Parse`](crate::Parse) entry points.
3. [`ch03_combinators`](crate::guide::ch03_combinators) — composition: sequencing with
   [`then`](crate::ParseInput::then), repetition with
   [`repeated`](crate::TryParseInput::repeated), separation with
   [`separated`](crate::TryParseInput::separated), and delimited shapes.
4. [`ch04_dispatch`](crate::guide::ch04_dispatch) — deterministic choice:
   [`DispatchOnKind`](crate::parser::DispatchOnKind) versus
   [`FusedDispatchOnKind`](crate::parser::FusedDispatchOnKind), and when each shape wins.
5. [`ch05_pratt`](crate::guide::ch05_pratt) — Pratt expression parsing with
   [`pratt_of`](crate::parser::pratt_of) and the built-in integer
   [`PrattPower`](crate::parser::PrattPower) impls.
6. [`ch06_backtracking`](crate::guide::ch06_backtracking) — speculation: [`attempt`](crate::InputRef::attempt) /
   [`try_attempt`](crate::InputRef::try_attempt), the [`Transaction`](crate::Transaction)
   guards and their [`DropPolicy`](crate::DropPolicy) typestate, and
   the [`InputRef`](crate::InputRef) session points.
7. [`ch07_diagnostics`](crate::guide::ch07_diagnostics) — diagnostics: [`Fatal`](crate::emitter::Fatal) versus
   [`Verbose`](crate::emitter::Verbose), [`Severity`](crate::emitter::Severity), labels,
   expected sets, and the [`diagnostics()`](crate::emitter::Verbose::diagnostics) view.
8. [`ch08_recovery`](crate::guide::ch08_recovery) — error recovery: [`sync_balanced`](crate::InputRef::sync_balanced),
   [`Hole`](crate::Hole)s, [`skip_then_retry`](crate::ParseInput::skip_then_retry), and the
   never-recoverable law.
9. [`ch09_streaming`](crate::guide::ch09_streaming) — partial input, Sans-I/O: the
   [`Completeness`](crate::Completeness) typestate and the
   [`parse_partial`](crate::parse_partial) refill loop.
10. Chapter 10, Testing (requires the `conformance` feature), covers the `conformance` kit
    for custom lexers and the [`traced`](crate::traced) combinator for debugging.
11. [`ch11_real_parser`](crate::guide::ch11_real_parser) — turning a tutorial fragment into a
    complete parser program.
12. [`ch12_calculator_example`](crate::guide::ch12_calculator_example) — the token-level Pratt
    calculator evaluator.
13. [`ch13_s_expression_example`](crate::guide::ch13_s_expression_example) — a recursive-descent
    S-expression parser and evaluator.
14. [`ch14_json_example`](crate::guide::ch14_json_example) — borrowed JSON values, delimiters,
    and tentative choice.
15. [`ch15_c_expression_example`](crate::guide::ch15_c_expression_example) — an AST-level Pratt
    parser with complex postfix forms.
16. Chapter 16, Lossless CSTs with Rowan, requires the `rowan` feature.

The four `examples/` programs in the repository (`json`, `calculator`, `s_expression`, and
`c_expression`) are canonical complete programs; the applied chapters explain how to reproduce
their structure without copying their source into the guide.
