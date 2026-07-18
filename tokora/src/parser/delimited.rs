//! Delimited shape atoms: commit an opener, run an inner sub-parser, commit the
//! closer, and wrap the three in a span-carrying [`Delimited`].
//!
//! [`delimited`] is the generic form, taking the [`Delimiter`](crate::delimiter::Delimiter)
//! pair as a type parameter through the [`TypedDelimiter`] capability; [`parens`],
//! [`braces`], [`brackets`], and [`angles`] are the named conveniences fixing that pair
//! to a built-in. Each returns a builder-form parser — a `for<'c> FnMut(&mut InputRef<…>)`
//! closure — so it composes over every lexer, source, and emitter, and drops straight
//! into another combinator with no adapter.
//!
//! A missing closer is a hard error: the closer's unexpected-token or end-of-input error
//! propagates rather than fabricating a delimiter, so an unterminated group fails under a
//! fail-fast and a collecting emitter alike. This family never fires the
//! [`Unclosed`](crate::error::Unclosed)/[`Unopened`](crate::error::Unopened)/
//! [`Undelimited`](crate::error::Undelimited) recovery vocabulary — a caller that wants it
//! holds the region's start cursor and can map at the call site.

use crate::{
  ErrorOf, Lexer, ParseCtx, Token,
  delimiter::TypedDelimiter,
  error::{UnexpectedEot, token::UnexpectedToken},
  input::InputRef,
  punct::{
    CloseAngle, CloseBrace, CloseBracket, CloseParen, OpenAngle, OpenBrace, OpenBracket, OpenParen,
  },
  span::Span as _,
  token::PunctuatorToken,
  utils::Delimited,
};

/// The result the parser [`delimited`] builds yields: `inner`'s output wrapped in a
/// `D`-delimited [`Delimited`] spanning the whole construct, or the propagated error.
pub type DelimitedOf<'inp, D, L, Ctx, Lang, T> = Result<
  Delimited<
    <D as TypedDelimiter<'inp, L, Lang>>::OpenValue,
    <D as TypedDelimiter<'inp, L, Lang>>::CloseValue,
    T,
    <L as Lexer<'inp>>::Span,
  >,
  ErrorOf<'inp, L, Ctx, Lang>,
>;

