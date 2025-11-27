use super::{Leading, UnexpectedToken};
use crate::{Lexer, Token, punct::*};

macro_rules! alias {
  (
    $(
      $(#[$attr:meta])*
      $name:ident
    ), +$(,)?
  ) => {
    paste::paste! {
      $(
        $(#[$attr])*
        pub type [< UnexpectedLeading $name >] <'inp, Sep, L> = UnexpectedLeadingOf<'inp, Sep, L>;

        impl<T, Kind, S> UnexpectedToken<'_, T, Kind, S, Leading<$name>> {
          #[doc = "Create a new `UnexpectedToken` error indicating a leading `" $name "` was found."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< leading_ $name:snake >](
            span: S,
            token: T,
          ) -> Self {
            Self::leading(span, token)
          }
        }

        impl<T, Kind, S> ::core::fmt::Debug for UnexpectedToken<'_, T, Kind, S, Leading<$name>>
        where
          S: ::core::fmt::Debug,
          T: ::core::fmt::Debug,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_struct(stringify!([< UnexpectedLeading $name >]))
              .field("span", &self.span)
              .field("found", &self.found)
              .finish()
          }
        }

        impl<T, Kind, S> ::core::fmt::Display for UnexpectedToken<'_, T, Kind, S, Leading<$name>>
        where
          S: ::core::fmt::Display,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            write!(
              f,
              "unexpected leading {} token at {}",
              stringify!([< $name: snake >]),
              self.span
            )
          }
        }

        impl<T, Kind, S> ::core::error::Error for UnexpectedToken<'_, T, Kind, S, Leading<$name>>
        where
          S: ::core::fmt::Display + ::core::fmt::Debug,
          T: ::core::fmt::Debug,
        {
        }
      )*
    }
  };
}

alias! {
  /// A type alias for an `UnexpectedToken` error indicating a leading comma was found.
  Comma,
  /// A type alias for an `UnexpectedToken` error indicating a leading dot was found.
  Dot,
  /// A type alias for an `UnexpectedToken` error indicating a leading underscore was found.
  Underscore,
  /// A type alias for an `UnexpectedToken` error indicating a leading pipe was found.
  Pipe,
  /// A type alias for an `UnexpectedToken` error indicating a leading ampersand was found.
  Ampersand,
  /// A type alias for an `UnexpectedToken` error indicating a leading hyphen was found.
  Hyphen,
  /// A type alias for an `UnexpectedToken` error indicating a leading double colon was found.
  DoubleColon,
}

/// A type alias for an `UnexpectedPrefix` error indicating a leading punctuator was found for a given lexer and separator.
pub type UnexpectedLeadingOf<'inp, Sep, L> = UnexpectedToken<
  'inp,
  <L as Lexer<'inp>>::Token,
  <<L as Lexer<'inp>>::Token as Token<'inp>>::Kind,
  <L as Lexer<'inp>>::Span,
  Leading<Sep>,
>;
