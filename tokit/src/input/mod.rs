use core::marker::PhantomData;

use crate::{ParseContext, span::Span};

use super::*;

pub use checkpoint::Checkpoint;
pub use cursor::Cursor;
pub use input_ref::{Commit, DropPolicy, InputRef, Rollback, Transaction};

#[cfg(any(feature = "std", feature = "alloc"))]
pub use input_ref::{SavepointId, StackedTransaction};

mod checkpoint;
mod cursor;
mod input_ref;

/// Storage for [`Input`]'s live-checkpoint lineage stack: inline for 8 ids so the common
/// many-small-parses workload backtracks with no per-parse heap allocation (live-checkpoint
/// nesting depth is typically 1-4), spilling to the heap only past that.
#[cfg(all(any(feature = "std", feature = "alloc"), feature = "smallvec_1"))]
pub(crate) type LineageStack = smallvec_1::SmallVec<[u64; 8]>;
#[cfg(all(any(feature = "std", feature = "alloc"), not(feature = "smallvec_1")))]
pub(crate) type LineageStack = std::vec::Vec<u64>;

/// Storage for a [`StackedTransaction`]'s live savepoints: inline for 2 (savepoint depth per
/// transaction is typically 1-4) so opening a transaction needs no per-parse heap allocation on
/// the common path, spilling to the heap only past that.
#[cfg(all(any(feature = "std", feature = "alloc"), feature = "smallvec_1"))]
pub(crate) type SavepointStack<'inp, 'closure, L> =
  smallvec_1::SmallVec<[(u64, Checkpoint<'inp, 'closure, L>); 2]>;
#[cfg(all(any(feature = "std", feature = "alloc"), not(feature = "smallvec_1")))]
pub(crate) type SavepointStack<'inp, 'closure, L> =
  std::vec::Vec<(u64, Checkpoint<'inp, 'closure, L>)>;

/// Debug-only witness of the input identity a checkpoint was created under, used by
/// [`InputRef::restore`] to reject a checkpoint restored into a foreign input.
///
/// The live-checkpoint *lineage* stack that enforces the last-in, first-out discipline
/// lives on [`Input`] itself (see [`Input::live_ckpts`]) and is maintained in **every**
/// allocator build; this witness carries only the cross-input identity, whose atomic id
/// source keeps it behind the debug + `target_has_atomic = "ptr"` gate. In release builds,
/// and in allocator-less builds, it is absent and `restore` performs no foreign-input
/// check.
#[cfg(all(
  debug_assertions,
  any(feature = "std", feature = "alloc"),
  target_has_atomic = "ptr"
))]
pub(crate) use witness::Witness;

#[cfg(all(
  debug_assertions,
  any(feature = "std", feature = "alloc"),
  target_has_atomic = "ptr"
))]
mod witness {
  use core::sync::atomic::{AtomicUsize, Ordering};

  /// Hands out a distinct identity to every [`Input`](super::Input) so a checkpoint
  /// carries a witness of the input that produced it.
  static NEXT_INPUT_ID: AtomicUsize = AtomicUsize::new(0);

  /// A process-unique identity for one input, stamped into every checkpoint it saves so
  /// a restore can reject a checkpoint that belongs to a different input.
  #[derive(Debug)]
  pub(crate) struct Witness {
    input_id: usize,
  }

  impl Witness {
    /// Creates a witness with a fresh, process-unique input identity.
    pub(crate) fn new() -> Self {
      Self {
        input_id: NEXT_INPUT_ID.fetch_add(1, Ordering::Relaxed),
      }
    }

    /// The identity of the input this witness belongs to.
    pub(crate) fn input_id(&self) -> usize {
      self.input_id
    }
  }

  impl Clone for Witness {
    /// A clone is a **new** input: it gets a fresh identity, so a clone's checkpoints and
    /// the original's can never be confused for one another.
    fn clone(&self) -> Self {
      Self::new()
    }
  }
}

/// The context for parsing input
pub struct InputContext<E, C> {
  emitter: E,
  cache: C,
}

impl<E, C> InputContext<E, C> {
  /// Creates a new `InputContext` with the given emitter and cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(emitter: E, cache: C) -> Self {
    Self { emitter, cache }
  }

  /// Decomposes this context into its emitter and cache components.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (E, C) {
    (self.emitter, self.cache)
  }
}

