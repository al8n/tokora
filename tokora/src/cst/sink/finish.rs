//! Materialization: the one place the buffered event stream becomes a rowan green tree —
//! losslessness and no-duplication enforced as **one function**, never panicking.
//!
//! The walk validates as it drives the builder: balance (an orphan finish or a leftover
//! open is a typed error — rowan's silent one-level absorb under the root wrapper is
//! unreachable, because the sink's own stack refuses first), retro-wrap integrity (stale
//! `StartAt` targets, dangling `forward_parent` pointers — the journal's finish-time
//! canary), kind hygiene (the reserved tombstone band), span discipline (monotone,
//! non-overlapping, in-bounds, u32-fitting), the **token-channel wall** (a balanced
//! stream that builds structure without one committed token over a nonempty source is
//! the half-forwarding-wrapper signature, refused instead of dressed up by tiling — see
//! [`FinishError::StructureWithoutTokens`]), and **gap tiling**: every source byte no
//! committed token covers becomes a `gap_kind` token, which is what makes
//! `tree.text() == source` structural for every input — poisoned, error-bearing, and
//! truncated parses included.
//!
//! # The gap-coverage law (and the one deliberate channel coupling)
//!
//! Gap tiling is not unconditional: a tiled byte must be *explained*. `finish` tiles a gap
//! only where a **recorded lexer-error diagnostic** covers it (the lexer saw bytes it could
//! not tokenize, said so, and committed no token there); a gap with no covering error and no
//! covering token is a **dropped committed token** — the partial-forwarding-wrapper signature
//! the zero-token wall cannot see — and is refused ([`FinishError::UncoveredGap`]). This is
//! the sink's one **deliberate coupling of the diagnostic and CST channels**: elsewhere they
//! are independent (a `Diag` slot is invisible to the tree), but at `finish` a lexer error's
//! *span* is what licenses its gap. The audit kept the channels separate; this law crosses
//! them on purpose, and only at materialization. [`finish_partial`](Sink::finish_partial)
//! is exempt — it tiles every gap, tolerating an incomplete parse the way it tolerates open
//! nodes (see its own boundary note on the fail-fast emitter case).

use std::{collections::BTreeMap, vec::Vec};

use rowan::{GreenNode, GreenNodeBuilder, SyntaxKind};

use crate::{Lexer, span::Span};

use super::{
  super::event::{Event, TOMBSTONE},
  Sink, TriviaPolicy,
};

