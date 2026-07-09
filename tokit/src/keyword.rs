/// Defines a keyword.
///
/// # Examples
/// ```rust
/// use tokit::keyword;
///
/// keyword! {
///   (MyKeyword, "MY_KEYWORD", "my_keyword"),
///   (AnotherKeyword, "ANOTHER_KEYWORD", "another_keyword"),
/// }
/// ```
#[macro_export]
macro_rules! keyword {
  ($(
    $(#[$meta:meta])*
    (
      $name:ident, $syntax_tree_display: literal, $kw:literal
    )
  ),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "The `" $kw "` keyword"]
        $(#[$meta])*
        #[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq, ::core::hash::Hash)]
        pub struct $name<S = $crate::__private::span::SimpleSpan, C = (), Lang: ?::core::marker::Sized = ()> {
          span: S,
          source: C,
          _lang: ::core::marker::PhantomData<Lang>,
        }

        impl $name<()> {
          #[doc = "A unit instance of the `" $kw "` keyword."]
          pub const UNIT: Self = {
            ::core::assert!(::core::mem::size_of::<Self>() == 0);
            ::core::assert!(::core::mem::align_of::<Self>() == 1);

            Self::new(())
          };

          #[doc = "Returns a unit instance of the `" $kw "` keyword."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn unit() -> Self {
            Self::UNIT
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::convert::AsRef<::core::primitive::str> for $name<S, C, Lang> {
          #[inline]
          fn as_ref(&self) -> &str {
            $kw
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::borrow::Borrow<str> for $name<S, C, Lang> {
          #[inline]
          fn borrow(&self) -> &str {
            ::core::convert::AsRef::<str>::as_ref(self)
          }
        }

        impl<S> $name<S> {
          /// Creates a new keyword.
          #[doc = "Creates a new `" $kw "` keyword."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn new(span: S) -> Self {
            Self { span, source: (), _lang: ::core::marker::PhantomData }
          }
        }

        impl<S, C> $name<S, C> {
          #[doc = "Creates a new `" $kw "` keyword with the given content."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn with_content(span: S, content: C) -> Self {
            Self { span, source: content, _lang: ::core::marker::PhantomData }
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $name<S, C, Lang> {
          #[doc = "Changes the language type of the `" $kw "` keyword."]
          #[inline]
          pub fn change_language<N: ?::core::marker::Sized>(self) -> $name<S, C, N> {
            $name { span: self.span, source: self.source, _lang: ::core::marker::PhantomData }
          }

          #[doc = "Changes the language type of the `" $kw "` keyword."]
          #[inline]
          pub const fn change_language_const<N: ?::core::marker::Sized>(self) -> $name<S, C, N>
          where
            S: ::core::marker::Copy,
            C: ::core::marker::Copy,
          {
            $name { span: self.span, source: self.source, _lang: ::core::marker::PhantomData }
          }

          #[doc = "Returns the raw string literal of the `" $kw "` keyword."]
          #[inline]
          pub const fn raw() -> &'static ::core::primitive::str {
            $kw
          }

          #[doc = "Returns the raw string literal of the `" $kw "` keyword."]
          #[inline]
          pub const fn as_str(&self) -> &'static ::core::primitive::str {
            Self::raw()
          }

          #[doc = "Returns the span of the `" $kw "` keyword."]
          #[inline]
          pub const fn span(&self) -> &S {
            &self.span
          }

          #[doc = "Returns a reference to the content of the `" $kw "` keyword."]
          #[inline]
          pub const fn content(&self) -> &C {
            &self.source
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::cmp::PartialEq<::core::primitive::str> for $name<S, C, Lang> {
          #[inline]
          fn eq(&self, other: &::core::primitive::str) -> bool {
            self.as_str().eq(other)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::cmp::PartialOrd<::core::primitive::str> for $name<S, C, Lang> {
          #[inline]
          fn partial_cmp(&self, other: &::core::primitive::str) -> ::core::option::Option<::core::cmp::Ordering> {
            self.as_str().partial_cmp(other)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::cmp::PartialEq<$name<S, C, Lang>> for ::core::primitive::str {
          #[inline]
          fn eq(&self, other: &$name<S, C, Lang>) -> bool {
            self.eq(other.as_str())
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::cmp::PartialOrd<$name<S, C, Lang>> for ::core::primitive::str {
          #[inline]
          fn partial_cmp(&self, other: &$name<S, C, Lang>) -> ::core::option::Option<::core::cmp::Ordering> {
            self.partial_cmp(other.as_str())
          }
        }

        impl<'inp, T, S, C, Lang> $crate::__private::Check<T, ::core::primitive::bool> for $name<S, C, Lang>
        where
          T: $crate::__private::token::KeywordToken<'inp> + ?::core::marker::Sized + 'inp,
          Lang: ?::core::marker::Sized,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn check(&self, target: &T) -> ::core::primitive::bool {
            ::core::cmp::PartialEq::eq(
              &$crate::__private::token::KeywordToken::keyword(target),
              &::core::option::Option::Some($kw),
            )
          }
        }

        impl $name {
          #[doc = "A parser that parses a token and returns a `" $name "` instance if it matches the `" $kw "` keyword."]
          ///
          /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not the
          /// `" $kw "` keyword, and promises no valid token is consumed.
          pub fn try_parse<'inp, L, Ctx>(
            inp: &mut $crate::InputRef<'inp, '_, L, Ctx>,
          ) -> ::core::result::Result<
            $crate::try_parse_input::ParseAttempt<$name<L::Span, ()>>,
            <Ctx::Emitter as $crate::Emitter<'inp, L>>::Error,
          >
          where
            L: $crate::Lexer<'inp>,
            L::Token: $crate::__private::token::KeywordToken<'inp>,
            Ctx: $crate::ParseContext<'inp, L>,
            <Ctx::Emitter as $crate::Emitter<'inp, L>>::Error:
              ::core::convert::From<$crate::error::UnexpectedEot<L::Offset>>,
          {
            Self::try_parse_of(inp)
          }

          #[doc = "A parser that parses a token and returns a `" $name "` instance if it matches the `" $kw "` keyword for a specific language."]
          ///
          /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not the
          /// `" $kw "` keyword, and promises no valid token is consumed.
          pub fn try_parse_of<'inp, L, Ctx, Lang: ?::core::marker::Sized>(
            inp: &mut $crate::InputRef<'inp, '_, L, Ctx, Lang>,
          ) -> ::core::result::Result<
            $crate::try_parse_input::ParseAttempt<$name<L::Span, (), Lang>>,
            <Ctx::Emitter as $crate::Emitter<'inp, L, Lang>>::Error,
          >
          where
            L: $crate::Lexer<'inp>,
            L::Token: $crate::__private::token::KeywordToken<'inp>,
            Ctx: $crate::ParseContext<'inp, L, Lang>,
            <Ctx::Emitter as $crate::Emitter<'inp, L, Lang>>::Error:
              ::core::convert::From<$crate::error::UnexpectedEot<L::Offset, Lang>>,
          {
            inp
              .try_expect(|t| ::core::cmp::PartialEq::eq(
                &$crate::__private::token::KeywordToken::keyword(t.into_data()),
                &::core::option::Option::Some($kw),
              ))
              .map(|res| res.map(|tok| $name::new(tok.into_span()).change_language()).into())
          }

          #[doc = "A parser that parses the `" $kw "` keyword, erroring when the next token is not that keyword."]
          ///
          /// Unlike [`try_parse`](Self::try_parse), an unexpected token is converted into an
          /// `UnexpectedToken` error carrying the found token, and end of input into an
          /// `UnexpectedEot` error.
          ///
          /// The error carries the found token; it does not carry an expected-token entry — the expected keyword is statically known at the call site.
          pub fn parse<'inp, L, Ctx>(
            inp: &mut $crate::InputRef<'inp, '_, L, Ctx>,
          ) -> ::core::result::Result<$name<L::Span, ()>, <Ctx::Emitter as $crate::Emitter<'inp, L>>::Error>
          where
            L: $crate::Lexer<'inp>,
            L::Token: $crate::__private::token::KeywordToken<'inp>,
            Ctx: $crate::ParseContext<'inp, L>,
            <Ctx::Emitter as $crate::Emitter<'inp, L>>::Error:
              ::core::convert::From<$crate::error::UnexpectedEot<L::Offset>>
              + ::core::convert::From<$crate::error::token::UnexpectedToken<'inp, L::Token, <L::Token as $crate::Token<'inp>>::Kind, L::Span>>,
          {
            Self::parse_of(inp)
          }

          #[doc = "A parser that parses the `" $kw "` keyword for a specific language, erroring when the next token is not that keyword."]
          ///
          /// Unlike [`try_parse_of`](Self::try_parse_of), an unexpected token is converted into an
          /// `UnexpectedToken` error carrying the found token, and end of input into an
          /// `UnexpectedEot` error.
          ///
          /// The error carries the found token; it does not carry an expected-token entry — the expected keyword is statically known at the call site.
          pub fn parse_of<'inp, L, Ctx, Lang: ?::core::marker::Sized>(
            inp: &mut $crate::InputRef<'inp, '_, L, Ctx, Lang>,
          ) -> ::core::result::Result<$name<L::Span, (), Lang>, <Ctx::Emitter as $crate::Emitter<'inp, L, Lang>>::Error>
          where
            L: $crate::Lexer<'inp>,
            L::Token: $crate::__private::token::KeywordToken<'inp>,
            Ctx: $crate::ParseContext<'inp, L, Lang>,
            <Ctx::Emitter as $crate::Emitter<'inp, L, Lang>>::Error:
              ::core::convert::From<$crate::error::UnexpectedEot<L::Offset, Lang>>
              + ::core::convert::From<$crate::error::token::UnexpectedToken<'inp, L::Token, <L::Token as $crate::Token<'inp>>::Kind, L::Span, Lang>>,
          {
            match inp.next()? {
              ::core::option::Option::Some(spanned) => {
                if ::core::cmp::PartialEq::eq(
                  &$crate::__private::token::KeywordToken::keyword(spanned.data()),
                  &::core::option::Option::Some($kw),
                ) {
                  ::core::result::Result::Ok($name::new(spanned.into_span()).change_language())
                } else {
                  let (span, tok) = spanned.into_components();
                  ::core::result::Result::Err(
                    $crate::error::token::UnexpectedToken::of(span).with_found(tok).into(),
                  )
                }
              }
              ::core::option::Option::None => ::core::result::Result::Err(
                $crate::error::UnexpectedEot::eot_of(
                  $crate::__private::span::Span::end(inp.span()),
                ).into(),
              ),
            }
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::span::AsSpan<S> for $name<S, C, Lang> {
          #[inline]
          fn as_span(&self) -> &S {
            self.span()
          }
        }

       impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::span::IntoSpan<S> for $name<S, C, Lang> {
          #[inline]
          fn into_span(self) -> S {
            self.span
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::IntoComponents for $name<S, C, Lang> {
          type Components = (S, C);

          #[inline]
          fn into_components(self) -> Self::Components {
            (self.span, self.source)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::fmt::Display for $name<S, C, Lang> {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            ::core::fmt::Write::write_str(f, $kw)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::human_display::DisplayHuman for $name<S, C, Lang> {
          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::sdl_display::DisplayCompact for $name<S, C, Lang> {
          type Options = ();

          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>, _: &Self::Options) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::sdl_display::DisplayPretty for $name<S, C, Lang> {
          type Options = ();

          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>, _: &Self::Options) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }
      )*
    }
  };
}

#[cfg(all(test, feature = "std", feature = "logos"))]
mod tests {
  use crate::{
    Emitter, InputRef, Lexer, Parse, Parser, ParserContext, SimpleSpan, Token as TokenTrait,
    error::{
      UnexpectedEot,
      token::{UnexpectedToken, UnexpectedTokenOf},
    },
    input::Cursor,
    lexer::LogosLexer,
    logos::{self, Logos},
    span::Spanned,
    token::KeywordToken,
  };

  // A test-local invocation of the `keyword!` macro under test.
  keyword! {
    (If, "IF_KW", "if"),
    (Else, "ELSE_KW", "else"),
  }

  #[derive(Debug, Clone, Logos, PartialEq)]
  #[logos(crate = logos, skip r"[ \t\r\n]+")]
  enum Token {
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum TokenKind {
    If,
    Else,
    Ident,
  }

  impl core::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TokenKind::If => write!(f, "if"),
        TokenKind::Else => write!(f, "else"),
        TokenKind::Ident => write!(f, "identifier"),
      }
    }
  }

  impl TokenTrait<'_> for Token {
    type Kind = TokenKind;
    type Error = ();

    fn kind(&self) -> TokenKind {
      match self {
        Token::If => TokenKind::If,
        Token::Else => TokenKind::Else,
        Token::Ident => TokenKind::Ident,
      }
    }

    fn is_trivia(&self) -> bool {
      false
    }
  }

  impl KeywordToken<'_> for Token {
    fn keyword(&self) -> Option<&'static str> {
      match self {
        Token::If => Some("if"),
        Token::Else => Some("else"),
        Token::Ident => None,
      }
    }
  }

  type TestLexer<'a> = LogosLexer<'a, Token>;

  #[derive(Debug, PartialEq)]
  enum E {
    Lex,
    Eot,
    Unexpected { found: Option<TokenKind> },
  }

  impl From<()> for E {
    fn from(_: ()) -> Self {
      E::Lex
    }
  }

  impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for E {
    fn from(_: UnexpectedEot<O, Lang>) -> Self {
      E::Eot
    }
  }

  impl<'a, S, Lang: ?Sized> From<UnexpectedToken<'a, Token, TokenKind, S, Lang>> for E {
    fn from(err: UnexpectedToken<'a, Token, TokenKind, S, Lang>) -> Self {
      let (_span, found, _expected) = err.into_components();
      E::Unexpected {
        found: found.map(|t| t.kind()),
      }
    }
  }

  struct TestEm;

  impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEm {
    type Error = E;

    fn emit_lexer_error(
      &mut self,
      _: Spanned<
        <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
        <TestLexer<'inp> as Lexer<'inp>>::Span,
      >,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(E::Lex)
    }

    fn emit_unexpected_token(
      &mut self,
      _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(E::Unexpected { found: None })
    }

    fn emit_error(
      &mut self,
      err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(err.into_data())
    }

    fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>)
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
    }
  }

  fn ctx() -> ParserContext<'static, TestLexer<'static>, TestEm> {
    ParserContext::new(TestEm)
  }

  // ── typed parsers: try_parse ────────────────────────────────────────────

  #[test]
  fn if_try_parse_accepts_if_token() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<bool, E> {
      Ok(If::try_parse(inp)?.is_accept())
    }
    let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("if");
    assert!(r.unwrap());
  }

  #[test]
  fn if_try_parse_declines_else_without_consuming() {
    // Declining on `else` must not consume it: a following `Else::try_parse`
    // still accepts the same token.
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<(bool, bool), E> {
      let declined = If::try_parse(inp)?.is_decline();
      let else_still_there = Else::try_parse(inp)?.is_accept();
      Ok((declined, else_still_there))
    }
    let r: Result<(bool, bool), _> = Parser::with_context(ctx()).apply(parse).parse_str("else");
    assert_eq!(r.unwrap(), (true, true));
  }

  #[test]
  fn if_try_parse_declines_non_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<bool, E> {
      Ok(If::try_parse(inp)?.is_decline())
    }
    let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("foo");
    assert!(r.unwrap());
  }

  // ── typed parsers: parse (committed) ────────────────────────────────────

  #[test]
  fn if_parse_accepts_if_token() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<If<SimpleSpan>, E> {
      If::parse(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("if");
    assert_eq!(*r.unwrap().span(), SimpleSpan::new(0, 2));
  }

  #[test]
  fn if_parse_errors_on_else_carrying_found_token() {
    // The committed parser falls back to `UnexpectedToken`, which carries the
    // found token (`else`); the emitter error records it.
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<If<SimpleSpan>, E> {
      If::parse(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("else");
    assert_eq!(
      r.unwrap_err(),
      E::Unexpected {
        found: Some(TokenKind::Else)
      }
    );
  }

  #[test]
  fn if_parse_errors_on_empty_input() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<If<SimpleSpan>, E> {
      If::parse(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("");
    assert_eq!(r.unwrap_err(), E::Eot);
  }

  // ── Lang-defaulted type name compiles bare ──────────────────────────────

  #[test]
  fn lang_defaulted_type_name_compiles() {
    let kw: If<SimpleSpan> = If::new(SimpleSpan::new(0, 2));
    assert_eq!(kw.as_str(), "if");
  }

  // ── as_str / raw consistency + PartialEq<str> both directions ───────────

  #[test]
  fn as_str_and_raw_are_consistent() {
    let kw = If::new(SimpleSpan::new(0, 2));
    assert_eq!(kw.as_str(), "if");
    assert_eq!(If::<SimpleSpan>::raw(), "if");
    assert_eq!(kw.as_str(), If::<SimpleSpan>::raw());
  }

  #[test]
  fn partial_eq_str_round_trip() {
    let kw = If::new(SimpleSpan::new(0, 2));
    assert!(kw == *"if");
    assert!(*"if" == kw);
    assert!(kw != *"else");
    assert!(*"else" != kw);
  }

  // ── UNIT / unit / change_language ───────────────────────────────────────

  #[test]
  fn unit_is_zero_sized_and_matches_literal() {
    assert_eq!(core::mem::size_of::<If<()>>(), 0);
    let kw = If::<()>::unit();
    assert_eq!(kw.as_str(), "if");
    assert_eq!(If::<()>::UNIT.as_str(), "if");
  }

  #[test]
  fn change_language_preserves_literal() {
    struct LangA;
    struct LangB;
    let kw: If<SimpleSpan, (), LangA> = If::new(SimpleSpan::new(0, 2)).change_language();
    let kw2: If<SimpleSpan, (), LangB> = kw.change_language();
    assert_eq!(kw2.as_str(), "if");
  }

  // ── Check impl against KeywordToken ─────────────────────────────────────

  #[test]
  fn check_matches_only_its_own_keyword() {
    use crate::Check;
    let if_kw = If::new(SimpleSpan::new(0, 2));
    assert!(if_kw.check(&Token::If));
    assert!(!if_kw.check(&Token::Else));
    assert!(!if_kw.check(&Token::Ident));
  }
}
