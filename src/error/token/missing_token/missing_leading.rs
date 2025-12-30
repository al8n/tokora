use super::{Leading, MissingToken, Ownable};
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
        pub type [< MissingLeading $name >] <'inp, L, Lang = ()> = MissingLeadingOf<'inp, $name, L, Lang>;

        impl<Kind: Ownable, O> MissingToken<'_, Kind, O, Leading<$name>> {
          #[doc = "Create a new `MissingToken` error indicating a leading `" $name "` was missing for a specific language."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< leading_ $name:snake>](
            offset: O,
          ) -> Self {
            Self::[< leading_ $name:snake _of>](offset)
          }
        }

        impl<Kind: Ownable, O, Lang: ?Sized> MissingToken<'_, Kind, O, Leading<$name, Lang>> {
          #[doc = "Create a new `MissingToken` error indicating a leading `" $name "` was missing for a specific language."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< leading_ $name:snake _of>](
            offset: O,
          ) -> Self {
            Self::leading_of(offset)
          }
        }

        impl<Kind: Ownable, O, Lang: ?Sized> ::core::fmt::Debug for MissingToken<'_, Kind, O, Leading<$name, Lang>>
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

        impl<Kind: Ownable, O, Lang: ?Sized> ::core::fmt::Display for MissingToken<'_, Kind, O, Leading<$name, Lang>>
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

        impl<Kind: Ownable, O, Lang: ?Sized> ::core::error::Error for MissingToken<'_, Kind, O, Leading<$name, Lang>>
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
pub type MissingLeadingOf<'inp, Sep, L, Lang = ()> = MissingToken<
  'inp,
  <<L as Lexer<'inp>>::Token as Token<'inp>>::Kind,
  <L as Lexer<'inp>>::Offset,
  Leading<Sep, Lang>,
>;