/// Why a materialization was refused. Every variant names the offending **event index**
/// (the buffer position of the event that broke the law), so the failure is diagnosable
/// against the recorded stream without exposing the stream itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FinishError {
  /// A `FinishNode` arrived with no node open — the orphan-finish shape (a start rolled
  /// back apart from its finish). Under a plain rowan root wrapper this imbalance would
  /// be silently absorbed; here it is refused before the builder ever sees it.
  #[error(
    "finish event at index {index} closes no open node (its start was rolled back or never \
     emitted)"
  )]
  OrphanFinish {
    /// The buffer index of the orphan `FinishNode`.
    index: u64,
  },

  /// The walk ended with nodes still open — a fatal abort, an unguarded unwind, or a raw
  /// bracketing bug. `finish` refuses; `finish_partial` closes them instead (the explicit
  /// tooling opt-in).
  #[error("{open} node(s) still open at the end of the event stream")]
  UnclosedNodes {
    /// How many starts never saw their finish.
    open: u64,
  },

  /// A `StartAt`'s target is not a live tombstone — out of bounds, or the slot holds a
  /// different event. Unreachable through the validated emission surface (marks panic at
  /// spend time); refused here as the release backstop.
  #[error("retro-wrap at index {index} targets {target}, which is not a live tombstone")]
  StaleStartAt {
    /// The buffer index of the `StartAt` event.
    index: u64,
    /// The target it names.
    target: u64,
  },

  /// A tombstone's `forward_parent` pointer does not name a `StartAt` targeting it — the
  /// dangling-pointer shape of an abandoned wrap that escaped the undo journal. With the
  /// journal reverse-replayed on every rewind this is unreachable; it is checked anyway,
  /// because the silent alternative is a stolen start.
  #[error(
    "tombstone at index {index} carries a forward_parent that names no retro-wrap of it \
     (an abandoned wrap escaped the undo journal)"
  )]
  DanglingForwardParent {
    /// The buffer index of the corrupt tombstone.
    index: u64,
  },

  /// A finish would close a retro-wrap before the buffer position of the `StartAt` that
  /// declared it — the wrap crosses a node boundary instead of enclosing whole subtrees
  /// (a mark taken inside a node, wrapped after the node closed).
  #[error(
    "finish at index {finish} closes the retro-wrap declared at index {start_at} before \
     its declaration (the wrap crosses a node boundary)"
  )]
  ImproperWrap {
    /// The buffer index of the `StartAt` whose node was closed too early.
    start_at: u64,
    /// The buffer index of the offending finish.
    finish: u64,
  },

  /// An event carries the reserved tombstone kind (`u16::MAX`) where a real kind is
  /// required — the dialect mapper or a raw caller leaked the reserved band. The
  /// emission-time debug assert is the detect-at-cause form; this is the release wall.
  #[error("event at index {index} carries the reserved tombstone kind (u16::MAX)")]
  ReservedKind {
    /// The buffer index of the offending event.
    index: u64,
  },

  /// The dialect root kind itself is the reserved tombstone kind.
  #[error("the root kind is the reserved tombstone kind (u16::MAX)")]
  ReservedRootKind,

  /// The stream builds structure but carries **no committed token** over a nonempty
  /// source — the half-forwarding-wrapper signature. A wrapper emitter that forwards the
  /// [`CstEmitter`](crate::emitter::CstEmitter) structuring surface (satisfying the
  /// `node()` bound) but inherits the defaulted no-op
  /// [`Emitter::commit_token`](crate::emitter::Emitter::commit_token) produces exactly
  /// this shape: the parse succeeds, every structuring event flows, and every committed
  /// token silently vanishes between the input layer and the sink. Gap tiling would
  /// happily return a *plausible* tree (full text, empty nodes), so `finish` refuses
  /// instead — the loud failure the wrapper contract promises. A parse that legitimately
  /// consumed nothing builds either no structure (nothing to refuse) or a tree over an
  /// empty source (also not refused); a fatally-aborted parse inspected through
  /// [`finish_partial`](Sink::finish_partial) still has its open nodes as the abort
  /// witness and is likewise not refused.
  #[error(
    "the event stream builds structure but carries no committed token over a nonempty \
     source (a wrapper emitter forwarding the CstEmitter structuring surface without \
     Emitter::commit_token produces exactly this shape)"
  )]
  StructureWithoutTokens,

  /// A run of source bytes that **no committed token covers and no recorded lexer-error
  /// diagnostic explains** — an *unexplained gap*. Gap tiling makes `tree.text() == source`
  /// structural, but only bytes a lexer legitimately refused (a diagnostic was recorded, with
  /// this span, and no token settled there) may become `gap_kind` tokens. A byte no token and
  /// no error accounts for is a **dropped committed token** — the partial-forwarding-wrapper
  /// signature the zero-token wall ([`StructureWithoutTokens`](Self::StructureWithoutTokens))
  /// cannot see because a token *did* survive — or, under a fail-fast emitter, an unconsumed
  /// tail an abort left un-diagnosed. `finish` refuses it rather than tile a plausible-but-lossy
  /// tree; [`finish_partial`](Sink::finish_partial) tiles it (the tooling door that tolerates
  /// an incomplete parse). The first (leftmost) uncovered run is named.
  #[error(
    "source bytes {start}..{end} are covered by neither a committed token nor a recorded \
     lexer-error diagnostic (a dropped committed token, or an un-diagnosed unconsumed region)"
  )]
  UncoveredGap {
    /// The first uncovered byte offset.
    start: u32,
    /// The end (exclusive) of the first maximal uncovered run.
    end: u32,
  },

  /// A token span starts before the end of the previous token — a double emission or a
  /// non-monotone stream. Rejecting it here is what makes the no-duplication half of the
  /// round-trip law structural.
  #[error("token at index {index} overlaps the previous token's span")]
  OverlappingSpans {
    /// The buffer index of the offending token event.
    index: u64,
  },

  /// A token offset does not fit rowan's `u32` text size. Nothing is truncated; the
  /// materialization is refused whole.
  #[error("token at index {index} has an offset beyond u32::MAX (rowan text sizes are u32)")]
  OffsetOverflow {
    /// The buffer index of the offending token event.
    index: u64,
  },

  /// A token span does not slice the given source (beyond its end, or off a UTF-8
  /// boundary) — the events and the source disagree.
  #[error("token at index {index} does not slice the given source")]
  SpanOutOfBounds {
    /// The buffer index of the offending token event.
    index: u64,
  },
}

