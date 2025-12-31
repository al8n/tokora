use mayber::Maybe::{Owned, Ref};

use crate::{lexer::IdentifierToken, types::Ident};

use super::*;

impl Ident<(), (), ()> {
  /// A parser that parses an identifier token and returns an `Ident` instance.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not an identifier,
  /// and promise no valid token is consumed.
  pub fn parse_optional<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Option<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: IdentifierToken<'inp, Source = <L::Source as Source<L::Offset>>::Slice<'inp>>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    let end = inp.cursor().as_inner().clone();
    let tok = inp.sync_until_token()?;

    match tok {
      None => Err(UnexpectedEot::eot_of(end).into()),
      Some(ct) => match ct {
        Ref(ct) => {
          if !ct.token().data().is_identifier() {
            return Ok(None);
          }

          Ok(inp.next_token()?.map(|t| {
            let (span, tok) = t.into_components();
            Ident::new(
              span,
              tok
                .try_into_identifier()
                .expect("token checked to be identifier"),
            )
          }))
        }
        Owned(ct) => {
          let (span, tok) = ct.into_token().into_components();

          Ok(match tok.try_into_identifier() {
            Ok(ident) => Some(Ident::new(span, ident)),
            Err(_) => None,
          })
        }
      },
    }
  }
}
