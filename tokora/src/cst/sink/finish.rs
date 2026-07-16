//! Materialization: the one place the buffered event stream becomes a rowan green tree —
//! losslessness and no-duplication enforced as **one function**, never panicking.
//!
//! The walk validates as it drives the builder: balance (an orphan finish or a leftover
//! open is a typed error — rowan's silent one-level absorb under the root wrapper is
//! unreachable, because the sink's own stack refuses first), retro-wrap integrity (stale
//! `StartAt` targets, dangling `forward_parent` pointers — the journal's finish-time
//! canary), kind hygiene (the reserved tombstone band), span discipline (monotone,
//! non-overlapping, in-bounds, u32-fitting), and **gap tiling**: every source byte no
//! committed token covers becomes a `gap_kind` token, which is what makes
//! `tree.text() == source` structural for every input — poisoned, error-bearing, and
//! truncated parses included.

use std::{collections::BTreeMap, vec::Vec};

use rowan::{GreenNode, GreenNodeBuilder, SyntaxKind};

use crate::{Lexer, span::Span};

use super::{
  super::event::{Event, TOMBSTONE},
  CstSink, TriviaPolicy,
};

/// Why a materialization was refused. Every variant names the offending **event index**
/// (the buffer position of the event that broke the law), so the failure is diagnosable
/// against the recorded stream without exposing the stream itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CstFinishError {
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

impl<'inp, L, E> CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
{
  /// Materializes the buffered events into a green tree wrapped in `root_kind`, returning
  /// the inner emitter either way — the sink is consumed exactly once, and the
  /// diagnostics survive the tree.
  ///
  /// The replay validates and builds in one walk: balance,
  /// retro-wrap integrity, kind hygiene, span discipline — and **tiles every uncovered
  /// source byte as a `gap_kind` token**, so on success `tree.text() == source` holds for
  /// every input, lexer errors and truncated tails included. On the first violation the
  /// half-built green state is dropped and a typed [`CstFinishError`] comes back instead;
  /// this method **never panics**.
  ///
  /// # Abort semantics
  ///
  /// - An `Incomplete` parse (needs more input) should not be materialized: keep the sink
  ///   — the buffered events *are* the resumable state.
  /// - A fatal abort leaves open nodes; `finish` refuses them
  ///   ([`CstFinishError::UnclosedNodes`]) — [`finish_partial`](Self::finish_partial) is
  ///   the explicit opt-in that closes them for tooling.
  pub fn finish(self, root_kind: u16, source: &str) -> (Result<GreenNode, CstFinishError>, E)
  where
    L::Offset: TryInto<u32>,
  {
    self.materialize(root_kind, source, false)
  }

  /// [`finish`](Self::finish), but open nodes at the end of the stream are **closed**
  /// instead of refused — the apollo-style partial tree for tooling that wants to inspect
  /// a fatally-aborted parse. Every other law (balance underflow, wrap integrity, span
  /// discipline, gap tiling) is enforced identically; the round-trip law holds on the
  /// partial tree too.
  pub fn finish_partial(
    self,
    root_kind: u16,
    source: &str,
  ) -> (Result<GreenNode, CstFinishError>, E)
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
  ) -> (Result<GreenNode, CstFinishError>, E)
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
fn offset_to_u32<O>(offset: O, index: u64) -> Result<u32, CstFinishError>
where
  O: TryInto<u32>,
{
  offset
    .try_into()
    .map_err(|_| CstFinishError::OffsetOverflow { index })
}

