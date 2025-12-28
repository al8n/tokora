use crate::error::syntax::MissingSyntaxOf;

use super::*;

impl<'inp, L, O, S, Sep, E, Lang: ?Sized> SeparatedEmitter<'inp, O, Sep, L, Lang>
  for Verbose<E, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  E: FromSeparatedError<'inp, O, Sep, L, Lang>,
  S: Span + Ord + Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    err: MissingSeparatorOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    let off = err.offset_ref().clone();
    self
      .errs
      .insert(S::new(off.clone(), off), E::from_missing_separator(err));
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L, Lang>,
  ) -> Result<(), Self::Error>
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
