//! The read-side rendering bridge for [`Verbose`](super::Verbose): a borrowing
//! [`Diagnostic`] view and the [`Diagnostics`] iterator that replays both diagnostic
//! channels in emission order.
//!
//! These types are deliberately renderer-agnostic — they borrow straight out of the emitter's
//! storage and carry no formatting policy — so a downstream adapter (ariadne, miette, a bespoke
//! reporter) can map each entry onto its own report kind without tokit taking on any dependency.

use std::{collections::BTreeMap, vec::Vec};

use crate::emitter::Severity;

/// A borrowing, read-side view of a single collected diagnostic.
///
/// Yielded by [`Verbose::diagnostics`](super::Verbose::diagnostics), a `Diagnostic` bundles the
/// four facts a renderer needs about one entry — its source [`span`](Self::span), its
/// [`severity`](Self::severity) tier, the *"while parsing X"* [`labels`](Self::labels) that were
/// open when it was emitted, and the [`payload`](Self::payload) (the collected error/warning
/// value) — all as borrows into the emitter, so building the view allocates nothing.
#[derive(Debug, Clone, Copy)]
pub struct Diagnostic<'a, S, E> {
  span: &'a S,
  severity: Severity,
  labels: &'a [&'static str],
  payload: &'a E,
}

impl<'a, S, E> Diagnostic<'a, S, E> {
  /// Bundles the borrowed facts of one collected diagnostic.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(
    span: &'a S,
    severity: Severity,
    labels: &'a [&'static str],
    payload: &'a E,
  ) -> Self {
    Self {
      span,
      severity,
      labels,
      payload,
    }
  }

  /// The source span of this diagnostic.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> &'a S {
    self.span
  }

  /// The [`Severity`] tier of this diagnostic — which channel it came from.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn severity(&self) -> Severity {
    self.severity
  }

  /// The open-label snapshot captured when this diagnostic was emitted, outermost
  /// [`labelled`](crate::labelled) context first. Empty when the emission was unlabelled.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn labels(&self) -> &'a [&'static str] {
    self.labels
  }

  /// The collected payload — the error or warning value recorded at this entry.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn payload(&self) -> &'a E {
    self.payload
  }
}

/// An iterator over every collected diagnostic of a [`Verbose`](super::Verbose) emitter — both
/// the error and warning channels — in true emission order.
///
/// Constructed by [`Verbose::diagnostics`](super::Verbose::diagnostics). It walks the emitter's
/// shared emission `log`; each log entry names a channel (via its [`Severity`] tag) and a span,
/// and the iterator hands back the matching payload and label snapshot from that channel. A
/// per-channel, per-span cursor tracks how far into each span's group the walk has advanced, so
/// same-span diagnostics come out in the order they were emitted. The result is the two
/// span-keyed maps *interleaved* on one timeline — the ordering a renderer wants and that
/// neither map can express alone.
#[derive(Debug)]
pub struct Diagnostics<'a, S, E> {
  log: &'a [(Severity, S)],
  errs: &'a BTreeMap<S, Vec<E>>,
  err_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
  warns: &'a BTreeMap<S, Vec<E>>,
  warn_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
  index: usize,
  err_cursor: BTreeMap<&'a S, usize>,
  warn_cursor: BTreeMap<&'a S, usize>,
}

impl<'a, S, E> Diagnostics<'a, S, E> {
  /// Builds the iterator from the emitter's channels and shared log.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn new(
    log: &'a [(Severity, S)],
    errs: &'a BTreeMap<S, Vec<E>>,
    err_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
    warns: &'a BTreeMap<S, Vec<E>>,
    warn_labels: &'a BTreeMap<S, Vec<Vec<&'static str>>>,
  ) -> Self {
    Self {
      log,
      errs,
      err_labels,
      warns,
      warn_labels,
      index: 0,
      err_cursor: BTreeMap::new(),
      warn_cursor: BTreeMap::new(),
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
    let (severity, span) = self.log.get(self.index)?;
    self.index += 1;

    // The `Severity` tag routes to the channel this entry was recorded in; the per-channel
    // cursor advances one step into this span's group, mirroring how `record`/`record_warning`
    // appended it. Group index == prior same-span emissions in this channel.
    let (groups, labels, cursor) = match severity {
      Severity::Error => (self.errs, self.err_labels, &mut self.err_cursor),
      Severity::Warning => (self.warns, self.warn_labels, &mut self.warn_cursor),
    };
    let slot = cursor.entry(span).or_insert(0);
    let idx = *slot;
    *slot += 1;

    let payload = &groups[span][idx];
    let labels = labels[span][idx].as_slice();
    Some(Diagnostic::new(span, *severity, labels, payload))
  }
}
