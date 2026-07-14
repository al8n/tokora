use crate::{Lexer, Token, error::token::UnexpectedToken, punct::*, span::Spanned, utils::CowStr};

/// A trait for any delimiter consisting of an opening and a closing punctuator.
pub trait Delimiter<'inp, L, Lang: ?Sized = ()> {
  /// The opening punctuator.
  type Open: Punctuator<'inp, L, Lang>;
  /// The closing punctuator.
  type Close: Punctuator<'inp, L, Lang>;

  /// The name of the delimiter.
  fn name() -> CowStr;

  /// Checks if the given token kind is the opening delimiter.
  #[inline(always)]
  fn is_open(knd: &<L::Token as Token<'inp>>::Kind) -> bool
  where
    L: Lexer<'inp>,
  {
    <Self::Open as Punctuator<'inp, L, Lang>>::eval(knd)
  }

  /// Checks if the given token kind is the closing delimiter.
  #[inline(always)]
  fn is_close(knd: &<L::Token as Token<'inp>>::Kind) -> bool
  where
    L: Lexer<'inp>,
  {
    <Self::Close as Punctuator<'inp, L, Lang>>::eval(knd)
  }

  /// Creates an `UnexpectedToken` error for an unexpected opening token.
  #[inline(always)]
  fn unexpected_open_token(
    tok: Spanned<L::Token, L::Span>,
  ) -> UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>
  where
    L: Lexer<'inp>,
  {
    <Self::Open as Punctuator<'inp, L, Lang>>::unexpected_token(tok)
  }

  /// Creates an `UnexpectedToken` error for an unexpected closing token.
  #[inline(always)]
  fn unexpected_close_token(
    tok: Spanned<L::Token, L::Span>,
  ) -> UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>
  where
    L: Lexer<'inp>,
  {
    <Self::Close as Punctuator<'inp, L, Lang>>::unexpected_token(tok)
  }
}

macro_rules! impl_builtin_delimiter {
  ($($name:ident { description: $description:literal, open: $open:ident, close: $close:ident $(,)? }),+$(,)?) => {
    $(
      impl<'inp, S, C, L, Lang: ?Sized> Delimiter<'inp, L, Lang> for $name<S, C, Lang>
      where
        L: Lexer<'inp>,
        $open<S, C, Lang>: Punctuator<'inp, L, Lang>,
        $close<S, C, Lang>: Punctuator<'inp, L, Lang>,
      {
        type Open = $open<S, C, Lang>;

        type Close = $close<S, C, Lang>;

        #[inline(always)]
        fn name() -> CowStr {
          CowStr::from_static($description)
        }
      }
    )*
  };
}

impl_builtin_delimiter! {
  Paren { description: "()", open: OpenParen, close: CloseParen },
  Angle { description: "<>", open: OpenAngle, close: CloseAngle },
  Bracket { description: "[]", open: OpenBracket, close: CloseBracket },
  Brace { description: "{}", open: OpenBrace, close: CloseBrace },
}

macro_rules! impl_deref {
  (@impl<$ty:ty>) => {
    type Open = <$ty>::Open;
    type Close = <$ty>::Close;

    #[inline(always)]
    fn name() -> CowStr {
      <$ty>::name()
    }

    #[inline(always)]
    fn is_open(knd: &<<L>::Token as Token<'inp>>::Kind) -> bool
    where
      L: Lexer<'inp>,
    {
      <$ty>::is_open(knd)
    }

    #[inline(always)]
    fn is_close(knd: &<<L>::Token as Token<'inp>>::Kind) -> bool
    where
      L: Lexer<'inp>,
    {
      <$ty>::is_close(knd)
    }

    #[inline(always)]
    fn unexpected_open_token(
      tok: Spanned<L::Token, L::Span>,
    ) -> UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>
    where
      L: Lexer<'inp>,
    {
      <$ty>::unexpected_open_token(tok)
    }

    #[inline(always)]
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

impl<'inp, L, Lang: ?Sized, D: ?Sized> Delimiter<'inp, L, Lang> for &D
where
  L: Lexer<'inp>,
  D: Delimiter<'inp, L, Lang>,
{
  impl_deref!(@impl<D>);
}

impl<'inp, L, Lang: ?Sized, D: ?Sized> Delimiter<'inp, L, Lang> for &mut D
where
  L: Lexer<'inp>,
  D: Delimiter<'inp, L, Lang>,
{
  impl_deref!(@impl<D>);
}
