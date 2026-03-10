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

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg(feature = "logos_0_16")]
mod tests {
  use super::super::{Lexer, Token as TokenTrait};
  use crate::span::Span;

  use ::logos_0_16 as logos;

  #[derive(Debug, Clone, PartialEq, logos::Logos)]
  #[logos(crate = logos, skip r"[ \t\r\n]+")]
  enum TestTok {
    #[token("+")]
    Plus,
    #[regex(r"[0-9]+")]
    Num,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum TestKind {
    Plus,
    Num,
  }

  impl core::fmt::Display for TestKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TestKind::Plus => write!(f, "+"),
        TestKind::Num => write!(f, "number"),
      }
    }
  }

  impl core::fmt::Display for TestTok {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TestTok::Plus => write!(f, "+"),
        TestTok::Num => write!(f, "number"),
      }
    }
  }

  impl TokenTrait<'_> for TestTok {
    type Kind = TestKind;
    type Error = ();

    fn kind(&self) -> TestKind {
      match self {
        TestTok::Plus => TestKind::Plus,
        TestTok::Num => TestKind::Num,
      }
    }

    fn is_trivia(&self) -> bool {
      false
    }
  }

  type TestLexer<'a> = super::logos_0_16::LogosLexer<'a, TestTok>;

  #[test]
  fn logos_lexer_new() {
    let lexer = TestLexer::new("42 + 1");
    let _ = lexer;
  }

  #[test]
  fn logos_lexer_with_state() {
    let lexer = TestLexer::with_state("42 + 1", ());
    let _ = lexer;
  }

  #[test]
  fn logos_lexer_lex_tokens() {
    let mut lexer = TestLexer::new("42 + 1");
    let tok1 = lexer.lex().unwrap().unwrap();
    assert_eq!(tok1.kind(), TestKind::Num);
    let tok2 = lexer.lex().unwrap().unwrap();
    assert_eq!(tok2.kind(), TestKind::Plus);
    let tok3 = lexer.lex().unwrap().unwrap();
    assert_eq!(tok3.kind(), TestKind::Num);
    assert!(lexer.lex().is_none());
  }

  #[test]
  fn logos_lexer_source() {
    let mut lexer = TestLexer::new("hello");
    // Need to lex at least once to have a valid source reference
    assert_eq!(lexer.source(), "hello");
  }

  #[test]
  fn logos_lexer_state() {
    let lexer = TestLexer::new("42");
    let _state: &() = lexer.state();
  }

  #[test]
  fn logos_lexer_state_mut() {
    let mut lexer = TestLexer::new("42");
    let _state: &mut () = lexer.state_mut();
  }

  #[test]
  fn logos_lexer_into_state() {
    let lexer = TestLexer::new("42");
    let _state: () = lexer.into_state();
  }

  #[test]
  fn logos_lexer_check() {
    let lexer = TestLexer::new("42");
    assert!(lexer.check().is_ok());
  }

  #[test]
  fn logos_lexer_span() {
    let mut lexer = TestLexer::new("42 + 1");
    let _ = lexer.lex(); // consume "42"
    let span = lexer.span();
    assert_eq!(span.start(), 0);
    assert_eq!(span.end(), 2);
  }

  #[test]
  fn logos_lexer_slice() {
    let mut lexer = TestLexer::new("42 + 1");
    let _ = lexer.lex(); // consume "42"
    assert_eq!(lexer.slice(), "42");
  }

  #[test]
  fn logos_lexer_bump() {
    let mut lexer = TestLexer::new("42 + 1");
    lexer.bump(&1);
    let _ = lexer;
  }

  #[test]
  fn logos_lexer_inner() {
    let lexer = TestLexer::new("42");
    let _inner = lexer.inner();
  }

  #[test]
  fn logos_lexer_inner_mut() {
    let mut lexer = TestLexer::new("42");
    let _inner = lexer.inner_mut();
  }

  #[test]
  fn logos_lexer_into_inner() {
    let lexer = TestLexer::new("42");
    let _inner = lexer.into_inner();
  }

  #[test]
  fn logos_lexer_into_lexer_trait() {
    use super::super::IntoLexer;
    use ::logos_0_16::Logos;
    let raw_lexer = TestTok::lexer("42");
    let _logos_lexer: TestLexer<'_> = raw_lexer.into_lexer();
  }

  #[test]
  fn logos_lexer_from_logos_identity() {
    use super::logos_0_16::FromLogos;
    let tok = TestTok::Plus;
    let converted = TestTok::from_logos(tok.clone());
    assert_eq!(converted, tok);
  }
}
