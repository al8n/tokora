pub use fatal::Fatal;
pub use ignored::Ignored;
pub use silent::Silent;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use verbose::{Diagnostic, DiagnosticKind, Diagnostics, Verbose};

mod fatal;
mod ignored;
mod silent;

#[cfg(any(feature = "std", feature = "alloc"))]
mod verbose;
