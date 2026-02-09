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