/// Commits the `D` opener, runs `inner`, commits the `D` closer, and returns the three as
/// a [`Delimited`] whose span covers the whole construct.
///
/// `D` is a [`Delimiter`](crate::delimiter::Delimiter) pair passed as the first type
/// parameter (`delimited::<Paren, …>(inner)`, mirroring the many-builder's
/// [`.delimited::<D>()`](crate::parser::Separated::delimited) and the atom
/// [`separated1::<Sep, …>`](crate::parser::separated1)); its [`TypedDelimiter`] impl
/// supplies the span-carrying punctuator values the result stores. For the built-in pairs,
/// `delimited::<Paren, …>(inner)` is equivalent to [`parens(inner)`](parens) — and likewise
/// for [`braces`]/[`brackets`]/[`angles`] — for any vocabulary whose two capability
/// declarations (`PunctuatorToken::open_paren` and `Kind: From<OpenParen<(), (), ()>>`)
/// agree, as all the built-in fixtures do.
///
/// `inner` runs between the committed delimiters and its output becomes the [`Delimited`]
/// data. A missing closer is not recovered: the closer's error — an unexpected token or end
/// of input — propagates, so an unterminated group fails rather than fabricating a delimiter.
///
/// # Examples
///
/// ```rust
/// # use core::{convert::Infallible, fmt};
/// # use tokora::{
/// #   FatalContext, InputRef, Lexer, SimpleSpan, Token,
/// #   error::{UnexpectedEot, syntax::{FullContainer, MissingSyntax, TooFew, TooMany}, token::{MissingToken, SeparatedError, UnexpectedToken}},
/// #   punct::{CloseAngle, CloseBrace, CloseBracket, CloseParen, OpenAngle, OpenBrace, OpenBracket, OpenParen},
/// #   span::Span as _,
/// #   token::PunctuatorToken,
/// # };
/// # #[derive(Debug, PartialEq)]
/// # struct Error;
/// # impl From<Infallible> for Error { fn from(e: Infallible) -> Self { match e {} } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for Error { fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, K, S, Lang>> for Error { fn from(_: SeparatedError<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, K: Clone, O, Lang: ?Sized> From<MissingToken<'a, K, O, Lang>> for Error { fn from(_: MissingToken<'a, K, O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for Error { fn from(_: UnexpectedEot<O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for Error { fn from(_: MissingSyntax<O, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for Error { fn from(_: FullContainer<S, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for Error { fn from(_: TooFew<S, Lang>) -> Self { Error } }
/// # #[derive(Debug, Clone, PartialEq)]
/// # enum Tok { Digit(u32), LParen, RParen, LBracket, RBracket, LBrace, RBrace, LAngle, RAngle }
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # enum Kind { Digit, LParen, RParen, LBracket, RBracket, LBrace, RBrace, LAngle, RAngle }
/// # impl fmt::Display for Kind { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") } }
/// # impl Token<'_> for Tok {
/// #   type Kind = Kind;
/// #   type Error = Infallible;
/// #   fn kind(&self) -> Kind { match self {
/// #     Tok::Digit(_) => Kind::Digit,
/// #     Tok::LParen => Kind::LParen, Tok::RParen => Kind::RParen,
/// #     Tok::LBracket => Kind::LBracket, Tok::RBracket => Kind::RBracket,
/// #     Tok::LBrace => Kind::LBrace, Tok::RBrace => Kind::RBrace,
/// #     Tok::LAngle => Kind::LAngle, Tok::RAngle => Kind::RAngle } }
/// #   fn is_trivia(&self) -> bool { false }
/// # }
/// # impl PunctuatorToken<'_> for Tok {
/// #   fn open_paren() -> Option<Kind> { Some(Kind::LParen) }
/// #   fn close_paren() -> Option<Kind> { Some(Kind::RParen) }
/// #   fn open_bracket() -> Option<Kind> { Some(Kind::LBracket) }
/// #   fn close_bracket() -> Option<Kind> { Some(Kind::RBracket) }
/// #   fn open_brace() -> Option<Kind> { Some(Kind::LBrace) }
/// #   fn close_brace() -> Option<Kind> { Some(Kind::RBrace) }
/// #   fn open_angle() -> Option<Kind> { Some(Kind::LAngle) }
/// #   fn close_angle() -> Option<Kind> { Some(Kind::RAngle) }
/// # }
/// # impl From<OpenParen<(), (), ()>> for Kind { fn from(_: OpenParen<(), (), ()>) -> Self { Kind::LParen } }
/// # impl From<CloseParen<(), (), ()>> for Kind { fn from(_: CloseParen<(), (), ()>) -> Self { Kind::RParen } }
/// # impl From<OpenBracket<(), (), ()>> for Kind { fn from(_: OpenBracket<(), (), ()>) -> Self { Kind::LBracket } }
/// # impl From<CloseBracket<(), (), ()>> for Kind { fn from(_: CloseBracket<(), (), ()>) -> Self { Kind::RBracket } }
/// # impl From<OpenBrace<(), (), ()>> for Kind { fn from(_: OpenBrace<(), (), ()>) -> Self { Kind::LBrace } }
/// # impl From<CloseBrace<(), (), ()>> for Kind { fn from(_: CloseBrace<(), (), ()>) -> Self { Kind::RBrace } }
/// # impl From<OpenAngle<(), (), ()>> for Kind { fn from(_: OpenAngle<(), (), ()>) -> Self { Kind::LAngle } }
/// # impl From<CloseAngle<(), (), ()>> for Kind { fn from(_: CloseAngle<(), (), ()>) -> Self { Kind::RAngle } }
/// # struct CharLexer<'a> { src: &'a str, pos: usize, tok: SimpleSpan, state: () }
/// # impl<'a> Lexer<'a> for CharLexer<'a> {
/// #   type State = (); type Source = str; type Token = Tok; type Span = SimpleSpan; type Offset = usize;
/// #   fn new(src: &'a str) -> Self { Self { src, pos: 0, tok: SimpleSpan::new(0, 0), state: () } }
/// #   fn with_state(src: &'a str, _: ()) -> Self { Self::new(src) }
/// #   fn check(&self) -> Result<(), Infallible> { Ok(()) }
/// #   fn state(&self) -> &() { &self.state }
/// #   fn state_mut(&mut self) -> &mut () { &mut self.state }
/// #   fn into_state(self) -> Self::State {}
/// #   fn source(&self) -> &'a str { self.src }
/// #   fn span(&self) -> SimpleSpan { self.tok }
/// #   fn slice(&self) -> &'a str { &self.src[self.tok.start()..self.tok.end()] }
/// #   fn lex(&mut self) -> Option<Result<Tok, Infallible>> {
/// #     let bytes = self.src.as_bytes();
/// #     while self.pos < bytes.len() && bytes[self.pos] == b' ' { self.pos += 1; }
/// #     if self.pos >= bytes.len() { return None; }
/// #     let (start, c) = (self.pos, bytes[self.pos] as char);
/// #     self.pos += 1;
/// #     self.tok = SimpleSpan::new(start, self.pos);
/// #     Some(Ok(match c {
/// #       '0'..='9' => Tok::Digit(c as u32 - '0' as u32),
/// #       '(' => Tok::LParen, ')' => Tok::RParen, '[' => Tok::LBracket, ']' => Tok::RBracket,
/// #       '{' => Tok::LBrace, '}' => Tok::RBrace, '<' => Tok::LAngle, '>' => Tok::RAngle,
/// #       _ => Tok::Digit(0),
/// #     }))
/// #   }
/// #   fn bump(&mut self, n: &usize) { self.pos += n; }
/// # }
/// # type Ctx<'a> = FatalContext<'a, CharLexer<'a>, Error>;
/// # fn digit<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
/// #   match inp.try_expect(|t| matches!(t.data, Tok::Digit(_)))? {
/// #     Some(sp) => match sp.data { Tok::Digit(n) => Ok(n), _ => unreachable!() },
/// #     None => Err(Error),
/// #   }
/// # }
/// use tokora::{Parse, Parser, punct::Paren, parser::delimited};
///
/// // A parenthesized digit through the generic form, fixing the pair to `Paren`.
/// fn paren_digit<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
///     delimited::<Paren, _, _, _, _, _>(digit)(inp).map(|d| *d.data())
/// }
///
/// assert_eq!(Parser::with_parser(paren_digit).parse_str("(1)").unwrap(), 1);
/// // A missing closer is a hard error, not a fabricated delimiter.
/// assert!(Parser::with_parser(paren_digit).parse_str("(1").is_err());
/// ```
#[inline]
pub fn delimited<'inp, D, L, Ctx, Lang, P, T>(
  mut inner: P,
) -> impl for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> DelimitedOf<'inp, D, L, Ctx, Lang, T>
where
  D: TypedDelimiter<'inp, L, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseCtx<'inp, L, Lang>,
  Lang: ?Sized,
  P: for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> Result<T, ErrorOf<'inp, L, Ctx, Lang>>,
  ErrorOf<'inp, L, Ctx, Lang>: From<UnexpectedEot<L::Offset, Lang>>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
{
  move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
    let cursor = inp.cursor().clone();
    let open_span = match inp.next()? {
      Some(sp) if D::is_open(&sp.data().kind()) => sp.into_span(),
      Some(sp) => return Err(D::unexpected_open_token(sp).into()),
      None => return Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    };
    let data = inner(inp)?;
    let close_span = match inp.next()? {
      Some(sp) if D::is_close(&sp.data().kind()) => sp.into_span(),
      Some(sp) => return Err(D::unexpected_close_token(sp).into()),
      None => return Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    };
    Ok(Delimited::new(
      D::open_value(open_span),
      D::close_value(close_span),
      data,
      inp.span_since(&cursor),
    ))
  }
}

