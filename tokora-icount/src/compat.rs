//! The tokora items the workloads name whose paths have moved between releases, imported in one
//! place.
//!
//! `ci/icount/run.sh` builds this package against the branch's `tokora/` and again against the
//! merge-base's, so `fixture.rs` and `workloads.rs` have to compile unchanged on both sides. A
//! branch that moves an item they name imports it here, behind a cfg that a build script sets
//! from the sibling tree, with a second arm for the base side's path; the workloads then stay
//! the constant of the comparison.

pub(crate) use tokora::{
  ParseAttempt,
  error::{
    FullContainer, MissingSyntax, MissingToken, SeparatedError, TooFew, TooMany, UnexpectedToken,
  },
  parse::Action,
};
