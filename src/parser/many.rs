use crate::{
  Decision, Emitter, ParseContext, ParseInput, Window,
  lexer::{InputRef, Lexer},
  utils::Spanned,
};

use super::*;
use handler::*;

pub use allow_leading::AllowLeading;
pub use allow_trailing::AllowTrailing;
pub use at_least::*;
pub use at_most::*;
pub use bounded::*;
pub use delim::*;
pub use repeated::*;
pub use repeated_while::*;
pub use require_leading::RequireLeading;
pub use require_trailing::RequireTrailing;
pub use sep::*;
pub use sep_while::*;

mod allow_leading;
mod allow_trailing;
mod at_least;
mod at_most;
mod bounded;
mod delim;
mod handler;
mod repeated;
mod repeated_while;
mod require_leading;
mod require_trailing;
mod sep;
mod sep_while;

/// A marker type representing the maximum number of elements allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Maximum(usize);

impl Maximum {
  /// The maximum possible value for `Maximum`.
  pub const MAX: Self = Self::new(usize::MAX);

  /// Creates a new `Maximum`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(n: usize) -> Self {
    Self(n)
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

/// A marker type representing the minimum number of elements required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Minimum(usize);

impl Minimum {
  /// The minimum possible value for `Minimum`.
  pub const MIN: Self = Self::new(0);

  /// Creates a new `Minimum`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(n: usize) -> Self {
    Self(n)
  }

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

struct Unbounded;
