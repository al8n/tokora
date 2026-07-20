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

  The close-status diagnostic is the **primary**: the `separated`/`separated_while` delimited
  drivers emit it **before** the end-state secondaries (`TooFew`, separator policy), so a
  fail-fast emitter fails with `Unclosed` on e.g. `[` under `at_least(1)` or `[1,2,` at
  end-of-input rather than letting the secondary short-circuit it, and a recovering emitter
  records primary-then-secondaries in order. The plain `repeated`/`repeated_while` delimited
  drivers already ordered the close-status diagnostic before their bound checks.

- **The delimited many-builders commit the closer without re-lexing it, fixing a
  blackhole-cache (`ParserContext<_, _, ()>`) double-scan on the success path.** Internal,
  non-breaking. `InputRef::probe_close` used to classify the closer by scanning it and then
  push the scanned token back to the cache for a follow-up `try_expect` to commit; under the
  blackhole cache `()` the push-back is a no-op, so the closer was dropped and the follow-up
  `try_expect` **re-scanned** it. That second scan is observable to a stateful or
  resource-limited lexer — a valid delimited list (e.g. `(a)`) could trip its limiter, or hit
  the "unreachable" recovery path, on otherwise-valid input. `probe_close` now classifies the
  closer without consuming it — carrying the scanned token out by value, or leaving a cached
  closer at the front — and a new by-value commit primitive advances the cursor over it once, at
  the driver's own commit point, with zero re-scans in every cache capacity. Because the probe
  stays cursor-neutral until that commit, the deferred (`separated`/`separated_while`) drivers
  span their elements correctly and an error before the commit leaves the closer available for
  recovery. All four delimited many-builders (`repeated`, `repeated_while`, `separated_by_*`,
  `separated_by_*_while`) adopt it; the `DefaultCache` path is unchanged (it already scanned the
  closer exactly once). This also removes the same latent double-scan from the `Unclosed` fix
  above, which shipped the identical push-back pattern.

- **Unterminated delimited shape parsers now report the opener as `Unclosed` through the
  emitter, matching the many-builders.** The delimited shape parsers (`delimited::<D>`,
  `parens`, `braces`, `brackets`, `angles`, and their `try_` twins) used to raise a plain
  unexpected-token / end-of-input error when the closing delimiter never arrived; they now
  follow the same four-way close-miss law as the delimited many-builders above. End of input
  with the opener still open emits an `Unclosed` diagnostic carrying the **opener's span** and
  the delimiter pair's name (a fail-fast `Fatal` emitter fails with it; a recovering `Verbose`
  emitter records it and recovers, yielding the construct with a closer synthesized at the
  insertion point — a zero-width span); a wrong token where the closer belongs stays the
  existing unexpected-token (expected-close) diagnostic, not `Unclosed`; and a terminal scanner
  stop surfaces the committed form's end-of-input error, adding no `Unclosed`. The `try_` twins
  keep their decline law (absent opener ⇒ `Ok(None)`, zero consumption) and, once the opener is
  committed, report `Unclosed` on an unterminated group — never a silent decline.

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
- The delimited **shape** parsers (`delimited`/`parens`/`braces`/`brackets`/`angles` and their
  `try_` twins) gained the same two bounds as the many-builders — `Ctx::Emitter:
  UnclosedEmitter<'inp, L, Lang>` and `<Ctx::Emitter as Emitter<…>>::Error: From<Unclosed<(),
  L::Span, Lang>>`. Source-breaking for shape-parser consumers whose emitter or error types do
  not already satisfy them, on the same footing as the many-builder change above.

### Migration

- Add a `From<Unclosed<…>>` arm to any error type used with `.delimited::<D>().collect()`, e.g.
  `impl<D, S, Lang: ?Sized> From<Unclosed<D, S, Lang>> for MyError { … }`. See the `json`
  example's `JsonError::Unclosed` arm for a worked pattern.
- If you use a custom emitter (not `Fatal`/`Verbose`/`Silent`/`Ignored`), implement
  `UnclosedEmitter` for it, mirroring your `FullContainerEmitter` impl: a fail-fast emitter
  converts the `Unclosed` to `Err` via `From`; a recovering emitter records it on its diagnostic
  log and returns `Ok(())`; a dropping emitter returns `Ok(())`.
- The same two migration steps apply to the delimited **shape** parsers (`delimited`/`parens`/
  `braces`/`brackets`/`angles` and their `try_` twins): add a `From<Unclosed<…>>` arm to the
  error type, and implement `UnclosedEmitter` for a custom emitter.
