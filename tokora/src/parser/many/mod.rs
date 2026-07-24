use crate::{
  Decision, Emitter, ParseContext, ParseInput, Window,
  input::{CloseStatus, InputRef},
  lexer::Lexer,
  span::Spanned,
};

use super::*;
use handler::*;

pub use delim::*;
pub use handler::{DelimiterHandler, SeparatorHandler};
pub use options::*;
pub use repeated::*;
pub use repeated_while::*;
pub use sep::*;
pub use sep_while::*;

#[macro_use]
mod macros;

mod delim;
mod handler;
mod repeated;
mod repeated_while;

mod options;
mod sep;
mod sep_while;

#[cfg(test)]
mod gate_census {
  //! GATE_CENSUS — the section-4 never-recoverable gate sites, locked by count.
  //!
  //! Every resilient emit-and-continue loop body in the try-driven collection families
  //! must gate on `Cmpl::is_incomplete_error` FIRST, and re-raise a terminal scanner stop
  //! (`inp.at_latched_boundary()`) alongside it, so neither a frontier `Incomplete` nor a
  //! tripped limit from the element parser is spent as a diagnostic. One gate per swallow
  //! site; the census pins the total, the per-file placement, and the terminal re-raise so a
  //! new resilient loop cannot land ungated or without the terminal dual (extend the list,
  //! then gate it both ways).

  #[test]
  fn every_resilient_swallow_site_is_gated() {
    let sites = [
      ("many/repeated/mod.rs", include_str!("repeated/mod.rs")),
      ("many/delim/repeated.rs", include_str!("delim/repeated.rs")),
      ("many/sep/parse/mod.rs", include_str!("sep/parse/mod.rs")),
      ("many/sep/delim/mod.rs", include_str!("sep/delim/mod.rs")),
    ];
    let mut gates = 0;
    for (name, src) in sites {
      let swallows = src.matches("emit_error(Spanned::new(span,").count();
      let gated = src.matches("if Cmpl::is_incomplete_error(&").count();
      assert_eq!(
        swallows, gated,
        "{name}: every emit-and-continue swallow needs exactly one incomplete gate"
      );
      // The terminal dual: every incomplete gate carries the terminal re-raise in the same
      // guard, so a tripped limit re-raises instead of being emitted-and-continued.
      let terminal = src.matches("|| inp.at_latched_boundary()").count();
      assert_eq!(
        gated, terminal,
        "{name}: every incomplete gate must re-raise a terminal stop too \
         (`|| inp.at_latched_boundary()`)"
      );
      gates += gated;
    }
    assert_eq!(
      gates, 4,
      "the try-driven families carry exactly four gated loop bodies"
    );
  }
}