/// A zero-copy token stream adapter that bridges Logos and Chumsky.
///
/// `Input` is the core integration layer between [Logos](https://github.com/maciejhirsz/logos)
/// lexical analysis and [Chumsky](https://github.com/zesterer/chumsky) parser combinators.
/// It efficiently wraps a Logos token source and implements all necessary Chumsky input traits,
/// allowing you to use Chumsky parsers directly on Logos tokens.
///
/// # Zero-Copy Design
///
/// `Input` doesn't allocate or copy tokens. Instead, it maintains a cursor position
/// and calls Logos on-demand as the parser consumes tokens. This makes it efficient for
/// large inputs and streaming scenarios.
///
/// # State Management
///
/// For stateful lexers (those with non-`()` `Extras`), `Input` maintains the lexer
/// state and passes it through token-by-token. This allows for context-sensitive lexing
/// patterns. Because the adapter clones `Extras` each time it polls Logos, it is best to
/// keep your state `Copy` or otherwise cheap to clone. If you need heavy state, consider
/// storing handles (e.g. `Arc`) inside `Extras` so clones stay inexpensive.
///
/// # Type Parameters
///
/// - `'inp`: The lifetime of the input source
/// - `T`: The token type implementing [`Token<'inp>`]
///
/// # Implemented Traits
///
/// This type implements all core Chumsky input traits:
/// - [`Input`](chumsky::input::Input) - Basic input stream functionality
/// - [`ValueInput`](chumsky::input::ValueInput) - Token-by-token consumption
/// - [`SliceInput`](chumsky::input::SliceInput) - Slice extraction from source
/// - [`ExactSizeInput`](chumsky::input::ExactSizeInput) - Known input length
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,ignore
/// use tokit::{Token, Input, TokenExt};
/// use logos::Logos;
/// use chumsky::prelude::*;
///
/// #[derive(Logos, Debug, Clone, Copy, PartialEq)]
/// #[logos(skip r"[ \t\n]+")]
/// enum MyTokens {
///     #[regex(r"[0-9]+")]
///     Number,
///     #[token("+")]
///     Plus,
/// }
///
/// // Create a token stream from input
/// let input = "42 + 13";
/// let stream = MyToken::lexer(input); // Returns Input<'_, MyToken>
///
/// // Use with Chumsky parsers
/// let parser = any().repeated().collect::<Vec<_>>();
/// let tokens = parser.parse(stream).into_result();
/// ```
///
/// ## Stateful Lexing
///
/// ```rust,ignore
/// #[derive(Default, Clone)]
/// struct LexerState {
///     brace_count: usize,
/// }
///
/// #[derive(Logos, Debug, Clone, Copy)]
/// #[logos(extras = LexerState)]
/// enum MyTokens {
///     #[token("{", |lex| lex.extras.brace_count += 1)]
///     LBrace,
///     #[token("}", |lex| lex.extras.brace_count -= 1)]
///     RBrace,
/// }
///
/// let input = "{ { } }";
/// let initial_state = LexerState::default();
/// let stream = Input::with_state(input, initial_state);
/// ```
///
/// ## Cloning and Backtracking
///
/// Input supports cloning (when the token type and extras are Clone/Copy),
/// which is essential for Chumsky's backtracking:
///
/// ```rust,ignore
/// let stream = MyToken::lexer(input);
/// let checkpoint = stream.clone(); // Save position for backtracking
///
/// // Try to parse something
/// if let Err(_) = try_parser.parse(stream) {
///     // Backtrack by using the cloned stream
///     alternative_parser.parse(checkpoint);
/// }
/// ```
pub(crate) struct Input<'inp, L, Ctx = DefaultCache<'inp, L>, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  input: &'inp L::Source,
  state: L::State,
  span: L::Span,
  cache: Ctx::Cache,
  /// High-water mark: lexer errors whose span ends at or before this offset have
  /// already been emitted (e.g. during a peek that lexed past this point), so the
  /// consume path must not report them again when it re-lexes the same region.
  emitted_error_end: L::Offset,
  /// Sticky limit-error boundary, held at the input level so it survives across
  /// the fresh lexers `InputRef` builds per operation.
  ///
  /// `None` is unpoisoned. `Some(off)` records the *durable frontier* a limit trip
  /// latched at: the offset up to which the pre-trip tokens remain reproducible by
  /// re-lexing. When a lexer surfaces a state/limit error (its
  /// [`check`](crate::Lexer::check) fails after scanning a token) the temporary
  /// lexer latches to EOF, but that latch dies with the lexer; persisting the
  /// frontier here means a scanner whose lex position has reached `off`
  /// short-circuits to its poisoned outcome **without** rebuilding a lexer or
  /// rescanning the tripping token, keeping error-recovery work bounded on
  /// untrusted input. Lexing *strictly before* the frontier still proceeds, so a
  /// cache prefix drained after the trip stays replayable. It is a fact of one
  /// lineage: [`save`](InputRef::save) captures it and [`restore`](InputRef::restore)
  /// copies it back verbatim, since a last-in, first-out restore returns to exactly
  /// the lineage the checkpoint recorded.
  poison_boundary: Option<L::Offset>,
  /// Monotone count of tokens the cache has accepted over this input's life, bumped by
  /// every successful cache push. A [`Checkpoint`] captures it at save time and
  /// [`InputRef::restore`] drops the entries pushed since — the ones lexed on the
  /// abandoned continuation — so their region re-lexes and re-emits its scan side effects.
  /// It is correctness state in every build.
  cache_pushes: u64,
  /// Input-global savepoint sequence counter for [`StackedTransaction`], bumped by
  /// [`InputRef::next_savepoint_seq`] on each [`savepoint`](StackedTransaction::savepoint).
  ///
  /// It is monotone across every stacked transaction of this input and never reset, so a
  /// [`SavepointId`]'s `seq` is unique for the whole life of the input: an id that crosses
  /// transactions (nested or sequential) can never collide with a live savepoint's `seq`
  /// in another transaction's stack. There is no atomic and no process-wide state — the
  /// counter is per-input.
  #[cfg(any(feature = "std", feature = "alloc"))]
  savepoint_seq: u64,
  /// The live-checkpoint lineage stack: the ids of the checkpoints that have been saved and
  /// neither restored nor invalidated by restoring an older one, youngest last. [`save`](InputRef::save)
  /// pushes the fresh id, [`restore`](InputRef::restore) pops the stack down through the
  /// restored id (invalidating it and every younger one), and a committed checkpoint is
  /// forgotten by [`forget_checkpoint`](InputRef::forget_checkpoint). State surgery leaves it
  /// untouched — checkpoints survive state replacement, which is transactional.
  ///
  /// It is the single source of truth for lineage validity in **every** allocator build — no
  /// atomics, no interior mutability, just a `Vec` — so [`StackedTransaction`] can reject a
  /// savepoint whose checkpoint a raw restore below it invalidated, on release and
  /// no-`target_has_atomic`-ptr targets alike. In debug + ptr builds the same stack also
  /// backs `restore`'s non-LIFO and foreign-input panics.
  #[cfg(any(feature = "std", feature = "alloc"))]
  live_ckpts: LineageStack,
  /// Monotone id source for [`live_ckpts`](Self::live_ckpts): each [`save`](InputRef::save)
  /// takes the current value and bumps it, so an id is never reused for the life of the input
  /// and a popped id can never be mistaken for a live one.
  #[cfg(any(feature = "std", feature = "alloc"))]
  next_ckp_id: u64,
  /// The pinned checkpoint ids: the begin-point checkpoint of every currently-live transaction
  /// guard and [`attempt`](InputRef::attempt)/[`try_attempt`](InputRef::try_attempt). A
  /// guard/attempt logically borrows the timeline from its begin point forward, so a raw
  /// [`restore`](InputRef::restore) that would pop a pinned id off
  /// [`live_ckpts`](Self::live_ckpts) — tearing that begin point out from under a live guard —
  /// **panics at the restore** rather than silently invalidating it. Every guard/attempt
  /// constructor pins its held id on entry and every settle path unpins, so this holds exactly
  /// the live begin points and stays bounded across commit-heavy loops. It lives beside
  /// `live_ckpts` under the same allocator gate; allocator-less builds maintain no pin set and
  /// fall back on the detect-at-use backstops (unspecified-but-bounded on misuse).
  #[cfg(any(feature = "std", feature = "alloc"))]
  pinned: LineageStack,
  /// Debug-only witness of the input identity a checkpoint was created under (see
  /// [`InputRef::restore`]); the lineage stack itself is [`live_ckpts`](Self::live_ckpts).
  #[cfg(all(
    debug_assertions,
    any(feature = "std", feature = "alloc"),
    target_has_atomic = "ptr"
  ))]
  witness: Witness,
}