/// One open node during the replay walk: a direct start, a hoisted retro-wrap (carrying
/// the buffer index of its `StartAt` declaration), or the synthetic dialect root.
enum Frame {
  /// The dialect root wrapper.
  Root,
  /// A direct `StartNode`.
  Start,
  /// A retro-wrap hoisted to its target's position; the payload is the `StartAt`'s own
  /// buffer index (a finish may close it only at or after that position).
  Wrap(u64),
}

impl<'inp, L, E> Sink<'inp, L, E>
where
  L: Lexer<'inp>,
{
  /// Materializes the buffered events into a green tree wrapped in `root_kind`, returning
  /// the inner emitter either way — the sink is consumed exactly once, and the
  /// diagnostics survive the tree.
  ///
  /// The replay validates and builds in one walk: balance,
  /// retro-wrap integrity, kind hygiene, span discipline, the token-channel wall
  /// ([`FinishError::StructureWithoutTokens`] — structure with zero committed tokens
  /// over a nonempty source is a severed `commit_token` channel, not a tree), the
  /// **gap-coverage law** (every uncovered byte tiles as a `gap_kind` token only where a
  /// recorded lexer error explains it; an unexplained gap is a dropped committed token —
  /// [`FinishError::UncoveredGap`]) — so on success `tree.text() == source` holds and
  /// every gap in it is a byte the lexer legitimately refused. On the first violation the
  /// half-built green state is dropped and a typed [`FinishError`] comes back instead;
  /// this method **never panics**.
  ///
  /// # Abort semantics
  ///
  /// - An `Incomplete` parse (needs more input) should not be materialized: keep the sink
  ///   — the buffered events *are* the resumable state.
  /// - A fatal abort leaves open nodes; `finish` refuses them
  ///   ([`FinishError::UnclosedNodes`]) — [`finish_partial`](Self::finish_partial) is
  ///   the explicit opt-in that closes them for tooling.
  ///
  /// # The fail-fast boundary (precise)
  ///
  /// The gap-coverage guarantee holds for a **collecting** inner emitter (the lossless
  /// case): every lexer error is recorded, with its span, before the parse moves on, so
  /// every refused byte is explained and `finish` succeeds. Under a **fail-fast** emitter
  /// ([`Fatal`](crate::emitter::Fatal)) the first lexer error aborts the parse, so the bytes
  /// past it are never lexed and no diagnostic covers them: `finish` then refuses (an
  /// [`UncoveredGap`](FinishError::UncoveredGap), or [`UnclosedNodes`](FinishError::UnclosedNodes)
  /// if the abort left a node open). That is by design — inspect such a partial parse through
  /// [`finish_partial`](Self::finish_partial), which tiles the un-diagnosed tail, or accept no
  /// tree. The guarantee is stated only for the collecting case for exactly this reason.
  pub fn finish(self, root_kind: u16, source: &str) -> (Result<GreenNode, FinishError>, E)
  where
    L::Offset: TryInto<u32>,
  {
    self.materialize(root_kind, source, false)
  }

  /// [`finish`](Self::finish), but the two **incompleteness** signals a partial parse leaves
  /// are tolerated instead of refused, so tooling can inspect a fatally-aborted or truncated
  /// parse: open nodes at the end of the stream are **closed** (not
  /// [`UnclosedNodes`](FinishError::UnclosedNodes)), and an uncovered gap is **tiled** (not
  /// [`UncoveredGap`](FinishError::UncoveredGap)) — the un-diagnosed tail of a fail-fast
  /// abort becomes one `gap_kind` run so `tree.text() == source` still holds. Every other law
  /// (balance underflow, wrap integrity, span discipline, and the token-channel wall for
  /// *balanced* streams — the zero-token severance is corruption, not mere incompleteness) is
  /// enforced identically. The exemptions are the two ways an incomplete parse differs from a
  /// complete one; refusing them would defeat the door.
  pub fn finish_partial(self, root_kind: u16, source: &str) -> (Result<GreenNode, FinishError>, E)
  where
    L::Offset: TryInto<u32>,
  {
    self.materialize(root_kind, source, true)
  }

  /// The one replay walk behind [`finish`](Self::finish) and
  /// [`finish_partial`](Self::finish_partial).
  fn materialize(
    self,
    root_kind: u16,
    source: &str,
    close_open_nodes: bool,
  ) -> (Result<GreenNode, FinishError>, E)
  where
    L::Offset: TryInto<u32>,
  {
    let events = self.events;
    let gap_kind = self.gap_kind;
    let inner = self.inner;
    // TriviaPolicy::AsEmitted is the only variant today: the replay below IS that policy
    // (tokens land in whichever node is open at their buffer position).
    let TriviaPolicy::AsEmitted = self.trivia;

    let result = replay(&events, root_kind, gap_kind, source, close_open_nodes);
    (result, inner)
  }
}

