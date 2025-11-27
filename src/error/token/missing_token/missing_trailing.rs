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
        pub type [< MissingTrailing $name >] <'inp, Sep, L> = MissingTrailingOf<'inp, Sep, L>;

        impl<Kind, O> MissingToken<'_, Kind, O, Trailing<$name>> {
          #[doc = "Create a new `MissingToken` error indicating a trailing `" $name "` was found."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< trailing_ $name:snake >](
            offset: O,
          ) -> Self {
            Self::trailing(offset)
          }
        }

        impl<Kind, O> ::core::fmt::Debug for MissingToken<'_, Kind, O, Trailing<$name>>
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

        impl<Kind, O> ::core::fmt::Display for MissingToken<'_, Kind, O, Trailing<$name>>
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

        impl<Kind, O> ::core::error::Error for MissingToken<'_, Kind, O, Trailing<$name>>
        where
          O: ::core::fmt::Display + ::core::fmt::Debug,
        {
        }
      )*
    }
  };
}

alias! {
  /// A type alias for an `MissingToken` error indicating a trailing comma was found.
  Comma,
  /// A type alias for an `MissingToken` error indicating a trailing dot was found.
  Dot,
  /// A type alias for an `MissingToken` error indicating a trailing underscore was found.
  Underscore,
  /// A type alias for an `MissingToken` error indicating a trailing pipe was found.
  Pipe,
  /// A type alias for an `MissingToken` error indicating a trailing ampersand was found.
  Ampersand,
  /// A type alias for an `MissingToken` error indicating a trailing hyphen was found.
  Hyphen,
  /// A type alias for an `MissingToken` error indicating a trailing double colon was found.
  DoubleColon,
}

/// A type alias for an `MissingPrefix` error indicating a leading punctuator was found for a given lexer and separator.
pub type MissingTrailingOf<'inp, Sep, L> = MissingToken<
  'inp,
  <<L as Lexer<'inp>>::Token as Token<'inp>>::Kind,
  <L as Lexer<'inp>>::Offset,
  Trailing<Sep>,
>;
