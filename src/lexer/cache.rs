use core::mem::MaybeUninit;

use mayber::MaybeRef;

use super::{CachedToken, Checkpoint, Lexer, Span, Token};

mod blackhole;
mod generic_arraydeque;

/// A trait for caching lookahead tokens in the tokenizer.
///
/// `Cache` provides a buffer for tokens that have been lexed but not yet consumed,
/// enabling efficient lookahead and backtracking operations. The cache acts as a
/// queue (FIFO - First In, First Out) between the lexer and the parser.
///
/// # Purpose
///
/// The cache serves several critical functions:
/// - **Lookahead**: Allows peeking at future tokens without consuming them
/// - **Backtracking**: Supports parser backtracking via checkpoint/rewind operations
/// - **Efficiency**: Avoids re-lexing tokens that have already been processed
/// - **State Management**: Preserves lexer state (extras) alongside each token
///
/// # Design Patterns
///
/// Different implementations support different use cases:
/// - **Fixed-size arrays**: Bounded lookahead with known maximum (e.g., `[CachedToken; 4]`)
/// - **Dynamic buffers**: Unlimited lookahead using `Vec` or `VecDeque`
/// - **BlackHole**: No caching at all, for streaming-only scenarios without backtracking
///
/// Note: Tokens cannot be overwritten until explicitly consumed, as they must remain
/// available for backtracking operations. This means the cache can become full and
/// refuse new tokens if capacity is reached.
///
/// # Cache Operations
///
/// The cache supports standard queue operations:
/// - `push_back`: Add newly lexed tokens to the end (fails if cache is full)
/// - `pop_front`: Remove and return the oldest token
/// - `peek`: View tokens without removing them
/// - `rewind`: Restore to a previous state (for backtracking)
///
/// # Safety
///
/// The `peek` method is marked unsafe because it requires implementations to guarantee
/// that returned slices only contain properly initialized tokens. This is enforced by
/// the trait's contract.
///
/// # Example
///
/// ```ignore
/// // A simple fixed-size cache using a VecDeque-like structure
/// struct BoundedCache<'a, T: Token<'a>> {
///     tokens: VecDeque<CachedToken<'a, T>>,
///     capacity: usize,
/// }
///
/// impl<'a, T: Token<'a>> Cache<'a, T> for BoundedCache<'a, T> {
///     fn len(&self) -> usize {
///         self.tokens.len()
///     }
///
///     fn remaining(&self) -> usize {
///         self.capacity - self.tokens.len()
///     }
///
///     fn push_back(&mut self, tok: CachedToken<'a, T>) -> Result<&CachedToken<'a, T>, CachedToken<'a, T>> {
///         if self.tokens.len() < self.capacity {
///             self.tokens.push_back(tok);
///             Ok(self.tokens.back().unwrap())
///         } else {
///             Err(tok) // Cache full, cannot overwrite!
///         }
///     }
///     // ... other methods
/// }
/// ```
pub trait Cache<'a, T: Token<'a>, L: Lexer<'a, T>> {
  /// Returns `true` if the cache contains no tokens.
  ///
  /// This is a convenience method that checks if `len() == 0`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Returns the number of tokens currently stored in the cache.
  ///
  /// This count includes all cached tokens from front to back.
  fn len(&self) -> usize;

  /// Returns the number of additional tokens that can be cached.
  ///
  /// For unbounded caches (like `Vec`), this might return a large number.
  /// For fixed-size caches, this returns the number of free slots.
  /// For `BlackHole`, this always returns 0.
  fn remaining(&self) -> usize;

  /// Rewinds the cache to a previously saved checkpoint.
  ///
  /// This operation restores the cache state to match the checkpoint, typically
  /// by clearing any tokens that were added after the checkpoint was created.
  /// This is used for parser backtracking.
  fn rewind(&mut self, checkpoint: &Checkpoint<'a, '_, T, L>)
  where
    Self: Sized;

  /// Attempts to add a token to the back of the cache.
  ///
  /// If successful, returns `Ok` with a reference to the cached token.
  /// If the cache is full, returns `Err` with the token so the caller can handle it
  /// (e.g., by processing it immediately without caching).
  ///
  /// # Example
  ///
  /// ```ignore
  /// match cache.push_back(token) {
  ///     Ok(cached_ref) => {
  ///         // Token was cached successfully
  ///     }
  ///     Err(token) => {
  ///         // Cache is full, handle token directly
  ///     }
  /// }
  /// ```
  fn push_back(
    &mut self,
    tok: CachedToken<'a, T, L>,
  ) -> Result<&CachedToken<'a, T, L>, CachedToken<'a, T, L>>;

  /// Removes and returns the token at the front of the cache.
  ///
  /// Returns `None` if the cache is empty. This is the primary way to consume
  /// cached tokens.
  #[allow(clippy::type_complexity)]
  fn pop_front(&mut self) -> Option<CachedToken<'a, T, L>>;

  /// Removes and returns the token at the back of the cache.
  ///
  /// Returns `None` if the cache is empty. This is less commonly used than
  /// `pop_front` but can be useful for certain cache management operations.
  #[allow(clippy::type_complexity)]
  fn pop_back(&mut self) -> Option<CachedToken<'a, T, L>>;

  /// Removes all tokens from the cache.
  ///
  /// After calling this method, `len()` returns 0 and `is_empty()` returns `true`.
  fn clear(&mut self);

  /// Conditionally removes and returns the front token if it matches a predicate.
  ///
  /// Peeks at the first token in the cache and checks if it satisfies the predicate.
  /// If it does, removes and returns it. Otherwise, returns `None` without modifying
  /// the cache.
  ///
  /// # Example
  ///
  /// ```ignore
  /// // Pop token only if it's a specific type
  /// if let Some(token) = cache.pop_front_if(|t| matches!(t.token().data, Lexed::Token(_))) {
  ///     // Process valid token
  /// }
  /// ```
  #[allow(clippy::type_complexity)]
  fn pop_front_if<F>(&mut self, predicate: F) -> Option<CachedToken<'a, T, L>>
  where
    F: FnOnce(&CachedToken<'a, T, L>) -> bool,
  {
    if let Some(peeked) = self.first() {
      if predicate(peeked) {
        return self.pop_front();
      }
    }
    None
  }

  /// Peeks at the first cached token without removing it.
  ///
  /// Returns `Some(MaybeRef)` with either a reference to the cached token or
  /// an owned token (if cache implementation requires). Returns `None` if the
  /// cache is empty.
  ///
  /// This is a convenience wrapper around `peek` for looking at just one token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn peek_one<'c>(&self) -> Option<MaybeRef<'_, CachedToken<'a, T, L>>>
  where
    'a: 'c,
  {
    let mut buf: [MaybeUninit<MaybeRef<'_, CachedToken<'a, T, L>>>; 1] = [MaybeUninit::uninit()];
    let feed = unsafe { self.peek(&mut buf) };
    if feed.is_empty() {
      return None;
    }

    // SAFETY: We just checked that the buffer is not empty, so the first element is initialized.
    buf.into_iter().next().map(|m| unsafe { m.assume_init() })
  }

  /// Peeks at multiple cached tokens without removing them.
  ///
  /// Fills the provided buffer with references to cached tokens (or owned tokens if
  /// necessary). The returned slice contains only the successfully initialized tokens,
  /// which may be fewer than requested if the cache doesn't have enough tokens.
  ///
  /// # Parameters
  ///
  /// - `buf`: A buffer of uninitialized `MaybeRef` entries to fill with peeked tokens
  ///
  /// # Returns
  ///
  /// A mutable slice containing initialized token references. The slice length indicates
  /// how many tokens were actually available.
  ///
  /// # Safety
  ///
  /// Implementations must guarantee that:
  /// - The returned slice contains only properly initialized tokens
  /// - All cached tokens are filled into the buffer if the buffer is large enough
  /// - The slice bounds are correct and don't include uninitialized memory
  ///
  /// Callers must ensure the returned slice is not used beyond its lifetime.
  #[allow(clippy::mut_from_ref)]
  unsafe fn peek<'p, 'b>(
    &'p self,
    buf: &'b mut [MaybeUninit<MaybeRef<'p, CachedToken<'a, T, L>>>],
  ) -> &'b mut [MaybeRef<'p, CachedToken<'a, T, L>>];

  /// Pushes multiple tokens into the cache at once.
  ///
  /// Attempts to cache all tokens from the iterator. If the cache becomes full,
  /// returns an iterator over the tokens that could not be cached.
  ///
  /// # Example
  ///
  /// ```ignore
  /// let overflow = cache.push_many(token_iter);
  /// for token in overflow {
  ///     // Handle tokens that didn't fit in cache
  /// }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_many<'p>(
    &'p mut self,
    toks: impl Iterator<Item = CachedToken<'a, T, L>> + 'p,
  ) -> impl Iterator<Item = CachedToken<'a, T, L>> + 'p {
    toks.filter_map(move |tok| self.push_back(tok).err())
  }

  /// Returns a reference to the first (oldest) cached token.
  ///
  /// Returns `None` if the cache is empty. This does not remove the token.
  fn first(&self) -> Option<&CachedToken<'a, T, L>>;

  /// Returns a reference to the last (newest) cached token.
  ///
  /// Returns `None` if the cache is empty. This does not remove the token.
  fn last(&self) -> Option<&CachedToken<'a, T, L>>;

  /// Returns the combined span covering all cached tokens.
  ///
  /// If the cache has tokens, returns a span from the start of the first token
  /// to the end of the last token. Returns `None` if the cache is empty.
  ///
  /// This is useful for error reporting or understanding the range of lookahead.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn span(&self) -> Option<L::Span> {
    match (self.first(), self.last()) {
      (Some(first), Some(last)) => Some(L::Span::new(
        first.token().span_ref().start(),
        last.token().span_ref().end(),
      )),
      _ => None,
    }
  }

  /// Returns the span of the first cached token.
  ///
  /// Returns `None` if the cache is empty. This is often used to determine
  /// where the next consumed token will come from.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn first_span<'s>(&'s self) -> Option<&'s L::Span>
  where
    'a: 's,
  {
    self.first().map(move |t| t.token().span_ref())
  }

  /// Returns the span of the last cached token.
  ///
  /// Returns `None` if the cache is empty. This can be used to determine
  /// where the cache's lookahead ends.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn last_span<'s>(&'s self) -> Option<&'s L::Span>
  where
    'a: 's,
  {
    self.last().map(move |t| t.token().span_ref())
  }
}
