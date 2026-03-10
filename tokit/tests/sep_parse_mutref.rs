#![cfg(all(feature = "std", feature = "logos"))]

//! Tests exercising the **mut-ref** (`Collect<&mut ..., &mut Container>`) code
//! path for every separator-policy x count-modifier combination in the
//! `sep/parse` directory (non-delimited).
//!
//! 8 policies x 4 count variants + 4 base (no policy) = 36 tests.

mod common;

use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait,
  emitter::{
    FromSeparatedError, FromUnexpectedLeadingSeparatorError, FromUnexpectedTrailingSeparatorError,
    FullContainerEmitter, MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    SeparatedEmitter, TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
    UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    UnexpectedEot,
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  parser::{
    AllowLeading, AllowTrailing, AtLeast, AtMost, Bounded, Collect, RequireLeading,
    RequireTrailing, Separated,
  },
  punct::Comma,
  span::Spanned,
  try_parse_input::ParseAttempt,
  utils::CowStr,
};

use common::{TestLexer, Token};

// -- Error type ---------------------------------------------------------------

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

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for E {
  fn from(_: FullContainer<S, Lang>) -> Self {
    E
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for E {
  fn from(_: TooFew<S, Lang>) -> Self {
    E
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for E {
  fn from(_: TooMany<S, Lang>) -> Self {
    E
  }
}

impl From<UnexpectedEot> for E {
  fn from(_: UnexpectedEot) -> Self {
    E
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for E {
  fn from_missing_separator(_: CowStr, _: MissingTokenOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    E
  }

  fn from_missing_element(_: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    E
  }
}

impl<'inp> FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for E {
  fn from_unexpected_leading_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    E
  }
}

impl<'inp> FromUnexpectedTrailingSeparatorError<'inp, TestLexer<'inp>> for E {
  fn from_unexpected_trailing_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    E
  }
}

// -- Full emitter -------------------------------------------------------------

struct FullEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for FullEmitter {
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

  fn emit_unexpected_token(&mut self, _: UnexpectedTokenOf<'inp, TestLexer<'inp>>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }

  fn emit_error(&mut self, err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E>
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

impl<'inp> SeparatedEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }

  fn emit_missing_element(&mut self, _: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> FullContainerEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_full_container(
    &mut self,
    _: FullContainer<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> TooFewEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_too_few(&mut self, _: TooFew<<TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> TooManyEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_too_many(&mut self, _: TooMany<<TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_unexpected_leading_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_trailing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_leading_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

fn full_ctx() -> ParserContext<'static, TestLexer<'static>, FullEmitter> {
  ParserContext::new(FullEmitter)
}

// -- Element parser -----------------------------------------------------------

fn try_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
{
  inp
    .try_expect(|t| matches!(t.data(), Token::Num(_)))
    .map(|opt| match opt {
      None => ParseAttempt::Decline,
      Some(tok) => ParseAttempt::Accept(match tok.into_data() {
        Token::Num(n) => n,
        _ => unreachable!(),
      }),
    })
}

// -- Test macro ---------------------------------------------------------------

macro_rules! sep_mr {
  ($name:ident, |$s:ident| $build:expr, $input:expr) => {
    paste::paste! {
      fn [<$name _mr>]<'inp, Ctx>(
        inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
      ) -> Result<Vec<i64>, E>
      where
        Ctx: ParseContext<'inp, TestLexer<'inp>>,
        Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
          + SeparatedEmitter<'inp, TestLexer<'inp>>
          + FullContainerEmitter<'inp, TestLexer<'inp>>
          + TooFewEmitter<'inp, TestLexer<'inp>>
          + TooManyEmitter<'inp, TestLexer<'inp>>
          + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
          + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
          + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
          + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
      {
        let mut f = try_num;
        let $s = Separated::new::<Comma>(&mut f);
        let mut inner = $build;
        let mut container = Vec::<i64>::new();
        let mut collect = Collect::new(&mut inner, &mut container);
        let _span = collect.parse_input(inp)?;
        Ok(core::mem::take(&mut container))
      }

      #[test]
      fn [<$name _mutref>]() {
        let r = Parser::with_context(full_ctx())
          .apply([<$name _mr>])
          .parse_str($input)
          .unwrap();
        assert!(!r.is_empty());
      }
    }
  };
}

// == Base (no policy) -- 4 tests ==

sep_mr!(base_unb, |sep| sep, "1,2,3");
sep_mr!(base_min, |sep| AtLeast::new(sep, 2), "1,2,3");
sep_mr!(base_max, |sep| AtMost::new(sep, 3), "1,2,3");
sep_mr!(base_bnd, |sep| Bounded::new(sep, 4, 2), "1,2,3");

// == 1. allow_leading (4 count variants) ==

sep_mr!(al_unb, |sep| AllowLeading::new(sep), ",1,2,3");
sep_mr!(
  al_min,
  |sep| AllowLeading::new(AtLeast::new(sep, 2)),
  ",1,2,3"
);
sep_mr!(
  al_max,
  |sep| AllowLeading::new(AtMost::new(sep, 3)),
  ",1,2,3"
);
sep_mr!(
  al_bnd,
  |sep| AllowLeading::new(Bounded::new(sep, 4, 2)),
  ",1,2,3"
);

// == 2. allow_trailing (4 count variants) ==

sep_mr!(at_unb, |sep| AllowTrailing::new(sep), "1,2,3,");
sep_mr!(
  at_min,
  |sep| AllowTrailing::new(AtLeast::new(sep, 2)),
  "1,2,3,"
);
sep_mr!(
  at_max,
  |sep| AllowTrailing::new(AtMost::new(sep, 3)),
  "1,2,3,"
);
sep_mr!(
  at_bnd,
  |sep| AllowTrailing::new(Bounded::new(sep, 4, 2)),
  "1,2,3,"
);

// == 3. allow_surrounded (allow_leading + allow_trailing) (4 count variants) ==

sep_mr!(
  as_unb,
  |sep| AllowLeading::new(AllowTrailing::new(sep)),
  ",1,2,3,"
);
sep_mr!(
  as_min,
  |sep| AllowLeading::new(AllowTrailing::new(AtLeast::new(sep, 2))),
  ",1,2,3,"
);
sep_mr!(
  as_max,
  |sep| AllowLeading::new(AllowTrailing::new(AtMost::new(sep, 3))),
  ",1,2,3,"
);
sep_mr!(
  as_bnd,
  |sep| AllowLeading::new(AllowTrailing::new(Bounded::new(sep, 4, 2))),
  ",1,2,3,"
);

// == 4. require_leading (4 count variants) ==

sep_mr!(rl_unb, |sep| RequireLeading::new(sep), ",1,2,3");
sep_mr!(
  rl_min,
  |sep| RequireLeading::new(AtLeast::new(sep, 2)),
  ",1,2,3"
);
sep_mr!(
  rl_max,
  |sep| RequireLeading::new(AtMost::new(sep, 3)),
  ",1,2,3"
);
sep_mr!(
  rl_bnd,
  |sep| RequireLeading::new(Bounded::new(sep, 4, 2)),
  ",1,2,3"
);

// == 5. require_trailing (4 count variants) ==

sep_mr!(rt_unb, |sep| RequireTrailing::new(sep), "1,2,3,");
sep_mr!(
  rt_min,
  |sep| RequireTrailing::new(AtLeast::new(sep, 2)),
  "1,2,3,"
);
sep_mr!(
  rt_max,
  |sep| RequireTrailing::new(AtMost::new(sep, 3)),
  "1,2,3,"
);
sep_mr!(
  rt_bnd,
  |sep| RequireTrailing::new(Bounded::new(sep, 4, 2)),
  "1,2,3,"
);

// == 6. require_surrounded (require_leading + require_trailing) (4 count variants) ==

sep_mr!(
  rs_unb,
  |sep| RequireLeading::new(RequireTrailing::new(sep)),
  ",1,2,3,"
);
sep_mr!(
  rs_min,
  |sep| RequireLeading::new(RequireTrailing::new(AtLeast::new(sep, 2))),
  ",1,2,3,"
);
sep_mr!(
  rs_max,
  |sep| RequireLeading::new(RequireTrailing::new(AtMost::new(sep, 3))),
  ",1,2,3,"
);
sep_mr!(
  rs_bnd,
  |sep| RequireLeading::new(RequireTrailing::new(Bounded::new(sep, 4, 2))),
  ",1,2,3,"
);

// == 7. allow_leading_require_trailing (4 count variants) ==

sep_mr!(
  alrt_unb,
  |sep| AllowLeading::new(RequireTrailing::new(sep)),
  ",1,2,3,"
);
sep_mr!(
  alrt_min,
  |sep| AllowLeading::new(RequireTrailing::new(AtLeast::new(sep, 2))),
  ",1,2,3,"
);
sep_mr!(
  alrt_max,
  |sep| AllowLeading::new(RequireTrailing::new(AtMost::new(sep, 3))),
  ",1,2,3,"
);
sep_mr!(
  alrt_bnd,
  |sep| AllowLeading::new(RequireTrailing::new(Bounded::new(sep, 4, 2))),
  ",1,2,3,"
);

// == 8. require_leading_allow_trailing (4 count variants) ==

sep_mr!(
  rlat_unb,
  |sep| RequireLeading::new(AllowTrailing::new(sep)),
  ",1,2,3,"
);
sep_mr!(
  rlat_min,
  |sep| RequireLeading::new(AllowTrailing::new(AtLeast::new(sep, 2))),
  ",1,2,3,"
);
sep_mr!(
  rlat_max,
  |sep| RequireLeading::new(AllowTrailing::new(AtMost::new(sep, 3))),
  ",1,2,3,"
);
sep_mr!(
  rlat_bnd,
  |sep| RequireLeading::new(AllowTrailing::new(Bounded::new(sep, 4, 2))),
  ",1,2,3,"
);
