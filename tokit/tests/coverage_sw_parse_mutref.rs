#![cfg(all(feature = "std", feature = "logos"))]

//! Tests exercising the **mut-ref** (`Collect<&mut ..., &mut Container>`) code
//! path for every separator-policy x count-modifier combination in the
//! `sep_while/parse` directory (non-delimited).
//!
//! 8 policies x 4 count variants + 4 base (no policy) = 36 tests.

mod common;

use generic_arraydeque::typenum::U1;
use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait,
  cache::Peeked,
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
    Action, AllowLeading, AllowTrailing, AtLeast, AtMost, Bounded, Collect, RequireLeading,
    RequireTrailing, SeparatedWhile,
  },
  punct::Comma,
  span::Spanned,
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

// -- Condition + element parser -----------------------------------------------

fn decide_num<'inp, Ctx>(
  mut peeked: Peeked<'_, 'inp, TestLexer<'inp>, U1>,
  _: &mut Ctx::Emitter,
) -> Result<Action, <Ctx::Emitter as Emitter<'inp, TestLexer<'inp>>>::Error>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
{
  Ok(match peeked.pop_front() {
    None => Action::Stop,
    Some(tok) => {
      let tok = tok
        .as_maybe_ref()
        .map(|t| t.token().copied(), |t| t.token())
        .into_inner();
      if matches!(**tok.data(), Token::Num(_)) {
        Action::Continue
      } else {
        Action::Stop
      }
    }
  })
}

fn parse_num<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
{
  match inp.next()? {
    None => Err(E),
    Some(tok) => match tok.into_data() {
      Token::Num(n) => Ok(n),
      _ => Err(E),
    },
  }
}

// -- Test macro ---------------------------------------------------------------

macro_rules! sw_mr {
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
        let mut f = parse_num;
        let mut cond = decide_num::<Ctx>;
        let $s: SeparatedWhile<_, Comma, _, i64, U1, _, Ctx, _> =
          SeparatedWhile::new::<Comma>(&mut f, &mut cond);
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

sw_mr!(base_unb, |sw| sw, "1,2,3");
sw_mr!(base_min, |sw| AtLeast::new(sw, 2), "1,2,3");
sw_mr!(base_max, |sw| AtMost::new(sw, 3), "1,2,3");
sw_mr!(base_bnd, |sw| Bounded::new(sw, 4, 2), "1,2,3");

// == 1. allow_leading (4 count variants) ==

sw_mr!(al_unb, |sw| AllowLeading::new(sw), ",1,2,3");
sw_mr!(
  al_min,
  |sw| AllowLeading::new(AtLeast::new(sw, 2)),
  ",1,2,3"
);
sw_mr!(al_max, |sw| AllowLeading::new(AtMost::new(sw, 3)), ",1,2,3");
sw_mr!(
  al_bnd,
  |sw| AllowLeading::new(Bounded::new(sw, 4, 2)),
  ",1,2,3"
);

// == 2. allow_trailing (4 count variants) ==

sw_mr!(at_unb, |sw| AllowTrailing::new(sw), "1,2,3,");
sw_mr!(
  at_min,
  |sw| AllowTrailing::new(AtLeast::new(sw, 2)),
  "1,2,3,"
);
sw_mr!(
  at_max,
  |sw| AllowTrailing::new(AtMost::new(sw, 3)),
  "1,2,3,"
);
sw_mr!(
  at_bnd,
  |sw| AllowTrailing::new(Bounded::new(sw, 4, 2)),
  "1,2,3,"
);

// == 3. allow_surrounded (allow_leading + allow_trailing) (4 count variants) ==

sw_mr!(
  as_unb,
  |sw| AllowLeading::new(AllowTrailing::new(sw)),
  ",1,2,3,"
);
sw_mr!(
  as_min,
  |sw| AllowLeading::new(AllowTrailing::new(AtLeast::new(sw, 2))),
  ",1,2,3,"
);
sw_mr!(
  as_max,
  |sw| AllowLeading::new(AllowTrailing::new(AtMost::new(sw, 3))),
  ",1,2,3,"
);
sw_mr!(
  as_bnd,
  |sw| AllowLeading::new(AllowTrailing::new(Bounded::new(sw, 4, 2))),
  ",1,2,3,"
);

// == 4. require_leading (4 count variants) ==

sw_mr!(rl_unb, |sw| RequireLeading::new(sw), ",1,2,3");
sw_mr!(
  rl_min,
  |sw| RequireLeading::new(AtLeast::new(sw, 2)),
  ",1,2,3"
);
sw_mr!(
  rl_max,
  |sw| RequireLeading::new(AtMost::new(sw, 3)),
  ",1,2,3"
);
sw_mr!(
  rl_bnd,
  |sw| RequireLeading::new(Bounded::new(sw, 4, 2)),
  ",1,2,3"
);

// == 5. require_trailing (4 count variants) ==

sw_mr!(rt_unb, |sw| RequireTrailing::new(sw), "1,2,3,");
sw_mr!(
  rt_min,
  |sw| RequireTrailing::new(AtLeast::new(sw, 2)),
  "1,2,3,"
);
sw_mr!(
  rt_max,
  |sw| RequireTrailing::new(AtMost::new(sw, 3)),
  "1,2,3,"
);
sw_mr!(
  rt_bnd,
  |sw| RequireTrailing::new(Bounded::new(sw, 4, 2)),
  "1,2,3,"
);

// == 6. require_surrounded (require_leading + require_trailing) (4 count variants) ==

sw_mr!(
  rs_unb,
  |sw| RequireLeading::new(RequireTrailing::new(sw)),
  ",1,2,3,"
);
sw_mr!(
  rs_min,
  |sw| RequireLeading::new(RequireTrailing::new(AtLeast::new(sw, 2))),
  ",1,2,3,"
);
sw_mr!(
  rs_max,
  |sw| RequireLeading::new(RequireTrailing::new(AtMost::new(sw, 3))),
  ",1,2,3,"
);
sw_mr!(
  rs_bnd,
  |sw| RequireLeading::new(RequireTrailing::new(Bounded::new(sw, 4, 2))),
  ",1,2,3,"
);

// == 7. allow_leading_require_trailing (4 count variants) ==

sw_mr!(
  alrt_unb,
  |sw| AllowLeading::new(RequireTrailing::new(sw)),
  ",1,2,3,"
);
sw_mr!(
  alrt_min,
  |sw| AllowLeading::new(RequireTrailing::new(AtLeast::new(sw, 2))),
  ",1,2,3,"
);
sw_mr!(
  alrt_max,
  |sw| AllowLeading::new(RequireTrailing::new(AtMost::new(sw, 3))),
  ",1,2,3,"
);
sw_mr!(
  alrt_bnd,
  |sw| AllowLeading::new(RequireTrailing::new(Bounded::new(sw, 4, 2))),
  ",1,2,3,"
);

// == 8. require_leading_allow_trailing (4 count variants) ==

sw_mr!(
  rlat_unb,
  |sw| RequireLeading::new(AllowTrailing::new(sw)),
  ",1,2,3,"
);
sw_mr!(
  rlat_min,
  |sw| RequireLeading::new(AllowTrailing::new(AtLeast::new(sw, 2))),
  ",1,2,3,"
);
sw_mr!(
  rlat_max,
  |sw| RequireLeading::new(AllowTrailing::new(AtMost::new(sw, 3))),
  ",1,2,3,"
);
sw_mr!(
  rlat_bnd,
  |sw| RequireLeading::new(AllowTrailing::new(Bounded::new(sw, 4, 2))),
  ",1,2,3,"
);
