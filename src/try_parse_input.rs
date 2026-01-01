use crate::{lexer::{InputRef, PunctuatorToken}, parser::{Repeated, Separated}, punct::*};

use super::*;

macro_rules! define_separated_by {
  ($($name:ident),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Creates a `Separated` combinator which separates elements by the `" $name:snake "` separator and applies this parser repeatedly."]
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn [< separated_by_ $name:snake>](
          self,
        ) -> Separated<Self, $name, O, L, Ctx, Lang>
        where
          Self: Sized,
          L: Lexer<'inp>,
          L::Token: PunctuatorToken<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
        {
          Separated::new(self, <$name>::PHANTOM)
        }
      )*
    }
  };
}

/// Tentative parsing trait for optional token consumption with automatic backtracking.
///
/// Unlike [`ParseInput`] which must produce a value or error, `TryParseInput` allows parsers
/// to inspect the input and decide whether to consume it based on lookahead. If the parser
/// returns `Ok(None)`, **no valid tokens are consumed** - the input position only advances
/// past any error tokens that were consumed by the emitter.
pub trait TryParseInput<'inp, L, O, Ctx, Lang: ?Sized = ()> {
  /// Attempts to parse `O` from the input without committing.
  ///
  /// **IMPORTANT:**
  ///
  /// Implementations **must** uphold this contract:
  /// - ✅ `Ok(Some(value))` - Parser succeeded, tokens consumed, value produced
  /// - ✅ `Ok(None)` - Parser declined, **no valid tokens consumed** (error tokens may be consumed by emitter)
  /// - ✅ `Err(error)` - Parser encountered an error (may have consumed tokens)
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Option<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  /// Creates a `Repeated` combinator that applies this parser repeatedly,
  /// the returned parser will stop when this parser returns `Ok(None)` or an error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn repeated(self) -> Repeated<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Repeated::new(self)
  }

  /// Creates a `Separated` combinator which separates elements by the given separator parser
  /// and applies this parser for each element.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn separated<SepClassifier>(
    self,
    sep_classifier: SepClassifier,
  ) -> Separated<Self, SepClassifier, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    SepClassifier: Check<L::Token>,
  {
    Separated::new(self, sep_classifier)
  }

  define_separated_by!(
    Comma,
    Semicolon,
    Dot,
    Colon,
    Pipe,
    Ampersand,
    Hyphen,
    Underscore,
    DoubleColon,
    Arrow,
    FatArrow,
    Tilde,
    Slash,
    BackSlash,
    Percent,
    Dollar,
    Hash,
    At,
  );
}
