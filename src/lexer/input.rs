// use core::{mem::MaybeUninit, ops::Range};

// use mayber::MaybeRef;

// use crate::utils::{Span, Spanned};

use super::*;

// /// Iterators for [`Input`]
// pub mod iter;

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
/// use logosky::{Token, Input, TokenExt};
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
pub struct Input<'inp, T: Token<'inp>, L: Lexer<'inp, T>, C = ()> {
  input: &'inp L::Source,
  state: L::State,
  cursor: L::Offset,
  cache: C,
}

impl<'inp, T, L, C> Clone for Input<'inp, T, L, C>
where
  T: Token<'inp>,
  L: Lexer<'inp, T>,
  L::State: Clone,
  C: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    Self {
      input: self.input,
      state: self.state.clone(),
      cursor: self.cursor.clone(),
      cache: self.cache.clone(),
    }
  }
}

impl<'inp, T, L, C> core::fmt::Debug for Input<'inp, T, L, C>
where
  T: Token<'inp>,
  L::Source: core::fmt::Debug,
  L: Lexer<'inp, T>,
  L::State: core::fmt::Debug,
  C: core::fmt::Debug,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Input")
      .field("input", &self.input)
      .field("state", &self.state)
      .field("cursor", &self.cursor)
      .field("cache", &self.cache)
      .finish()
  }
}

impl<'inp, T, L> Input<'inp, T, L>
where
  T: Token<'inp>,
  L: Lexer<'inp, T>,
  L::State: Default,
{
  /// Creates a new lexer from the given input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(input: &'inp L::Source) -> Self {
    Self::with_state(input, L::State::default())
  }
}

impl<'inp, T, L> Input<'inp, T, L>
where
  T: Token<'inp>,
  L: Lexer<'inp, T>,
{
  /// Creates a new lexer from the given input and state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_state(input: &'inp L::Source, state: L::State) -> Self {
    Self {
      input,
      state,
      cursor: L::Offset::default(),
      cache: (),
    }
  }
}

impl<'inp, T, L, C> Input<'inp, T, L, C>
where
  T: Token<'inp>,
  L: Lexer<'inp, T>,
{
  /// Creates a zero-copy reference adapter for this input.
  pub const fn as_ref<'closure>(&'inp mut self) -> InputRef<'inp, 'closure, T, L, C> {
    InputRef {
      input: &self.input,
      state: &mut self.state,
      cursor: &mut self.cursor,
      cache: &mut self.cache,
    }
  }
}

// impl<'inp, T, L, C> Input<'inp, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
// {
//   /// Returns a reference to the tokenizer's cache.
//   ///
//   /// The cache stores peeked tokens that have been lexed but not yet consumed.
//   /// This can be useful for inspecting the cache state or implementing custom
//   /// lookahead logic.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub const fn cache(&self) -> &C {
//     &self.cache
//   }

//   /// Returns a reference to the underlying input source.
//   ///
//   /// This allows access to the raw source being tokenized, which is typically
//   /// a `&str` or `&[u8]` depending on your Logos token definition.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub const fn input(&self) -> &L::Source {
//     self.input
//   }

