use super::*;

use crate::{TryParseInput, error::UnexpectedEot, lexer::PunctuatorToken, punct::*};

macro_rules! define_parsers {
  ($($name:ident::$fn:ident),+$(,)?) => {
    paste::paste! {
      $(
        impl $name {
          #[doc = "A parser that parses a token and returns `" $name "` instance if matches."]
          ///
          /// If the function returns `Ok(None)`, it means the next token does not match,
          /// and promises no valid token is consumed.
          pub fn try_parse<'inp, L, Ctx>(
            inp: &mut InputRef<'inp, '_, L, Ctx>,
          ) -> Result<Option<$name<L::Span, ()>>, <Ctx::Emitter as Emitter<'inp, L>>::Error>
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
          ) -> Result<Option<$name<L::Span, (), Lang>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
          where
            L: Lexer<'inp>,
            L::Token: PunctuatorToken<'inp>,
            Ctx: ParseContext<'inp, L, Lang>,
            <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
          {
            <$name>::new(()).try_parse_input(inp)
          }
        }

        impl<'inp, L, Ctx, Lang> TryParseInput<'inp, L, $name<L::Span, (), Lang>, Ctx, Lang>
          for $name
        where
          L: Lexer<'inp>,
          L::Token: PunctuatorToken<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
          <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
          Lang: ?Sized,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn try_parse_input(
            &mut self,
            inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
          ) -> Result<
            Option<$name<L::Span, (), Lang>>,
            <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
          > {
            let end = inp.cursor().as_inner().clone();
            let tok = inp.sync_until_token()?;

            match tok {
              None => Err(UnexpectedEot::eot_of(end).into()),
              Some(ct) => {
                let (span, matches) = ct
                  .map(
                    |t| {
                      let (span, tok) = t.into_token().into_components();
                      (span.clone(), tok.$fn())
                    },
                    |t| {
                      let (span, tok) = t.into_token().into_components();
                      (span, tok.$fn())
                    },
                  )
                  .into_inner();
                if matches {
                  inp.skip_one();
                  Ok(Some($name::new(span).change_language()))
                } else {
                  Ok(None)
                }
              }
            }
          }
        }
      )*
    }
  };
}

define_parsers!(
  Dot::is_dot,
  Comma::is_comma,
  Colon::is_colon,
  Semicolon::is_semicolon,
  Exclamation::is_exclamation,
  DoubleQuote::is_double_quote,
  Apostrophe::is_apostrophe,
  Hash::is_hash,
  Dollar::is_dollar,
  Percent::is_percent,
  Ampersand::is_ampersand,
  Asterisk::is_asterisk,
  Plus::is_plus,
  Hyphen::is_minus,
  Slash::is_slash,
  BackSlash::is_backslash,
  OpenAngle::is_open_angle,
  Equal::is_equal,
  CloseAngle::is_close_angle,
  Question::is_question,
  At::is_at,
  OpenBracket::is_open_bracket,
  CloseBracket::is_close_bracket,
  OpenBrace::is_open_brace,
  CloseBrace::is_close_brace,
  OpenParen::is_open_paren,
  CloseParen::is_close_paren,
  Backtick::is_backtick,
  Pipe::is_pipe,
  Caret::is_caret,
  Underscore::is_underscore,
  Tilde::is_tilde,
  Space::is_space,
  Tab::is_tab,
  Newline::is_newline,
  CarriageReturn::is_carriage_return,
  CarriageReturnNewline::is_crlf,
);
