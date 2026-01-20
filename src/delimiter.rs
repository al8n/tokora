use crate::{
  Lexer, Token,
  error::token::UnexpectedToken,
  punct::{Brace, Bracket, CloseBrace, CloseBracket, OpenBrace, OpenBracket, Punctuator},
  span::Spanned,
  utils::Message,
};

/// A trait for any delimiter consisting of an opening and a closing punctuator.
pub trait DelimiterSelector<'inp, L, Lang: ?Sized = ()> {
  /// The opening punctuator.
  type Open: Punctuator<'inp, L, Lang>;
  /// The closing punctuator.
  type Close: Punctuator<'inp, L, Lang>;

  /// The name of the delimiter.
  fn name() -> Message;

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

impl<'inp, S, C, L, Lang: ?Sized> DelimiterSelector<'inp, L, Lang> for Brace<S, C, Lang>
where
  L: Lexer<'inp>,
  OpenBrace<S, C, Lang>: Punctuator<'inp, L, Lang>,
  CloseBrace<S, C, Lang>: Punctuator<'inp, L, Lang>,
{
  type Open = OpenBrace<S, C, Lang>;

  type Close = CloseBrace<S, C, Lang>;

  fn name() -> Message {
    Message::from_static("{}")
  }
}

impl<'inp, S, C, L, Lang: ?Sized> DelimiterSelector<'inp, L, Lang> for Bracket<S, C, Lang>
where
  L: Lexer<'inp>,
  OpenBracket<S, C, Lang>: Punctuator<'inp, L, Lang>,
  CloseBracket<S, C, Lang>: Punctuator<'inp, L, Lang>,
{
  type Open = OpenBracket<S, C, Lang>;

  type Close = CloseBracket<S, C, Lang>;

  fn name() -> Message {
    Message::from_static("[]")
  }
}

macro_rules! impl_deref {
  (@impl<$ty:ty>) => {
    type Open = <$ty>::Open;
    type Close = <$ty>::Close;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn name() -> Message {
      <$ty>::name()
    }

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
