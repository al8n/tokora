// `CstText` exists only with `rowan`, as did `cst/text.rs`, where these impls were.
#[cfg(feature = "rowan")]
mod cst_text;
mod equivalent;
mod slice;
mod source;
mod to_equivalent;

#[cfg(test)]
#[allow(warnings)]
mod tests;
