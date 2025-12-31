use crate::{error::UnexpectedEot, lexer::IdentifierToken, types::Ident};

use super::*;

/// A parser that parses a token and returns an `Ident` instance if matches.
///
/// If the function returns `Ok(None)`, it means the next token is not an identifier,
/// and promise no valid token is consumed.
pub fn try_ident<'inp, L, Ctx>(
  inp: &mut InputRef<'inp, '_, L, Ctx>,
) -> Result<
  Option<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>>,
  <Ctx::Emitter as Emitter<'inp, L>>::Error,
>
where
  L: Lexer<'inp>,
  L::Token: IdentifierToken<'inp>,
  Ctx: ParseContext<'inp, L>,
  <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
{
  try_ident_of(inp)
}

/// A parser that parses a token and returns an `Ident` instance if matches for a specific language.
///
/// If the function returns `Ok(None)`, it means the next token is not an identifier,
/// and promise no valid token is consumed.
pub fn try_ident_of<'inp, L, Ctx, Lang: ?Sized>(
  inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
) -> Result<
  Option<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
>
where
  L: Lexer<'inp>,
  L::Token: IdentifierToken<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
{
  let end = inp.cursor().as_inner().clone();
  let tok = inp.sync_until_token()?;

  match tok {
    None => Err(UnexpectedEot::eot_of(end).into()),
    Some(ct) => {
      let (span, ident) = ct
        .map(
          |t| {
            let (span, t) = t.into_token().into_components();
            (span.clone(), t.is_identifier())
          },
          |t| {
            let (span, t) = t.into_token().into_components();
            (span, t.is_identifier())
          },
        )
        .into_inner();

      if !ident {
        return Ok(None);
      }

      inp.skip_one();
      Ok(Some(Ident::new(span, inp.slice())))
    }
  }
}
