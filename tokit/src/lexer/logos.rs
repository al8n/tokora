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

    /// A lexer implementation for [`logos`](https://docs.rs/logos)-based lexers.
    ///
    /// # Limit-error latching
    ///
    /// When the lexer state (`extras`) reports a limit error from
    /// [`check`](Lexer::check) after a token is scanned, [`lex`](Lexer::lex) returns that
    /// error **once** and then latches: every subsequent [`lex`](Lexer::lex) returns
    /// `None`, exactly as if the input were exhausted. This bounds the work an
    /// error-recovery caller performs on untrusted input — once the limiter trips the
    /// lexer stops scanning rather than re-failing on every remaining token.
    ///
    /// After latching, [`span`](Lexer::span), [`slice`](Lexer::slice) and
    /// [`bump`](Lexer::bump) continue to delegate to the inner lexer, which is positioned
    /// at the token that tripped the limit; they behave exactly as they do once the input
    /// is exhausted (the latch performs no further scanning).
    ///
    /// Mutating the state through [`state_mut`](Lexer::state_mut) (e.g. resetting the
    /// limiter) does **not** clear the latch: poisoning is sticky for the lifetime of the
    /// lexer instance. A fresh lexer must be constructed to lex again.
    pub struct LogosLexer<'inp, T: FromLogos<'inp>> {
      inner: $lib::Lexer<'inp, T::Logos>,
      poisoned: bool,
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
          poisoned: false,
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
        // logos guarantees `start <= end` for every token span, so construct the
        // span directly and skip the checked `From<Range>` -> `new` bounds assert.
        let range = self.inner.span();
        SimpleSpan {
          start: range.start,
          end: range.end,
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
      fn lex(&mut self) -> Option<Result<T, T::Error>>
      where
        T: Token<'inp>,
      {
        // Once a limit error has been reported, latch: report EOF forever so an
        // error-recovery caller cannot be made to scan the whole input.
        if self.poisoned {
          return None;
        }
        match self.inner.next() {
          Some(Ok(tok)) => match self.check() {
            Ok(_) => Some(Ok(T::from_logos(tok))),
            Err(e) => {
              self.poisoned = true;
              Some(Err(e))
            }
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

  // ── Limit-error latching ─────────────────────────────────────────────────

  use crate::state::token_tracker::{TokenLimitExceeded, TokenLimiter};

  #[derive(Debug, Clone, PartialEq)]
  enum LimitErr {
    Lex,
    Limit(TokenLimitExceeded),
  }

  impl From<()> for LimitErr {
    fn from(_: ()) -> Self {
      LimitErr::Lex
    }
  }

  impl From<TokenLimitExceeded> for LimitErr {
    fn from(e: TokenLimitExceeded) -> Self {
      LimitErr::Limit(e)
    }
  }

  #[derive(Debug, Clone, PartialEq, logos::Logos)]
  #[logos(crate = logos, extras = TokenLimiter, skip r"[ \t\r\n]+")]
  enum LimitedTok {
    // Each scanned token bumps the limiter; the over-limit condition is caught by
    // `LogosLexer::lex` via `check()`, not by the callback itself.
    #[regex(r"[0-9]+", |lex| { lex.extras.increase(); })]
    Num,
  }

  impl core::fmt::Display for LimitedTok {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      write!(f, "number")
    }
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum LimitedKind {
    Num,
  }

  impl core::fmt::Display for LimitedKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      write!(f, "number")
    }
  }

  impl TokenTrait<'_> for LimitedTok {
    type Kind = LimitedKind;
    type Error = LimitErr;

    fn kind(&self) -> LimitedKind {
      LimitedKind::Num
    }

    fn is_trivia(&self) -> bool {
      false
    }
  }

  type LimitedLexer<'a> = super::logos_0_16::LogosLexer<'a, LimitedTok>;

  #[test]
  fn logos_lexer_latches_after_limit_error() {
    // Limit of 2: the third scanned token trips `check()`.
    let mut lexer = LimitedLexer::with_state("1 2 3 4 5 6", TokenLimiter::with_limitation(2));

    assert!(matches!(lexer.lex(), Some(Ok(_))), "first token");
    assert!(matches!(lexer.lex(), Some(Ok(_))), "second token");

    // Third token trips the limiter: exactly one limit error is returned.
    assert!(
      matches!(lexer.lex(), Some(Err(LimitErr::Limit(_)))),
      "limit error on the tripping token"
    );

    let tokens_at_trip = lexer.state().tokens();
    assert_eq!(
      tokens_at_trip, 3,
      "three tokens were scanned before latching"
    );

    // Latched: every subsequent `lex()` is `None` and NO further scanning happens
    // (the counting callback proves bounded work — the count never advances).
    for _ in 0..5 {
      assert!(lexer.lex().is_none(), "latched to EOF");
    }
    assert_eq!(
      lexer.state().tokens(),
      tokens_at_trip,
      "no further tokens scanned after the latch"
    );
  }

  #[test]
  fn logos_lexer_latch_inherited_by_lex_spanned() {
    use super::super::Lexed;

    // The `lex_spanned`/iterator surface routes through `lex`, so it inherits the latch.
    let mut lexer = LimitedLexer::with_state("1 2 3 4 5", TokenLimiter::with_limitation(2));

    let mut errors = 0usize;
    let mut last_was_error = false;
    while let Some(spanned) = Lexed::lex_spanned(&mut lexer) {
      let (_, lexed) = spanned.into_components();
      last_was_error = lexed.is_error();
      if last_was_error {
        errors += 1;
      }
    }

    assert_eq!(
      errors, 1,
      "exactly one limit error surfaced via lex_spanned"
    );
    assert!(
      last_was_error,
      "iteration stopped right after the limit error"
    );
    assert_eq!(
      lexer.state().tokens(),
      3,
      "bounded work: scanning stopped at the trip point"
    );
  }
}
