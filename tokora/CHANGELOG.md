# Changelog

All notable changes to this crate are documented here. The project follows semantic
versioning; before 1.0, a minor bump (0.x → 0.(x+1)) signals a breaking change.

## Unreleased (0.4.0)

### Fixed

- **Unterminated delimited many-builders now report the opener as `Unclosed` through the
  emitter instead of silently accepting the input.** A delimited many-builder
  (`item.repeated(…)`, `item.repeated_while(…)`, `item.separated_by_*(…)`, or
  `item.separated_by_*_while(…)` closed with `.delimited::<D>().collect()`) driven over input
  whose closing delimiter never arrives before end-of-input — e.g. `"(1 2"`, `"[1 2"`, `"{1 2"` —
  used to return `Ok` with the elements parsed so far. It now emits an `Unclosed` diagnostic
  carrying the **opener's span** and the delimiter pair's name:
  - under a fail-fast `Fatal` emitter the parse fails with it (via the `From<Unclosed<…>>`
    conversion);
  - under a recovering `Verbose` emitter the diagnostic is recorded and the parse recovers,
    yielding the elements collected so far.

  A wrong token where the closer belongs still reports the existing unexpected-token
  (expected-close) vocabulary. The `separated`+delimited driver — which previously *did* error
  at end-of-input, but with a stale unexpected-token pointing at the last element rather than at
  the opener — now reports `Unclosed` at the opener like the other three drivers.

### Changed (breaking)

- Added `UnclosedEmitter`, a new atomically-composable emitter sub-trait
  (`tokora::emitter::UnclosedEmitter`) with a single `emit_unclosed` method, implemented by the
  built-in `Fatal`, `Verbose`, `Silent`, and `Ignored` emitters.
- The delimited many-builder `ParseInput`/`Collect` implementations gained two bounds:
  - `Ctx::Emitter: UnclosedEmitter<'inp, L, Lang>` — **a custom emitter must now implement
    `UnclosedEmitter`** to be usable with `.delimited::<D>().collect()`;
  - `<Ctx::Emitter as Emitter<…>>::Error: From<Unclosed<(), L::Span, Lang>>` — **an error type
    used with a delimited many-builder must gain a `From<Unclosed<…>>` arm.**

  Both are source-breaking for consumers whose emitter or error types do not already satisfy the
  new bounds, hence the 0.4.0 (breaking) classification. The delimiter identity travels in the
  `Unclosed`'s name (`CowStr`); the type-level delimiter tag is the erased `()` (the builder
  reborrows the delimiter internally, so a `Delim`-parameterized bound would not unify across the
  builder's own indirection).

### Migration

- Add a `From<Unclosed<…>>` arm to any error type used with `.delimited::<D>().collect()`, e.g.
  `impl<D, S, Lang: ?Sized> From<Unclosed<D, S, Lang>> for MyError { … }`. See the `json`
  example's `JsonError::Unclosed` arm for a worked pattern.
- If you use a custom emitter (not `Fatal`/`Verbose`/`Silent`/`Ignored`), implement
  `UnclosedEmitter` for it, mirroring your `FullContainerEmitter` impl: a fail-fast emitter
  converts the `Unclosed` to `Err` via `From`; a recovering emitter records it on its diagnostic
  log and returns `Ok(())`; a dropping emitter returns `Ok(())`.
