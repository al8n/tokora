use super::{Leading, MissingToken};
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
        pub type [< MissingLeading $name >] <'inp, Sep, L> = MissingLeadingOf<'inp, Sep, L>;

        impl<Kind, O> MissingToken<'_, Kind, O, Leading<$name>> {
          #[doc = "Create a new `MissingToken` error indicating a leading `" $name "` was missing."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< leading_ $name:snake >](
            offset: O,
          ) -> Self {
            Self::leading(offset)
          }
        }

        impl<Kind, O> ::core::fmt::Debug for MissingToken<'_, Kind, O, Leading<$name>>
        where
          O: ::core::fmt::Debug,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_struct(stringify!([< MissingLeading $name >]))
              .field("offset", &self.offset)
              .finish()
          }
        }

        impl<Kind, O> ::core::fmt::Display for MissingToken<'_, Kind, O, Leading<$name>>
        where
          O: ::core::fmt::Display,
        {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            write!(
              f,
              "missing leading {} token at {}",
              stringify!([< $name: snake >]),
              self.offset
            )
          }
        }

        impl<Kind, O> ::core::error::Error for MissingToken<'_, Kind, O, Leading<$name>>
        where
          O: ::core::fmt::Display + ::core::fmt::Debug,
        {
        }
      )*
    }
  };
}

alias! {
  /// A type alias for an `MissingToken` error indicating a leading comma was missing.
  Comma,
  /// A type alias for an `MissingToken` error indicating a leading dot was missing.
  Dot,
  /// A type alias for an `MissingToken` error indicating a leading underscore was missing.
  Underscore,
  /// A type alias for an `MissingToken` error indicating a leading pipe was missing.
  Pipe,
  /// A type alias for an `MissingToken` error indicating a leading ampersand was missing.
  Ampersand,
  /// A type alias for an `MissingToken` error indicating a leading hyphen was missing.
  Hyphen,
  /// A type alias for an `MissingToken` error indicating a leading double colon was missing.
  DoubleColon,
}

/// A type alias for an `MissingPrefix` error indicating a leading punctuator was missing for a given lexer and separator.
pub type MissingLeadingOf<'inp, Sep, L> = MissingToken<
  'inp,
  <<L as Lexer<'inp>>::Token as Token<'inp>>::Kind,
  <L as Lexer<'inp>>::Offset,
  Leading<Sep>,
>;
