#[cfg(feature = "trace")]
mod trace;

// The label must be popped on EVERY exit of the scope, unwind included. The error path
// is already bracketed (`res` is produced, then `exit_label` runs); the unguarded path is
// exactly panic unwind, where the pop is skipped and the label stack — plain emitter state
// that survives `catch_unwind` — keeps the stale context for the emitter's whole life.
mod unwind;
