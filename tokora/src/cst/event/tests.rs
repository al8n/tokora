use super::*;

/// The F-A5 core, at the vocabulary level: a truncation at or below a mark's index,
/// recorded after the mark's era, makes the mark stale; a truncation strictly above it
/// does not; and marks issued after a truncation are untouched by it.
#[test]
fn ledger_staleness_is_truncate_at_or_below_after_issue() {
  let mut ledger = TruncationLedger::new();
  let mark_era = ledger.era();

  // No truncations yet: nothing is stale.
  assert!(!ledger.is_stale(mark_era, 3));

  // Truncation strictly above the mark's index leaves it live.
  ledger.record_truncation(4);
  assert!(!ledger.is_stale(mark_era, 3));

  // Truncation reaching the index kills it — the tombstone at 3 was dropped, whatever
  // regrew at index 3 later is not it.
  ledger.record_truncation(3);
  assert!(ledger.is_stale(mark_era, 3));

  // A mark issued NOW (after both truncations) is untouched by them.
  let fresh_era = ledger.era();
  assert!(!ledger.is_stale(fresh_era, 3));
}

/// The merge invariant: a deeper truncation subsumes shallower ones recorded before it,
/// so the stack stays strictly increasing in `low` — and staleness answers stay exact
/// across the merge.
#[test]
fn ledger_merges_subsumed_truncations() {
  let mut ledger = TruncationLedger::new();
  let old = ledger.era();
  ledger.record_truncation(10);
  ledger.record_truncation(7);
  ledger.record_truncation(3);
  // All three collapse to the deepest record; answers are unchanged by the merge.
  assert_eq!(ledger.entries.len(), 1);
  assert!(ledger.is_stale(old, 3));
  assert!(ledger.is_stale(old, 9));
  assert!(ledger.is_stale(old, 100));
  assert!(!ledger.is_stale(old, 2));

  // Regrow-then-truncate-shallow keeps both records: they invalidate different ranges.
  let mid = ledger.era();
  ledger.record_truncation(8);
  assert_eq!(ledger.entries.len(), 2);
  assert!(ledger.is_stale(mid, 8), "the new truncation reaches 8");
  assert!(!ledger.is_stale(mid, 7), "below the new low-water mark");
  assert!(
    ledger.is_stale(old, 5),
    "the old, deeper truncation still applies"
  );
}

/// The era source is monotone and bumps exactly once per recorded truncation.
#[test]
fn era_source_is_monotone_per_truncation() {
  let mut ledger = TruncationLedger::new();
  assert_eq!(ledger.era(), 0);
  ledger.record_truncation(5);
  assert_eq!(ledger.era(), 1);
  ledger.record_truncation(5);
  assert_eq!(ledger.era(), 2);
}

/// The depth model, event by event: tombstones are neutral, wraps count at their own
/// index, tokens and diag slots are invisible to balance.
#[test]
fn depth_deltas_follow_the_model() {
  let start: Event<()> = Event::StartNode {
    kind: 7,
    forward_parent: None,
  };
  let tomb: Event<()> = Event::StartNode {
    kind: TOMBSTONE,
    forward_parent: None,
  };
  let wrap: Event<()> = Event::StartAt {
    kind: 7,
    target: 0,
    prev: None,
  };
  let fin: Event<()> = Event::FinishNode { kind: 7 };
  let demote: Event<()> = Event::Demote { target: 0 };
  let tok: Event<()> = Event::Token { kind: 7, span: () };
  let diag: Event<()> = Event::Diag { error_span: None };

  assert_eq!(start.depth_delta(), 1);
  assert_eq!(tomb.depth_delta(), 0);
  assert_eq!(wrap.depth_delta(), 1);
  assert_eq!(fin.depth_delta(), -1);
  assert_eq!(
    demote.depth_delta(),
    -1,
    "the failing exit un-opens exactly what its start opened: Start(+1) … Demote(−1) nets \
     zero, the same as the canonicalized tombstone"
  );
  assert_eq!(tok.depth_delta(), 0);
  assert_eq!(diag.depth_delta(), 0);

  assert!(tomb.is_tombstone());
  assert!(!start.is_tombstone());
  assert!(
    !demote.is_tombstone(),
    "a Demote names a tombstone-to-be; it is not one"
  );
}

/// The `Demote` variant costs the log nothing: `StartAt` already carries a `u64` target
/// alongside a kind and a link, so the widest variant is unchanged and the buffer's
/// per-event footprint does not move.
#[test]
fn the_demote_variant_does_not_widen_the_event() {
  use core::mem::size_of;

  type Ev = Event<crate::span::SimpleSpan>;
  assert_eq!(
    size_of::<Ev>(),
    32,
    "the event log's element is 32 bytes for a two-usize span; adding Demote must not \
     widen it (StartAt already carries u64 + u16 + Option<NonZeroU32>)"
  );
}

