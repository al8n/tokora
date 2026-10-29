//! The items the workloads name whose paths moved under the 0.12 export layout (#350), imported
//! from whichever layout the sibling `../tokora` has.
//!
//! `build.rs` sets `tokora_0_11_layout` when it is the merge-base's 0.11 tree. Both arms import
//! the same names, so `fixture.rs` and `workloads.rs` compile unchanged against either side —
//! which is what keeps the instrument the constant of a comparison.

// The 0.11 paths: remove this arm, `build.rs` and the cfg once main carries the 0.12 layout.
#[cfg(tokora_0_11_layout)]
pub(crate) use tokora::{
  error::{
    syntax::{FullContainer, MissingSyntax, TooFew, TooMany},
    token::{MissingToken, SeparatedError, UnexpectedToken},
  },
  parser::Action,
  try_parse_input::ParseAttempt,
};

#[cfg(not(tokora_0_11_layout))]
pub(crate) use tokora::{
  ParseAttempt,
  error::{
    FullContainer, MissingSyntax, MissingToken, SeparatedError, TooFew, TooMany, UnexpectedToken,
  },
  parse::Action,
};
