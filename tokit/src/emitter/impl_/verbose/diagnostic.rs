//! The read-side rendering bridge for [`Verbose`](super::Verbose): a borrowing
//! [`Diagnostic`] view, its [`DiagnosticKind`] payload tag, and the [`Diagnostics`] iterator
//! that replays every recorded channel — errors, warnings, and recovery holes — in emission
//! order.
//!
//! These types are deliberately renderer-agnostic — they borrow straight out of the emitter's
//! storage and carry no formatting policy — so a downstream adapter (ariadne, miette, a bespoke
//! reporter) can map each entry onto its own report kind without tokit taking on any dependency.

use std::{collections::BTreeMap, vec::Vec};

use derive_more::{IsVariant, TryUnwrap, Unwrap};

use super::Channel;
use crate::emitter::Severity;

/// What one collected [`Diagnostic`] carries — the record kind and its payload.
///
/// The two payload channels carry a borrowed error value ([`Error`](Self::Error) /
/// [`Warning`](Self::Warning)); the recovery-hole channel carries only the skipped-token count
/// ([`SkippedRegion`](Self::SkippedRegion)), since a hole has no error value. The span and the
/// captured label snapshot are common to every kind and live on the outer [`Diagnostic`] rather
/// than being duplicated into each variant.
#[derive(Debug, IsVariant, Unwrap, TryUnwrap)]
pub enum DiagnosticKind<'a, E> {
  /// A hard error payload — the [`Severity::Error`] channel.
  Error(&'a E),
  /// A soft warning payload — the [`Severity::Warning`] channel.
  Warning(&'a E),
  /// A recovery hole recorded by [`emit_skipped_region`](crate::Emitter::emit_skipped_region):
  /// the count of tokens a balanced skip discarded. Payload-less — the span it covers is on the
  /// outer [`Diagnostic`].
  SkippedRegion(usize),
}

// Hand-written `Copy`/`Clone` (not derived): the variants hold only `&'a E` and `usize`, both of
// which are `Copy` for *every* `E`, so the type is unconditionally `Copy`. `#[derive(Copy,
// Clone)]` would instead emit a spurious `E: Copy`/`E: Clone` bound (the classic derive-adds-a-
// bound-it-does-not-need trap), which would in turn force that bound onto [`Diagnostic::kind`].
impl<E> Clone for DiagnosticKind<'_, E> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    *self
  }
}

impl<E> Copy for DiagnosticKind<'_, E> {}

/// A borrowing, read-side view of a single collected diagnostic.
///
/// Yielded by [`Verbose::diagnostics`](super::Verbose::diagnostics), a `Diagnostic` bundles the
/// facts a renderer needs about one entry — its source [`span`](Self::span), the *"while parsing
/// X"* [`labels`](Self::labels) that were open when it was emitted, and its
/// [`kind`](Self::kind) (the record kind plus payload) — all as borrows into the emitter, so
/// building the view allocates nothing. [`severity`](Self::severity) and
/// [`payload`](Self::payload) are convenience projections of [`kind`](Self::kind).
#[derive(Debug)]
pub struct Diagnostic<'a, S, E> {
  span: &'a S,
  labels: &'a [&'static str],
  kind: DiagnosticKind<'a, E>,
}

// Hand-written `Copy`/`Clone` for the same reason as [`DiagnosticKind`]: every field is `Copy`
// for all `S`/`E` (`&S`, `&[..]`, and the `Copy` `DiagnosticKind`), so deriving would attach
// spurious `S: Copy`/`E: Copy` bounds.
impl<S, E> Clone for Diagnostic<'_, S, E> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    *self
  }
}

impl<S, E> Copy for Diagnostic<'_, S, E> {}

impl<'a, S, E> Diagnostic<'a, S, E> {
  /// Bundles the borrowed facts of one collected diagnostic.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(
    span: &'a S,
    labels: &'a [&'static str],
    kind: DiagnosticKind<'a, E>,
  ) -> Self {
    Self { span, labels, kind }
  }

  /// The source span of this diagnostic.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> &'a S {
    self.span
  }

  /// The open-label snapshot captured when this diagnostic was emitted, outermost
  /// [`labelled`](crate::labelled) context first. Empty when the emission was unlabelled.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn labels(&self) -> &'a [&'static str] {
    self.labels
  }

  /// The record kind and payload — the primary way to tell an error, a warning, and a recovery
  /// hole apart and read each one's data.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn kind(&self) -> DiagnosticKind<'a, E> {
    self.kind
  }

  /// The [`Severity`] tier of this diagnostic.
  ///
  /// An error reports [`Severity::Error`]; a warning reports [`Severity::Warning`]. A
  /// [`SkippedRegion`](DiagnosticKind::SkippedRegion) is a soft, non-fatal recovery event and so
  /// also reports [`Severity::Warning`] — use [`kind`](Self::kind) to distinguish a hole from a
  /// genuine warning payload.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn severity(&self) -> Severity {
    match self.kind {
      DiagnosticKind::Error(_) => Severity::Error,
      DiagnosticKind::Warning(_) | DiagnosticKind::SkippedRegion(_) => Severity::Warning,
    }
  }

  /// The collected error/warning payload, or `None` for a
  /// [`SkippedRegion`](DiagnosticKind::SkippedRegion) hole (which carries a skipped-token count,
  /// not an error value — read it via [`kind`](Self::kind)).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn payload(&self) -> Option<&'a E> {
    match self.kind {
      DiagnosticKind::Error(e) | DiagnosticKind::Warning(e) => Some(e),
      DiagnosticKind::SkippedRegion(_) => None,
    }
  }
}

