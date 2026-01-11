use super::*;

use crate::{
  error::UnexpectedEot, punct::*, token::PunctuatorToken, try_parse_input::ParseAttempt,
};

macro_rules! define_parsers {
  ($($name:ident::$kind:ident::$punct_char:literal),+$(,)?) => {
    paste::paste! {
      $(
        impl $name {
          #[doc = "A parser that parses a token and returns `" $name "` instance if matches."]
          ///
          /// If the function returns `Ok(None)`, it means the next token does not match,
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

          #[doc = "A parser that parses a token and returns `" $name " ` instance if matches for a specific language."]
          ///
          /// If the function returns `Ok(None)`, it means the next token does not match,
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

          #[doc = "A parser that parses a token and returns `" $name "` instance if matches."]
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

          #[doc = "A parser that parses a token and returns `" $name " ` instance if matches for a specific language."]
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
          fn name() -> &'static str {
            stringify!($knd)
          }

          #[cfg_attr(not(tarpaulin), inline(always))]
          fn description() -> Option<&'static str> {
            Some(concat!("The `", $punct_char, "` punctuator."))
          }

          #[inline]
          fn kind() -> <L::Token as Token<'inp>>::Kind {
            <<L::Token as Token<'inp>>::Kind as From<_>>::from(<$name>::PHANTOM)
          }
        }
      )*
    }
  };
}

define_parsers!(
  Dot::dot::".",
  Comma::comma::",",
  Colon::colon::":",
  Semicolon::semicolon::";",
  Exclamation::exclamation::"!",
  DoubleQuote::double_quote::"\"",
  Apostrophe::apostrophe::"'",
  Hash::hash::"#",
  Dollar::dollar::"$",
  Percent::percent::"%",
  Ampersand::ampersand::"&",
  Asterisk::asterisk::"*",
  Plus::plus::"+",
  Hyphen::minus::"-",
  Slash::slash::"/",
  Backslash::backslash::"\\",
  OpenAngle::open_angle::"<",
  Equal::equal::"=",
  CloseAngle::close_angle::">",
  Question::question::"?",
  At::at::"@",
  OpenBracket::open_bracket::"[",
  CloseBracket::close_bracket::"]",
  OpenBrace::open_brace::"{",
  CloseBrace::close_brace::"}",
  OpenParen::open_paren::"(",
  CloseParen::close_paren::")",
  Backtick::backtick::"`",
  Pipe::pipe::"|",
  Caret::caret::"^",
  Underscore::underscore::"_",
  Tilde::tilde::"~",
  Space::space::" ",
  Tab::tab::"\t",
  Newline::newline::"\n",
  CarriageReturn::carriage_return::"\r",
  CarriageReturnNewline::crlf::"\r\n",
);
