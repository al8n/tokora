A guided tour of tokora: build a small language end-to-end.

The crate's reference documentation explains each API in isolation; this guide tells the
story in order. Across ten chapters it builds **Calc**, a tiny calculator language with
variables, and uses every major capability of the crate along the way — lexing, hand-written
and combinator parsers, dispatch, Pratt expressions, backtracking, diagnostics, recovery,
partial input, and testing.

```text
program := stmt+
stmt    := "let" ident "=" expr ";"        bind a variable
         | "print" expr ("," expr)* ";"    print one or more values
         | expr ";"                        evaluate and discard
expr    := integers, variables, + - * / ^, unary -, ( ) grouping
```

# How to read this guide

Every code block is a **complete, runnable program** — the doctest suite compiles and runs
each one, so nothing here can drift from the real API. Later chapters hide the setup lines
established by earlier chapters (expand any example in the HTML docs to see them). The
chapters build on each other, but each states what it teaches up front, so you can jump in
anywhere.

# Chapters

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
10. [`ch10_testing`](crate::guide::ch10_testing) — testing your language: the [`conformance`](crate::conformance) kit
    for custom lexers and the [`traced`](crate::traced) combinator for debugging.

The four `examples/` programs in the repository (`json`, `calculator`, `s_expression`,
`c_expression`) are the guide's bigger siblings — full programs in the same style.