/// Converts one span endpoint, mapping failures to the event's typed error.
fn offset_to_u32<O>(offset: O, index: u64) -> Result<u32, FinishError>
where
  O: TryInto<u32>,
{
  offset
    .try_into()
    .map_err(|_| FinishError::OffsetOverflow { index })
}

/// Records the leftmost source run in `[lo, hi)` that no `cover` interval spans, keeping the
/// first one found across all tiled gaps (the most useful diagnostic, and the reason the
/// tiling order — interior gaps in source order, then the trailing gap — matters). `cover`
/// is the sorted, non-overlapping merge of the recorded lexer-error spans.
fn record_uncovered(lo: u32, hi: u32, cover: &[(u32, u32)], first: &mut Option<(u32, u32)>) {
  if first.is_none() {
    *first = first_uncovered(lo, hi, cover);
  }
}

/// The first sub-range of `[lo, hi)` that the sorted, non-overlapping `cover` intervals do
/// not span, or `None` when `[lo, hi)` is fully covered (or empty).
fn first_uncovered(lo: u32, hi: u32, cover: &[(u32, u32)]) -> Option<(u32, u32)> {
  if lo >= hi {
    return None;
  }
  let mut cursor = lo;
  for &(s, e) in cover {
    if e <= cursor {
      continue; // interval entirely before the uncovered cursor
    }
    if s > cursor {
      return Some((cursor, s.min(hi))); // a hole precedes this interval
    }
    cursor = cursor.max(e);
    if cursor >= hi {
      return None; // fully covered
    }
  }
  Some((cursor, hi))
}

