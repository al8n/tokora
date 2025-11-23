use crate::utils::Span;

use super::{Lexer, Source, State, Token};

impl<'source, T, L> Lexer<'source, T> for logos::Lexer<'source, L>
where
  T: From<L> + Token<'source> + 'source,
  T::Error: From<L::Error> + From<<L::Extras as State>::Error>,
  L: logos::Logos<'source> + 'source,
  L::Extras: State,
  L::Source: Source<usize, Slice<'source> = <L::Source as logos::Source>::Slice<'source>>,
{
  type State = L::Extras;
  type Source = L::Source;
  // type Cursor = usize;
  type Span = Span;
  type Offset = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn new(src: &'source Self::Source) -> Self
  where
    Self::State: Default,
  {
    logos::Lexer::new(src)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn with_state(src: &'source Self::Source, state: Self::State) -> Self {
    logos::Lexer::with_extras(src, state)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn state(&self) -> &Self::State {
    &self.extras
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn state_mut(&mut self) -> &mut Self::State {
    &mut self.extras
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&self) -> Result<(), T::Error>
  where
    T: Token<'source>,
  {
    self.extras.check().map_err(Into::into)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_state(self) -> Self::State {
    self.extras
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn source(&self) -> &'source Self::Source
  where
    T: Token<'source>,
  {
    self.source()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn span(&self) -> Span {
    self.span().into()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lex(&mut self) -> Option<Result<T, T::Error>>
  where
    T: Token<'source>,
  {
    match self.next() {
      Some(Ok(tok)) => match <Self as Lexer<'_, T>>::check(self) {
        Ok(()) => Some(Ok(T::from(tok))),
        Err(err) => Some(Err(err)),
      },
      Some(Err(err)) => Some(Err(err.into())),
      None => None,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn slice(&self) -> <Self::Source as Source<Self::Offset>>::Slice<'source>
  where
    T: Token<'source>,
  {
    self.slice()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn bump(&mut self, n: &usize) {
    self.bump(*n);
  }
}