/// An inert mark can never name a real slot: its index is `u64::MAX`, above any
/// reachable buffer length, so the bounds half of validation rejects it before the era
/// half is consulted.
#[test]
fn inert_mark_is_out_of_every_buffer() {
  let m = EventMark::inert();
  assert_eq!(m.index(), u64::MAX);
  assert_eq!(m.era(), u64::MAX);
}

// ── GUIDE_EVENT_CENSUS ────────────────────────────────────────────────────────────────
//
// The guide chapter that teaches this vocabulary heads its enumeration with a **count** —
// `### The six events` — and that count is a reader's only signal that the list under it is
// exhaustive. It went wrong once and silently, and nothing read it: `Demote` landed and the
// heading kept saying five over a five-bullet list (#227). That error lands on exactly the
// population the guide exists for, and it is undetectable from inside the text, because an
// enumeration that counts itself reads as complete whether or not it is.
//
// The other way to close it is to drop the number from the heading. That removes the
// *visible* error and keeps the *actual* one: a bulleted enumeration reads as exhaustive
// whether or not it counts itself, so a seventh variant with no seventh entry would still
// leave the chapter teaching a model with a hole in it — just without the arithmetic that
// gave it away. Dropping the count turns a checkable claim into an unstated one. So the
// count stays and becomes machine-checked instead, which is the trade `RECORD_CENSUS`,
// `FORWARDING_CENSUS` and `CACHE_CALL_CENSUS` already make: a hand-maintained number is
// fine exactly when something reds the moment it drifts.
//
// The census reads NAMES as well as the count, because a count alone cannot see a swap:
// rename a variant, rewrite one unrelated entry, and the arithmetic still holds. It does
// NOT read the entries' ORDER — the chapter introduces `Demote` after `StartAt` because its
// encoding argument *is* `StartAt`'s, and prose order should serve the reader rather than
// the declaration order of an enum.
//
// Both readers take their text as an argument, so `the_census_reds_on_the_drift_it_claims`
// can drive the real ones over inputs that are wrong in each way the real files can be
// wrong. A census that has only ever agreed with itself is not a census.

/// The guide chapter whose enumeration this census pins.
const GUIDE_CHAPTER: &str = include_str!("../../guide/arch_event_stream_cst.md");

/// The module's own `mod.rs`, read as text. Rust has no variant reflection, so the census parses
/// the source that *declares* [`Event`] rather than the type — the `CACHE_CALL_CENSUS` posture,
/// exact for declaration syntax rather than for a list of spellings.
const EVENT_SOURCE: &str = include_str!("mod.rs");

/// The one heading the census reads, split around the count it carries.
const HEADING_PREFIX: &str = "### The ";
const HEADING_SUFFIX: &str = " events";

/// The spelling every entry of the enumeration opens with. One spelling, and everything
/// else is refused: an entry this cannot read is a variant the census would silently miss.
const BULLET: &str = "- **`";

/// The number words a heading may spell, indexed by the value each names. A count past the
/// end of this table reds rather than passes — an event vocabulary that grows that far is
/// precisely when somebody should be made to look at the chapter.
const NUMBER_WORDS: &[&str] = &[
  "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
];

/// Every variant the `Event` enum in `source` declares, in declaration order.
///
/// Read through `syn`, which refuses what it cannot parse instead of scanning for a shape:
/// three review rounds of #183 each patched a substring matcher and each was followed by a
/// new gap in it, which is a property of text scanning rather than of any one pattern.
fn declared_variants(source: &str) -> std::vec::Vec<std::string::String> {
  let file = syn::parse_file(source)
    .expect("GUIDE_EVENT_CENSUS: `cst/event.rs` must parse, or the census reads nothing");
  let mut enums = file.items.iter().filter_map(|item| match item {
    syn::Item::Enum(item) if item.ident == "Event" => Some(item),
    _ => None,
  });
  let declared = enums
    .next()
    .expect("GUIDE_EVENT_CENSUS: no `enum Event` in `cst/event.rs` — the census would pin nothing");
  assert!(
    enums.next().is_none(),
    "GUIDE_EVENT_CENSUS: more than one `enum Event` in `cst/event.rs`; the census cannot \
     tell which one the chapter enumerates"
  );
  declared
    .variants
    .iter()
    .map(|variant| variant.ident.to_string())
    .collect()
}

