use crate::{
  Decision, Emitter, ParseContext, ParseInput, Window, input::InputRef, lexer::Lexer, span::Spanned,
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
  //! must gate on `Cmpl::is_incomplete_error` FIRST, so a frontier `Incomplete` from the
  //! element parser re-raises instead of being spent as a diagnostic. One gate per
  //! swallow site; the census pins both the total and the per-file placement so a new
  //! resilient loop cannot land ungated (extend the list, then gate it).

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
      gates += gated;
    }
    assert_eq!(
      gates, 4,
      "the try-driven families carry exactly four gated loop bodies"
    );
  }
}
