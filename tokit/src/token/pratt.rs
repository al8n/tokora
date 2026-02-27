use crate::parser::{PrattLHS, PrattRHS};

use super::Token;

/// A trait for tokens that can run a pratt on them.
pub trait PrattToken<'a, Expr: ?Sized, Power = i64>: Token<'a> {
  /// Returns `true` if the token is kind of an operand token or a prefix token of the `Expr`.
  fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), Power>>;

  /// Returns `true` if the token is kind of an operand token or a prefix token of the `Expr`.
  fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), Power>>;
}
