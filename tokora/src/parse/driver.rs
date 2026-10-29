//! The drivers — [`Parser`], [`Parse`] and the free [`parse()`] family — with the
//! configuration types they take and the [`Action`] answer every `*_while` callback returns.
//!
//! They are declared here rather than in `combinator`: a `pub` item declared in that module's
//! own file is reachable at `tokora::combinator::…`, and the combinator namespace holds no driver
//! and no trait. [`parse`](super) re-exports each of them.

use core::marker::PhantomData;

use derive_more::{IsVariant, TryUnwrap, Unwrap};

use crate::{
  Emitter, Lexer,
  emitter::{Fatal, FromTokenErrors},
  input::{Input, Partial, SurfaceIncomplete},
};

use super::{
  context::{ErrorOf, FatalContext, ParseContext, ParserContext},
  input::ParseInput,
};

/// Wrapper for cache configuration in parsers.
///
/// Wraps a cache type `C` to distinguish it from bare `()` in type parameters,
/// preventing trait overlap in Parse implementations.
#[repr(transparent)]
pub struct WithCache<'inp, L, C> {
  cache: C,
  _marker: PhantomData<&'inp L>,
}

/// Wrapper for emitter configuration in parsers.
///
/// Wraps an emitter type `E` to distinguish it from bare `()` in type parameters,
/// preventing trait overlap in Parse implementations.
#[repr(transparent)]
pub struct WithEmitter<E: ?Sized>(E);

/// A parser: a parsing function plus the context (emitter, cache, recursion budget) it runs
/// against.
///
/// Reach for [`parse()`], [`parse_with`] or [`parse_with_state`] when the source is already in
/// hand — they take it as a value and infer everything from it. `Parser` is the builder for
/// the flows those three do not cover: configuring an emitter or cache options *before* the
/// source exists.
///
/// # Type Parameters
///
/// - `F`: the parsing function
/// - `L`: the lexer type
/// - `O`: the output type
/// - `Ctx`: the [`ParseContext`] — the emitter and cache pair, plus the recursion budget its
///   [`provide`](ParseContext::provide) hands over with them
///
/// The error type is **not** a parameter. It is
/// [`ErrorOf<'inp, L, Ctx, Lang>`](crate::ErrorOf), a projection of `Ctx`, so carrying it here
/// would only let a caller write a second, independent spelling of a type the context already
/// determines.
///
/// # Examples
///
/// ```ignore
/// // Fail fast, default cache.
/// let p = Parser::new().apply(one_int);
///
/// // A collecting emitter, configured before the source exists.
/// let p = Parser::with_context((&mut emitter, cache)).apply(one_int);
/// ```
pub struct Parser<F, L: ?Sized, O: ?Sized, Ctx> {
  f: F,
  ctx: Ctx,
  _l: PhantomData<L>,
  _o: PhantomData<O>,
}

impl<F, L, O, Ctx> core::ops::Deref for Parser<F, L, O, Ctx> {
  type Target = F;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.f
  }
}

impl<F, L, O, Ctx> core::ops::DerefMut for Parser<F, L, O, Ctx> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.f
  }
}

impl<'inp, L, O, E, Lang> Default for Parser<(), L, O, FatalContext<'inp, L, E, Lang>>
where
  L: Lexer<'inp>,
  E: FromTokenErrors<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline(always)]
  fn default() -> Self {
    Parser::new()
  }
}

impl Parser<(), (), (), ()> {
  /// A parser without any behavior, wired to the fail-fast [`FatalContext`].
  ///
  /// One constructor serves every language: `E` and `Lang` are both recoverable from the
  /// returned `FatalContext<'inp, L, E, Lang>`, which names each of them in a parameter
  /// position, so wherever the context is pinned downstream neither needs a turbofish.
  #[inline(always)]
  pub const fn new<'inp, L, O, E, Lang>() -> Parser<(), L, O, FatalContext<'inp, L, E, Lang>>
  where
    L: Lexer<'inp>,
    E: FromTokenErrors<'inp, L, Lang>,
    Lang: ?Sized,
  {
    Self::with_context(ParserContext::of(Fatal::of()))
  }

