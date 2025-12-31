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
          #[doc = "A phantom instance of the `" $punct "` punctuator."]
          pub const PHANTOM: Self = {
            ::core::assert!(::core::mem::size_of::<Self>() == 0);
            ::core::assert!(::core::mem::align_of::<Self>() == 1);

            Self::new(())
          };
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

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::AsSpan<S> for $name<S, C, Lang> {
          #[inline]
          fn as_span(&self) -> &S {
            self.span()
          }
        }

        impl<S, C, Lang: ?::core::marker::Sized> $crate::__private::utils::IntoSpan<S> for $name<S, C, Lang> {
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
  (OpenAngle, "OPEN_ANGLE", "<"),
  (CloseAngle, "CLOSE_ANGLE", ">"),
  (Angle, "ANGLE", "<>"),
  (OpenBrace, "OPEN_BRACE", "{"),
  (CloseBrace, "CLOSE_BRACE", "}"),
  (Brace, "BRACE", "{}"),
  (OpenParen, "OPEN_PAREN", "("),
  (CloseParen, "CLOSE_PAREN", ")"),
  (Paren, "PAREN", "()"),
  (OpenBracket, "OPEN_BRACKET", "["),
  (CloseBracket, "CLOSE_BRACKET", "]"),
  (Bracket, "BRACKET", "[]"),
  (Comma, "COMMA", ","),
  (Semicolon, "SEMICOLON", ";"),
  (Colon, "COLON", ":"),
  (Dot, "DOT", "."),
  (Tilde, "TILDE", "~"),
  (Underscore, "UNDERSCORE", "_"),
  (Equal, "EQUAL", "="),
  (Hyphen, "HYPHEN", "-"),
  #[doc(alias = "ThinArrow")]
  (Arrow, "ARROW", "->"),
  (FatArrow, "FAT_ARROW", "=>"),
  (ColonEq, "COLON_EQ", ":="),
  (DoubleColon, "DOUBLE_COLON", "::"),
  (Tab, "TAB", "\t"),
  (Newline, "NEWLINE", "\n"),
  (CarriageReturn, "CARRIAGE_RETURN", "\r"),
  (CarriageReturnNewline, "CARRIAGE_RETURN_NEWLINE", "\r\n"),
  (Space, "SPACE", " "),
  (Pipe, "PIPE", "|"),
  (Ampersand, "AMPERSAND", "&"),
  (Percent, "PERCENT", "%"),
  (Slash, "SLASH", "/"),
  (BackSlash, "BACKSLASH", "\\"),
  (Dollar, "DOLLAR", "$"),
  (Hash, "HASH", "#"),
  (At, "AT", "@"),
  (Asterisk, "ASTERISK", "*"),
  (Apostrophe, "APOSTROPHE", "'"),
  (DoubleQuote, "DOUBLE_QUOTE", "\""),
  (Plus, "PLUS", "+"),
  (Exclamation, "EXCLAMATION", "!"),
  (Question, "QUESTION", "?"),
  (Backtick, "BACKTICK", "`"),
  (Trivia, "TRIVIA", "any trivia characters"),
  (Caret, "CARET", "^"),
}