/// The result [`parens`] returns: `inner`'s output wrapped in a paren-delimited
/// [`Delimited`] spanning the whole `( … )`, or the propagated error.
pub type ParensOf<'inp, L, Ctx, Lang, T> = Result<
  Delimited<
    OpenParen<<L as Lexer<'inp>>::Span, (), Lang>,
    CloseParen<<L as Lexer<'inp>>::Span, (), Lang>,
    T,
    <L as Lexer<'inp>>::Span,
  >,
  ErrorOf<'inp, L, Ctx, Lang>,
>;

/// Commits the `(` opener, runs `inner`, commits the `)` closer, and returns the
/// three as a [`Delimited`] whose span covers the whole `( … )`.
///
/// `inner` runs between the committed delimiters and its output becomes the
/// [`Delimited`] data. A missing closer is not recovered: the closer atom's error
/// — an unexpected token or end of input — propagates, so an unterminated `( …`
/// fails rather than fabricating a paren.
///
/// # Examples
///
/// ```rust
/// # use core::{convert::Infallible, fmt};
/// # use tokora::{
/// #   FatalContext, InputRef, Lexer, SimpleSpan, Token,
/// #   error::{UnexpectedEot, syntax::{FullContainer, MissingSyntax, TooFew, TooMany}, token::{MissingToken, SeparatedError, UnexpectedToken}},
/// #   punct::{CloseAngle, CloseBrace, CloseBracket, CloseParen, OpenAngle, OpenBrace, OpenBracket, OpenParen},
/// #   span::Span as _,
/// #   token::PunctuatorToken,
/// # };
/// # #[derive(Debug, PartialEq)]
/// # struct Error;
/// # impl From<Infallible> for Error { fn from(e: Infallible) -> Self { match e {} } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for Error { fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, K, S, Lang>> for Error { fn from(_: SeparatedError<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, K: Clone, O, Lang: ?Sized> From<MissingToken<'a, K, O, Lang>> for Error { fn from(_: MissingToken<'a, K, O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for Error { fn from(_: UnexpectedEot<O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for Error { fn from(_: MissingSyntax<O, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for Error { fn from(_: FullContainer<S, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for Error { fn from(_: TooFew<S, Lang>) -> Self { Error } }
/// # #[derive(Debug, Clone, PartialEq)]
/// # enum Tok { Digit(u32), LParen, RParen }
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # enum Kind { Digit, LParen, RParen }
/// # impl fmt::Display for Kind { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") } }
/// # impl Token<'_> for Tok {
/// #   type Kind = Kind;
/// #   type Error = Infallible;
/// #   fn kind(&self) -> Kind { match self { Tok::Digit(_) => Kind::Digit, Tok::LParen => Kind::LParen, Tok::RParen => Kind::RParen } }
/// #   fn is_trivia(&self) -> bool { false }
/// # }
/// # impl PunctuatorToken<'_> for Tok {
/// #   fn open_paren() -> Option<Kind> { Some(Kind::LParen) }
/// #   fn close_paren() -> Option<Kind> { Some(Kind::RParen) }
/// # }
/// # impl From<OpenParen<(), (), ()>> for Kind { fn from(_: OpenParen<(), (), ()>) -> Self { Kind::LParen } }
/// # impl From<CloseParen<(), (), ()>> for Kind { fn from(_: CloseParen<(), (), ()>) -> Self { Kind::RParen } }
/// # struct CharLexer<'a> { src: &'a str, pos: usize, tok: SimpleSpan, state: () }
/// # impl<'a> Lexer<'a> for CharLexer<'a> {
/// #   type State = (); type Source = str; type Token = Tok; type Span = SimpleSpan; type Offset = usize;
/// #   fn new(src: &'a str) -> Self { Self { src, pos: 0, tok: SimpleSpan::new(0, 0), state: () } }
/// #   fn with_state(src: &'a str, _: ()) -> Self { Self::new(src) }
/// #   fn check(&self) -> Result<(), Infallible> { Ok(()) }
/// #   fn state(&self) -> &() { &self.state }
/// #   fn state_mut(&mut self) -> &mut () { &mut self.state }
/// #   fn into_state(self) -> Self::State {}
/// #   fn source(&self) -> &'a str { self.src }
/// #   fn span(&self) -> SimpleSpan { self.tok }
/// #   fn slice(&self) -> &'a str { &self.src[self.tok.start()..self.tok.end()] }
/// #   fn lex(&mut self) -> Option<Result<Tok, Infallible>> {
/// #     let bytes = self.src.as_bytes();
/// #     while self.pos < bytes.len() && bytes[self.pos] == b' ' { self.pos += 1; }
/// #     if self.pos >= bytes.len() { return None; }
/// #     let (start, c) = (self.pos, bytes[self.pos] as char);
/// #     self.pos += 1;
/// #     self.tok = SimpleSpan::new(start, self.pos);
/// #     Some(Ok(match c { '0'..='9' => Tok::Digit(c as u32 - '0' as u32), '(' => Tok::LParen, ')' => Tok::RParen, _ => Tok::Digit(0) }))
/// #   }
/// #   fn bump(&mut self, n: &usize) { self.pos += n; }
/// # }
/// # type Ctx<'a> = FatalContext<'a, CharLexer<'a>, Error>;
/// # fn digit<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
/// #   match inp.try_expect(|t| matches!(t.data, Tok::Digit(_)))? {
/// #     Some(sp) => match sp.data { Tok::Digit(n) => Ok(n), _ => unreachable!() },
/// #     None => Err(Error),
/// #   }
/// # }
/// use tokora::{Parse, Parser, parser::parens};
///
/// fn wrapped<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
///     parens(digit)(inp).map(|d| *d.data())
/// }
///
/// assert_eq!(Parser::with_parser(wrapped).parse_str("(1)").unwrap(), 1);
/// assert!(Parser::with_parser(wrapped).parse_str("(1").is_err());
/// ```
#[inline]
pub fn parens<'inp, L, Ctx, Lang, P, T>(
  mut inner: P,
) -> impl for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> ParensOf<'inp, L, Ctx, Lang, T>
where
  L: Lexer<'inp>,
  L::Token: PunctuatorToken<'inp>,
  Ctx: ParseCtx<'inp, L, Lang>,
  Lang: ?Sized,
  P: for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> Result<T, ErrorOf<'inp, L, Ctx, Lang>>,
  ErrorOf<'inp, L, Ctx, Lang>: From<UnexpectedEot<L::Offset, Lang>>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
{
  move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
    let cursor = inp.cursor().clone();
    let open = OpenParen::parse_of(inp)?;
    let data = inner(inp)?;
    let close = CloseParen::parse_of(inp)?;
    Ok(Delimited::new(open, close, data, inp.span_since(&cursor)))
  }
}

