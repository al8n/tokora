# The Tokora Guide

Tokora writes parsers as plain Rust functions with the reach of a combinator library. It lexes on
demand as the parser pulls tokens — there is no separate tokenize pass — and gives you typed,
structured errors with rich diagnostics, explicit backtracking and recovery, streaming over
partial input, and optional [lossless concrete syntax trees](crate::cst) that preserve every byte
(whitespace and comments included) for formatters, refactoring tools, and language servers.

This guide is the tour. The reference documentation explains each API in isolation; the guide
tells the story in order, building **Calc** — a tiny calculator language with variables — end to
end, then reusing it to introduce the harder machinery.

```text
program := stmt+
stmt    := "let" ident "=" expr ";"        bind a variable
         | "print" expr ("," expr)* ";"    print one or more values
         | expr ";"                        evaluate and discard
expr    := integers, variables, + - * / ^, unary -, ( ) grouping
```

## The five parts

- **Part I — Getting Started** (this introduction) frames tokora and sketches Calc.
- **Part II — Core Concepts** is the tutorial: ten chapters that build Calc, from
  [tokens and the lexer](crate::guide::ch01_tokens) and
  [first parsers](crate::guide::ch02_parsers) through composition, deterministic choice, Pratt
  expressions, backtracking, diagnostics, recovery, partial input, and testing. Read it to
  *build* a parser.
- **Part III — Design & Architecture** is the internals: how the
  [parse-while-lexing engine](crate::guide::arch_parsing_engine), the
  [checkpoint and rewind timeline](crate::guide::arch_checkpoint_rewind), the
  [atomic emitter](crate::guide::arch_atomic_emitter), the event-stream CST engine, and the
  [source and slice layer](crate::guide::arch_source_slice) actually work. Read it to
  *understand the design*, or before contributing.
- **Part IV — Recipes & Applied Parsers** puts it together: the
  [anatomy of a real parser](crate::guide::ch11_real_parser), a
  [custom (non-logos) lexer](crate::guide::recipe_custom_lexer), and four full walkthroughs —
  [calculator](crate::guide::ch12_calculator_example),
  [S-expressions](crate::guide::ch13_s_expression_example),
  [JSON](crate::guide::ch14_json_example), and
  [C expressions](crate::guide::ch15_c_expression_example) — capped by lossless CSTs with Rowan.
  Read it for *worked examples*.
- **Part V — Reference** is the catalog: the
  [combinator and atom reference](crate::guide::ref_combinators); the
  [errors, emitters, and context](crate::guide::ref_errors_emitters_context) model; the
  [Pratt reference](crate::guide::ref_pratt); the
  [reusable AST building blocks](crate::guide::ref_types_syntax); and the
  [vocabulary, macros, and feature flags](crate::guide::ref_vocabulary_macros_features). Read it
  to *look something up*.

Two topics ride a feature flag: Testing (Part II) needs `conformance`, and both Lossless CSTs
(Part IV) and the event-stream CST engine (Part III) need `rowan`.

## How to read this guide

Every non-ignored Rust fence is a **doctest** — the suite compiles and runs it, so the examples
cannot quietly drift from the API. Later chapters may hide reduced token and error definitions to
keep the visible code focused (expand an example in the HTML docs to see them). Chapters build on
each other, but each states what it teaches up front, so you can jump in anywhere. Pick a path:

- **New to tokora** — read Part I, work Part II in order, then open the matching walkthrough in
  Part IV.
- **Using tokora as a library** — Part V is the lookup catalog; its entries point back to the
  chapter that teaches each API.
- **Contributing, or just curious how it works** — Part III is the internals tour; it assumes
  Part II.

The four `examples/` programs in the repository (`json`, `calculator`, `s_expression`, and
`c_expression`) are canonical complete programs; the applied chapters explain how to reproduce
their structure without copying their source into the guide.
