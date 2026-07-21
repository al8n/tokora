use crate::{Lexer, Token, error::token::UnexpectedToken, punct::*, span::Spanned, utils::CowStr};

/// A trait for any delimiter consisting of an opening and a closing punctuator.
pub trait Delimiter<'inp, L, Lang: ?Sized = ()> {
  /// The stable type-level identity carried by delimiter diagnostics.
  ///
  /// Delimited parser adapters may reborrow their marker while they drive an inner parser.
  /// `ErrorTag` preserves the original delimiter identity across those `&D`/`&mut D`
  /// wrappers, so an unclosed built-in bracket always reports `Unclosed<Bracket, …>`
  /// rather than a reference-shaped tag.
  type ErrorTag;

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

/// A [`Delimiter`] whose punctuators can be materialized as typed, span-carrying values —
/// the capability the [`delimited`](crate::parser::delimited) shape parser uses to build
/// its [`Delimited`](crate::utils::Delimited) result.
///
/// The base [`Delimiter`] trait is a classifier and error helper; this additive subtrait
/// adds the *consumption* side, turning a committed opener's or closer's span into the
/// span-carrying punctuator value the shape parsers store. It is implemented for the
/// built-in pairs [`Paren`], [`Brace`], [`Bracket`], and [`Angle`].
///
/// # Custom delimiter pairs
///
/// Any user pair works the same way: implement [`Punctuator`] for the two punctuator types
/// (or define them with [`punctuator!`](crate::punctuator)), then implement [`Delimiter`]
/// and `TypedDelimiter` for the pair. It then drops straight into
/// [`delimited::<MyPair, …>`](crate::parser::delimited) — see that function's example.
pub trait TypedDelimiter<'inp, L, Lang: ?Sized = ()>: Delimiter<'inp, L, Lang>
where
  L: Lexer<'inp>,
{
  /// The span-carrying opening punctuator value this delimiter materializes.
  type OpenValue;
  /// The span-carrying closing punctuator value this delimiter materializes.
  type CloseValue;

  /// Materializes the opening punctuator value from its committed token's span.
  fn open_value(span: L::Span) -> Self::OpenValue;

  /// Materializes the closing punctuator value from its committed token's span.
  fn close_value(span: L::Span) -> Self::CloseValue;
}

macro_rules! impl_builtin_delimiter {
  ($($name:ident { description: $description:literal, open: $open:ident, close: $close:ident $(,)? }),+$(,)?) => {
    $(
      impl<'inp, S, C, MarkerLang: ?Sized, L, Lang: ?Sized> Delimiter<'inp, L, Lang>
        for $name<S, C, MarkerLang>
      where
        L: Lexer<'inp>,
        $open<S, C, Lang>: Punctuator<'inp, L, Lang>,
        $close<S, C, Lang>: Punctuator<'inp, L, Lang>,
      {
        type ErrorTag = $name;

        type Open = $open<S, C, Lang>;

        type Close = $close<S, C, Lang>;

        #[inline(always)]
        fn name() -> CowStr {
          CowStr::from_static($description)
        }
      }

      impl<'inp, S, C, MarkerLang: ?Sized, L, Lang: ?Sized> TypedDelimiter<'inp, L, Lang>
        for $name<S, C, MarkerLang>
      where
        L: Lexer<'inp>,
        $open<S, C, Lang>: Punctuator<'inp, L, Lang>,
        $close<S, C, Lang>: Punctuator<'inp, L, Lang>,
      {
        type OpenValue = $open<L::Span, (), Lang>;

        type CloseValue = $close<L::Span, (), Lang>;

        #[inline(always)]
        fn open_value(span: L::Span) -> Self::OpenValue {
          $open::new(span).change_language()
        }

        #[inline(always)]
        fn close_value(span: L::Span) -> Self::CloseValue {
          $close::new(span).change_language()
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
    type ErrorTag = <$ty>::ErrorTag;
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