/// The result [`braces`] returns: `inner`'s output wrapped in a brace-delimited
/// [`Delimited`] spanning the whole `{ … }`, or the propagated error.
pub type BracesOf<'inp, L, Ctx, Lang, T> = Result<
  Delimited<
    OpenBrace<<L as Lexer<'inp>>::Span, (), Lang>,
    CloseBrace<<L as Lexer<'inp>>::Span, (), Lang>,
    T,
    <L as Lexer<'inp>>::Span,
  >,
  ErrorOf<'inp, L, Ctx, Lang>,
>;

/// Commits the `{` opener, runs `inner`, commits the `}` closer, and returns the
/// three as a [`Delimited`] whose span covers the whole `{ … }`.
///
/// `inner` runs between the committed delimiters and its output becomes the
/// [`Delimited`] data. A missing closer is not recovered: the closer atom's error
/// — an unexpected token or end of input — propagates, so an unterminated `{ …`
/// fails rather than fabricating a brace.
///
/// # Examples
///
/// ```rust
/// # use core::{convert::Infallible, fmt};
/// # use tokora::{
/// #   FatalContext, InputRef, Lexer, SimpleSpan, Token,
/// #   error::{UnexpectedEot, syntax::{FullContainer, MissingSyntax, TooFew, TooMany}, token::{MissingToken, SeparatedError, UnexpectedToken}},
/// #   punct::{CloseAngle, CloseBrace, CloseBracket, CloseParen, OpenAngle, OpenBrace, OpenBracket, OpenParen},
/// #   span::Span as _,
/// #   token::PunctuatorToken,
/// # };
/// # #[derive(Debug, PartialEq)]
/// # struct Error;
/// # impl From<Infallible> for Error { fn from(e: Infallible) -> Self { match e {} } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for Error { fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, K, S, Lang>> for Error { fn from(_: SeparatedError<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, K: Clone, O, Lang: ?Sized> From<MissingToken<'a, K, O, Lang>> for Error { fn from(_: MissingToken<'a, K, O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for Error { fn from(_: UnexpectedEot<O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for Error { fn from(_: MissingSyntax<O, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for Error { fn from(_: FullContainer<S, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for Error { fn from(_: TooFew<S, Lang>) -> Self { Error } }
/// # #[derive(Debug, Clone, PartialEq)]
/// # enum Tok { Digit(u32), LBrace, RBrace }
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # enum Kind { Digit, LBrace, RBrace }
/// # impl fmt::Display for Kind { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") } }
/// # impl Token<'_> for Tok {
/// #   type Kind = Kind;
/// #   type Error = Infallible;
/// #   fn kind(&self) -> Kind { match self { Tok::Digit(_) => Kind::Digit, Tok::LBrace => Kind::LBrace, Tok::RBrace => Kind::RBrace } }
/// #   fn is_trivia(&self) -> bool { false }
/// # }
/// # impl PunctuatorToken<'_> for Tok {
/// #   fn open_brace() -> Option<Kind> { Some(Kind::LBrace) }
/// #   fn close_brace() -> Option<Kind> { Some(Kind::RBrace) }
/// # }
/// # impl From<OpenBrace<(), (), ()>> for Kind { fn from(_: OpenBrace<(), (), ()>) -> Self { Kind::LBrace } }
/// # impl From<CloseBrace<(), (), ()>> for Kind { fn from(_: CloseBrace<(), (), ()>) -> Self { Kind::RBrace } }
/// # struct CharLexer<'a> { src: &'a str, pos: usize, tok: SimpleSpan, state: () }
/// # impl<'a> Lexer<'a> for CharLexer<'a> {
/// #   type State = (); type Source = str; type Token = Tok; type Span = SimpleSpan; type Offset = usize;
/// #   fn new(src: &'a str) -> Self { Self { src, pos: 0, tok: SimpleSpan::new(0, 0), state: () } }
/// #   fn with_state(src: &'a str, _: ()) -> Self { Self::new(src) }
/// #   fn check(&self) -> Result<(), Infallible> { Ok(()) }
/// #   fn state(&self) -> &() { &self.state }
/// #   fn state_mut(&mut self) -> &mut () { &mut self.state }
/// #   fn into_state(self) -> Self::State {}
/// #   fn source(&self) -> &'a str { self.src }
/// #   fn span(&self) -> SimpleSpan { self.tok }
/// #   fn slice(&self) -> &'a str { &self.src[self.tok.start()..self.tok.end()] }
/// #   fn lex(&mut self) -> Option<Result<Tok, Infallible>> {
/// #     let bytes = self.src.as_bytes();
/// #     while self.pos < bytes.len() && bytes[self.pos] == b' ' { self.pos += 1; }
/// #     if self.pos >= bytes.len() { return None; }
/// #     let (start, c) = (self.pos, bytes[self.pos] as char);
/// #     self.pos += 1;
/// #     self.tok = SimpleSpan::new(start, self.pos);
/// #     Some(Ok(match c { '0'..='9' => Tok::Digit(c as u32 - '0' as u32), '{' => Tok::LBrace, '}' => Tok::RBrace, _ => Tok::Digit(0) }))
/// #   }
/// #   fn bump(&mut self, n: &usize) { self.pos += n; }
/// # }
/// # type Ctx<'a> = FatalContext<'a, CharLexer<'a>, Error>;
/// # fn digit<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
/// #   match inp.try_expect(|t| matches!(t.data, Tok::Digit(_)))? {
/// #     Some(sp) => match sp.data { Tok::Digit(n) => Ok(n), _ => unreachable!() },
/// #     None => Err(Error),
/// #   }
/// # }
/// use tokora::{Parse, Parser, parser::braces};
///
/// fn wrapped<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
///     braces(digit)(inp).map(|d| *d.data())
/// }
///
/// assert_eq!(Parser::with_parser(wrapped).parse_str("{1}").unwrap(), 1);
/// assert!(Parser::with_parser(wrapped).parse_str("{1").is_err());
/// ```
#[inline]
pub fn braces<'inp, L, Ctx, Lang, P, T>(
  mut inner: P,
) -> impl for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> BracesOf<'inp, L, Ctx, Lang, T>
where
  L: Lexer<'inp>,
  L::Token: PunctuatorToken<'inp>,
  Ctx: ParseCtx<'inp, L, Lang>,
  Lang: ?Sized,
  P: for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> Result<T, ErrorOf<'inp, L, Ctx, Lang>>,
  ErrorOf<'inp, L, Ctx, Lang>: From<UnexpectedEot<L::Offset, Lang>>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
{
  move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
    let cursor = inp.cursor().clone();
    let open = OpenBrace::parse_of(inp)?;
    let data = inner(inp)?;
    let close = CloseBrace::parse_of(inp)?;
    Ok(Delimited::new(open, close, data, inp.span_since(&cursor)))
  }
}

