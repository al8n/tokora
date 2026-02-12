macro_rules! bail {
  ($lib:ident) => {
    use core::marker::PhantomData;

    use $lib::Logos;

    use crate::span::SimpleSpan;

    use super::super::{IntoLexer, Lexer, Source, State, Token};

    /// A trait for token types that can be created from `logos::Logos` types.
    pub trait FromLogos<'inp>: Token<'inp> {
      /// The type which implements `logos::Logos`.
      type Logos: Logos<'inp>;

      /// Converts a `logos::Logos` token into this token type.
      fn from_logos(logos_token: Self::Logos) -> Self;
    }

    impl<'inp, T> FromLogos<'inp> for T
    where
      T: Token<'inp> + Logos<'inp>,
    {
      type Logos = T;

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn from_logos(t: Self::Logos) -> Self {
        t
      }
    }

    /// A lexer implementation for [`logos`]-based lexers.
    #[repr(transparent)]
    pub struct LogosLexer<'inp, T: FromLogos<'inp>> {
      inner: $lib::Lexer<'inp, T::Logos>,
      _marker: PhantomData<T>,
    }

    impl<'inp, T, L> IntoLexer<'inp, T> for $lib::Lexer<'inp, L>
    where
      T: FromLogos<'inp, Logos = L> + Token<'inp> + 'inp,
      T::Error: From<L::Error> + From<<L::Extras as State>::Error>,
      L: Logos<'inp> + 'inp,
      L::Extras: State,
      L::Source: Source<usize, Slice<'inp> = <L::Source as $lib::Source>::Slice<'inp>>,
    {
      type Lexer = LogosLexer<'inp, T>;

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn into_lexer(self) -> Self::Lexer {
        LogosLexer {
          inner: self,
          _marker: PhantomData,
        }
      }
    }

    impl<'inp, T> LogosLexer<'inp, T>
    where
      T: FromLogos<'inp>,
    {
      /// Consumes the lexer and returns the inner `$lib::Lexer`.
      #[cfg_attr(not(tarpaulin), inline(always))]
      pub fn into_inner(self) -> $lib::Lexer<'inp, T::Logos> {
        self.inner
      }

      /// Returns a reference to the inner `$lib::Lexer`.
      #[cfg_attr(not(tarpaulin), inline(always))]
      pub const fn inner(&self) -> &$lib::Lexer<'inp, T::Logos> {
        &self.inner
      }

      /// Returns a reference to the inner `$lib::Lexer`.
      #[cfg_attr(not(tarpaulin), inline(always))]
      pub const fn inner_mut(&mut self) -> &mut $lib::Lexer<'inp, T::Logos> {
        &mut self.inner
      }
    }

    impl<'inp, T> Lexer<'inp> for LogosLexer<'inp, T>
    where
      T: FromLogos<'inp> + Token<'inp>,
      T::Error: From<<T::Logos as Logos<'inp>>::Error>
        + From<<<T::Logos as Logos<'inp>>::Extras as State>::Error>,
      <T::Logos as Logos<'inp>>::Extras: State + Default,
      <T::Logos as Logos<'inp>>::Source: Source<
          usize,
          Slice<'inp> = <<T::Logos as Logos<'inp>>::Source as $lib::Source>::Slice<'inp>,
        >,
    {
      type State = <T::Logos as Logos<'inp>>::Extras;
      type Source = <T::Logos as Logos<'inp>>::Source;
      type Token = T;
      type Span = SimpleSpan;
      type Offset = usize;

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn new(src: &'inp Self::Source) -> Self {
        $lib::Lexer::new(src).into_lexer()
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn with_state(src: &'inp Self::Source, state: Self::State) -> Self {
        $lib::Lexer::with_extras(src, state).into_lexer()
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn check(&self) -> Result<(), T::Error>
      where
        T: Token<'inp>,
      {
        self.inner.extras.check().map_err(Into::into)
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
      fn span(&self) -> Self::Span {
        self.inner.span().into()
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn slice(&self) -> <Self::Source as Source<Self::Offset>>::Slice<'inp>
      where
        T: Token<'inp>,
      {
        self.inner.slice()
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn lex(&mut self) -> Option<Result<T, T::Error>>
      where
        T: Token<'inp>,
      {
        match self.inner.next() {
          Some(Ok(tok)) => match self.check() {
            Ok(_) => Some(Ok(T::from_logos(tok))),
            Err(e) => Some(Err(e)),
          },
          Some(Err(err)) => Some(Err(err.into())),
          None => None,
        }
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn bump(&mut self, n: &usize) {
        self.inner.bump(*n);
      }
    }
  };
}

#[cfg(feature = "logos_0_16")]
pub use self::logos_0_16::{FromLogos, LogosLexer};

#[cfg(all(feature = "logos_0_15", not(feature = "logos_0_16")))]
pub use self::logos_0_15::{FromLogos, LogosLexer};

#[cfg(all(
  feature = "logos_0_14",
  not(any(feature = "logos_0_15", feature = "logos_0_16"))
))]
pub use self::logos_0_14::{FromLogos, LogosLexer};

/// A module containing integrations with the `logos` lexer library version 0.16.
#[cfg(feature = "logos_0_16")]
#[cfg_attr(docsrs, doc(cfg(feature = "logos_0_16")))]
pub mod logos_0_16 {
  bail!(logos_0_16);
}

/// A module containing integrations with the `logos` lexer library version 0.15.
#[cfg(feature = "logos_0_15")]
#[cfg_attr(docsrs, doc(cfg(feature = "logos_0_15")))]
pub mod logos_0_15 {
  bail!(logos_0_15);
}

/// A module containing integrations with the `logos` lexer library version 0.14.
#[cfg(feature = "logos_0_14")]
#[cfg_attr(docsrs, doc(cfg(feature = "logos_0_14")))]
pub mod logos_0_14 {
  bail!(logos_0_14);
}
