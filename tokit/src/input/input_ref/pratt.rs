use crate::{
  emitter::PrattEmitter,
  error::{UnexpectedEoLhs, UnexpectedEoRhs},
  parser::{
    PrattFoldTokenInfix, PrattFoldTokenPostfix, PrattFoldTokenPrefix, PrattInfix, PrattLHS,
    PrattPower, PrattRHS, Precedenced,
  },
  token::PrattToken,
};

use super::*;

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// a
  pub fn pratt<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power>(
    &mut self,
    mut fold_prefix: FoldPrefix,
    mut fold_infix: FoldInfix,
    mut fold_postfix: FoldPostfix,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L::Token: PrattToken<'inp, Expr, Power>,
    Ctx::Emitter: PrattEmitter<'inp, L, Lang>,
    Power: PrattPower,
    FoldPrefix: PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang>,
    FoldInfix: PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang>,
    FoldPostfix: PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang>,
  {
    self.pratt_in(
      Power::default(),
      &mut fold_prefix,
      &mut fold_infix,
      &mut fold_postfix,
    )
  }

  fn pratt_in<FoldPrefix, FoldInfix, FoldPostfix, Expr, Power>(
    &mut self,
    min_power: Power,
    fold_prefix: &mut FoldPrefix,
    fold_infix: &mut FoldInfix,
    fold_postfix: &mut FoldPostfix,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L::Token: PrattToken<'inp, Expr, Power>,
    Ctx::Emitter: PrattEmitter<'inp, L, Lang>,
    Power: PrattPower,
    FoldPrefix: PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang>,
    FoldInfix: PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang>,
    FoldPostfix: PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang>,
  {
    let Some((lhs, tok)) = self.try_expect_map(|tok| tok.try_pratt_lhs())? else {
      return Ok(None);
    };

    let mut lhs = match lhs {
      PrattLHS::Operand(_) => tok,
      PrattLHS::Prefix(precedenced) => {
        let power = precedenced.into_precedence();
        let Some(operand) = self.pratt_in(power, fold_prefix, fold_infix, fold_postfix)? else {
          self
            .emitter
            .emit_unexpected_end_of_lhs(UnexpectedEoLhs::eolhs_of(self.offset().clone()))?;
          return Ok(Some(tok));
        };

        fold_prefix.fold_prefix(tok, operand, self.emitter())?
      }
    };

    // Step 2: parse rhs -- either an infix/postfix operator or the end of this pratt expression
    loop {
      if self.is_eoi() {
        break;
      }

      let Some((rhs, tok)) = self.try_expect_map(|tok| {
        tok.try_pratt_rhs().and_then(|rhs| match rhs {
          PrattRHS::Postfix(precedenced) => {
            let power = precedenced.into_precedence();
            if power >= min_power {
              Some(PrattRHS::Postfix(Precedenced::new((), power)))
            } else {
              None
            }
          }
          PrattRHS::Infix(precedenced) => {
            let (infix, lpower) = precedenced.into_components();
            let rpower = match infix {
              PrattInfix::Left(_) => lpower.next(),
              PrattInfix::Right(_) => lpower.prev(),
              PrattInfix::Neither(_) => lpower.next(),
            };

            if lpower.lt(&min_power) {
              None
            } else {
              Some(PrattRHS::Infix(Precedenced::new(infix, rpower)))
            }
          }
        })
      })?
      else {
        break;
      };

      match rhs {
        PrattRHS::Postfix(_) => lhs = fold_postfix.fold_postfix(lhs, tok, self.emitter())?,
        PrattRHS::Infix(infix) => {
          let (infix, power) = infix.into_components();
          let Some(rhs) = self.pratt_in(power, fold_prefix, fold_infix, fold_postfix)? else {
            self
              .emitter
              .emit_unexpected_end_of_rhs(UnexpectedEoRhs::eorhs_of(self.offset().clone()))?;
            return Ok(Some(lhs));
          };
          let infix = {
            let (span, tok) = tok.into_components();
            let infix = match infix {
              PrattInfix::Left(_) => PrattInfix::Left(tok),
              PrattInfix::Right(_) => PrattInfix::Right(tok),
              PrattInfix::Neither(_) => PrattInfix::Neither(tok),
            };
            Spanned::new(span, infix)
          };
          lhs = fold_infix.fold_infix(lhs, rhs, infix, self.emitter())?;
        }
      }
    }

    Ok(Some(lhs))
  }
}