/// The validating replay: one forward walk over the surviving events, driving the green
/// builder only with already-checked operations (so rowan can never panic under it).
fn replay<S>(
  events: &[Event<S>],
  root_kind: u16,
  gap_kind: u16,
  source: &str,
  close_open_nodes: bool,
) -> Result<GreenNode, CstFinishError>
where
  S: Span,
  S::Offset: TryInto<u32>,
{
  if root_kind == TOMBSTONE {
    return Err(CstFinishError::ReservedRootKind);
  }

  // Pre-pass: group the retro-wraps by target (reverse buffer order opens outermost
  // first), validating targets and the forward_parent canaries.
  let mut wraps: BTreeMap<u64, Vec<(u64, u16)>> = BTreeMap::new();
  for (index, event) in events.iter().enumerate() {
    let index = index as u64;
    match event {
      Event::StartAt { kind, target } => {
        if *kind == TOMBSTONE {
          return Err(CstFinishError::ReservedKind { index });
        }
        let live = *target < index && events[*target as usize].is_tombstone();
        if !live {
          return Err(CstFinishError::StaleStartAt {
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
          return Err(CstFinishError::DanglingForwardParent { index });
        }
      }
      Event::StartNode { kind, .. } if *kind == TOMBSTONE => {}
      Event::StartNode { kind, .. } | Event::Token { kind, .. } => {
        if *kind == TOMBSTONE {
          return Err(CstFinishError::ReservedKind { index });
        }
      }
      Event::FinishNode | Event::Diag { .. } => {}
    }
  }

  let mut builder = GreenNodeBuilder::new();
  let mut stack: Vec<Frame> = Vec::new();
  builder.start_node(SyntaxKind(root_kind));
  stack.push(Frame::Root);

  // The tiling cursor: the end of the last covered source byte, in u32 space.
  let mut covered: u32 = 0;
  let source_len =
    u32::try_from(source.len()).map_err(|_| CstFinishError::OffsetOverflow { index: 0 })?;

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
        builder.start_node(SyntaxKind(*kind));
        stack.push(Frame::Start);
      }
      Event::Token { kind, span } => {
        let start = offset_to_u32(span.start(), index)?;
        let end = offset_to_u32(span.end(), index)?;
        if start < covered || end < start {
          return Err(CstFinishError::OverlappingSpans { index });
        }
        if end > source_len {
          return Err(CstFinishError::SpanOutOfBounds { index });
        }
        // Tile the gap this token reveals: bytes no committed token covered (a skipped
        // lexer error, an undrained region) become one gap token in the currently open
        // node — losslessness by construction, not by lexer luck.
        if start > covered {
          let gap = source
            .get(covered as usize..start as usize)
            .ok_or(CstFinishError::SpanOutOfBounds { index })?;
          builder.token(SyntaxKind(gap_kind), gap);
        }
        let text = source
          .get(start as usize..end as usize)
          .ok_or(CstFinishError::SpanOutOfBounds { index })?;
        builder.token(SyntaxKind(*kind), text);
        covered = end;
      }
      Event::FinishNode => {
        match stack.last() {
          None | Some(Frame::Root) => {
            // The sink's own wall: rowan would silently absorb one level of imbalance
            // under the root wrapper; the walk refuses before the builder sees it.
            return Err(CstFinishError::OrphanFinish { index });
          }
          Some(Frame::Wrap(start_at)) => {
            // A hoisted wrap may only close at or after its declaration: closing
            // earlier means the wrap crosses a node boundary (the mark was taken
            // inside a node that closed before the wrap was declared).
            if *start_at > index {
              return Err(CstFinishError::ImproperWrap {
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
      .ok_or(CstFinishError::SpanOutOfBounds {
        index: events.len() as u64,
      })?;
    builder.token(SyntaxKind(gap_kind), gap);
  }

  // Balance at the end: everything but the root must have closed. (The root frame is
  // always present here — the orphan wall above refuses every pop that could reach it —
  // but `finish` promises to never panic, so the arithmetic saturates instead of
  // assuming.)
  let open = (stack.len() as u64).saturating_sub(1);
  if open > 0 {
    if !close_open_nodes {
      return Err(CstFinishError::UnclosedNodes { open });
    }
    for _ in 0..open {
      builder.finish_node();
    }
  }

  builder.finish_node();
  Ok(builder.finish())
}
