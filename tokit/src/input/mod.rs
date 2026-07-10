use core::marker::PhantomData;

use crate::{ParseContext, span::Span};

use super::*;

pub use checkpoint::Checkpoint;
pub use cursor::Cursor;
pub use input_ref::{InputRef, Transaction};

mod checkpoint;
mod cursor;
mod input_ref;

/// Debug-only exact witness for the last-in, first-out checkpoint discipline that
/// [`InputRef::restore`] documents.
///
/// It exists only in debug builds that have an allocator (`std` or `alloc`); in
/// release builds, and in allocator-less builds, it is absent and `restore` is an
/// unchecked pure copy of the saved lineage state.
#[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
pub(crate) use witness::Witness;

#[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
mod witness {
  use core::{
    cell::{Cell, RefCell},
    sync::atomic::{AtomicUsize, Ordering},
  };

  use std::vec::Vec;

  /// Hands out a distinct identity to every [`Input`](super::Input) so a checkpoint
  /// carries a witness of the input that produced it.
  static NEXT_INPUT_ID: AtomicUsize = AtomicUsize::new(0);

  /// Tracks the checkpoints that are still live for one input.
  ///
  /// A live checkpoint is one that has been saved and neither restored nor
  /// invalidated by restoring an older one. Restoring pops the stack down through
  /// the restored id (inclusive), which is exactly the set of checkpoints the
  /// restore invalidates. The counter is monotone and never reused, so an id popped
  /// off the stack can never be mistaken for a live one.
  ///
  /// The cells give `save` (which takes `&self`) a place to record without widening
  /// its signature; the whole type is debug-only scaffolding.
  #[derive(Debug)]
  pub(crate) struct Witness {
    input_id: usize,
    next_ckp_id: Cell<u64>,
    live: RefCell<Vec<u64>>,
  }

  impl Witness {
    /// Creates a witness with a fresh, process-unique input identity and no live
    /// checkpoints.
    pub(crate) fn new() -> Self {
      Self {
        input_id: NEXT_INPUT_ID.fetch_add(1, Ordering::Relaxed),
        next_ckp_id: Cell::new(0),
        live: RefCell::new(Vec::new()),
      }
    }

    /// The identity of the input this witness belongs to.
    pub(crate) fn input_id(&self) -> usize {
      self.input_id
    }

    /// Records a freshly saved checkpoint and returns its id.
    pub(crate) fn push(&self) -> u64 {
      let id = self.next_ckp_id.get();
      self.next_ckp_id.set(id + 1);
      self.live.borrow_mut().push(id);
      id
    }

    /// Returns whether `id` is still a live checkpoint of this input.
    pub(crate) fn contains(&self, id: u64) -> bool {
      self.live.borrow().contains(&id)
    }

    /// Pops the live stack down through `id` inclusive, invalidating it and every
    /// checkpoint saved after it.
    pub(crate) fn pop_through(&self, id: u64) {
      let mut live = self.live.borrow_mut();
      if let Some(pos) = live.iter().position(|&x| x == id) {
        live.truncate(pos);
      }
    }

    /// Drops `id` from the live stack without restoring it — the checkpoint is kept
    /// (committed) rather than rewound, so its id must not linger and grow the stack
    /// across commit-heavy loops. O(1) when `id` is the stack top (the common case
    /// for a committed checkpoint); a linear removal otherwise (e.g. a raw checkpoint
    /// saved above it was dropped without restoring). Removing a non-top id keeps the
    /// rest of the stack in order, so an older restore still pops cleanly through it.
    pub(crate) fn forget(&self, id: u64) {
      let mut live = self.live.borrow_mut();
      if live.last() == Some(&id) {
        live.pop();
      } else if let Some(pos) = live.iter().position(|&x| x == id) {
        live.remove(pos);
      }
    }

    /// Invalidates every live checkpoint (used when lexer state is replaced).
    pub(crate) fn clear(&self) {
      self.live.borrow_mut().clear();
    }

    /// The number of live checkpoints. Test-only observability for the no-growth
    /// guarantee that `forget` gives `commit`/`attempt`/`try_attempt`.
    #[cfg(test)]
    pub(crate) fn live_len(&self) -> usize {
      self.live.borrow().len()
    }
  }

  impl Clone for Witness {
    /// A clone is a **new** input: it gets a fresh identity and no live checkpoints,
    /// so a clone's checkpoints and the original's can never be confused for one
    /// another.
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
  /// Debug-only witness of the last-in, first-out checkpoint discipline (see
  /// [`InputRef::restore`]).
  #[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
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
      // A clone is a new input: `Witness::clone` mints a fresh identity and an empty
      // live-checkpoint stack, so the clone's checkpoints and the original's never
      // cross.
      #[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
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
      #[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
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
      #[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
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
      #[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
      witness: &mut self.witness,
      emitter,
      _marker: PhantomData,
    }
  }
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests;
