#![doc = include_str!("README.md")]

// The chapters are markdown so the same source renders as rustdoc *and* as the mdbook book
// (`tokora/book.toml`); `include_str!` keeps every code block a compiled doctest. The files must
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

pub mod arch_parsing_engine {
  #![doc = include_str!("arch_parsing_engine.md")]
}

pub mod arch_checkpoint_rewind {
  #![doc = include_str!("arch_checkpoint_rewind.md")]
}

pub mod arch_source_slice {
  #![doc = include_str!("arch_source_slice.md")]
}

pub mod ch11_real_parser {
  #![doc = include_str!("ch11_real_parser.md")]
}

pub mod recipe_custom_lexer {
  #![doc = include_str!("recipe_custom_lexer.md")]
}

pub mod ch12_calculator_example {
  #![doc = include_str!("ch12_calculator_example.md")]
}

pub mod ch13_s_expression_example {
  #![doc = include_str!("ch13_s_expression_example.md")]
}

pub mod ch14_json_example {
  #![doc = include_str!("ch14_json_example.md")]
}

pub mod ch15_c_expression_example {
  #![doc = include_str!("ch15_c_expression_example.md")]
}

#[cfg(feature = "rowan")]
#[cfg_attr(docsrs, doc(cfg(feature = "rowan")))]
pub mod ch16_lossless_cst {
  #![doc = include_str!("ch16_lossless_cst.md")]
}

pub mod ref_combinators {
  #![doc = include_str!("ref_combinators.md")]
}

pub mod ref_errors_emitters_context {
  #![doc = include_str!("ref_errors_emitters_context.md")]
}

pub mod ref_vocabulary_macros_features {
  #![doc = include_str!("ref_vocabulary_macros_features.md")]
}

pub mod ref_pratt {
  #![doc = include_str!("ref_pratt.md")]
}

pub mod ref_types_syntax {
  #![doc = include_str!("ref_types_syntax.md")]
}
