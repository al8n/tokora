//! Tells `src/compat.rs` which export layout the sibling `../tokora` has.
//!
//! `ci/icount/run.sh` builds this package twice from one tree: once against the branch's
//! `tokora/`, once with the merge-base's `tokora/` swapped in, so the instrument is the constant
//! and the library is the variable. A few items the workloads name moved under the 0.12 export
//! layout (#350). This script sets `tokora_0_11_layout` when the sibling tree predates that
//! layout — its `tokora/src/parse/driver.rs`, which the layout introduced, is absent — and
//! `src/compat.rs` then imports those names from their 0.11 paths.
//!
//! Remove this script, with the cfg and `src/compat.rs`'s 0.11 arm, once main carries the 0.12
//! layout.

use std::path::Path;

fn main() {
  println!("cargo::rustc-check-cfg=cfg(tokora_0_11_layout)");
  // The whole directory rather than the marker file: the marker exists on one side only, and a
  // `rerun-if-changed` path that does not exist reruns the script on every build.
  println!("cargo::rerun-if-changed=../tokora/src");
  let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
    .expect("cargo sets CARGO_MANIFEST_DIR for build scripts");
  if !Path::new(&manifest_dir)
    .join("../tokora/src/parse/driver.rs")
    .exists()
  {
    println!("cargo::rustc-cfg=tokora_0_11_layout");
  }
}