//   /// Returns a reference to the current lexer state (extras)
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn state(&self) -> &L::State {
//     &self.state
//   }

//   /// Manually sets the lexer state (for context-sensitive lexing)
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn set_state(&mut self, state: L::State) {
//     self.state = state;
//   }

//   /// Returns an iterator over the tokens of the lexer.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub const fn iter(&mut self) -> iter::Iter<'inp, '_, T, L, C> {
//     iter::Iter::new(self)
//   }

//   /// Consumes the lexer and returns an iterator over the tokens of the lexer.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub const fn into_iter(self) -> iter::IntoIter<'inp, T, L, C> {
//     iter::IntoIter::new(self)
//   }

//   /// Creates a Logos lexer positioned at the end of the cache or current cursor.
//   ///
//   /// This internal method constructs a fresh Logos lexer with the current state and
//   /// positions it to continue lexing from where the cache ends (or from the cursor
//   /// if the cache is empty).
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn lexer(&self) -> L
//   where
//     L::State: Clone,
//     C: Cache<'inp, T, L>,
//   {
//     let mut lexer = L::with_state(self.input, self.state.clone());
//     lexer.bump(
//       self
//         .cache
//         .span_last()
//         .map(|s| s.end())
//         .unwrap_or(self.cursor),
//     );
//     lexer
//   }

//   /// Sets the cursor to the specified position, clamped to the input length.
//   ///
//   /// This ensures the cursor never exceeds the bounds of the input source.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn set_cursor(&mut self, new: usize) {
//     self.cursor = new.min(self.input.len());
//   }

//   /// Sets the cursor to the latest position between the new value and the cache start.
//   ///
//   /// This method ensures the cursor is positioned at or after the first cached token
//   /// (if any), preventing the cursor from moving backwards past cached tokens.
//   /// The cursor is also clamped to the input length.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn set_cursor_after_consume(&mut self, new: usize)
//   where
//     C: Cache<'inp, T, L>,
//   {
//     self.cursor = new
//       .max(self.cache.span_first().map(|s| s.start()).unwrap_or(new))
//       .min(self.input.len());
//     debug_assert!(
//       self.cursor <= self.input.len(),
//       "Cursor exceeded input bounds"
//     );
//   }
// }

// impl<'inp, T, L, C> Input<'inp, T, L, C>
// where
//   T: Token<'inp>,
//   L: Lexer<'inp, T>,
//   L::State: Clone,
//   C: Cache<'inp, T, L>,
// {
//   // /// Try parsing, returns `None` on failure (no error propagation)
//   // pub fn attempt<F, R>(&mut self, f: F) -> Option<R>
//   // where
//   //   F: FnOnce(&mut Self) -> Option<R>,
//   // {
//   //   let cur = self.cursor().cursor;
//   //   let state = self.state.clone();

//   //   match f(self) {
//   //     Some(result) => Some(result),
//   //     None => {
//   //       let ckp = Checkpoint::new(Cursor::new(cur, self), state);
//   //       self.go(ckp);
//   //       None
//   //     }
//   //   }
//   // }

//   /// Consumes a token if it matches the predicate, returns `None` otherwise (no cursor advance on failure)
//   pub fn accept<F>(&mut self, pred: F) -> Option<Spanned<T>>
//   where
//     F: FnOnce(&T) -> bool,
//   {
//     if let Some(peeked) = self.cache.first() {
//       match peeked.token().data() {
//         Lexed::Token(tk) if pred(tk) => {
//           let tok = self.cache.pop_front().unwrap();
//           let (spanned_lexed, extras) = tok.into_components();
//           let (span, lexed) = spanned_lexed.into_components();
//           self.set_cursor_after_consume(span.end());
//           self.state = extras;
//           return Some(Spanned::new(span, lexed.unwrap_token()));
//         }
//         _ => return None,
//       }
//     }

//     let mut lexer = self.lexer();
//     if let Some(lexed) = Lexed::<T>::lex_spanned(&mut lexer) {
//       let (span, lexed) = lexed.into_components();

//       if let Lexed::Token(tk) = &lexed {
//         if pred(tk) {
//           self.set_cursor_after_consume(lexer.span().end());
//           self.state = lexer.into_state();
//           return Some(Spanned::new(span, lexed.unwrap_token()));
//         }
//       }

//       // cache the token as it was peeked
//       let ct = CachedToken::new(Spanned::new(span, lexed), lexer.into_state());
//       match self.cache.push_back(ct) {
//         Ok(_) => {}
//         Err(_) => {
//           // cache full, do nothing
//         }
//       }
//     }

//     None
//   }

//   /// Consumes the next token if it matches the predicate, otherwise returns an error.
//   pub fn expect<F, Error>(
//     &mut self,
//     pred: F,
//     error_fn: impl FnOnce(Lexed<'inp, T>) -> Error,
//   ) -> Result<Option<Spanned<T>>, Error>
//   where
//     F: FnOnce(&T) -> bool,
//   {
//     if let Some(peeked) = self.cache.first() {
//       match peeked.token().data() {
//         Lexed::Token(tk) if pred(tk) => {
//           let tok = self.cache.pop_front().unwrap();
//           let (spanned_lexed, extras) = tok.into_components();
//           let (span, lexed) = spanned_lexed.into_components();
//           self.set_cursor_after_consume(span.end());
//           self.state = extras;
//           return Ok(Some(Spanned::new(span, lexed.unwrap_token())));
//         }
//         _ => {
//           let tok = self.cache.pop_front().unwrap();
//           let (spanned_lexed, extras) = tok.into_components();
//           let (span, lexed) = spanned_lexed.into_components();
//           self.set_cursor_after_consume(span.end());
//           self.state = extras;
//           return Err(error_fn(lexed));
//         }
//       }
//     }

//     let mut lexer = self.lexer();

//     if let Some(lexed) = Lexed::lex_spanned(&mut lexer) {
//       let (span, lexed) = lexed.into_components();

//       match &lexed {
//         Lexed::Token(tk) if pred(tk) => {
//           self.set_cursor_after_consume(lexer.span().end());
//           self.state = lexer.into_state();
//           return Ok(Some(Spanned::new(span, lexed.unwrap_token())));
//         }
//         _ => {
//           self.set_cursor_after_consume(lexer.span().end());
//           self.state = lexer.into_state();
//           return Err(error_fn(lexed));
//         }
//       }
//     }

//     Ok(None)
//   }

//   /// Returns a slice of the input source from the given cursor to the current cursor of the tokenizer.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn slice_since(
//     &self,
//     cursor: &Cursor<'inp, '_, T, L, C>,
//   ) -> Option<<L::Source as Source>::Slice<'inp>> {
//     let start = cursor.cursor;
//     let end = self.cursor().cursor;
//     self.input.slice(start..end)
//   }

//   /// Returns a slice of the input source from the given cursor to the end of the input.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn slice_from(
//     &self,
//     cursor: &Cursor<'inp, '_, T, L, C>,
//   ) -> Option<<L::Source as Source>::Slice<'inp>> {
//     let start = cursor.cursor;
//     let end = self.input.len();
//     self.input.slice(start..end)
//   }

//   /// Returns a slice of the input source from the current cursor of the tokenizer to the end of the input.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn slice(
//     &self,
//     range: Range<&Cursor<'inp, '_, T, L, C>>,
//   ) -> Option<<L::Source as Source>::Slice<'inp>> {
//     let start = range.start.cursor;
//     let end = range.end.cursor;
//     // SAFETY: The range is guaranteed to be within bounds as both cursors are within input length and comes from the same input.
//     self.input.slice(start..end)
//   }

//   /// Returns a span from the given cursor to the current cursor of the tokenizer.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn span_since(&self, cursor: &Cursor<'inp, '_, T, L, C>) -> Span {
//     Span::new(cursor.cursor, self.cursor().cursor)
//   }

//   /// Returns a span from the given cursor to the end of the input.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn span_from(&self, cursor: &Cursor<'inp, '_, T, L, C>) -> Span {
//     Span::new(cursor.cursor, self.input.len())
//   }

//   /// Returns a span from the current cursor of the tokenizer to the end of the input.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn span(&self, range: Range<&Cursor<'inp, '_, T, L, C>>) -> Span {
//     Span::new(range.start.cursor, range.end.cursor)
//   }

//   /// Consumes one token from the peeked tokens and returns the consumed token if any, the cursor is advanced.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   #[allow(clippy::type_complexity)]
//   pub fn consume_one(&mut self) -> Option<Spanned<Lexed<'inp, T>>> {
//     let tok = self.cache.pop_front()?;
//     let (tok, extras): (Spanned<Lexed<'inp, T>>, _) = tok.into_components();
//     self.set_cursor_after_consume(tok.span().end());
//     self.state = extras;
//     Some(tok)
//   }

//   /// Consumes tokens from cache until the predicate returns `true`, the cursor is advanced to the end of the last consumed token.
//   ///
//   /// Returns the last consumed token.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn consume_until<F>(&mut self, mut f: F) -> Option<Spanned<Lexed<'inp, T>>>
//   where
//     F: FnMut(&CachedToken<'inp, T, L>) -> bool,
//   {
//     let mut last = None;
//     // pop from cache if not matching
//     while let Some(tok) = self.cache.pop_front_if(|t| !f(t)) {
//       self.set_cursor_after_consume(tok.token().span().end());
//       let (tok, state) = tok.into_components();
//       self.state = state;
//       last = Some(tok);
//     }

//     last
//   }

//   /// Consumes tokens from cache while the predicate returns `true`, the cursor is advanced to the end of the last consumed token.
//   ///
//   /// Returns the last consumed token.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn consume_while<F>(&mut self, mut f: F) -> Option<Spanned<Lexed<'inp, T>>>
//   where
//     F: FnMut(&CachedToken<'inp, T, L>) -> bool,
//   {
//     self.consume_until(|t| !f(t))
//   }

//   /// Consumes all cached tokens, the cursor is advanced to the end of the last cached token.
//   ///
//   /// Returns the last consumed token.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn consume_cached(&mut self) -> Option<Spanned<Lexed<'inp, T>>> {
//     let last = self.cache.pop_back()?;
//     self.cache.clear();
//     let (tok, extras): (Spanned<Lexed<'inp, T>>, _) = last.into_components();
//     self.set_cursor_after_consume(tok.span().end());
//     self.state = extras;
//     Some(tok)
//   }

//   /// Skips one token, advancing the cursor.
//   ///
//   /// If there's a token in the cache, it pops and discards it. Otherwise,
//   /// it lexes the next token and discards it.
//   ///
//   /// Returns `true` if a token was skipped, `false` if the end of input was reached.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn skip_one(&mut self) -> bool {
//     if let Some(cached_token) = self.cache.pop_front() {
//       let (spanned_lexed, extras) = cached_token.into_components();
//       let (span, _lexed) = spanned_lexed.into_components();
//       self.set_cursor_after_consume(span.end());
//       self.state = extras;
//       true
//     } else {
//       self.next().is_some()
//     }
//   }

//   /// Skips tokens until a valid token is found or the end of input is reached.
//   ///
//   /// Returns the first valid token found, but without consuming it.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn skip_until<F>(&mut self, mut pred: F) -> Option<MaybeRef<'_, CachedToken<'inp, T, L>>>
//   where
//     F: FnMut(&Spanned<Lexed<'inp, T>>) -> bool,
//   {
//     // pop from cache if not matching
//     while let Some(tok) = self.cache.pop_front_if(|t| !pred(t.token())) {
//       self.set_cursor_after_consume(tok.token().span().end());
//       self.state = tok.state;
//     }

//     // as the matched token will not be consumed, we just peek it
//     match !self.cache.is_empty() {
//       // If the matched token is in cache, return it
//       true => self.cache.peek_one(),
//       // Otherwise, let's skip the input
//       false => {
//         let mut lexer = self.lexer();
//         let mut end = self.cursor;
//         let mut state = self.state.clone();

//         while let Some(lexed) = Lexed::<T>::lex_spanned(&mut lexer) {
//           // if the token matches, we cache it and return it
//           if pred(&lexed) {
//             let ct = CachedToken::new(lexed, lexer.state().clone());
//             self.set_cursor_after_consume(end);
//             self.state = state;

//             return match self.cache.push_back(ct) {
//               Ok(tok) => Some(MaybeRef::Ref(tok)),
//               Err(ct) => Some(MaybeRef::Owned(ct)),
//             };
//           }

//           end = lexer.span().end();
//           state = lexer.state().clone();
//         }

//         // No matched token found, we just update the cursor and state
//         self.set_cursor_after_consume(lexer.span().end());
//         self.state = lexer.into_state();

//         None
//       }
//     }
//   }

//   /// Skips tokens while the predicate returns `true`.
//   ///
//   /// Returns the first token that does not match the predicate, but without consuming it.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn skip_while<F>(&mut self, mut pred: F) -> Option<MaybeRef<'_, CachedToken<'inp, T, L>>>
//   where
//     F: FnMut(&Spanned<Lexed<'inp, T>>) -> bool,
//   {
//     self.skip_until(|t| !pred(t))
//   }

//   /// Skips error tokens until a valid token is found or the end of input is reached.
//   ///
//   /// Returns the first valid token found, but without consuming it.
//   pub fn skip_until_valid(&mut self) -> Option<MaybeRef<'_, CachedToken<'inp, T, L>>> {
//     self.skip_until(|t| matches!(t.data, Lexed::Token(_)))
//   }

//   /// Skips tokens until the predicate returns `true`, emitting errors using the provided emitter.
//   ///
//   /// This method advances through the token stream, skipping tokens until it finds one that
//   /// matches the predicate. Any lexer errors encountered are emitted via the provided emitter.
//   /// If a fatal error occurs during emission, the method returns immediately with that error.
//   ///
//   /// Returns the first token that matches the predicate, but without consuming it.
//   /// If no matching token is found, returns `None`.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn skip_until_with_emitter<F, E>(
//     &mut self,
//     mut pred: F,
//     mut emitter: E,
//   ) -> Result<Option<MaybeRef<'_, CachedToken<'inp, T, L>>>, E::Error>
//   where
//     F: FnMut(Spanned<&T>, &mut E) -> bool,
//     E: Emitter<'inp, T>,
//   {
//     // pop from cache if not matching
//     while let Some(tok) = self.cache.pop_front_if(|t| {
//       let span = t.token().span();
//       match t.token().data() {
//         Lexed::Token(tok) => !pred(Spanned::new(span, tok), &mut emitter),
//         Lexed::Error(_) => true,
//       }
//     }) {
//       let span = tok.token().span();
//       self.set_cursor_after_consume(span.end());
//       self.state = tok.state;

//       // Note: cursor/state are updated before emission. If emission fails,
//       // the error token has still been consumed (no backtracking here).
//       if let Lexed::Error(e) = tok.token.into_data() {
//         emitter.emit_token_error(Spanned::new(span, e))?;
//       }
//     }

//     // as the matched token will not be consumed, we just peek it
//     match !self.cache.is_empty() {
//       // If the matched token is in cache, return it
//       true => Ok(self.cache.peek_one()),
//       // Otherwise, let's skip the input
//       false => {
//         let mut lexer = self.lexer();

//         let mut end = self.cursor;
//         let mut state = self.state.clone();

//         while let Some(Spanned { span, data: tok }) = Lexed::<T>::lex_spanned(&mut lexer) {
//           match tok {
//             Lexed::Error(err) => match emitter.emit_token_error(Spanned::new(span, err)) {
//               Ok(_) => {
//                 end = lexer.span().end();
//                 state = lexer.state().clone();
//               }
//               Err(e) => {
//                 self.set_cursor_after_consume(lexer.span().end());
//                 self.state = lexer.into_state();
//                 return Err(e);
//               }
//             },
//             Lexed::Token(tok) => {
//               let tok = Spanned::new(span, tok);
//               // if the token matches, we cache it and return it
//               if pred(tok.as_ref(), &mut emitter) {
//                 let ct = CachedToken::new(tok.map_data(Lexed::Token), lexer.into_state());
//                 self.set_cursor_after_consume(end);
//                 self.state = state;
//                 return Ok(match self.cache.push_back(ct) {
//                   Ok(tok) => Some(MaybeRef::Ref(tok)),
//                   Err(ct) => Some(MaybeRef::Owned(ct)),
//                 });
//               }

//               end = lexer.span().end();
//               state = lexer.state().clone();
//             }
//           }
//         }

//         // No matched token found, we just update the cursor and state
//         self.set_cursor_after_consume(lexer.span().end());
//         self.state = lexer.into_state();

//         Ok(None)
//       }
//     }
//   }

//   /// Peeks the next token without advancing the cursor.
//   #[inline]
//   pub fn peek_one(&mut self) -> Option<MaybeRef<'_, CachedToken<'inp, T, L>>> {
//     let mut buf: [MaybeUninit<MaybeRef<'_, CachedToken<'inp, T, L>>>; 1] = [MaybeUninit::uninit()];
//     let feed = self.peek(&mut buf);
//     if feed.is_empty() {
//       return None;
//     }

//     // SAFETY: We just checked that the buffer is not empty, so the first element is initialized.
//     buf.into_iter().next().map(|m| unsafe { m.assume_init() })
//   }

//   // /// Peeks the tokens until find
//   // pub fn peek_until(&mut self, pred: impl Fn(&Lexed<'inp, T>) -> bool) -> Option<Lexed<'inp, T>> {
//   //   if let Some(cached_token) = self.cache.peek() {
//   //     return Some(cached_token.data.clone());
//   //   }

//   //   let state = self.state.clone();
//   //   let mut lexer = logos::Lexer::<T::Logos>::with_extras(self.input, state);
//   //   lexer.bump(self.cursor);
//   //   Lexed::lex(&mut lexer).map(|tok| {
//   //     self.cache_token(lexer.span().into(), lexer.extras.clone(), tok)
//   //   })
//   // }

//   /// Try to peeks tokens to fill the provided buffer, if not enough tokens are cached, lex more tokens to fill the buffer.
//   ///
//   /// The returned slice will contain only the initialized tokens.
//   #[inline]
//   pub fn peek<'p, 'b>(
//     &'p mut self,
//     buf: &'b mut [MaybeUninit<MaybeRef<'p, CachedToken<'inp, T, L>>>],
//   ) -> &'b mut [MaybeRef<'p, CachedToken<'inp, T, L>>] {
//     let buf_len = buf.len();
//     let mut in_cache = self.cache.len();
//     let mut want = buf_len.saturating_sub(in_cache);

//     // If we already have enough tokens cached, just peek from cache
//     if want == 0 {
//       // SAFETY: Cache guarantees peek() returns only initialized tokens up to cache.len()
//       return unsafe { self.cache.peek(buf) };
//     }

//     // Otherwise, lex additional tokens to fill the request
//     let mut lexer = self.lexer();
//     while want > 0 {
//       if let Some(lexed) = Lexed::lex_spanned(&mut lexer) {
//         let (span, lexed) = lexed.into_components();
//         let cached = CachedToken::new(Spanned::new(span, lexed), lexer.state().clone());

//         // Try to cache the token; if cache is full, write directly to output buffer
//         match self.cache.push_back(cached) {
//           Ok(_) => {
//             in_cache += 1;
//           }
//           Err(ct) => {
//             // Cache full: write overflow tokens directly to buffer
//             // Position: buf[buf_len - want] is the next unfilled slot
//             buf[buf_len - want].write(MaybeRef::Owned(ct));
//           }
//         }
//         want -= 1;
//       } else {
//         break;
//       }
//     }

//     // Fill buffer from cache (this covers both cached tokens and any we just added)
//     // SAFETY: Cache.peek() returns slice of initialized tokens, guaranteed by trait contract
//     let output = unsafe { self.cache.peek(&mut buf[..in_cache]) };
//     debug_assert!(
//       output.len() == in_cache,
//       "Cache peek returned unexpected number of tokens"
//     );
//     output
//   }

//   /// Saves the current state of the tokenizer as a checkpoint.
//   ///
//   /// This creates a snapshot of the current position and lexer state, which can
//   /// later be restored using [`go`](Self::go). Checkpoints are essential for
//   /// implementing backtracking in parsers.
//   ///
//   /// # Example
//   ///
//   /// ```ignore
//   /// let checkpoint = tokenizer.save();
//   /// // Try parsing something...
//   /// if parsing_failed {
//   ///     tokenizer.go(checkpoint); // Restore state
//   /// }
//   /// ```
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn save(&self) -> Checkpoint<'inp, '_, T, L, C> {
//     Checkpoint::new(self.cursor(), self.state.clone())
//   }

//   /// Returns the current cursor position of the tokenizer.
//   ///
//   /// The cursor represents the byte offset in the input where the tokenizer will
//   /// continue lexing. If there are cached tokens, the cursor points to the start
//   /// of the first cached token; otherwise, it points to the current position.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn cursor(&self) -> Cursor<'inp, '_, T, L, C> {
//     Cursor::new(
//       self
//         .cache
//         .span_first()
//         .map(|s| s.start())
//         .unwrap_or(self.cursor),
//       self,
//     )
//   }

//   /// Restores the tokenizer state to a previously saved checkpoint.
//   ///
//   /// This rewinds the cache, resets the cursor position, and restores the lexer
//   /// state, effectively undoing all operations since the checkpoint was created.
//   /// This is commonly used for parser backtracking.
//   #[doc(alias = "rewinds")]
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn go(&mut self, checkpoint: Checkpoint<'inp, '_, T, L, C>) {
//     self.cache.rewind(&checkpoint);
//     self.set_cursor(checkpoint.cursor().cursor);
//     self.state = checkpoint.state;
//   }

//   /// Advances the cursor and returns the next valid token, emitting errors via the provided emitter.
//   ///
//   /// This method skips over lexer errors, emitting them through the provided emitter.
//   /// Non-fatal errors are emitted and the method continues to the next token. If a
//   /// fatal error occurs during emission, it's returned and processing stops.
//   ///
//   /// Returns `Ok(Some(token))` for valid tokens, `Ok(None)` at end of input, or
//   /// `Err(error)` if a fatal error occurred.
//   pub fn next_valid_with<E>(&mut self, mut emitter: E) -> Result<Option<Spanned<T>>, E::Error>
//   where
//     E: Emitter<'inp, T>,
//   {
//     // First, consume from cache if available
//     while let Some(cached_token) = self.cache.pop_front() {
//       let (spanned_lexed, extras) = cached_token.into_components();
//       let (span, lexed) = spanned_lexed.into_components();
//       self.set_cursor_after_consume(span.end());
//       self.state = extras;
//       match lexed {
//         Lexed::Token(t) => return Ok(Some(Spanned::new(span, t))),
//         Lexed::Error(e) => {
//           emitter.emit_token_error(Spanned::new(span, e))?;
//           continue;
//         }
//       }
//     }

//     // then, construct a lexer and lex until a valid token is found
//     let mut lexer = self.lexer();

//     while let Some(lexed) = Lexed::lex_spanned(&mut lexer) {
//       let (span, lexed) = lexed.into_components();
//       self.set_cursor_after_consume(lexer.span().end());
//       self.state = lexer.state().clone();

//       match lexed {
//         Lexed::Token(t) => return Ok(Some(Spanned::new(span, t))),
//         Lexed::Error(e) => {
//           emitter.emit_token_error(Spanned::new(span, e))?;
//           continue;
//         }
//       }
//     }

//     Ok(None)
//   }

//   // /// Advances the cursor and returns the next valid token, emit non-fatal errors, fatal errors are returned and stop the process.
//   // pub fn next_valid<E>(&mut self, emitter: E) -> Result<Option<Spanned<T>>, E::Error>
//   // where
//   //   E: Emitter<'inp, T>,
//   //   E::Error: From<<T::Logos as Logos<'inp>>::Error>,
//   // {
//   //   self.next_valid_with(emitter)
//   // }

//   /// Advances the cursor and returns the next token (valid or error).
//   ///
//   /// Unlike [`next_valid_with`](Self::next_valid_with), this method returns both
//   /// valid tokens and lexer errors wrapped in [`Lexed`]. The cursor advances
//   /// regardless of whether a valid token or error is returned.
//   ///
//   /// Returns `Some(Spanned<Lexed>)` with either a token or error, or `None` at
//   /// end of input.
//   #[allow(clippy::should_implement_trait)]
//   pub fn next(&mut self) -> Option<Spanned<Lexed<'inp, T>>> {
//     if let Some(cached_token) = self.cache.pop_front() {
//       let (spanned_lexed, extras) = cached_token.into_components();
//       let (span, lexed) = spanned_lexed.into_components();
//       self.set_cursor_after_consume(span.end());
//       self.state = extras;
//       return Some(Spanned::new(span, lexed));
//     }

//     let mut lexer = self.lexer();
//     Lexed::lex_spanned(&mut lexer).inspect(|_| {
//       self.set_cursor_after_consume(lexer.span().end());
//       self.state = lexer.state().clone();
//     })
//   }

//   // #[cfg_attr(not(tarpaulin), inline(always))]
//   // pub(crate) fn next_at(&mut self, cursor: &mut usize) -> Option<Spanned<Lexed<'inp, T>>> {
//   //   let state = self.state.clone();
//   //   let mut lexer = logos::Lexer::<T::Logos>::with_extras(self.input, state);
//   //   lexer.bump(*cursor);
//   //   Lexed::lex_spanned(&mut lexer).inspect(|_| {
//   //     *cursor = lexer.span().end;
//   //     self.state = lexer.extras.clone();
//   //   })
//   // }
// }
