use crate::{Lexer, Token, error::token::UnexpectedToken, span::Spanned, utils::CowStr};

/// The carriage return newline (`\r\n`) punctuator.
pub type Crnl<S, C = (), Lang = ()> = CarriageReturnNewline<S, C, Lang>;

/// Defines the punctuators.
///
/// # Examples
/// ```rust
/// use tokit::punctuator;
///
/// punctuator! {
///   (LAngle, "L_ANGLE", "<"),
///   (RAngle, "R_ANGLE", ">"),
/// }
/// ```
#[macro_export]
macro_rules! punctuator {
  ($(
    $(#[$attr:meta])*
    ($name:ident, $syntax_tree_display: literal, $punct:literal)),+$(,)?
  ) => {
    paste::paste! {
      $(
        $(#[$attr])*
        #[doc = "The `" $punct "` punctuator"]
        #[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq, ::core::hash::Hash)]
        pub struct $name<S = (), C = (), Lang: ?::core::marker::Sized = ()> {
          span: S,
          source: C,
          _lang: ::core::marker::PhantomData<Lang>,
        }

        impl $name<()> {
          #[doc = "A unit instance of the `" $punct "` punctuator."]
          pub const UNIT: Self = {
            ::core::assert!(::core::mem::size_of::<Self>() == 0);
            ::core::assert!(::core::mem::align_of::<Self>() == 1);

            Self::new(())
          };

          #[doc = "Returns a unit instance of the `" $punct "` punctuator."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn unit() -> Self {
            Self::UNIT
          }
        }

        impl $name {
          #[doc = "Returns the raw string literal of the `" $punct "` punctuator."]
          #[inline]
          pub const fn raw() -> &'static ::core::primitive::str {
            $punct
          }
        }

        impl<S> $name<S> {
          /// Creates a new punctuator with the given span.
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn new(span: S) -> Self {
            Self { span, source: (), _lang: ::core::marker::PhantomData }
          }
        }

        impl<S, C> $name<S, C> {
          #[doc = "Creates a new `" $punct "` punctuator with the given span and content."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn with_content(span: S, content: C) -> Self {
            Self { span, source: content, _lang: ::core::marker::PhantomData }
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $name<S, C, Lang> {
          #[doc = "Changes the language type of the `" $punct "` punctuator."]
          #[inline]
          pub fn change_language<N: ?::core::marker::Sized>(self) -> $name<S, C, N> {
            $name { span: self.span, source: self.source, _lang: ::core::marker::PhantomData }
          }

          #[doc = "Changes the language type of the `" $punct "` punctuator."]
          #[inline]
          pub const fn change_language_const<N: ?::core::marker::Sized>(self) -> $name<S, C, N>
          where
            S: ::core::marker::Copy,
            C: ::core::marker::Copy,
          {
            $name { span: self.span, source: self.source, _lang: ::core::marker::PhantomData }
          }

          #[doc = "Returns the raw string literal of the `" $punct "` punctuator."]
          #[inline]
          pub const fn as_str(&self) -> &'static ::core::primitive::str {
            <$name>::raw()
          }

          #[doc = "Returns the span of the `" $punct "` punctuator."]
          #[inline]
          pub const fn span(&self) -> &S {
            &self.span
          }

          #[doc = "Returns a reference to the content of the `" $punct "` punctuator."]
          #[inline]
          pub const fn content(&self) -> &C {
            &self.source
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::cmp::PartialEq<::core::primitive::str> for $name<S, C, Lang> {
          #[inline]
          fn eq(&self, other: &::core::primitive::str) -> bool {
            self.as_str().eq(other)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::cmp::PartialOrd<::core::primitive::str> for $name<S, C, Lang> {
          #[inline]
          fn partial_cmp(&self, other: &::core::primitive::str) -> ::core::option::Option<::core::cmp::Ordering> {
            self.as_str().partial_cmp(other)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::cmp::PartialEq<$name<S, C, Lang>> for ::core::primitive::str {
          #[inline]
          fn eq(&self, other: &$name<S, C, Lang>) -> bool {
            self.eq(other.as_str())
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::cmp::PartialOrd<$name<S, C, Lang>> for ::core::primitive::str {
          #[inline]
          fn partial_cmp(&self, other: &$name<S, C, Lang>) -> ::core::option::Option<::core::cmp::Ordering> {
            self.partial_cmp(other.as_str())
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::borrow::Borrow<::core::primitive::str> for $name<S, C, Lang> {
          #[inline]
          fn borrow(&self) -> &::core::primitive::str {
            self.as_str()
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::convert::AsRef<::core::primitive::str> for $name<S, C, Lang> {
          #[inline]
          fn as_ref(&self) -> &::core::primitive::str {
            self.as_str()
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::span::AsSpan<S> for $name<S, C, Lang> {
          #[inline]
          fn as_span(&self) -> &S {
            self.span()
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::span::IntoSpan<S> for $name<S, C, Lang> {
          #[inline]
          fn into_span(self) -> S {
            self.span
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::IntoComponents for $name<S, C, Lang> {
          type Components = (S, C);

          #[inline]
          fn into_components(self) -> Self::Components {
            (self.span, self.source)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> ::core::fmt::Display for $name<S, C, Lang> {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt($punct, f)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::human_display::DisplayHuman for $name<S, C, Lang> {
          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::sdl_display::DisplayCompact for $name<S, C, Lang> {
          type Options = ();

          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>, _: &Self::Options) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::sdl_display::DisplayPretty for $name<S, C, Lang> {
          type Options = ();

          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>, _: &Self::Options) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }
      )*
    }
  };
}

punctuator! {
  // Delimiters
  #[doc(alias = "LessThan")]
  (OpenAngle, "OPEN_ANGLE", "<"),
  #[doc(alias = "GreaterThan")]
  (CloseAngle, "CLOSE_ANGLE", ">"),
  (Angle, "ANGLE", "<>"),
  (OpenBrace, "OPEN_BRACE", "{"),
  (CloseBrace, "CLOSE_BRACE", "}"),
  (OpenParen, "OPEN_PAREN", "("),
  (CloseParen, "CLOSE_PAREN", ")"),
  (Paren, "PAREN", "()"),
  (Brace, "BRACE", "{}"),
  (Bracket, "BRACKET", "[]"),
  (OpenBracket, "OPEN_BRACKET", "["),
  (CloseBracket, "CLOSE_BRACKET", "]"),

  // ASCII Punctuation
  (At, "AT", "@"),
  (Asterisk, "ASTERISK", "*"),
  #[doc(alias = "And")]
  (Ampersand, "AMPERSAND", "&"),
  (Apostrophe, "APOSTROPHE", "'"),
  (Backtick, "BACKTICK", "`"),
  (Backslash, "BACKSLASH", "\\"),
  #[doc(alias = "BitXor")]
  (Caret, "CARET", "^"),
  (Comma, "COMMA", ","),
  (Colon, "COLON", ":"),
  (Dot, "DOT", "."),
  (Dollar, "DOLLAR", "$"),
  (DoubleQuote, "DOUBLE_QUOTE", "\""),
  (Equal, "EQUAL", "="),
  #[doc(alias = "Bang")]
  (Exclamation, "EXCLAMATION", "!"),
  (Hash, "HASH", "#"),
  #[doc(alias = "Minus")]
  (Hyphen, "HYPHEN", "-"),
  (Pipe, "PIPE", "|"),
  (Plus, "PLUS", "+"),
  (Percent, "PERCENT", "%"),
  (Question, "QUESTION", "?"),
  (Slash, "SLASH", "/"),
  (Semicolon, "SEMICOLON", ";"),
  (Tilde, "TILDE", "~"),
  (Underscore, "UNDERSCORE", "_"),

  // Multi-character Punctuators
  #[doc(alias = "ThinArrow")]
  (Arrow, "ARROW", "->"),
  (FatArrow, "FAT_ARROW", "=>"),
  #[doc(alias = "PipeForward")]
  (PipeArrow, "PIPE_ARROW", "|>"),

  // Equal related
  #[doc(alias = "ColonAssign")]
  #[doc(alias = "ShortDeclaration")]
  #[doc(alias = "ColonEquals")]
  (ColonEqual, "COLON_EQUAL", ":="),
  (LogicalEqual, "LOGICAL_EQUAL", "=="),
  (LogicalNotEqual, "LOGICAL_NOT_EQUAL", "!="),
  (StrictEqual, "STRICT_EQUAL", "==="),
  (StrictNotEqual, "STRICT_NOT_EQUAL", "!=="),
  (LessThanOrEqual, "LESS_THAN_OR_EQUAL", "<="),
  (GreaterThanOrEqual, "GREATER_THAN_OR_EQUAL", ">="),
  (StrictLessThanOrEqual, "STRICT_LESS_THAN_OR_EQUAL", "<=="),
  (StrictGreaterThanOrEqual, "STRICT_GREATER_THAN_OR_EQUAL", ">=="),

  #[doc(alias = "AddAssign")]
  (PlusEqual, "PLUS_EQUAL", "+="),
  #[doc(alias = "SubAssign")]
  (HyphenEqual, "HYPHEN_EQUAL", "-="),
  #[doc(alias = "MulAssign")]
  (AsteriskEqual, "ASTERISK_EQUAL", "*="),
  #[doc(alias = "ExponentiationAssign")]
  (ExponentiationEqual, "EXPONENTIATION_EQUAL", "**="),
  #[doc(alias = "DivAssign")]
  (SlashEqual, "SLASH_EQUAL", "/="),
  (BackslashEqual, "BACKSLASH_EQUAL", "\\="),
  #[doc(alias = "RemAssign")]
  (PercentEqual, "PERCENT_EQUAL", "%="),
  #[doc(alias = "AndAssign")]
  (AmpersandEqual, "AMPERSAND_EQUAL", "&="),
  #[doc(alias = "OrAssign")]
  (PipeEqual, "PIPE_EQUAL", "|="),
  #[doc(alias = "XorAssign")]
  (CaretEqual, "CARET_EQUAL", "^="),
  #[doc(alias = "ShlAssign")]
  (ShlEqual, "SHL_EQUAL", "<<="),
  #[doc(alias = "ShrAssign")]
  (ShrEqual, "SHR_EQUAL", ">>="),
  #[doc(alias = "SarAssign")]
  (SarEqual, "SAR_EQUAL", ">>>="),


  (ShiftLeft, "SHIFT_LEFT", "<<"),
  (ShiftRight, "SHIFT_RIGHT", ">>"),
  (ShiftArithmeticRight, "SHIFT_ARITHMETIC_RIGHT", ">>>"),

  (Increment, "INCREMENT", "++"),
  (Decrement, "DECREMENT", "--"),
  (Exponentiation, "EXPONENTIATION", "**"),

  (LogicalAnd, "LOGICAL_AND", "&&"),
  (LogicalOr, "LOGICAL_OR", "||"),

  (DoubleColon, "DOUBLE_COLON", "::"),

  (Spread, "SPREAD", "..."),
  #[doc(alias = "NullishCoalescing")]
  (NullCoalesce, "NULL_COALESCE", "??"),
  (OptionalChain, "OPTIONAL_CHAIN", "?."),

  // Trivia
  (Tab, "TAB", "\t"),
  (Newline, "NEWLINE", "\n"),
  (CarriageReturn, "CARRIAGE_RETURN", "\r"),
  (CarriageReturnNewline, "CARRIAGE_RETURN_NEWLINE", "\r\n"),
  (Space, "SPACE", " "),
  (Trivia, "TRIVIA", "any trivia characters"),
}

/// A trait for any punctuator.
pub trait Punctuator<'inp, L, Lang: ?Sized = ()> {
  /// Returns the name of the punctuator.
  fn name() -> CowStr;

  /// Returns the description of the punctuator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn description() -> Option<CowStr> {
    None
  }

  /// Returns the kind of the punctuator.
  fn kind() -> <L::Token as Token<'inp>>::Kind
  where
    L: Lexer<'inp>;

  /// Evaluates whether the given token kind matches the punctuator's kind.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eval(knd: &<L::Token as Token<'inp>>::Kind) -> bool
  where
    L: Lexer<'inp>,
  {
    Self::kind().eq(knd)
  }

  /// Creates an `UnexpectedToken` error for the punctuator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unexpected_token(
    tok: Spanned<L::Token, L::Span>,
  ) -> UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>
  where
    L: Lexer<'inp>,
  {
    let (span, tok) = tok.into_components();
    UnexpectedToken::expected_one(span, Self::kind()).with_found(tok)
  }
}

macro_rules! impl_deref {
  (@impl<$ty:ty>) => {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn name() -> CowStr {
      <$ty>::name()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn description() -> Option<CowStr> {
      <$ty>::description()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn kind() -> <L::Token as Token<'inp>>::Kind
    where
      L: Lexer<'inp>,
    {
      <$ty>::kind()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn eval(knd: &<<L>::Token as Token<'inp>>::Kind) -> bool
    where
      L: Lexer<'inp>,
    {
      <$ty>::eval(knd)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn unexpected_token(
      tok: Spanned<<L>::Token, <L>::Span>,
    ) -> UnexpectedToken<'inp, <L>::Token, <<L>::Token as Token<'inp>>::Kind, <L>::Span, Lang>
    where
      L: Lexer<'inp>,
    {
      <$ty>::unexpected_token(tok)
    }
  };
}

impl<'inp, L, Lang: ?Sized, P> Punctuator<'inp, L, Lang> for &P
where
  L: Lexer<'inp>,
  P: Punctuator<'inp, L, Lang>,
{
  impl_deref!(@impl<P>);
}

impl<'inp, L, Lang: ?Sized, P> Punctuator<'inp, L, Lang> for &mut P
where
  L: Lexer<'inp>,
  P: Punctuator<'inp, L, Lang>,
{
  impl_deref!(@impl<P>);
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;
  use core::borrow::Borrow;
  use std::format;

  #[test]
  fn comma_unit_is_zero_sized() {
    assert_eq!(core::mem::size_of::<Comma>(), 0);
  }

  #[test]
  fn comma_unit_returns_unit() {
    let c = Comma::unit();
    assert_eq!(c.as_str(), ",");
  }

  #[test]
  fn comma_raw_returns_literal() {
    assert_eq!(Comma::raw(), ",");
  }

  #[test]
  fn comma_new_with_span() {
    let c = Comma::<usize>::new(42);
    assert_eq!(*c.span(), 42);
    assert_eq!(c.as_str(), ",");
  }

  #[test]
  fn comma_with_content() {
    let c = Comma::<usize, &str>::with_content(10, "hello");
    assert_eq!(*c.span(), 10);
    assert_eq!(*c.content(), "hello");
  }

  #[test]
  fn punctuator_display() {
    let c = Comma::unit();
    assert_eq!(format!("{}", c), ",");

    let s = Semicolon::unit();
    assert_eq!(format!("{}", s), ";");

    let d = Dot::unit();
    assert_eq!(format!("{}", d), ".");
  }

  #[test]
  fn punctuator_debug() {
    let c = Comma::unit();
    let dbg = format!("{:?}", c);
    assert!(dbg.contains("Comma"));
  }

  #[test]
  fn punctuator_partial_eq_str() {
    let c = Comma::unit();
    assert!(c == *",");
    assert!(!(c == *";"));
  }

  #[test]
  fn str_partial_eq_punctuator() {
    let c = Comma::unit();
    assert!(*"," == c);
    assert!((*";" != c));
  }

  #[test]
  fn punctuator_partial_ord_str() {
    let c = Comma::unit();
    let ord = c.partial_cmp(",");
    assert_eq!(ord, Some(core::cmp::Ordering::Equal));
  }

  #[test]
  fn str_partial_ord_punctuator() {
    let c = Comma::unit();
    let ord = ",".partial_cmp(&c);
    assert_eq!(ord, Some(core::cmp::Ordering::Equal));
  }

  #[test]
  fn punctuator_borrow_str() {
    let c = Comma::unit();
    let s: &str = c.borrow();
    assert_eq!(s, ",");
  }

  #[test]
  fn punctuator_as_ref_str() {
    let c = Comma::unit();
    let s: &str = c.as_ref();
    assert_eq!(s, ",");
  }

  #[test]
  fn punctuator_clone_copy() {
    let c = Comma::unit();
    let c2 = c;
    let c3 = c;
    assert_eq!(c2.as_str(), c3.as_str());
  }

  #[test]
  fn punctuator_eq_hash() {
    let c1 = Comma::unit();
    let c2 = Comma::unit();
    assert_eq!(c1, c2);
  }

  #[test]
  fn change_language() {
    struct LangA;
    struct LangB;
    let c: Comma<(), (), LangA> = Comma {
      span: (),
      source: (),
      _lang: core::marker::PhantomData,
    };
    let c2: Comma<(), (), LangB> = c.change_language();
    assert_eq!(c2.as_str(), ",");
  }

  #[test]
  fn change_language_const() {
    struct LangA;
    struct LangB;
    let c: Comma<(), (), LangA> = Comma {
      span: (),
      source: (),
      _lang: core::marker::PhantomData,
    };
    let c2: Comma<(), (), LangB> = c.change_language_const();
    assert_eq!(c2.as_str(), ",");
  }

  #[test]
  fn into_components() {
    use crate::utils::IntoComponents;
    let c = Comma::<usize, &str>::with_content(42, "test");
    let (span, content) = c.into_components();
    assert_eq!(span, 42);
    assert_eq!(content, "test");
  }

  #[test]
  fn into_span() {
    use crate::span::IntoSpan;
    let c = Comma::<usize>::new(99);
    let span = c.into_span();
    assert_eq!(span, 99);
  }

  #[test]
  fn as_span() {
    use crate::span::AsSpan;
    let c = Comma::<usize>::new(77);
    assert_eq!(*c.as_span(), 77);
  }

  #[test]
  fn various_punctuators_raw() {
    assert_eq!(OpenAngle::raw(), "<");
    assert_eq!(CloseAngle::raw(), ">");
    assert_eq!(OpenBrace::raw(), "{");
    assert_eq!(CloseBrace::raw(), "}");
    assert_eq!(OpenParen::raw(), "(");
    assert_eq!(CloseParen::raw(), ")");
    assert_eq!(OpenBracket::raw(), "[");
    assert_eq!(CloseBracket::raw(), "]");
    assert_eq!(At::raw(), "@");
    assert_eq!(Asterisk::raw(), "*");
    assert_eq!(Ampersand::raw(), "&");
    assert_eq!(Arrow::raw(), "->");
    assert_eq!(FatArrow::raw(), "=>");
    assert_eq!(Spread::raw(), "...");
    assert_eq!(DoubleColon::raw(), "::");
    assert_eq!(LogicalEqual::raw(), "==");
    assert_eq!(LogicalNotEqual::raw(), "!=");
    assert_eq!(Increment::raw(), "++");
    assert_eq!(Decrement::raw(), "--");
    assert_eq!(Exponentiation::raw(), "**");
    assert_eq!(LogicalAnd::raw(), "&&");
    assert_eq!(LogicalOr::raw(), "||");
    assert_eq!(NullCoalesce::raw(), "??");
    assert_eq!(OptionalChain::raw(), "?.");
    assert_eq!(Tab::raw(), "\t");
    assert_eq!(Newline::raw(), "\n");
    assert_eq!(Space::raw(), " ");
    assert_eq!(CarriageReturn::raw(), "\r");
    assert_eq!(CarriageReturnNewline::raw(), "\r\n");
  }

  #[test]
  fn crnl_type_alias() {
    // Crnl is an alias for CarriageReturnNewline
    let c = Crnl::unit();
    assert_eq!(c.as_str(), "\r\n");
  }

  #[test]
  fn punctuator_display_special_chars() {
    assert_eq!(format!("{}", Tab::unit()), "\t");
    assert_eq!(format!("{}", Newline::unit()), "\n");
    assert_eq!(format!("{}", Space::unit()), " ");
  }
}
