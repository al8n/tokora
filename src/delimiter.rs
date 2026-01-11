use crate::{Lexer, Token, error::token::UnexpectedToken, punct::Punctuator, span::Spanned};

/// A trait for any delimiter consisting of an opening and a closing punctuator.
pub trait DelimiterSelector<'inp, L, Lang: ?Sized = ()> {
  /// The opening punctuator.
  type Open: Punctuator<'inp, L, Lang>;
  /// The closing punctuator.
  type Close: Punctuator<'inp, L, Lang>;

  /// Checks if the given token kind is the opening delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open(knd: &<L::Token as Token<'inp>>::Kind) -> bool
  where
    L: Lexer<'inp>,
  {
    <Self::Open as Punctuator<'inp, L, Lang>>::eval(knd)
  }

  /// Checks if the given token kind is the closing delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close(knd: &<L::Token as Token<'inp>>::Kind) -> bool
  where
    L: Lexer<'inp>,
  {
    <Self::Close as Punctuator<'inp, L, Lang>>::eval(knd)
  }

  /// Creates an `UnexpectedToken` error for an unexpected opening token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unexpected_open_token(
    tok: Spanned<L::Token, L::Span>,
  ) -> UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>
  where
    L: Lexer<'inp>,
  {
    <Self::Open as Punctuator<'inp, L, Lang>>::unexpected_token(tok)
  }

  /// Creates an `UnexpectedToken` error for an unexpected closing token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unexpected_close_token(
    tok: Spanned<L::Token, L::Span>,
  ) -> UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>
  where
    L: Lexer<'inp>,
  {
    <Self::Close as Punctuator<'inp, L, Lang>>::unexpected_token(tok)
  }
}

macro_rules! impl_deref {
  (@impl<$ty:ty>) => {
    type Open = <$ty>::Open;
    type Close = <$ty>::Close;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_open(knd: &<<L>::Token as Token<'inp>>::Kind) -> bool
    where
      L: Lexer<'inp>,
    {
      <$ty>::is_open(knd)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_close(knd: &<<L>::Token as Token<'inp>>::Kind) -> bool
    where
      L: Lexer<'inp>,
    {
      <$ty>::is_close(knd)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn unexpected_open_token(
      tok: Spanned<L::Token, L::Span>,
    ) -> UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>
    where
      L: Lexer<'inp>,
    {
      <$ty>::unexpected_open_token(tok)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn unexpected_close_token(
      tok: Spanned<L::Token, L::Span>,
    ) -> UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>
    where
      L: Lexer<'inp>,
    {
      <$ty>::unexpected_close_token(tok)
    }
  };
}

impl<'inp, L, Lang: ?Sized, D: ?Sized> DelimiterSelector<'inp, L, Lang> for &D
where
  L: Lexer<'inp>,
  D: DelimiterSelector<'inp, L, Lang>,
{
  impl_deref!(@impl<D>);
}

impl<'inp, L, Lang: ?Sized, D: ?Sized> DelimiterSelector<'inp, L, Lang> for &mut D
where
  L: Lexer<'inp>,
  D: DelimiterSelector<'inp, L, Lang>,
{
  impl_deref!(@impl<D>);
}
