use crate::{
  error::{syntax::MissingSyntaxOf, token::MissingTokenOf},
  utils::CowStr,
};

use super::*;

impl<'inp, L, S, E, Lang: ?Sized> SeparatedEmitter<'inp, L, Lang> for Verbose<E, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  E: FromSeparatedError<'inp, L, Lang>,
  S: Span + Ord + Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    let off = err.offset_ref().clone();
    self.errs.insert(
      S::new(off.clone(), off),
      E::from_missing_separator(name, err),
    );
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(&mut self, err: MissingSyntaxOf<'inp, L, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    let off = err.offset_ref().clone();
    self
      .errs
      .insert(S::new(off.clone(), off), E::from_missing_element(err));
    Ok(())
  }
}
