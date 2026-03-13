#![cfg(all(feature = "std", feature = "logos"))]

//! Coverage tests exercising the **mut-ref** (`Collect<&mut DelimitedBy<...>, &mut Container>`)
//! path (Impl #3) for `SeparatedWhile` delimited combinations across all
//! 8 separator policies × 4 count variants = 32 tests.

mod common;

use generic_arraydeque::typenum::U1;
use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
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
    Action, AllowLeading, AllowTrailing, AtLeast, AtMost, Bounded, Collect, DelimitedBy,
    RequireLeading, RequireTrailing, SeparatedWhile,
  },
  punct::Bracket,
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
      <<TestLexer<'inp> as Lexer<'inp>>::Token as tokit::Token<'inp>>::Error,
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

// -- Comma type alias (zero-sized punctuator) ---------------------------------

use tokit::punct::Comma;

// -- Test macro ---------------------------------------------------------------

macro_rules! sw_delim_mutref_tests {
  ($name:ident, $wrap:expr, $input:expr) => {
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
        let mut f = |inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>| -> Result<i64, E> {
          parse_num(inp)
        };
        let cond = |peeked: Peeked<'_, 'inp, TestLexer<'inp>, U1>,
                     emitter: &mut Ctx::Emitter|
         -> Result<Action, E> { decide_num::<Ctx>(peeked, emitter) };
        let sw = SeparatedWhile::new::<Comma>(&mut f, cond);
        let wrap_fn: fn(
          SeparatedWhile<_, Comma, _, i64, U1, TestLexer<'inp>, Ctx>,
        ) -> _ = $wrap;
        let inner = wrap_fn(sw);
        let mut delim = DelimitedBy::<_, Bracket<(), (), ()>>::new(inner);
        let mut container = Vec::<i64>::new();
        let mut collect = Collect::new(&mut delim, &mut container);
        let _span: <TestLexer<'inp> as Lexer<'inp>>::Span = collect.parse_input(inp)?;
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

// == 1. No policy (plain) =====================================================

sw_delim_mutref_tests!(plain_unb, |sw| sw, "[1,2,3]");
sw_delim_mutref_tests!(plain_min, |sw| AtLeast::new(sw, 2), "[1,2,3]");
sw_delim_mutref_tests!(plain_max, |sw| AtMost::new(sw, 3), "[1,2,3]");
sw_delim_mutref_tests!(plain_bnd, |sw| Bounded::new(sw, 4, 2), "[1,2,3]");

// == 2. allow_leading =========================================================

sw_delim_mutref_tests!(al_unb, |sw| AllowLeading::new(sw), "[,1,2,3]");
sw_delim_mutref_tests!(
  al_min,
  |sw| AllowLeading::new(AtLeast::new(sw, 2)),
  "[,1,2,3]"
);
sw_delim_mutref_tests!(
  al_max,
  |sw| AllowLeading::new(AtMost::new(sw, 3)),
  "[,1,2,3]"
);
sw_delim_mutref_tests!(
  al_bnd,
  |sw| AllowLeading::new(Bounded::new(sw, 4, 2)),
  "[,1,2,3]"
);

// == 3. allow_trailing ========================================================

sw_delim_mutref_tests!(at_unb, |sw| AllowTrailing::new(sw), "[1,2,3,]");
sw_delim_mutref_tests!(
  at_min,
  |sw| AllowTrailing::new(AtLeast::new(sw, 2)),
  "[1,2,3,]"
);
sw_delim_mutref_tests!(
  at_max,
  |sw| AllowTrailing::new(AtMost::new(sw, 3)),
  "[1,2,3,]"
);
sw_delim_mutref_tests!(
  at_bnd,
  |sw| AllowTrailing::new(Bounded::new(sw, 4, 2)),
  "[1,2,3,]"
);

// == 4. allow_surrounded (allow_leading + allow_trailing) =====================

sw_delim_mutref_tests!(
  as_unb,
  |sw| AllowLeading::new(AllowTrailing::new(sw)),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  as_min,
  |sw| AllowLeading::new(AllowTrailing::new(AtLeast::new(sw, 2))),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  as_max,
  |sw| AllowLeading::new(AllowTrailing::new(AtMost::new(sw, 3))),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  as_bnd,
  |sw| AllowLeading::new(AllowTrailing::new(Bounded::new(sw, 4, 2))),
  "[,1,2,3,]"
);

// == 5. require_leading =======================================================

sw_delim_mutref_tests!(rl_unb, |sw| RequireLeading::new(sw), "[,1,2,3]");
sw_delim_mutref_tests!(
  rl_min,
  |sw| RequireLeading::new(AtLeast::new(sw, 2)),
  "[,1,2,3]"
);
sw_delim_mutref_tests!(
  rl_max,
  |sw| RequireLeading::new(AtMost::new(sw, 3)),
  "[,1,2,3]"
);
sw_delim_mutref_tests!(
  rl_bnd,
  |sw| RequireLeading::new(Bounded::new(sw, 4, 2)),
  "[,1,2,3]"
);

// == 6. require_trailing ======================================================

sw_delim_mutref_tests!(rt_unb, |sw| RequireTrailing::new(sw), "[1,2,3,]");
sw_delim_mutref_tests!(
  rt_min,
  |sw| RequireTrailing::new(AtLeast::new(sw, 2)),
  "[1,2,3,]"
);
sw_delim_mutref_tests!(
  rt_max,
  |sw| RequireTrailing::new(AtMost::new(sw, 3)),
  "[1,2,3,]"
);
sw_delim_mutref_tests!(
  rt_bnd,
  |sw| RequireTrailing::new(Bounded::new(sw, 4, 2)),
  "[1,2,3,]"
);

// == 7. require_surrounded (require_leading + require_trailing) ===============

sw_delim_mutref_tests!(
  rs_unb,
  |sw| RequireLeading::new(RequireTrailing::new(sw)),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  rs_min,
  |sw| RequireLeading::new(RequireTrailing::new(AtLeast::new(sw, 2))),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  rs_max,
  |sw| RequireLeading::new(RequireTrailing::new(AtMost::new(sw, 3))),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  rs_bnd,
  |sw| RequireLeading::new(RequireTrailing::new(Bounded::new(sw, 4, 2))),
  "[,1,2,3,]"
);

// == 8. allow_leading_require_trailing ========================================

sw_delim_mutref_tests!(
  alrt_unb,
  |sw| AllowLeading::new(RequireTrailing::new(sw)),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  alrt_min,
  |sw| AllowLeading::new(RequireTrailing::new(AtLeast::new(sw, 2))),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  alrt_max,
  |sw| AllowLeading::new(RequireTrailing::new(AtMost::new(sw, 3))),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  alrt_bnd,
  |sw| AllowLeading::new(RequireTrailing::new(Bounded::new(sw, 4, 2))),
  "[,1,2,3,]"
);

// == 9. require_leading_allow_trailing ========================================

sw_delim_mutref_tests!(
  rlat_unb,
  |sw| RequireLeading::new(AllowTrailing::new(sw)),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  rlat_min,
  |sw| RequireLeading::new(AllowTrailing::new(AtLeast::new(sw, 2))),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  rlat_max,
  |sw| RequireLeading::new(AllowTrailing::new(AtMost::new(sw, 3))),
  "[,1,2,3,]"
);
sw_delim_mutref_tests!(
  rlat_bnd,
  |sw| RequireLeading::new(AllowTrailing::new(Bounded::new(sw, 4, 2))),
  "[,1,2,3,]"
);