/// The result [`brackets`] returns: `inner`'s output wrapped in a
/// bracket-delimited [`Delimited`] spanning the whole `[ … ]`, or the propagated
/// error.
pub type BracketsOf<'inp, L, Ctx, Lang, T> = Result<
  Delimited<
    OpenBracket<<L as Lexer<'inp>>::Span, (), Lang>,
    CloseBracket<<L as Lexer<'inp>>::Span, (), Lang>,
    T,
    <L as Lexer<'inp>>::Span,
  >,
  ErrorOf<'inp, L, Ctx, Lang>,
>;

/// Commits the `[` opener, runs `inner`, commits the `]` closer, and returns the
/// three as a [`Delimited`] whose span covers the whole `[ … ]`.
///
/// `inner` runs between the committed delimiters and its output becomes the
/// [`Delimited`] data. A missing closer is not recovered: the closer atom's error
/// — an unexpected token or end of input — propagates, so an unterminated `[ …`
/// fails rather than fabricating a bracket.
///
/// # Examples
///
/// ```rust
/// # use core::{convert::Infallible, fmt};
/// # use tokora::{
/// #   FatalContext, InputRef, Lexer, SimpleSpan, Token,
/// #   error::{UnexpectedEot, syntax::{FullContainer, MissingSyntax, TooFew, TooMany}, token::{MissingToken, SeparatedError, UnexpectedToken}},
/// #   punct::{CloseAngle, CloseBrace, CloseBracket, CloseParen, OpenAngle, OpenBrace, OpenBracket, OpenParen},
/// #   span::Span as _,
/// #   token::PunctuatorToken,
/// # };
/// # #[derive(Debug, PartialEq)]
/// # struct Error;
/// # impl From<Infallible> for Error { fn from(e: Infallible) -> Self { match e {} } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for Error { fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, K, S, Lang>> for Error { fn from(_: SeparatedError<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, K: Clone, O, Lang: ?Sized> From<MissingToken<'a, K, O, Lang>> for Error { fn from(_: MissingToken<'a, K, O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for Error { fn from(_: UnexpectedEot<O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for Error { fn from(_: MissingSyntax<O, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for Error { fn from(_: FullContainer<S, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for Error { fn from(_: TooFew<S, Lang>) -> Self { Error } }
/// # #[derive(Debug, Clone, PartialEq)]
/// # enum Tok { Digit(u32), LBracket, RBracket }
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # enum Kind { Digit, LBracket, RBracket }
/// # impl fmt::Display for Kind { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") } }
/// # impl Token<'_> for Tok {
/// #   type Kind = Kind;
/// #   type Error = Infallible;
/// #   fn kind(&self) -> Kind { match self { Tok::Digit(_) => Kind::Digit, Tok::LBracket => Kind::LBracket, Tok::RBracket => Kind::RBracket } }
/// #   fn is_trivia(&self) -> bool { false }
/// # }
/// # impl PunctuatorToken<'_> for Tok {
/// #   fn open_bracket() -> Option<Kind> { Some(Kind::LBracket) }
/// #   fn close_bracket() -> Option<Kind> { Some(Kind::RBracket) }
/// # }
/// # impl From<OpenBracket<(), (), ()>> for Kind { fn from(_: OpenBracket<(), (), ()>) -> Self { Kind::LBracket } }
/// # impl From<CloseBracket<(), (), ()>> for Kind { fn from(_: CloseBracket<(), (), ()>) -> Self { Kind::RBracket } }
/// # struct CharLexer<'a> { src: &'a str, pos: usize, tok: SimpleSpan, state: () }
/// # impl<'a> Lexer<'a> for CharLexer<'a> {
/// #   type State = (); type Source = str; type Token = Tok; type Span = SimpleSpan; type Offset = usize;
/// #   fn new(src: &'a str) -> Self { Self { src, pos: 0, tok: SimpleSpan::new(0, 0), state: () } }
/// #   fn with_state(src: &'a str, _: ()) -> Self { Self::new(src) }
/// #   fn check(&self) -> Result<(), Infallible> { Ok(()) }
/// #   fn state(&self) -> &() { &self.state }
/// #   fn state_mut(&mut self) -> &mut () { &mut self.state }
/// #   fn into_state(self) -> Self::State {}
/// #   fn source(&self) -> &'a str { self.src }
/// #   fn span(&self) -> SimpleSpan { self.tok }
/// #   fn slice(&self) -> &'a str { &self.src[self.tok.start()..self.tok.end()] }
/// #   fn lex(&mut self) -> Option<Result<Tok, Infallible>> {
/// #     let bytes = self.src.as_bytes();
/// #     while self.pos < bytes.len() && bytes[self.pos] == b' ' { self.pos += 1; }
/// #     if self.pos >= bytes.len() { return None; }
/// #     let (start, c) = (self.pos, bytes[self.pos] as char);
/// #     self.pos += 1;
/// #     self.tok = SimpleSpan::new(start, self.pos);
/// #     Some(Ok(match c { '0'..='9' => Tok::Digit(c as u32 - '0' as u32), '[' => Tok::LBracket, ']' => Tok::RBracket, _ => Tok::Digit(0) }))
/// #   }
/// #   fn bump(&mut self, n: &usize) { self.pos += n; }
/// # }
/// # type Ctx<'a> = FatalContext<'a, CharLexer<'a>, Error>;
/// # fn digit<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
/// #   match inp.try_expect(|t| matches!(t.data, Tok::Digit(_)))? {
/// #     Some(sp) => match sp.data { Tok::Digit(n) => Ok(n), _ => unreachable!() },
/// #     None => Err(Error),
/// #   }
/// # }
/// use tokora::{Parse, Parser, parser::brackets};
///
/// fn wrapped<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
///     brackets(digit)(inp).map(|d| *d.data())
/// }
///
/// assert_eq!(Parser::with_parser(wrapped).parse_str("[1]").unwrap(), 1);
/// assert!(Parser::with_parser(wrapped).parse_str("[1").is_err());
/// ```
#[inline]
pub fn brackets<'inp, L, Ctx, Lang, P, T>(
  mut inner: P,
) -> impl for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> BracketsOf<'inp, L, Ctx, Lang, T>
where
  L: Lexer<'inp>,
  L::Token: PunctuatorToken<'inp>,
  Ctx: ParseCtx<'inp, L, Lang>,
  Lang: ?Sized,
  P: for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> Result<T, ErrorOf<'inp, L, Ctx, Lang>>,
  ErrorOf<'inp, L, Ctx, Lang>: From<UnexpectedEot<L::Offset, Lang>>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
{
  move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
    let cursor = inp.cursor().clone();
    let open = OpenBracket::parse_of(inp)?;
    let data = inner(inp)?;
    let close = CloseBracket::parse_of(inp)?;
    Ok(Delimited::new(open, close, data, inp.span_since(&cursor)))
  }
}

