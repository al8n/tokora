# Summary

# Part I — Getting Started

- [Introduction](README.md)

# Part II — Core Concepts

- [Tokens and the lexer](ch01_tokens.md)
- [First parsers](ch02_parsers.md)
- [Composition](ch03_combinators.md)
- [Deterministic choice](ch04_dispatch.md)
- [Expressions: Pratt parsing](ch05_pratt.md)
- [Backtracking](ch06_backtracking.md)
- [Diagnostics](ch07_diagnostics.md)
- [Recovery](ch08_recovery.md)
- [Partial input](ch09_streaming.md)
- [Testing](ch10_testing.md)

# Part III — Design & Architecture

<!-- Forthcoming: the parse-while-lexing engine, checkpoint & rewind, the atomic
     emitter design, the event-stream CST engine, and source & storage backends. -->

# Part IV — Recipes & Applied Parsers

- [Anatomy of a real Tokora parser](ch11_real_parser.md)
- [Walkthrough: calculator](ch12_calculator_example.md)
- [Walkthrough: S-expressions](ch13_s_expression_example.md)
- [Walkthrough: JSON](ch14_json_example.md)
- [Walkthrough: C expressions](ch15_c_expression_example.md)
- [Lossless CSTs with Rowan](ch16_lossless_cst.md)

# Part V — Reference

- [Combinator & atom reference](ref_combinators.md)
