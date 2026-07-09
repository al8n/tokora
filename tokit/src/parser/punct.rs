use super::*;

use crate::{
  error::UnexpectedEot, punct::*, token::PunctuatorToken, try_parse_input::ParseAttempt,
  utils::CowStr,
};

macro_rules! define_parsers {
  ($($name:ident::$kind:ident::$punct_char:literal),+$(,)?) => {
    paste::paste! {
      $(
        impl $name {
          #[doc = "A parser that parses a token and returns a `" $name "` instance if it matches."]
          ///
          /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token does not match,
          /// and promises no valid token is consumed.
          pub fn try_parse<'inp, L, Ctx>(
            inp: &mut InputRef<'inp, '_, L, Ctx>,
          ) -> Result<ParseAttempt<$name<L::Span, ()>>, <Ctx::Emitter as Emitter<'inp, L>>::Error>
          where
            L: Lexer<'inp>,
            L::Token: PunctuatorToken<'inp>,
            Ctx: ParseContext<'inp, L>,
            <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
          {
            Self::try_parse_of(inp)
          }

          #[doc = "A parser that parses a token and returns a `" $name " ` instance if it matches for a specific language."]
          ///
          /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token does not match,
          /// and promises no valid token is consumed.
          pub fn try_parse_of<'inp, L, Ctx, Lang: ?Sized>(
            inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
          ) -> Result<ParseAttempt<$name<L::Span, (), Lang>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
          where
            L: Lexer<'inp>,
            L::Token: PunctuatorToken<'inp>,
            Ctx: ParseContext<'inp, L, Lang>,
            <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
          {
            inp.[< try_expect_ $kind >]().map(|res| res.map(|tok| $name::new(tok.into_span()).change_language()).into())
          }

          #[doc = "A parser that parses a token and returns a `" $name "` instance if it matches."]
          pub fn parse<'inp, L, Ctx>(
            inp: &mut InputRef<'inp, '_, L, Ctx>,
          ) -> Result<$name<L::Span, ()>, <Ctx::Emitter as Emitter<'inp, L>>::Error>
          where
            L: Lexer<'inp>,
            L::Token: PunctuatorToken<'inp>,
            Ctx: ParseContext<'inp, L>,
            <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>
            + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>,
          {
            Self::parse_of(inp)
          }

          #[doc = "A parser that parses a token and returns a `" $name " ` instance if it matches for a specific language."]
          pub fn parse_of<'inp, L, Ctx, Lang>(
            inp: &mut InputRef<'inp, '_, L, Ctx, Lang>
          ) -> Result<$name<L::Span, (), Lang>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
          where
            L: Lexer<'inp>,
            L::Token: PunctuatorToken<'inp>,
            Ctx: ParseContext<'inp, L, Lang>,
            <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>> +
            From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
            Lang: ?Sized,
          {
            inp.[<expect_ $kind>]().map(|tok| $name::new(tok.into_span()).change_language())
          }
        }

        impl<'inp, L, S, C, Lang: ?Sized> Punctuator<'inp, L, Lang> for $name<S, C, Lang>
        where
          L: Lexer<'inp>,
          <L::Token as Token<'inp>>::Kind: From<$name<(), (), ()>>,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn name() -> CowStr {
            CowStr::from_static(stringify!($knd))
          }

          #[cfg_attr(not(tarpaulin), inline(always))]
          fn description() -> Option<CowStr> {
            Some(CowStr::from_static(concat!("The `", $punct_char, "` punctuator.")))
          }

          #[inline]
          fn kind() -> <L::Token as Token<'inp>>::Kind {
            <<L::Token as Token<'inp>>::Kind as From<_>>::from(<$name>::unit())
          }
        }
      )*
    }
  };
}

// define_punctuator_token_traits!(
//   open_angle: "<",
//   close_angle: ">",
//   open_brace: "{",
//   close_brace: "}",
//   open_paren: "(",
//   close_paren: ")",
//   open_bracket: "[",
//   close_bracket: "]",
//   comma: ",",
//   semicolon: ";",
//   colon: ":",
//   dot: ".",
//   tilde: "~",
//   underscore: "_",
//   equal: "=",
//   minus: "-",
//   #[doc(alias = "thin_arrow")]
//   arrow: "->",
//   fat_arrow: "=>",
//   pipe_arrow: "|>",
//   double_colon: "::",
//   tab: "\t",
//   newline: "\n",
//   carriage_return: "\r",
//   crlf: "\r\n",
//   space: " ",
//   pipe: "|",
//   ampersand: "&",
//   percent: "%",
//   slash: "/",
//   backslash: "\\",
//   dollar: "$",
//   hash: "#",
//   at: "@",
//   asterisk: "*",
//   apostrophe: "'",
//   double_quote: "\"",
//   plus: "+",
//   exclamation: "!",
//   question: "?",
//   backtick: "`",
//   caret: "^",
// );