/// The result [`angles`] returns: `inner`'s output wrapped in an angle-delimited
/// [`Delimited`] spanning the whole `< … >`, or the propagated error.
pub type AnglesOf<'inp, L, Ctx, Lang, T> = Result<
  Delimited<
    OpenAngle<<L as Lexer<'inp>>::Span, (), Lang>,
    CloseAngle<<L as Lexer<'inp>>::Span, (), Lang>,
    T,
    <L as Lexer<'inp>>::Span,
  >,
  ErrorOf<'inp, L, Ctx, Lang>,
>;

/// Commits the `<` opener, runs `inner`, commits the `>` closer, and returns the
/// three as a [`Delimited`] whose span covers the whole `< … >`.
///
/// `inner` runs between the committed delimiters and its output becomes the
/// [`Delimited`] data. A missing closer is not recovered: the closer atom's error
/// — an unexpected token or end of input — propagates, so an unterminated `< …`
/// fails rather than fabricating an angle.
///
/// # Examples
///
/// ```rust
/// # use core::{convert::Infallible, fmt};
/// # use tokora::{
/// #   FatalContext, InputRef, Lexer, SimpleSpan, Token,
/// #   error::{UnexpectedEot, syntax::{FullContainer, MissingSyntax, TooFew, TooMany}, token::{MissingToken, SeparatedError, UnexpectedToken}},
/// #   punct::{CloseAngle, CloseBrace, CloseBracket, CloseParen, OpenAngle, OpenBrace, OpenBracket, OpenParen},
/// #   span::Span as _,
/// #   token::PunctuatorToken,
/// # };
/// # #[derive(Debug, PartialEq)]
/// # struct Error;
/// # impl From<Infallible> for Error { fn from(e: Infallible) -> Self { match e {} } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for Error { fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, T, K: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, K, S, Lang>> for Error { fn from(_: SeparatedError<'a, T, K, S, Lang>) -> Self { Error } }
/// # impl<'a, K: Clone, O, Lang: ?Sized> From<MissingToken<'a, K, O, Lang>> for Error { fn from(_: MissingToken<'a, K, O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for Error { fn from(_: UnexpectedEot<O, Lang>) -> Self { Error } }
/// # impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for Error { fn from(_: MissingSyntax<O, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for Error { fn from(_: FullContainer<S, Lang>) -> Self { Error } }
/// # impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for Error { fn from(_: TooFew<S, Lang>) -> Self { Error } }
/// # #[derive(Debug, Clone, PartialEq)]
/// # enum Tok { Digit(u32), LAngle, RAngle }
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # enum Kind { Digit, LAngle, RAngle }
/// # impl fmt::Display for Kind { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") } }
/// # impl Token<'_> for Tok {
/// #   type Kind = Kind;
/// #   type Error = Infallible;
/// #   fn kind(&self) -> Kind { match self { Tok::Digit(_) => Kind::Digit, Tok::LAngle => Kind::LAngle, Tok::RAngle => Kind::RAngle } }
/// #   fn is_trivia(&self) -> bool { false }
/// # }
/// # impl PunctuatorToken<'_> for Tok {
/// #   fn open_angle() -> Option<Kind> { Some(Kind::LAngle) }
/// #   fn close_angle() -> Option<Kind> { Some(Kind::RAngle) }
/// # }
/// # impl From<OpenAngle<(), (), ()>> for Kind { fn from(_: OpenAngle<(), (), ()>) -> Self { Kind::LAngle } }
/// # impl From<CloseAngle<(), (), ()>> for Kind { fn from(_: CloseAngle<(), (), ()>) -> Self { Kind::RAngle } }
/// # struct CharLexer<'a> { src: &'a str, pos: usize, tok: SimpleSpan, state: () }
/// # impl<'a> Lexer<'a> for CharLexer<'a> {
/// #   type State = (); type Source = str; type Token = Tok; type Span = SimpleSpan; type Offset = usize;
/// #   fn new(src: &'a str) -> Self { Self { src, pos: 0, tok: SimpleSpan::new(0, 0), state: () } }
/// #   fn with_state(src: &'a str, _: ()) -> Self { Self::new(src) }
/// #   fn check(&self) -> Result<(), Infallible> { Ok(()) }
/// #   fn state(&self) -> &() { &self.state }
/// #   fn state_mut(&mut self) -> &mut () { &mut self.state }
/// #   fn into_state(self) -> Self::State {}
/// #   fn source(&self) -> &'a str { self.src }
/// #   fn span(&self) -> SimpleSpan { self.tok }
/// #   fn slice(&self) -> &'a str { &self.src[self.tok.start()..self.tok.end()] }
/// #   fn lex(&mut self) -> Option<Result<Tok, Infallible>> {
/// #     let bytes = self.src.as_bytes();
/// #     while self.pos < bytes.len() && bytes[self.pos] == b' ' { self.pos += 1; }
/// #     if self.pos >= bytes.len() { return None; }
/// #     let (start, c) = (self.pos, bytes[self.pos] as char);
/// #     self.pos += 1;
/// #     self.tok = SimpleSpan::new(start, self.pos);
/// #     Some(Ok(match c { '0'..='9' => Tok::Digit(c as u32 - '0' as u32), '<' => Tok::LAngle, '>' => Tok::RAngle, _ => Tok::Digit(0) }))
/// #   }
/// #   fn bump(&mut self, n: &usize) { self.pos += n; }
/// # }
/// # type Ctx<'a> = FatalContext<'a, CharLexer<'a>, Error>;
/// # fn digit<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
/// #   match inp.try_expect(|t| matches!(t.data, Tok::Digit(_)))? {
/// #     Some(sp) => match sp.data { Tok::Digit(n) => Ok(n), _ => unreachable!() },
/// #     None => Err(Error),
/// #   }
/// # }
/// use tokora::{Parse, Parser, parser::angles};
///
/// fn wrapped<'a>(inp: &mut InputRef<'a, '_, CharLexer<'a>, Ctx<'a>>) -> Result<u32, Error> {
///     angles(digit)(inp).map(|d| *d.data())
/// }
///
/// assert_eq!(Parser::with_parser(wrapped).parse_str("<1>").unwrap(), 1);
/// assert!(Parser::with_parser(wrapped).parse_str("<1").is_err());
/// ```
#[inline]
pub fn angles<'inp, L, Ctx, Lang, P, T>(
  mut inner: P,
) -> impl for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> AnglesOf<'inp, L, Ctx, Lang, T>
where
  L: Lexer<'inp>,
  L::Token: PunctuatorToken<'inp>,
  Ctx: ParseCtx<'inp, L, Lang>,
  Lang: ?Sized,
  P: for<'c> FnMut(&mut InputRef<'inp, 'c, L, Ctx, Lang>) -> Result<T, ErrorOf<'inp, L, Ctx, Lang>>,
  ErrorOf<'inp, L, Ctx, Lang>: From<UnexpectedEot<L::Offset, Lang>>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
{
  move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
    let cursor = inp.cursor().clone();
    let open = OpenAngle::parse_of(inp)?;
    let data = inner(inp)?;
    let close = CloseAngle::parse_of(inp)?;
    Ok(Delimited::new(open, close, data, inp.span_since(&cursor)))
  }
}
