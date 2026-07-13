#![doc = include_str!("README.md")]

// The chapters are markdown so the same source renders as rustdoc *and* as the mdbook book
// (`tokit/book.toml`); `include_str!` keeps every code block a compiled doctest. The files must
// live inside the package directory or `cargo package` would ship a crate that cannot build.
//
// Each chapter is included as an *inner* attribute of its module: an inner doc attribute resolves
// intra-doc links in the module's own scope, exactly as the `//!` comments it replaced did, so the
// chapters keep linking to each other as `super::chNN_name`.

pub mod ch01_tokens {
  #![doc = include_str!("ch01_tokens.md")]
}

pub mod ch02_parsers {
  #![doc = include_str!("ch02_parsers.md")]
}

pub mod ch03_combinators {
  #![doc = include_str!("ch03_combinators.md")]
}

pub mod ch04_dispatch {
  #![doc = include_str!("ch04_dispatch.md")]
}

pub mod ch05_pratt {
  #![doc = include_str!("ch05_pratt.md")]
}

pub mod ch06_backtracking {
  #![doc = include_str!("ch06_backtracking.md")]
}

pub mod ch07_diagnostics {
  #![doc = include_str!("ch07_diagnostics.md")]
}

pub mod ch08_recovery {
  #![doc = include_str!("ch08_recovery.md")]
}

pub mod ch09_streaming {
  #![doc = include_str!("ch09_streaming.md")]
}

#[cfg(feature = "conformance")]
#[cfg_attr(docsrs, doc(cfg(feature = "conformance")))]
pub mod ch10_testing {
  #![doc = include_str!("ch10_testing.md")]
}
