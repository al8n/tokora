use core::marker::PhantomData;

use crate::{IntoLexer, utils::Span};

use super::{Lexer, Source, State, Token};

/// a
#[repr(transparent)]
pub struct LogosLexer<'inp, T, L: logos::Logos<'inp>> {
  inner: logos::Lexer<'inp, L>,
  _marker: PhantomData<T>,
}

impl<'inp, T, L> IntoLexer<'inp, T> for logos::Lexer<'inp, L>
where
  T: From<L> + Token<'inp> + 'inp,
  T::Error: From<L::Error> + From<<L::Extras as State>::Error>,
  L: logos::Logos<'inp> + 'inp,
  L::Extras: State,
  L::Source: Source<usize, Slice<'inp> = <L::Source as logos::Source>::Slice<'inp>>,
{
  type Lexer = LogosLexer<'inp, T, L>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_lexer(self) -> Self::Lexer {
    LogosLexer {
      inner: self,
      _marker: PhantomData,
    }
  }
}

impl<'inp, T, L> Lexer<'inp> for LogosLexer<'inp, T, L>
where
  T: From<L> + Token<'inp> + 'inp,
  T::Error: From<L::Error> + From<<L::Extras as State>::Error>,
  L: logos::Logos<'inp> + 'inp,
  L::Extras: State,
  L::Source: Source<usize, Slice<'inp> = <L::Source as logos::Source>::Slice<'inp>>,
{
  type State = L::Extras;
  type Source = L::Source;
  type Token = T;
  // type Cursor = usize;
  type Span = Span;
  type Offset = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn new(src: &'inp Self::Source) -> Self
  where
    Self::State: Default,
  {
    logos::Lexer::new(src).into_lexer()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn with_state(src: &'inp Self::Source, state: Self::State) -> Self {
    logos::Lexer::with_extras(src, state).into_lexer()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn state(&self) -> &Self::State {
    &self.inner.extras
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn state_mut(&mut self) -> &mut Self::State {
    &mut self.inner.extras
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&self) -> Result<(), T::Error>
  where
    T: Token<'inp>,
  {
    self.inner.extras.check().map_err(Into::into)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_state(self) -> Self::State {
    self.inner.extras
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn source(&self) -> &'inp Self::Source
  where
    T: Token<'inp>,
  {
    self.inner.source()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn span(&self) -> Span {
    self.inner.span().into()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lex(&mut self) -> Option<Result<T, T::Error>>
  where
    T: Token<'inp>,
  {
    match self.inner.next() {
      Some(Ok(tok)) => match <Self as Lexer<'_>>::check(self) {
        Ok(()) => Some(Ok(T::from(tok))),
        Err(err) => Some(Err(err)),
      },
      Some(Err(err)) => Some(Err(err.into())),
      None => None,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn slice(&self) -> <Self::Source as Source<Self::Offset>>::Slice<'inp>
  where
    T: Token<'inp>,
  {
    self.inner.slice()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn bump(&mut self, n: &usize) {
    self.inner.bump(*n);
  }
}
