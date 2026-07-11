use core::marker::PhantomData;

use crate::{ParseContext, span::Span};

use super::*;

pub use checkpoint::Checkpoint;
pub use cursor::Cursor;
pub use input_ref::{Commit, DropPolicy, InputRef, Rollback, Transaction};
pub(crate) use lineage::Lineage;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use input_ref::{SavepointId, StackedTransaction};

mod checkpoint;
mod cursor;
mod input_ref;
mod lineage;

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
/// lives on [`Input`] itself (in its [`Lineage`] memos) and is maintained in **every**
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
  /// The lineage memos — the bookkeeping backtracking rewinds an abandoned continuation with:
  /// the live-checkpoint stack, the pin set, and the cache-push/checkpoint-id/savepoint counters.
  /// Gathered behind one guardian (see [`Lineage`] and its [module](lineage) for the
  /// single-writer taxonomy) and kept after the ground-truth cells above, so the scanner-hot
  /// fields pack ahead of it on the `next()` path.
  lineage: Lineage,
  /// Debug-only witness of the input identity a checkpoint was created under (see
  /// [`InputRef::restore`]); the lineage stack itself lives in the [`Lineage`] memos.
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
      // A clone is a new input that shares the cache contents: `Lineage::forked` carries the
      // cache-push and savepoint counters forward and starts a fresh, empty live-checkpoint
      // lineage and pin set (see it for the per-cell rationale).
      lineage: self.lineage.forked(),
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
      lineage: Lineage::new(),
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
      lineage: Lineage::new(),
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
      lineage: &mut self.lineage,
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
