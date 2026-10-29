/// GATED OFF — the chunked-event-equivalence property (the event-channel twin of this
/// driver's token-stream oracle): a chunked partial parse and the single-shot parse
/// must emit identical event streams.
///
/// **The two prerequisites this was parked on are now MET, and a different one replaced
/// them.** Re-derived rather than re-parked, because a stale gate string is how a deferral
/// becomes permanent:
///
/// - *"the scanner family becoming `Cmpl`-generic"* — **done.** `sync_to`, `sync_through`,
///   `skip_while` and `consume_cached` are all generic over the completeness parameter.
///   (The earlier check for this looked for the bound spelled `Cmpl: Completeness` and found
///   none; the scanners spell it `Cmpl: SurfaceIncomplete`. A census that greps one spelling
///   cannot see another, and its zero was not evidence of absence.)
/// - *"the cache-transparency matrix growing a Partial column"* — **done.**
///   `try_expect_partial_matrix`, `skip_while_partial_matrix`, `sync_to_partial_matrix` and
///   `sync_through_partial_matrix` are the column.
///
/// What blocks it now postdates both. `PartialSession::parse` requires
/// `Ctx::Emitter: ValueKeyedEmitter`, and a recording `Sink` deliberately is not one — so the
/// **session** route to a chunked CST is uncompilable by design, not by omission (see the 0.8.0
/// release notes' *Streaming CST is not in this release*). The supported shape is a
/// hand-rolled refill loop with a **fresh sink per attempt**, and writing this oracle against
/// that shape is the remaining work: it needs a per-attempt sink harness that the suite does
/// not have yet.
///
/// The law it would pin is the one repeatedly circled and stated by no automated check:
/// a chunked replay equals the one-shot parse.
#[test]
#[ignore = "gated: needs a fresh-sink-per-attempt harness. The two originally-named \
            prerequisites (Cmpl-generic scanners, matrix Partial column) are BOTH met; the \
            session route is type-excluded by `PartialSession::parse`'s ValueKeyedEmitter \
            bound, so this must drive a hand-rolled refill loop instead"]
fn chunked_event_equivalence_awaits_the_partial_gate() {
  unimplemented!(
    "drive a hand-rolled refill loop with a FRESH Sink per attempt against a single-shot \
     parse of the same source, and assert identical materialized trees"
  );
}
