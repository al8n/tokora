# 12. Walkthrough: calculator

Prerequisites: chapters 5, 10, and 11.

This walkthrough explains the maintained
[`calculator.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/calculator.rs).
It is a token-level Pratt evaluator: the parser classifies tokens, then folds directly to an
`f64` rather than allocating an expression AST.

| Maintained program | Symbols to follow |
| --- | --- |
| [`calculator.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/calculator.rs) | `Token`, `TokenKind`, `Power`, `PrattToken`, `fold_prefix`, `fold_infix`, `fold_postfix`, `calc_expr` |

## Define token, kind, lexer alias, and `CalcError`

The enum carries numeric payloads in `Token::Num(f64)` and leaves classification to a separate
`TokenKind`. The canonical program derives the Logos lexer, aliases it as `CalcLexer`, and has one
`CalcError` variant family for lexical errors, unexpected tokens, and an unexpected end. The
error conversions are what let a generic `Ctx::Emitter` return the application error.

The relevant public APIs are `Token`, [`token::PrattToken`](crate::token::PrattToken),
[`parser::PrattPower`](crate::parser::PrattPower),
[`parser::PrattLHS`](crate::parser::PrattLHS),
[`parser::PrattRHS`](crate::parser::PrattRHS),
[`parser::Precedenced`](crate::parser::Precedenced),
[`parser::PrattInfix`](crate::parser::PrattInfix), [`InputRef::pratt`](crate::InputRef::pratt),
`PrattEmitter`, `Spanned`, `Parser`, and `Parse::parse_str`.

## Define the precedence constants and grouping sentinel

`Power(i32)` names this language's ladder. It is useful for making the domain explicit, not for
orphan-rule reasons: Tokora implements `PrattPower` for the standard integer types too. The
grouping sentinel is below the default floor so an opening parenthesis can recurse at that lower
floor and consume its matching closing parenthesis without exposing it to the outer expression.

```rust
use tokora::parser::{PrattInfix, PrattLHS, PrattPower, PrattRHS, Precedenced};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Power(i32);

impl PrattPower for Power {
  fn next(&self) -> Self { Self(self.0 + 1) }
  fn prev(&self) -> Self { Self(self.0 - 1) }
}

const PAREN: Power = Power(-1);
const SUM: Power = Power(1);
const PRODUCT: Power = Power(2);
const NEGATE: Power = Power(3);
const EXPONENT: Power = Power(4);

#[derive(Clone, Copy)]
enum CalcToken { Number, Minus, Star, Caret, Open, Close }

fn lhs(token: CalcToken) -> Option<PrattLHS<(), (), Power>> {
  Some(match token {
    CalcToken::Number => PrattLHS::Operand(()),
    CalcToken::Minus => PrattLHS::Prefix(Precedenced::new((), NEGATE)),
    CalcToken::Open => PrattLHS::Prefix(Precedenced::new((), PAREN)),
    _ => return None,
  })
}

fn rhs(token: CalcToken) -> Option<PrattRHS<(), (), (), (), Power>> {
  Some(match token {
    CalcToken::Minus => PrattRHS::Infix(Precedenced::new(PrattInfix::Left(()), SUM)),
    CalcToken::Star => PrattRHS::Infix(Precedenced::new(PrattInfix::Left(()), PRODUCT)),
    CalcToken::Caret => PrattRHS::Infix(Precedenced::new(PrattInfix::Right(()), EXPONENT)),
    CalcToken::Close => PrattRHS::Postfix(Precedenced::new((), PAREN)),
    _ => return None,
  })
}

assert!(matches!(lhs(CalcToken::Number), Some(PrattLHS::Operand(()))));
assert!(matches!(rhs(CalcToken::Caret), Some(PrattRHS::Infix(_))));
assert_eq!(EXPONENT.prev(), NEGATE);
```

## Implement `try_pratt_lhs` and `try_pratt_rhs`

The canonical `PrattToken` implementation turns that table into the engine interface.
`try_pratt_lhs` accepts a number, prefix minus, or opening parenthesis; `try_pratt_rhs` accepts
infix operators and the closing-parenthesis postfix sentinel. Returning `None` tells the engine
that the token is not part of this expression and must remain in the input.

## Implement the named prefix, infix, and postfix folds

Use named functions rather than closures because the token-level fold traits require a
higher-ranked lifetime bound. `fold_prefix` negates a number or passes a grouped value through;
`fold_infix` extracts an operator from `PrattInfix` and computes the next `f64`; `fold_postfix`
acknowledges a closing parenthesis and returns its operand. Each fold returns a `Spanned<Token>`,
so the evaluated value goes back into `Token::Num`.

## Build `calc_expr`

`calc_expr` calls `InputRef::pratt` with the three folds, then unwraps the final `Token::Num`.
Its `Ctx` bounds add `PrattEmitter` to the ordinary `Emitter` bound because Pratt-specific
diagnostics travel through the emitter too.

## Reproduce the maintained assertion table

Run the canonical binary after changing a fold or precedence constant:

```sh
cargo run -p tokora --example calculator --features logos
```

Its table checks `1 + 2 * 3`, parentheses, `2 ^ 3 ^ 2`, unary minus versus exponentiation, and
left-associative division. Those are the behavior contract for the maintained evaluator, not
extra source to copy into this guide. Next: [chapter 13](super::ch13_s_expression_example).