/// An iterator over every collected diagnostic of a [`Verbose`](super::Verbose) emitter — the
/// error and warning channels **and** the recovery-hole channel — in true emission order.
///
/// Constructed by [`Verbose::diagnostics`](super::Verbose::diagnostics). It walks the emitter's
/// shared emission `log`; each log entry names a channel and a span, and the iterator hands back
/// the matching payload and label snapshot from that channel as a [`Diagnostic`] tagged with its
/// [`DiagnosticKind`]. A per-channel, per-span cursor tracks how far into each span's group the
/// walk has advanced, so same-span records come out in the order they were emitted. The result
/// is the three span-keyed maps *interleaved* on one timeline — the ordering a renderer wants and
/// that no single map can express alone.
#[derive(Debug)]
pub struct Diagnostics<'a, S, E> {
  log: &'a [(Channel, S)],
  errs: &'a BTreeMap<S, Vec<E>>,
  err_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
  warns: &'a BTreeMap<S, Vec<E>>,
  warn_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
  holes: &'a BTreeMap<S, Vec<usize>>,
  hole_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
  index: usize,
  err_cursor: BTreeMap<&'a S, usize>,
  warn_cursor: BTreeMap<&'a S, usize>,
  hole_cursor: BTreeMap<&'a S, usize>,
}

impl<'a, S, E> Diagnostics<'a, S, E> {
  /// Builds the iterator from the emitter's channels and shared log.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn new(
    log: &'a [(Channel, S)],
    errs: &'a BTreeMap<S, Vec<E>>,
    err_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
    warns: &'a BTreeMap<S, Vec<E>>,
    warn_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
    holes: &'a BTreeMap<S, Vec<usize>>,
    hole_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
  ) -> Self {
    Self {
      log,
      errs,
      err_labels,
      warns,
      warn_labels,
      holes,
      hole_labels,
      index: 0,
      err_cursor: BTreeMap::new(),
      warn_cursor: BTreeMap::new(),
      hole_cursor: BTreeMap::new(),
    }
  }
}

impl<'a, S, E> Iterator for Diagnostics<'a, S, E>
where
  S: Ord,
{
  type Item = Diagnostic<'a, S, E>;

  #[cfg_attr(not(tarpaulin), inline)]
  fn next(&mut self) -> Option<Self::Item> {
    let (channel, span) = self.log.get(self.index)?;
    self.index += 1;

    // The `Channel` tag routes to the maps this entry was recorded in; the per-channel cursor
    // advances one step into this span's group, mirroring how `record`/`record_warning`/
    // `record_hole` appended it. Group index == prior same-span emissions in this channel, so
    // the three timelines interleave exactly as they were emitted.
    match *channel {
      Channel::Diagnostic(severity) => {
        let (groups, labels, cursor) = match severity {
          Severity::Error => (self.errs, self.err_labels, &mut self.err_cursor),
          Severity::Warning => (self.warns, self.warn_labels, &mut self.warn_cursor),
        };
        let slot = cursor.entry(span).or_insert(0);
        let idx = *slot;
        *slot += 1;

        let payload = &groups[span][idx];
        let labels = labels[span][idx].as_slice();
        let kind = match severity {
          Severity::Error => DiagnosticKind::Error(payload),
          Severity::Warning => DiagnosticKind::Warning(payload),
        };
        Some(Diagnostic::new(span, labels, kind))
      }
      Channel::SkippedRegion => {
        let slot = self.hole_cursor.entry(span).or_insert(0);
        let idx = *slot;
        *slot += 1;

        let skipped = self.holes[span][idx];
        let labels = self.hole_labels[span][idx].as_slice();
        Some(Diagnostic::new(
          span,
          labels,
          DiagnosticKind::SkippedRegion(skipped),
        ))
      }
    }
  }
}