/// The count word in `chapter`'s enumeration heading, and the variant each of its entries
/// names.
///
/// The enumeration is the one contiguous block of lines under the heading — it ends at the
/// first blank line, so a list further down the section is none of this reader's business.
/// Inside the block a bullet the census cannot read is a **refusal**, not a skip: silently
/// ignoring an entry is how a census comes to agree only with itself.
fn enumerated_in_guide(chapter: &str) -> (std::string::String, std::vec::Vec<std::string::String>) {
  let lines: std::vec::Vec<&str> = chapter.lines().collect();
  let headings: std::vec::Vec<usize> = lines
    .iter()
    .enumerate()
    .filter(|(_, line)| {
      line.starts_with(HEADING_PREFIX)
        && line.ends_with(HEADING_SUFFIX)
        && line.len() > HEADING_PREFIX.len() + HEADING_SUFFIX.len()
    })
    .map(|(index, _)| index)
    .collect();
  assert_eq!(
    headings.len(),
    1,
    "GUIDE_EVENT_CENSUS: expected exactly one `{HEADING_PREFIX}<count>{HEADING_SUFFIX}` \
     heading in `guide/arch_event_stream_cst.md`, found {}",
    headings.len()
  );

  let heading = lines[headings[0]];
  let word = heading[HEADING_PREFIX.len()..heading.len() - HEADING_SUFFIX.len()].to_string();

  let block = lines[headings[0] + 1..]
    .iter()
    .skip_while(|line| line.trim().is_empty())
    .take_while(|line| !line.trim().is_empty());
  let mut named = std::vec::Vec::new();
  for line in block {
    if !line.starts_with("- ") {
      continue;
    }
    let rest = line.strip_prefix(BULLET).unwrap_or_else(|| {
      panic!(
        "GUIDE_EVENT_CENSUS: the enumeration carries a top-level bullet the census cannot \
         read. Every entry opens `{BULLET}Variant`, so that an entry added to the chapter \
         cannot be invisible to this check:\n  {line}"
      )
    });
    let name: std::string::String = rest
      .chars()
      .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
      .collect();
    assert!(
      !name.is_empty(),
      "GUIDE_EVENT_CENSUS: an entry opens `{BULLET}` with no variant name after it:\n  \
       {line}"
    );
    named.push(name);
  }
  assert!(
    !named.is_empty(),
    "GUIDE_EVENT_CENSUS: the `{HEADING_PREFIX}{word}{HEADING_SUFFIX}` heading is followed \
     by no enumeration at all — the reader found nothing to check, which must never read \
     as agreement"
  );
  (word, named)
}

/// The census itself: the guide's enumeration names every variant `Event` declares, nothing
/// it does not, and the heading counts them. Panics, naming the drift, otherwise.
fn census(source: &str, chapter: &str) {
  let declared = declared_variants(source);
  let (word, named) = enumerated_in_guide(chapter);

  let counted = NUMBER_WORDS
    .iter()
    .position(|candidate| *candidate == word)
    .unwrap_or_else(|| {
      panic!(
        "GUIDE_EVENT_CENSUS: the enumeration's heading spells `{word}`, which is not a \
         number word this census can read. Spell it out — one of {NUMBER_WORDS:?} — or \
         extend the table in the same commit as the variant that outgrew it."
      )
    });
  assert_eq!(
    counted,
    declared.len(),
    "GUIDE_EVENT_CENSUS drift: `guide/arch_event_stream_cst.md` heads its enumeration \
     `{HEADING_PREFIX}{word}{HEADING_SUFFIX}`, but `Event` declares {} variant(s): \
     {declared:?}. Correct the heading and the entries in the same commit as the variant \
     (grep GUIDE_EVENT_CENSUS).",
    declared.len()
  );

  let missing: std::vec::Vec<&std::string::String> = declared
    .iter()
    .filter(|variant| !named.contains(variant))
    .collect();
  assert!(
    missing.is_empty(),
    "GUIDE_EVENT_CENSUS drift: {missing:?} declared by `Event` but named by no entry of \
     the guide's enumeration. The section reads as exhaustive, so a reader learning the \
     event stream comes away with a model that has no {missing:?} in it — give each one \
     the treatment the others get: what it is, when the stream carries one, and what \
     materialization does with it (grep GUIDE_EVENT_CENSUS)."
  );

  let stale: std::vec::Vec<&std::string::String> = named
    .iter()
    .filter(|variant| !declared.contains(variant))
    .collect();
  assert!(
    stale.is_empty(),
    "GUIDE_EVENT_CENSUS drift: {stale:?} enumerated by the guide but declared by no \
     `Event` variant. Drop the entry, or restore the variant (grep GUIDE_EVENT_CENSUS)."
  );

  assert_eq!(
    named.len(),
    declared.len(),
    "GUIDE_EVENT_CENSUS drift: the enumeration carries {} entries for {} variant(s), so a \
     variant is named twice: {named:?}",
    named.len(),
    declared.len()
  );
}

/// GUIDE_EVENT_CENSUS — the shipped chapter enumerates exactly the variants [`Event`]
/// declares, and its heading counts them.
#[test]
fn the_guide_enumerates_every_event_variant() {
  census(EVENT_SOURCE, GUIDE_CHAPTER);
}

