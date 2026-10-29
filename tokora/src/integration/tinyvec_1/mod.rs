mod container;

// The handler traits exist only with `many`; these impls were in `combinator/many/handler`.
#[cfg(feature = "many")]
mod handler;