define_parsers!(
  OpenAngle::open_angle::"<",
  CloseAngle::close_angle::">",
  OpenBrace::open_brace::"{",
  CloseBrace::close_brace::"}",
  OpenParen::open_paren::"(",
  CloseParen::close_paren::")",
  OpenBracket::open_bracket::"[",
  CloseBracket::close_bracket::"]",
  Comma::comma::",",
  Semicolon::semicolon::";",
  Colon::colon::":",
  Dot::dot::".",
  Tilde::tilde::"~",
  Underscore::underscore::"_",
  Equal::equal::"=",
  Hyphen::hyphen::"-",
  Arrow::arrow::"->",
  FatArrow::fat_arrow::"=>",
  PipeArrow::pipe_arrow::"|>",
  DoubleColon::double_colon::"::",
  Spread::spread::"...",
  Space::space::" ",
  Tab::tab::"\t",
  Newline::newline::"\n",
  CarriageReturn::carriage_return::"\r",
  CarriageReturnNewline::crlf::"\r\n",
  Pipe::pipe::"|",
  Ampersand::ampersand::"&",
  Percent::percent::"%",
  Slash::slash::"/",
  Backslash::backslash::"\\",
  Dollar::dollar::"$",
  Hash::hash::"#",
  At::at::"@",
  Asterisk::asterisk::"*",
  Apostrophe::apostrophe::"'",
  DoubleQuote::double_quote::"\"",
  Plus::plus::"+",
  Exclamation::exclamation::"!",
  Question::question::"?",
  Backtick::backtick::"`",
  Caret::caret::"^",
);

#[cfg(all(test, feature = "std", feature = "logos"))]
mod tests {
  use super::*;

  use crate::{
    ParserContext,
    error::token::UnexpectedTokenOf,
    input::Cursor,
    lexer::LogosLexer,
    logos::{self, Logos},
    span::Spanned,
    token::Token as TokenTrait,
  };

  #[derive(Debug, Clone, Logos, PartialEq)]
  #[logos(crate = logos, skip r"[ \t\r\n]+")]
  enum Token {
    #[token("...")]
    Spread,
    #[regex(r"[0-9]+")]
    Num,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum TokenKind {
    Spread,
    Num,
  }

  impl core::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TokenKind::Spread => write!(f, "..."),
        TokenKind::Num => write!(f, "number"),
      }
    }
  }

  impl TokenTrait<'_> for Token {
    type Kind = TokenKind;
    type Error = ();

    fn kind(&self) -> TokenKind {
      match self {
        Token::Spread => TokenKind::Spread,
        Token::Num => TokenKind::Num,
      }
    }

    fn is_trivia(&self) -> bool {
      false
    }
  }

  impl PunctuatorToken<'_> for Token {
    fn spread() -> Option<Self::Kind> {
      Some(TokenKind::Spread)
    }
  }

  type TestLexer<'a> = LogosLexer<'a, Token>;

  #[derive(Debug)]
  struct E;

  impl From<()> for E {
    fn from(_: ()) -> Self {
      E
    }
  }

  impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
    fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
      E
    }
  }

  impl From<UnexpectedEot> for E {
    fn from(_: UnexpectedEot) -> Self {
      E
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
      Err(E)
    }

    fn emit_unexpected_token(
      &mut self,
      _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(E)
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

  #[test]
  fn spread_try_parse_accepts_spread_token() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<bool, E> {
      Ok(Spread::try_parse(inp)?.is_accept())
    }
    let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("...");
    assert!(r.unwrap());
  }

  #[test]
  fn spread_try_parse_declines_non_spread_token() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<(bool, bool), E> {
      let declined = Spread::try_parse(inp)?.is_decline();
      let next_is_num = inp
        .try_expect(|t| t.data.kind() == TokenKind::Num)?
        .is_some();
      Ok((declined, next_is_num))
    }
    let r: Result<(bool, bool), _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
    let (declined, next_is_num) = r.unwrap();
    assert!(declined);
    assert!(next_is_num);
  }
}