/// GUIDE_EVENT_CENSUS — the positive control, driven through the real readers.
///
/// Each pair below is the fixture pair with exactly one thing wrong, in one of the ways the
/// real files can be wrong, and each must red with the message that names it. The control
/// on the control is the first call: the unmutated pair passes, so a red below is the
/// mutation and not the harness.
#[test]
fn the_census_reds_on_the_drift_it_claims() {
  // Six variants, the real enum's shape with the bodies elided.
  const SOURCE: &str = "\
pub(crate) enum Event<S> {
  StartNode { kind: u16, forward_parent: Option<NonZeroU32> },
  Token { kind: u16, span: S },
  FinishNode { kind: u16 },
  Demote { target: u64 },
  StartAt { kind: u16, target: u64, prev: Option<NonZeroU32> },
  Diag { error_span: Option<S> },
}
";
  // Six entries under a heading that counts them, in the shipped chapter's shape.
  const CHAPTER: &str = "\
## The event vocabulary

### The six events

- **`StartNode { kind, forward_parent }`** opens a node.
- **`Token { kind, span }`** is one committed token.
- **`FinishNode { kind }`** closes the innermost open node.
- **`StartAt { kind, target, prev }`** retro-opens a node.
- **`Demote { target }`** un-opens a node.
- **`Diag { error_span }`** is a forwarded-diagnostic slot.

Balance is derived from the log.

### Marks carry an era and a witness
";

  fn red(source: &str, chapter: &str) -> std::string::String {
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| census(source, chapter)));
    let payload = caught.expect_err("the census must red on this pair");
    payload
      .downcast_ref::<std::string::String>()
      .cloned()
      .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
      .expect("a census refusal carries its message")
  }

  // The control on the control.
  census(SOURCE, CHAPTER);

  // A seventh variant the chapter never mentions: the exact shape #227 shipped.
  let grown = SOURCE.replace("  Demote {", "  Reparent { target: u64 },\n  Demote {");
  assert_ne!(
    grown, SOURCE,
    "the mutation must actually change the source"
  );
  let message = red(&grown, CHAPTER);
  assert!(
    message.contains("`Event` declares 7 variant(s)") && message.contains("Reparent"),
    "the count arm must name the enum's own variants: {message}"
  );

  // Heading corrected, entry still absent — the completeness half, on its own.
  let counted = CHAPTER.replace("### The six events", "### The seven events");
  let message = red(&grown, &counted);
  assert!(
    message.contains("[\"Reparent\"] declared by `Event` but named by no entry"),
    "the completeness arm must name the variant the chapter is missing: {message}"
  );

  // An entry for a variant that does not exist. It is added rather than substituted, so
  // that the completeness arm above has nothing to say and this arm is the one under test.
  let invented = CHAPTER.replace(
    "- **`Demote { target }`** un-opens a node.\n",
    "- **`Demote { target }`** un-opens a node.\n- **`Retract { target }`** withdrew.\n",
  );
  let message = red(SOURCE, &invented);
  assert!(
    message.contains("[\"Retract\"] enumerated by the guide"),
    "the staleness arm must name the entry with no variant behind it: {message}"
  );

  // One variant given two entries: every name resolves and every name is present, so only
  // the arithmetic can see it.
  let twice = CHAPTER.replace(
    "- **`Demote { target }`** un-opens a node.\n",
    "- **`Demote { target }`** un-opens a node.\n- **`Demote { target }`** again.\n",
  );
  let message = red(SOURCE, &twice);
  assert!(
    message.contains("carries 7 entries for 6 variant(s)"),
    "the arithmetic arm must catch a variant enumerated twice: {message}"
  );

  // A count word the reader cannot resolve is refused, never read as no opinion.
  let unspelled = CHAPTER.replace("### The six events", "### The 6 events");
  let message = red(SOURCE, &unspelled);
  assert!(
    message.contains("heading spells `6`"),
    "the heading arm must name the word it could not read: {message}"
  );

  // An entry spelled another way is refused rather than dropped from the enumeration — the
  // silent skip that would let a variant be "enumerated" by a line nothing parses.
  let unreadable = CHAPTER.replace("- **`Demote {", "- `Demote` {");
  let message = red(SOURCE, &unreadable);
  assert!(
    message.contains("a top-level bullet the census cannot read"),
    "the bullet arm must refuse what it cannot parse: {message}"
  );

  // And a source the census cannot read is a refusal too, not an empty variant list that
  // would make every chapter trivially correct.
  let message = red("pub struct NotAnEnum;", CHAPTER);
  assert!(
    message.contains("no `enum Event`"),
    "an unreadable source must refuse rather than pin nothing: {message}"
  );
}