  /// Creates a parser with the given context.
  ///
  /// Deliberately unbounded. A [`ParseContext`] bound would have to name `Lang`, and nothing
  /// here can pin it: [`ParseContext`] is implemented for `()` and for `(E, C)` at *every*
  /// `Lang`, so the marker is unrecoverable from the context alone and every call would need
  /// a turbofish to supply it. Storing a context is a value move; the obligations that
  /// context must actually meet are checked by [`apply`](Parser::apply) and by [`Parse`],
  /// each of which has a parsing function to read the marker off.
  #[inline(always)]
  pub const fn with_context<L, O, Ctx>(ctx: Ctx) -> Parser<(), L, O, Ctx>
  where
    L: ?Sized,
    O: ?Sized,
  {
    Parser {
      f: (),
      ctx,
      _l: PhantomData,
      _o: PhantomData,
    }
  }

  /// Creates a parser with a parsing function and the fail-fast [`FatalContext`].
  #[inline(always)]
  pub const fn with_parser<'inp, L, O, E, F, Lang>(
    f: F,
  ) -> Parser<F, L, O, FatalContext<'inp, L, E, Lang>>
  where
    L: Lexer<'inp>,
    E: FromTokenErrors<'inp, L, Lang>,
    F: ParseInput<'inp, L, O, FatalContext<'inp, L, E, Lang>, Lang>,
    Lang: ?Sized,
  {
    Self::with_parser_and_context(f, ParserContext::of(Fatal::of()))
  }

  /// Creates a parser with a parsing function and the given context.
  #[inline(always)]
  pub const fn with_parser_and_context<'inp, L, O, Ctx, F, Lang>(
    f: F,
    ctx: Ctx,
  ) -> Parser<F, L, O, Ctx>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    Lang: ?Sized,
  {
    Parser {
      f,
      ctx,
      _l: PhantomData,
      _o: PhantomData,
    }
  }
}

impl<'inp, L, O, Ctx> Parser<(), L, O, Ctx>
where
  L: Lexer<'inp>,
{
  /// Apply a parsing function to the parser.
  ///
  /// `Lang` is read off `f`'s [`InputRef`](crate::InputRef) argument, so a branded grammar writes the same
  /// call an unbranded one does.
  #[inline(always)]
  pub fn apply<F, Lang>(self, f: F) -> Parser<F, L, O, Ctx>
  where
    Ctx: ParseContext<'inp, L, Lang>,
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    Lang: ?Sized,
  {
    Parser {
      f,
      ctx: self.ctx,
      _l: PhantomData,
      _o: PhantomData,
    }
  }
}