/// The validating replay: one forward walk over the surviving events, driving the green
/// builder only with already-checked operations (so rowan can never panic under it).
fn replay<S>(
  events: &[Event<S>],
  root_kind: u16,
  gap_kind: u16,
  source: &str,
  close_open_nodes: bool,
) -> Result<GreenNode, FinishError>
where
  S: Span,
  S::Offset: TryInto<u32>,
{
  if root_kind == TOMBSTONE {
    return Err(FinishError::ReservedRootKind);
  }

  let source_len =
    u32::try_from(source.len()).map_err(|_| FinishError::OffsetOverflow { index: 0 })?;

  // Pre-pass: group the retro-wraps by target (reverse buffer order opens outermost
  // first), validating targets and the forward_parent canaries — and collect the recorded
  // lexer-error spans (the only diagnostic that legitimately explains an uncovered byte)
  // for the gap-coverage law below.
  let mut wraps: BTreeMap<u64, Vec<(u64, u16)>> = BTreeMap::new();
  let mut error_spans: Vec<(u32, u32)> = Vec::new();
  for (index, event) in events.iter().enumerate() {
    let index = index as u64;
    match event {
      Event::StartAt { kind, target } => {
        if *kind == TOMBSTONE {
          return Err(FinishError::ReservedKind { index });
        }
        let live = *target < index && events[*target as usize].is_tombstone();
        if !live {
          return Err(FinishError::StaleStartAt {
            index,
            target: *target,
          });
        }
        wraps.entry(*target).or_default().push((index, *kind));
      }
      Event::StartNode {
        kind: TOMBSTONE,
        forward_parent: Some(relative),
      } => {
        // The journal-integrity canary: a set pointer must name a StartAt of this
        // tombstone. A dangling pointer is the un-journaled abandoned wrap (F-A2/F-A3's
        // silent corruption), surfaced as a typed error instead of a stolen start.
        let target = index;
        let named = target + u64::from(relative.get());
        let names_this = matches!(
          events.get(named as usize),
          Some(Event::StartAt { target: t, .. }) if *t == target
        );
        if !names_this {
          return Err(FinishError::DanglingForwardParent { index });
        }
      }
      Event::StartNode { kind, .. } if *kind == TOMBSTONE => {}
      Event::StartNode { kind, .. } | Event::Token { kind, .. } => {
        if *kind == TOMBSTONE {
          return Err(FinishError::ReservedKind { index });
        }
      }
      Event::Diag {
        error_span: Some(span),
        ..
      } => {
        // A lexer error's span covers the bytes it refused. Clamp each endpoint to the
        // source: an out-of-source or u32-overflowing endpoint saturates to the source end,
        // so a malformed span can never over-cover a real gap.
        let to_u32 = |o: S::Offset| {
          o.try_into()
            .map(|v: u32| v.min(source_len))
            .unwrap_or(source_len)
        };
        let start = to_u32(span.start());
        let end = to_u32(span.end());
        if start < end {
          error_spans.push((start, end));
        }
      }
      Event::FinishNode | Event::Diag { .. } => {}
    }
  }

  // Merge the lexer-error spans into sorted, non-overlapping intervals so the coverage
  // check below is one linear sweep per gap.
  error_spans.sort_unstable();
  let mut error_cover: Vec<(u32, u32)> = Vec::with_capacity(error_spans.len());
  for (s, e) in error_spans {
    match error_cover.last_mut() {
      Some((_, last_end)) if s <= *last_end => *last_end = (*last_end).max(e),
      _ => error_cover.push((s, e)),
    }
  }

  let mut builder = GreenNodeBuilder::new();
  let mut stack: Vec<Frame> = Vec::new();
  builder.start_node(SyntaxKind(root_kind));
  stack.push(Frame::Root);

  // The tiling cursor: the end of the last covered source byte, in u32 space.
  let mut covered: u32 = 0;

  // The token-channel witnesses (see `StructureWithoutTokens`): whether any committed
  // token survived to materialization, and whether any real node did. Structure without a
  // single token over a nonempty source is the half-forwarding-wrapper signature — the
  // gap tiling below would otherwise dress it up as a plausible tree.
  let mut saw_token = false;
  let mut saw_structure = false;

  // The leftmost source run no committed token covers and no lexer error explains — the
  // dropped-committed-token signature (see `UncoveredGap`). Recorded during tiling and
  // refused at the end (by `finish`), so the zero-token wall keeps precedence for the
  // all-dropped case.
  let mut first_uncovered_gap: Option<(u32, u32)> = None;

  for (index, event) in events.iter().enumerate() {
    let index = index as u64;
    match event {
      Event::StartNode {
        kind: TOMBSTONE, ..
      } => {
        // An inert mark — unless retro-wraps target it: they open HERE, latest first
        // (the later wrap's finish comes later, so it is the outer node).
        if let Some(targeting) = wraps.get(&index) {
          for (start_at, kind) in targeting.iter().rev() {
            builder.start_node(SyntaxKind(*kind));
            stack.push(Frame::Wrap(*start_at));
          }
        }
      }
      Event::StartNode { kind, .. } => {
        saw_structure = true;
        builder.start_node(SyntaxKind(*kind));
        stack.push(Frame::Start);
      }
      Event::Token { kind, span } => {
        saw_token = true;
        let start = offset_to_u32(span.start(), index)?;
        let end = offset_to_u32(span.end(), index)?;
        if start < covered || end < start {
          return Err(FinishError::OverlappingSpans { index });
        }
        if end > source_len {
          return Err(FinishError::SpanOutOfBounds { index });
        }
        // Tile the gap this token reveals: bytes no committed token covered (a skipped
        // lexer error, an undrained region) become one gap token in the currently open
        // node — losslessness by construction, not by lexer luck.
        if start > covered {
          let gap = source
            .get(covered as usize..start as usize)
            .ok_or(FinishError::SpanOutOfBounds { index })?;
          record_uncovered(covered, start, &error_cover, &mut first_uncovered_gap);
          builder.token(SyntaxKind(gap_kind), gap);
        }
        let text = source
          .get(start as usize..end as usize)
          .ok_or(FinishError::SpanOutOfBounds { index })?;
        builder.token(SyntaxKind(*kind), text);
        covered = end;
      }
      Event::FinishNode => {
        match stack.last() {
          None | Some(Frame::Root) => {
            // The sink's own wall: rowan would silently absorb one level of imbalance
            // under the root wrapper; the walk refuses before the builder sees it.
            return Err(FinishError::OrphanFinish { index });
          }
          Some(Frame::Wrap(start_at)) => {
            // A hoisted wrap may only close at or after its declaration: closing
            // earlier means the wrap crosses a node boundary (the mark was taken
            // inside a node that closed before the wrap was declared).
            if *start_at > index {
              return Err(FinishError::ImproperWrap {
                start_at: *start_at,
                finish: index,
              });
            }
          }
          Some(Frame::Start) => {}
        }
        stack.pop();
        builder.finish_node();
      }
      Event::StartAt { .. } => {
        // Its node was opened at the target's position (the hoist above); the
        // declaration slot itself is structural silence.
        saw_structure = true;
      }
      Event::Diag { .. } => {
        // A diagnostic order-slot: invisible to the tree.
      }
    }
  }

  // The trailing gap: bytes after the last covered token (an undrained tail, a poisoned
  // truncation) tile into the root.
  if covered < source_len {
    let gap = source
      .get(covered as usize..)
      .ok_or(FinishError::SpanOutOfBounds {
        index: events.len() as u64,
      })?;
    record_uncovered(covered, source_len, &error_cover, &mut first_uncovered_gap);
    builder.token(SyntaxKind(gap_kind), gap);
  }

  // Balance at the end: everything but the root must have closed. (The root frame is
  // always present here — the orphan wall above refuses every pop that could reach it —
  // but `finish` promises to never panic, so the arithmetic saturates instead of
  // assuming.)
  let open = (stack.len() as u64).saturating_sub(1);
  if open > 0 {
    if !close_open_nodes {
      return Err(FinishError::UnclosedNodes { open });
    }
    for _ in 0..open {
      builder.finish_node();
    }
  } else if saw_structure && !saw_token && source_len > 0 {
    // The token-channel wall: a *balanced* stream that builds structure without one
    // committed token over a nonempty source is the half-forwarding-wrapper signature
    // (structuring forwarded, `Emitter::commit_token` inherited as the core no-op), and
    // gap tiling would return a plausible tree instead of a witness. Balanced-only on
    // purpose: a fatally-aborted parse inspected through `finish_partial` still has its
    // open nodes (the `open > 0` arm above), so the abort shape is never refused here.
    // Kept ahead of the gap-coverage law so the all-dropped case earns this precise
    // message rather than an `UncoveredGap` over the whole source.
    return Err(FinishError::StructureWithoutTokens);
  } else if !close_open_nodes {
    // The gap-coverage law, `finish`-only (the partial-drop generalization of the wall
    // above): an uncovered byte with no covering lexer error is a dropped committed token —
    // or, under a fail-fast emitter, an un-diagnosed unconsumed tail. `finish_partial` is
    // exempt and tiles the run, tolerating an incomplete parse as it tolerates open nodes.
    if let Some((start, end)) = first_uncovered_gap {
      return Err(FinishError::UncoveredGap { start, end });
    }
  }

  builder.finish_node();
  Ok(builder.finish())
}
