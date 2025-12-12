use core::marker::PhantomData;

use super::*;

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
pub(crate) struct Input<'inp, L, C = DefaultCache<'inp, L>>
where
  L: Lexer<'inp>,
{
  input: &'inp L::Source,
  state: L::State,
  span: L::Span,
  cursor: L::Offset,
  cache: C,
}

impl<'inp, L, C> Clone for Input<'inp, L, C>
where
  L: Lexer<'inp>,
  L::State: Clone,
  C: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    Self {
      input: self.input,
      state: self.state.clone(),
      span: self.span.clone(),
      cursor: self.cursor.clone(),
      cache: self.cache.clone(),
    }
  }
}

impl<'inp, L, C> core::fmt::Debug for Input<'inp, L, C>
where
  L: Lexer<'inp>,
  L::Source: core::fmt::Debug,
  L::State: core::fmt::Debug,
  C: core::fmt::Debug,
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

impl<'inp, L> Input<'inp, L>
where
  L: Lexer<'inp>,
  L::State: Default,
{
  /// Creates a new lexer from the given input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(dead_code)]
  pub fn new(input: &'inp L::Source) -> Self {
    Self::with_state(input, L::State::default())
  }
}

impl<'inp, L> Input<'inp, L>
where
  L: Lexer<'inp>,
{
  /// Creates a new lexer from the given input and state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(dead_code)]
  pub fn with_state(input: &'inp L::Source, state: L::State) -> Self {
    Self {
      input,
      state,
      cursor: L::Offset::default(),
      span: L::Span::new(L::Offset::default(), L::Offset::default()),
      cache: DefaultCache::<'inp, L>::default(),
    }
  }
}

impl<'inp, L, C> Input<'inp, L, C>
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_state_and_cache(input: &'inp L::Source, state: L::State, cache: C) -> Self
  where
    C: Cache<'inp, L>,
  {
    Self {
      input,
      state,
      cursor: L::Offset::default(),
      span: L::Span::new(L::Offset::default(), L::Offset::default()),
      cache,
    }
  }

  /// Creates a zero-copy reference adapter for this input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref<'closure, E, Lang: ?Sized>(
    &'closure mut self,
    emitter: &'closure mut E,
  ) -> InputRef<'inp, 'closure, L, E, C, Lang> {
    InputRef {
      input: &self.input,
      state: &mut self.state,
      cache: &mut self.cache,
      span: &mut self.span,
      emitter,
      _marker: PhantomData,
    }
  }
}