impl<'inp, L, Ctx, Lang: ?Sized> Clone for Input<'inp, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Cache: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    Self {
      input: self.input,
      state: self.state.clone(),
      span: self.span.clone(),
      cache: self.cache.clone(),
      emitted_error_end: self.emitted_error_end.clone(),
      poison_boundary: self.poison_boundary.clone(),
      // The clone shares the cache contents, so it carries the push count forward and its
      // own future saves and restores stay consistent with it.
      cache_pushes: self.cache_pushes,
      // Carried forward so the clone's savepoint seqs stay monotone; the clone is a
      // distinct struct with a distinct nonce anyway, so its ids never cross the
      // original's regardless of the starting value.
      #[cfg(any(feature = "std", feature = "alloc"))]
      savepoint_seq: self.savepoint_seq,
      // A clone is a new input: it starts with an empty lineage stack and a fresh id
      // counter, so a checkpoint from the original is never mistaken for one of the
      // clone's (restoring it is caught as a foreign input in debug + ptr builds).
      #[cfg(any(feature = "std", feature = "alloc"))]
      live_ckpts: LineageStack::new(),
      #[cfg(any(feature = "std", feature = "alloc"))]
      next_ckp_id: 0,
      // A clone is a new input with no live guards, so it starts with an empty pin set.
      #[cfg(any(feature = "std", feature = "alloc"))]
      pinned: LineageStack::new(),
      // A clone is a new input: `Witness::clone` mints a fresh identity, so the clone's
      // checkpoints and the original's never cross.
      #[cfg(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      ))]
      witness: self.witness.clone(),
    }
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> core::fmt::Debug for Input<'inp, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::Source: core::fmt::Debug,
  L::State: core::fmt::Debug,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Cache: core::fmt::Debug,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Input")
      .field("input", &self.input)
      .field("state", &self.state)
      .field("span", &self.span)
      .field("cache", &self.cache)
      .finish()
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> Input<'inp, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Default,
  Ctx: ParseContext<'inp, L, Lang, Cache = DefaultCache<'inp, L>>,
{
  /// Creates a new lexer from the given input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(dead_code)]
  pub fn new(input: &'inp L::Source) -> Self {
    Self::with_state(input, L::State::default())
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> Input<'inp, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang, Cache = DefaultCache<'inp, L>>,
{
  /// Creates a new lexer from the given input and state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(dead_code)]
  pub fn with_state(input: &'inp L::Source, state: L::State) -> Self {
    Self {
      input,
      state,
      span: L::Span::new(L::Offset::default(), L::Offset::default()),
      cache: DefaultCache::<'inp, L>::default(),
      emitted_error_end: L::Offset::default(),
      poison_boundary: None,
      cache_pushes: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      savepoint_seq: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      live_ckpts: LineageStack::new(),
      #[cfg(any(feature = "std", feature = "alloc"))]
      next_ckp_id: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      pinned: LineageStack::new(),
      #[cfg(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      ))]
      witness: Witness::new(),
    }
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> Input<'inp, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_state_and_cache(input: &'inp L::Source, state: L::State, cache: Ctx::Cache) -> Self
// where
  //   C: Cache<'inp, L>,
  {
    Self {
      input,
      state,
      span: L::Span::new(L::Offset::default(), L::Offset::default()),
      cache,
      emitted_error_end: L::Offset::default(),
      poison_boundary: None,
      cache_pushes: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      savepoint_seq: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      live_ckpts: LineageStack::new(),
      #[cfg(any(feature = "std", feature = "alloc"))]
      next_ckp_id: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      pinned: LineageStack::new(),
      #[cfg(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      ))]
      witness: Witness::new(),
    }
  }

  /// Creates a zero-copy reference adapter for this input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref<'closure>(
    &'closure mut self,
    emitter: &'closure mut Ctx::Emitter,
  ) -> InputRef<'inp, 'closure, L, Ctx, Lang> {
    InputRef {
      input: &self.input,
      state: &mut self.state,
      cache: &mut self.cache,
      span: &mut self.span,
      emitted_error_end: &mut self.emitted_error_end,
      poison_boundary: &mut self.poison_boundary,
      cache_pushes: &mut self.cache_pushes,
      #[cfg(any(feature = "std", feature = "alloc"))]
      savepoint_seq: &mut self.savepoint_seq,
      #[cfg(any(feature = "std", feature = "alloc"))]
      live_ckpts: &mut self.live_ckpts,
      #[cfg(any(feature = "std", feature = "alloc"))]
      next_ckp_id: &mut self.next_ckp_id,
      #[cfg(any(feature = "std", feature = "alloc"))]
      pinned: &mut self.pinned,
      #[cfg(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      ))]
      witness: &self.witness,
      emitter,
      _marker: PhantomData,
    }
  }
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests;
