use super::*;

impl<'a, L, S, E, Lang: ?Sized> FullContainerEmitter<'a, L, Lang> for Verbose<E, S, Lang>
where
  L: Lexer<'a, Span = S, Offset = S::Offset>,
  E: FromFullContainerError<'a, L, Lang>,
  S: Span + Ord + Clone,
  Verbose<E, S, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_full_container(&mut self, err: FullContainer<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    let span = err.span().clone();
    self
      .errs
      .entry(span)
      .or_default()
      .push(E::from_full_container(err));
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  const fn assert_noop_full_container_emitter<'a, L, Error, E>()
  where
    L: Lexer<'a>,
    E: FullContainerEmitter<'a, L, Error = Error>,
  {
  }

  assert_noop_full_container_emitter::<'_, DummyLexer, (), Verbose<()>>();
};