/// Entry-point trait: run a parser against a source.
///
/// This provides the ergonomic `.parse()` API similar to Chumsky and
/// Winnow. Implementations wire up `Input`, `Emitter`, and `Cache`
/// before delegating to [`ParseInput`].
///
/// A grammar that already holds its source reaches for the free
/// [`parse()`] / [`parse_with`] / [`parse_with_state`] instead: they take the
/// source as a value, so nothing has to be named before it exists.
pub trait Parse<'inp, L, O, Error, Lang: ?Sized = ()>: Sized {
  /// Parse using the lexer's default state.
  #[inline(always)]
  fn parse(self, src: &'inp L::Source) -> Result<O, Error>
  where
    L: Lexer<'inp>,
    L::State: Default,
  {
    self.parse_with_state(src, L::State::default())
  }

  /// Parse using an explicit lexer state.
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> Result<O, Error>
  where
    L: Lexer<'inp>;

  /// Parse from a raw string source.
  #[inline(always)]
  fn parse_str(self, src: &'inp str) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = str>,
    L::State: Default,
  {
    self.parse_str_with_state(src, Default::default())
  }

  /// Parse from a raw string source with an explicit lexer state.
  #[inline(always)]
  fn parse_str_with_state(self, src: &'inp str, state: L::State) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = str>,
  {
    self.parse_with_state(src, state)
  }

  /// Parse from a raw byte slice source.
  #[inline(always)]
  fn parse_slice(self, src: &'inp [u8]) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = [u8]>,
    L::State: Default,
  {
    self.parse_slice_with_state(src, Default::default())
  }

  /// Parse from a raw byte slice source with an explicit lexer state.
  #[inline(always)]
  fn parse_slice_with_state(self, src: &'inp [u8], state: L::State) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = [u8]>,
  {
    self.parse_with_state(src, state)
  }

  /// Parse from [`bytes::Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html) source.
  #[cfg(feature = "bytes_1")]
  #[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
  #[inline(always)]
  fn parse_bytes(self, src: &'inp bytes_1::Bytes) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = [u8]>,
    L::State: Default,
  {
    self.parse_bytes_with_state(src, Default::default())
  }

  /// Parse from [`bytes::Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html) source with an explicit lexer state.
  #[cfg(feature = "bytes_1")]
  #[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
  #[inline(always)]
  fn parse_bytes_with_state(self, src: &'inp bytes_1::Bytes, state: L::State) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = [u8]>,
  {
    self.parse_with_state(src.as_ref(), state)
  }

  /// Parse from [`bstr::BStr`](https://docs.rs/bstr/latest/bstr/struct.BStr.html) source.
  #[cfg(feature = "bstr_1")]
  #[cfg_attr(docsrs, doc(cfg(feature = "bstr_1")))]
  #[inline(always)]
  fn parse_bstr(self, src: &'inp bstr_1::BStr) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = [u8]>,
    L::State: Default,
  {
    self.parse_bstr_with_state(src, Default::default())
  }

  /// Parse from [`bstr::BStr`](https://docs.rs/bstr/latest/bstr/struct.BStr.html) source with an explicit lexer state.
  #[cfg(feature = "bstr_1")]
  #[cfg_attr(docsrs, doc(cfg(feature = "bstr_1")))]
  #[inline(always)]
  fn parse_bstr_with_state(self, src: &'inp bstr_1::BStr, state: L::State) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = [u8]>,
  {
    self.parse_with_state(src.as_ref(), state)
  }

  /// Parse from [`hipstr::HipStr`](https://docs.rs/hipstr/latest/hipstr/type.HipStr.html) source.
  #[cfg(feature = "hipstr_0_8")]
  #[cfg_attr(docsrs, doc(cfg(feature = "hipstr_0_8")))]
  #[inline(always)]
  fn parse_hipstr(self, src: &'inp hipstr_0_8::HipStr<'_>) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = str>,
    L::State: Default,
  {
    self.parse_hipstr_with_state(src, Default::default())
  }

  /// Parse from [`hipstr::HipStr`](https://docs.rs/hipstr/latest/hipstr/type.HipStr.html) source with an explicit lexer state.
  #[cfg(feature = "hipstr_0_8")]
  #[cfg_attr(docsrs, doc(cfg(feature = "hipstr_0_8")))]
  #[inline(always)]
  fn parse_hipstr_with_state(
    self,
    src: &'inp hipstr_0_8::HipStr<'_>,
    state: L::State,
  ) -> Result<O, Error>
  where
    L: Lexer<'inp, Source = str>,
  {
    self.parse_with_state(src.as_str(), state)
  }
}

impl<'inp, F, L, O, Error, Ctx, Lang: ?Sized> Parse<'inp, L, O, Error, Lang>
  for Parser<F, L, O, Ctx>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: Emitter<'inp, L, Lang, Error = Error>,
{
  #[inline(always)]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> Result<O, Error> {
    let Parser { mut f, ctx, .. } = self;

    let mut input = Input::with_state_and_context(src, state, ctx.provide());
    let mut input_ref = input.as_ref();
    f.parse_input(&mut input_ref)
  }
}

/// Runs `f` over `src`, failing fast on the first error.
///
/// The value-driven entry point: the source arrives as an argument, so the whole signature is
/// solved from the call. Contrast [`Parser`], which bakes a context *before* any source exists
/// and therefore cannot read the lexer off one.
///
/// # What is inferred, and from where
///
/// | Parameter | Comes from |
/// |---|---|
/// | `'inp`, `S` | the `&'inp S` argument |
/// | `L` | `L: Lexer<'inp, Source = S>` — the source **value** picks the lexer's source parameter |
/// | `Lang` | the [`InputRef`](crate::InputRef) in `f`'s signature |
/// | `O`, `E` | the annotated result |
///
/// `L` riding the source matters for a lexer that is generic over what it reads —
/// `MyLexer<'inp, Src>`, the shape a dialect with more than one storage backend has. Nothing in
/// `f` fixes `Src`; the source does.
///
/// # Examples
///
/// A source-generic lexer, a context-generic production, and **no turbofish anywhere**:
///
#[cfg_attr(feature = "any", doc = "```rust")]
#[cfg_attr(not(feature = "any"), doc = "```rust,ignore")]
/// # use core::{convert::Infallible, fmt};
/// # use tokora::{
/// #   ComposableParseContext, ErrorOf, InputRef, Lexer, ParseInput as _, SimpleSpan, Source,
/// #   Token, combinator::Any,
/// #   error::{
/// #     FullContainer, MissingSyntax, MissingToken, SeparatedError, TooFew, UnexpectedEot,
/// #     UnexpectedToken,
/// #   },
/// #   parse, span::Span as _,
/// # };
/// # #[derive(Debug)] struct Error;
/// # impl From<Infallible> for Error { fn from(e: Infallible) -> Self { match e {} } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for Error { fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, K, S, Lang>> for Error { fn from(_: SeparatedError<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, K: Clone, O, Lang: ?Sized> From<MissingToken<'a, K, O, Lang>> for Error { fn from(_: MissingToken<'a, K, O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for Error { fn from(_: MissingSyntax<O, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for Error { fn from(_: FullContainer<S, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for Error { fn from(_: TooFew<S, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEot<O, Lang, Set>> for Error { fn from(_: UnexpectedEot<O, Lang, Set>) -> Self { Error } }
/// # impl<'a, L: Lexer<'a>, Lang: ?Sized> tokora::emitter::FromUnclosed<'a, L, Lang> for Error { fn from_unclosed<D>(_: tokora::error::Unclosed<D, L::Span, Lang>) -> Self { Error } }
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] struct Kind;
/// # impl fmt::Display for Kind { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("digit") } }
/// # #[derive(Debug, Clone, Copy, PartialEq)] struct Digit(u32);
/// # impl Token<'_> for Digit { type Kind = Kind; type Error = Infallible; const SCAN_LOOKAHEAD: tokora::ScanLookahead = tokora::ScanLookahead::Unbounded; fn kind(&self) -> Kind { Kind } fn is_trivia(&self) -> bool { false } }
/// # /// The byte view a `Mini` lexer needs of whatever it is reading.
/// # trait Bytes: Source<usize> { fn byte(&self, at: usize) -> Option<u8>; }
/// # impl Bytes for str { fn byte(&self, at: usize) -> Option<u8> { self.as_bytes().get(at).copied() } }
/// # impl Bytes for [u8] { fn byte(&self, at: usize) -> Option<u8> { self.get(at).copied() } }
/// # /// A lexer generic over its source — one type, two storage backends.
/// # struct Mini<'a, Src: ?Sized> { src: &'a Src, pos: usize, tok: SimpleSpan, state: () }
/// # impl<'a, Src: Bytes + ?Sized + 'a> Lexer<'a> for Mini<'a, Src> {
/// #   type State = (); type Source = Src; type Token = Digit; type Span = SimpleSpan; type Offset = usize;
/// #   fn new(src: &'a Src) -> Self { Self { src, pos: 0, tok: SimpleSpan::new(0, 0), state: () } }
/// #   fn with_state(src: &'a Src, _: ()) -> Self { Self::new(src) }
/// #   fn check(&self) -> Result<(), Infallible> { Ok(()) }
/// #   fn state(&self) -> &() { &self.state }
/// #   fn state_mut(&mut self) -> &mut () { &mut self.state }
/// #   fn into_state(self) {}
/// #   fn source(&self) -> &'a Src { self.src }
/// #   fn span(&self) -> SimpleSpan { self.tok }
/// #   fn slice(&self) -> <Src as Source<usize>>::Slice<'a> { self.src.slice(self.tok.start()..self.tok.end()).unwrap() }
/// #   fn lex(&mut self) -> Option<Result<Digit, Infallible>> {
/// #     let byte = self.src.byte(self.pos)?;
/// #     let start = self.pos;
/// #     self.pos += 1;
/// #     self.tok = SimpleSpan::new(start, self.pos);
/// #     Some(Ok(Digit(u32::from(byte - b'0'))))
/// #   }
/// #   fn read_frontier(&self) -> tokora::ReadFrontier<usize> { tokora::ReadFrontier::SpanEnd }
/// #   fn bump(&mut self, n: &usize) { self.pos += n; }
/// # }
/// // A production the way a real grammar writes one: generic over the source type *and* over
/// // the context, naming neither the lexer's instantiation nor the error it produces.
/// fn two_digits<'inp, Src, Ctx>(
///   inp: &mut InputRef<'inp, '_, Mini<'inp, Src>, Ctx>,
/// ) -> Result<(u32, u32), ErrorOf<'inp, Mini<'inp, Src>, Ctx, ()>>
/// where
///   Src: Bytes + ?Sized + 'inp,
///   Ctx: ComposableParseContext<'inp, Mini<'inp, Src>>,
/// {
///   let a = Any::of().parse_input(inp)?;
///   let b = Any::of().parse_input(inp)?;
///   Ok((a.0, b.0))
/// }
///
/// let parsed: Result<(u32, u32), Error> = parse(two_digits, "42");
/// assert_eq!(parsed.unwrap(), (4, 2));
/// ```
///
/// # Negative control
///
/// The same function handed to the builder does **not** type-check, because the builder is
/// configured before a source exists and `Mini`'s source parameter has nothing else to come
/// from. This is the defect the free functions exist to remove; the block above is the same
/// call with the source supplied as a value.
///
#[cfg_attr(feature = "any", doc = "```compile_fail,E0283")]
#[cfg_attr(not(feature = "any"), doc = "```compile_fail,ignore")]
/// # use core::{convert::Infallible, fmt};
/// # use tokora::{
/// #   ComposableParseContext, ErrorOf, InputRef, Lexer, ParseInput as _, Parser, SimpleSpan,
/// #   Source, Token, combinator::Any,
/// #   error::{
/// #     FullContainer, MissingSyntax, MissingToken, SeparatedError, TooFew, UnexpectedEot,
/// #     UnexpectedToken,
/// #   },
/// #   span::Span as _,
/// # };
/// # #[derive(Debug)] struct Error;
/// # impl From<Infallible> for Error { fn from(e: Infallible) -> Self { match e {} } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for Error { fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, K, S, Lang>> for Error { fn from(_: SeparatedError<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, K: Clone, O, Lang: ?Sized> From<MissingToken<'a, K, O, Lang>> for Error { fn from(_: MissingToken<'a, K, O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for Error { fn from(_: MissingSyntax<O, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for Error { fn from(_: FullContainer<S, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for Error { fn from(_: TooFew<S, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEot<O, Lang, Set>> for Error { fn from(_: UnexpectedEot<O, Lang, Set>) -> Self { Error } }
/// # impl<'a, L: Lexer<'a>, Lang: ?Sized> tokora::emitter::FromUnclosed<'a, L, Lang> for Error { fn from_unclosed<D>(_: tokora::error::Unclosed<D, L::Span, Lang>) -> Self { Error } }
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] struct Kind;
/// # impl fmt::Display for Kind { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("digit") } }
/// # #[derive(Debug, Clone, Copy, PartialEq)] struct Digit(u32);
/// # impl Token<'_> for Digit { type Kind = Kind; type Error = Infallible; const SCAN_LOOKAHEAD: tokora::ScanLookahead = tokora::ScanLookahead::Unbounded; fn kind(&self) -> Kind { Kind } fn is_trivia(&self) -> bool { false } }
/// # trait Bytes: Source<usize> { fn byte(&self, at: usize) -> Option<u8>; }
/// # impl Bytes for str { fn byte(&self, at: usize) -> Option<u8> { self.as_bytes().get(at).copied() } }
/// # impl Bytes for [u8] { fn byte(&self, at: usize) -> Option<u8> { self.get(at).copied() } }
/// # struct Mini<'a, Src: ?Sized> { src: &'a Src, pos: usize, tok: SimpleSpan, state: () }
/// # impl<'a, Src: Bytes + ?Sized + 'a> Lexer<'a> for Mini<'a, Src> {
/// #   type State = (); type Source = Src; type Token = Digit; type Span = SimpleSpan; type Offset = usize;
/// #   fn new(src: &'a Src) -> Self { Self { src, pos: 0, tok: SimpleSpan::new(0, 0), state: () } }
/// #   fn with_state(src: &'a Src, _: ()) -> Self { Self::new(src) }
/// #   fn check(&self) -> Result<(), Infallible> { Ok(()) }
/// #   fn state(&self) -> &() { &self.state }
/// #   fn state_mut(&mut self) -> &mut () { &mut self.state }
/// #   fn into_state(self) {}
/// #   fn source(&self) -> &'a Src { self.src }
/// #   fn span(&self) -> SimpleSpan { self.tok }
/// #   fn slice(&self) -> <Src as Source<usize>>::Slice<'a> { self.src.slice(self.tok.start()..self.tok.end()).unwrap() }
/// #   fn lex(&mut self) -> Option<Result<Digit, Infallible>> {
/// #     let byte = self.src.byte(self.pos)?;
/// #     let start = self.pos;
/// #     self.pos += 1;
/// #     self.tok = SimpleSpan::new(start, self.pos);
/// #     Some(Ok(Digit(u32::from(byte - b'0'))))
/// #   }
/// #   fn read_frontier(&self) -> tokora::ReadFrontier<usize> { tokora::ReadFrontier::SpanEnd }
/// #   fn bump(&mut self, n: &usize) { self.pos += n; }
/// # }
/// # fn two_digits<'inp, Src, Ctx>(
/// #   inp: &mut InputRef<'inp, '_, Mini<'inp, Src>, Ctx>,
/// # ) -> Result<(u32, u32), ErrorOf<'inp, Mini<'inp, Src>, Ctx, ()>>
/// # where
/// #   Src: Bytes + ?Sized + 'inp,
/// #   Ctx: ComposableParseContext<'inp, Mini<'inp, Src>>,
/// # {
/// #   let a = Any::of().parse_input(inp)?;
/// #   let b = Any::of().parse_input(inp)?;
/// #   Ok((a.0, b.0))
/// # }
/// let parser = Parser::with_parser(two_digits);
/// ```
#[inline(always)]
pub fn parse<'inp, L, S, F, O, E, Lang>(f: F, src: &'inp S) -> Result<O, E>
where
  S: ?Sized,
  L: Lexer<'inp, Source = S>,
  L::State: Default,
  E: FromTokenErrors<'inp, L, Lang>,
  F: ParseInput<'inp, L, O, FatalContext<'inp, L, E, Lang>, Lang>,
  Lang: ?Sized,
{
  parse_with_state(f, src, L::State::default())
}

/// Runs `f` over `src` with an explicit lexer state, failing fast on the first error.
///
/// The stateful twin of [`parse()`]: same inference, plus the resume state a lexer that carries
/// one needs. See [`parse()`] for what is inferred from where.
#[inline(always)]
pub fn parse_with_state<'inp, L, S, F, O, E, Lang>(
  f: F,
  src: &'inp S,
  state: L::State,
) -> Result<O, E>
where
  S: ?Sized,
  L: Lexer<'inp, Source = S>,
  E: FromTokenErrors<'inp, L, Lang>,
  F: ParseInput<'inp, L, O, FatalContext<'inp, L, E, Lang>, Lang>,
  Lang: ?Sized,
{
  Parser::with_parser(f).parse_with_state(src, state)
}

/// Runs `f` over `src` against a caller-supplied context.
///
/// The context twin of [`parse()`], for a collecting emitter or a configured cache. The error
/// type is the context's own — [`ErrorOf`] — rather than a free parameter, so
/// there is nothing to annotate and nothing to keep in step.
#[inline(always)]
pub fn parse_with<'inp, L, S, F, O, Ctx, Lang>(
  f: F,
  src: &'inp S,
  ctx: Ctx,
) -> Result<O, ErrorOf<'inp, L, Ctx, Lang>>
where
  S: ?Sized,
  L: Lexer<'inp, Source = S>,
  L::State: Default,
  Ctx: ParseContext<'inp, L, Lang>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Lang: ?Sized,
{
  Parser::with_parser_and_context(f, ctx).parse(src)
}

/// Type-level function for configuration transformations.
///
/// This trait enables progressive parser configuration by transforming
/// one configuration type into another. For example:
///
/// - `()` → `WithEmitter<E>` (add emitter configuration)
/// - `()` → `WithCache<C>` (add cache configuration)
///
/// Used internally by `.with_emitter()` and `.with_cache()` methods.
pub trait Apply<State> {
  /// The input required to perform the transformation
  type Options;

  /// Transform `self` into `State` using the provided `options`.
  fn apply(self, options: Self::Options) -> State;
}

/// A hint used during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Action {
  /// Indicates the token belongs to another syntactic element, hint to stop parsing.
  #[unwrap(ignore)]
  #[try_unwrap(ignore)]
  Stop,
  /// Indicates a token belongs to an element was found, hint to continue parsing.
  #[unwrap(ignore)]
  #[try_unwrap(ignore)]
  Continue,
}

/// Opens a **Sans-I/O partial parse session** over `src` and drives the parser `f` — any
/// [`ParseInput`]`<'inp, L, O, Ctx, Lang, `[`Partial`]`>` — returning whatever
/// `f` returns.
///
/// This is the partial-mode entry point, the [`Partial`] instantiation of the same trait
/// vocabulary the complete-only [`Parse`] API drives at its default: a typed fn item, a named
/// combinator chain, or a parser written generic over its completeness parameter all satisfy the
/// bound (write once, run in both modes — see the [`input`](crate::input) module docs). `f`
/// receives an [`InputRef`](crate::InputRef) in [`Partial`] mode set to `is_final`, so the
/// [frontier rules](crate::input) are active while non-final: consuming across the frontier
/// surfaces an [`Incomplete`](crate::error::Incomplete) on the `Err` channel, which the parser
/// propagates out. `ctx` supplies the emitter and cache exactly as [`Parse`] does.
///
/// The `Partial: SurfaceIncomplete` bound requires that the emitter's error implements
/// `From<Incomplete<L::Offset>>` (constructing the frontier sentinel) and
/// [`MaybeIncomplete`](crate::error::MaybeIncomplete) (recognizing it — the atom-layer
/// never-recoverable gate and the caller's own refill check); complete parses see neither.
///
/// # The refill loop
///
/// The caller owns the buffer. Each attempt builds a fresh input over the current buffer slice, so
/// there is **no growable source inside tokora**: on an incomplete result the caller appends the
/// next chunk to *its own* buffer and calls this again over the larger slice. Inside `f`, parsing
/// under a rollback-on-drop [`Transaction`](crate::input::Transaction) means an incomplete attempt unwinds its emissions and
/// cursor cleanly before the retry — a transaction `f` opens, because this driver deliberately
/// wraps nothing. See the [`input`](crate::input) module docs for the full runnable loop and the one-token
/// frontier-latency guarantee.
///
/// # Cost
///
/// Every attempt re-lexes the whole prefix, so cumulative lexer work across a refill loop is
/// **Θ(Σ attempt lengths)** — the triangular figure under one-token refills. This function returns
/// nothing that could carry a committed frontier or a cross-attempt budget, so a caller facing
/// untrusted arrival fragmentation should drive [`PartialSession`](crate::input::PartialSession) instead, whose [`Budget`](crate::input::Budget) bounds
/// exactly that sum.
#[inline]
pub fn parse_partial<'inp, L, Ctx, Lang, O, P>(
  ctx: Ctx,
  src: &'inp L::Source,
  state: L::State,
  is_final: bool,
  mut f: P,
) -> Result<O, <Ctx::Emitter as crate::Emitter<'inp, L, Lang>>::Error>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Lang: ?Sized,
  Ctx: ParseContext<'inp, L, Lang>,
  Partial: SurfaceIncomplete<'inp, L, Ctx, Lang>,
  P: crate::ParseInput<'inp, L, O, Ctx, Lang, Partial>,
{
  let mut input = Input::<L, Ctx, Lang, Partial>::with_state_and_context(src, state, ctx.provide());
  // The driver states the world fact BEFORE the parser ever sees a handle — and it is the only
  // party that can. `seal` takes `&mut Input`, so it is unreachable from the `&mut InputRef` `f`
  // is handed: `f` cannot end the stream, at any depth, inside any speculative branch.
  if is_final {
    input.seal();
  }
  let mut input_ref = input.as_ref();
  f.parse_input(&mut input_ref)
}
