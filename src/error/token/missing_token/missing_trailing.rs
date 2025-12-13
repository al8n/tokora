use super::{MissingToken, Trailing};
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
        pub type [< MissingTrailing $name >] <'inp, L, Lang = ()> = MissingTrailingOf<'inp, $name, L, Lang>;

        impl<Kind, O> MissingToken<'_, Kind, O, Trailing<$name>> {
          #[doc = "Create a new `MissingToken` error indicating a trailing `" $name "` was missing for a specific language."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< trailing_ $name:snake>](
            offset: O,
          ) -> Self {
            Self::[< trailing_ $name:snake _of>](offset)
          }
        }

        impl<Kind, O, Lang: ?Sized> MissingToken<'_, Kind, O, Trailing<$name, Lang>> {
          #[doc = "Create a new `MissingToken` error indicating a trailing `" $name "` was missing for a specific language."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< trailing_ $name:snake _of>](
            offset: O,
          ) -> Self {
            Self::trailing_of(offset)
          }
        }

        impl<Kind, O, Lang: ?Sized> ::core::fmt::Debug for MissingToken<'_, Kind, O, Trailing<$name, Lang>>
        where
          O: ::core::fmt::Debug,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_struct(stringify!([< MissingTrailing $name >]))
              .field("offset", &self.offset)
              .finish()
          }
        }

        impl<Kind, O, Lang: ?Sized> ::core::fmt::Display for MissingToken<'_, Kind, O, Trailing<$name, Lang>>
        where
          O: ::core::fmt::Display,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            write!(
              f,
              "missing trailing {} token at {}",
              stringify!([< $name: snake >]),
              self.offset
            )
          }
        }

        impl<Kind, O, Lang: ?Sized> ::core::error::Error for MissingToken<'_, Kind, O, Trailing<$name, Lang>>
        where
          O: ::core::fmt::Display + ::core::fmt::Debug,
        {
        }
      )*
    }
  };
}

alias! {
  /// A type alias for an `MissingToken` error indicating a trailing comma was missing.
  Comma,
  /// A type alias for an `MissingToken` error indicating a trailing dot was missing.
  Dot,
  /// A type alias for an `MissingToken` error indicating a trailing underscore was missing.
  Underscore,
  /// A type alias for an `MissingToken` error indicating a trailing pipe was missing.
  Pipe,
  /// A type alias for an `MissingToken` error indicating a trailing ampersand was missing.
  Ampersand,
  /// A type alias for an `MissingToken` error indicating a trailing hyphen was missing.
  Hyphen,
  /// A type alias for an `MissingToken` error indicating a trailing double colon was missing.
  DoubleColon,
}

/// A type alias for an `MissingPrefix` error indicating a trailing punctuator was missing for a given lexer and separator.
pub type MissingTrailingOf<'inp, Sep, L, Lang = ()> = MissingToken<
  'inp,
  <<L as Lexer<'inp>>::Token as Token<'inp>>::Kind,
  <L as Lexer<'inp>>::Offset,
  Trailing<Sep, Lang>,
>;
