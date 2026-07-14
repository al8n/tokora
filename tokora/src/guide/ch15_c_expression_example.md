# 15. Walkthrough: C expressions

Prerequisites: chapters 5 and 11; chapter 12 is helpful.

The maintained [`c_expression.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/c_expression.rs)
uses AST-level Pratt parsing. Unlike the calculator, its folds construct typed `Expr` nodes and
some postfix folds consume further input for indexing, calls, and ternaries.

| Maintained program | Symbols to follow |
| --- | --- |
| [`c_expression.rs`](https://github.com/al8n/tokora/blob/main/tokora/examples/c_expression.rs) | `parse_lhs`, `parse_rhs`, `fold_prefix`, `fold_infix`, `fold_postfix`, `parse_cexpr` |

## Define `UnaryOp`, `BinOp`, `PostfixOp`, and `Expr`

The AST has typed variants for prefix, binary, postfix increment/decrement, index, call, and
ternary expressions. Separating operator tags from tree nodes lets the folds be small and keeps
the display implementation useful as an assertion oracle.

The public surface is [`parser::pratt_of`](crate::parser::pratt_of),
[`parser::PrattPower`](crate::parser::PrattPower),
[`parser::PrattLHS`](crate::parser::PrattLHS),
[`parser::PrattRHS`](crate::parser::PrattRHS),
[`parser::Precedenced`](crate::parser::Precedenced),
[`parser::PrattInfix`](crate::parser::PrattInfix), `ParseInput::parse_input`,
[`InputRef::next`](crate::InputRef::next), [`InputRef::try_expect`](crate::InputRef::try_expect),
`Parser`, and [`Parse::parse_str`](crate::Parse::parse_str).

## Define the precedence ladder

The ladder runs from a sentinel below the default floor through ternary, logical and bitwise
operators, comparison, arithmetic, prefix, and high-power postfix forms. The precise numeric
values matter only relative to one another; named constants make that relationship auditable.

```rust
use tokora::parser::{PrattInfix, PrattPower, PrattRHS, Precedenced};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Power(i32);
impl PrattPower for Power {
  fn next(&self) -> Self { Self(self.0 + 1) }
  fn prev(&self) -> Self { Self(self.0 - 1) }
}

const SENTINEL: Power = Power(-1);
const TERNARY: Power = Power(2);
const ADD: Power = Power(11);
const POSTFIX: Power = Power(14);

#[derive(Clone, Copy, Debug)]
enum Postfix { Index, Call, Ternary, Sentinel }

fn rhs(token: char) -> PrattRHS<(), (), (), Postfix, Power> {
  let sentinel = PrattRHS::Postfix(Precedenced::new(Postfix::Sentinel, SENTINEL));
  match token {
    '+' => PrattRHS::Infix(Precedenced::new(PrattInfix::Left(()), ADD)),
    '[' => PrattRHS::Postfix(Precedenced::new(Postfix::Index, POSTFIX)),
    '(' => PrattRHS::Postfix(Precedenced::new(Postfix::Call, POSTFIX)),
    '?' => PrattRHS::Postfix(Precedenced::new(Postfix::Ternary, TERNARY)),
    _ => sentinel,
  }
}

assert!(matches!(rhs('+'), PrattRHS::Infix(_)));
assert!(matches!(rhs('['), PrattRHS::Postfix(_)));
assert!(matches!(rhs(')'), PrattRHS::Postfix(_)));
```

## Implement `parse_lhs`

`parse_lhs` consumes a number or identifier as an operand, parses a grouped expression after
`(`, and classifies prefix `-`, `+`, `!`, `~`, `++`, and `--` operators. For grouping it calls
`parse_cexpr` recursively and consumes the matching close parenthesis with `try_expect`.

## Implement `parse_rhs` and the sentinel

`parse_rhs` maps infix tokens to left-associative `PrattInfix` values and maps postfix trigger
tokens to `PostfixOp` values. A non-operator is classified as the low-power `Sentinel`. Because
its power is below the current floor, the Pratt engine restores the checkpoint made before
`parse_rhs`, leaving that token for the surrounding grammar instead of losing it.

## Implement the three folds

`fold_prefix` wraps an operand in `Expr::Prefix`. `fold_infix` extracts the operator from
`PrattInfix` and constructs `Expr::Binary`. `fold_postfix` handles simple increment/decrement
nodes immediately and delegates the forms that need more tokens to the ordinary parser.

## Show how `fold_postfix` parses `[]`, calls, and `?:`

The fold receives the already-parsed left operand and an `InputRef`. For an index it parses an
expression until `]`. For a call it accepts `)` for an empty argument list or loops over
comma-separated expressions. For a ternary it parses the then-expression, requires `:`, and
parses the otherwise-expression. The parser is still in control of all delimiters.

```rust
#[derive(Debug, PartialEq)]
enum Expr {
  Name(&'static str),
  Index { base: Box<Expr>, index: Box<Expr> },
  Call { function: Box<Expr>, arguments: Vec<Expr> },
  Ternary { condition: Box<Expr>, then: Box<Expr>, otherwise: Box<Expr> },
}

fn index(base: Expr, index: Expr) -> Expr {
  Expr::Index { base: Box::new(base), index: Box::new(index) }
}

assert_eq!(
  index(Expr::Name("array"), Expr::Name("i")),
  Expr::Index { base: Box::new(Expr::Name("array")), index: Box::new(Expr::Name("i")) },
);
```

## Close the recursion in `parse_cexpr`

`parse_cexpr` calls `pratt_of(parse_lhs, parse_rhs, fold_prefix, fold_infix, fold_postfix)` and
then `parse_input`. Named functions close the mutual recursion through call-stack frames; they
do not require recursive parser types.

## Run the maintained AST-display assertion table

The canonical binary covers precedence, grouping, associativity, unary operators, increment,
ternary expressions, indexing, calls, shifts, and bitwise operators:

```sh
cargo run -p tokora --example c_expression --features logos
```

The optional chapter 16, Lossless CSTs with Rowan, takes a different route: it records source
tokens rather than reducing them to an AST. It requires the `rowan` feature, so it is named here
without a rustdoc link.
