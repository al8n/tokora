//! The parsing traits, the types their contracts name, and the drivers that run a parser.
//!
//! [`combinator`](crate::combinator) builds parsers; this module says what a parser *is* and
//! what the combinators accept:
//!
//! - **What a parser is** — [`ParseInput`], with [`ParseInputUnwrapExt`] beside it, and the
//!   tentative [`TryParseInput`], whose answer is a [`ParseAttempt`].
//! - **What combinators accept** — the choice traits [`ParseChoice`] and [`ParseTokenChoice`]
//!   with the [`Branch`] they dispatch on; [`Accumulator`], which `.collect()` on any repeated or
//!   separated parser needs in scope; the lookahead vocabulary [`Decision`], [`Window`] and
//!   [`Action`]; the classifiers [`Check`] and [`Require`]; the recovery bounds [`RecoverInput`]
//!   and [`InplaceRecoverInput`]; the repetition handlers `DelimiterHandler` and
//!   `SeparatorHandler` (feature `many`); and the [`ParseState`] a `*_with` callback receives.
//! - **Context** — [`ParseContext`] and its refinements [`ComposableParseContext`] and
//!   [`PolicyParseContext`], the concrete [`ParserContext`] and [`FatalContext`], and the
//!   [`ErrorOf`] projection a generic production's signature spells.
//! - **Configuration** — [`Apply`], [`WithCache`] and [`WithEmitter`].
//! - **Running a parser** — the [`Parser`] builder and its [`Parse`] entry point, and the free
//!   drivers [`parse()`], [`parse_with`], [`parse_with_state`] and [`parse_partial`].
//!
//! The names a production must have in scope are re-exported at the crate root as well, so
//! `use tokora::{ParseInput, Parser};` and `use tokora::parse::{ParseInput, Parser};` name the
//! same items. The module `parse` and the function `parse()` share the root in different
//! namespaces, the way `syn::parse` and `syn::parse()` do.

pub use choice::{Branch, ParseChoice, ParseTokenChoice};
pub use context::{
  ComposableParseContext, ErrorOf, FatalContext, ParseContext, ParserContext, PolicyParseContext,
};
pub use driver::{
  Action, Apply, Parse, Parser, WithCache, WithEmitter, parse, parse_partial, parse_with,
  parse_with_state,
};
pub use input::{Accumulator, Decision, ParseInput, ParseInputUnwrapExt, Window};
pub use state::ParseState;
pub use try_parse::input::{ParseAttempt, TryParseInput};

// The traits below are declared beside the code that consumes them and re-exported from here
// alone: a trait is never reachable under `combinator` (#350, rule 3).
pub use crate::check::Check;
pub use crate::combinator::recover::{InplaceRecoverInput, RecoverInput};
pub use crate::require::Require;

#[cfg(feature = "many")]
#[cfg_attr(docsrs, doc(cfg(feature = "many")))]
pub use crate::combinator::many::handler::{DelimiterHandler, SeparatorHandler};

pub(crate) mod choice;
pub(crate) mod context;
mod driver;
pub(crate) mod input;
pub(crate) mod state;
pub(crate) mod try_parse;
