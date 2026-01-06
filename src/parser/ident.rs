use crate::{
  error::UnexpectedEot,
  token::IdentifierToken,
  try_parse_input::ParseAttempt,
  types::Ident,
};

use super::*;

impl Ident<(), ()> {
  /// A parser that parses a token and returns an `Ident` instance if matches.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not an identifier,
  /// and promise no valid token is consumed.
  pub fn try_parse<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<
    ParseAttempt<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: IdentifierToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
  {
    Self::try_parse_of(inp)
  }

  /// A parser that parses a token and returns an `Ident` instance if matches for a specific language.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not an identifier,
  /// and promise no valid token is consumed.
  pub fn try_parse_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    ParseAttempt<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: IdentifierToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    inp.try_expect_valid(|t, _| {
      t.data.is_identifier()
    }).map(|res| res.map(|tok| Ident::new(tok.into_span(), inp.slice())).into())
  }
}
