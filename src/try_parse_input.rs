use crate::{
  lexer::{InputRef, PunctuatorToken},
  parser::{Repeated, Separated},
  punct::*,
};

use super::*;

macro_rules! define_separated_by {
  ($($name:ident),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Creates a `Separated` combinator which separates elements by the `" $name:snake "` separator and applies this parser repeatedly."]
        ///
        /// See [`separated`](crate::TryParseInput::separated) for details.
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

  /// Creates a `Repeated` combinator that applies this parser repeatedly until it signals completion.
  ///
  /// The parser will be called repeatedly until:
  /// - It returns `Ok(None)` - parser peeked ahead, didn't match (no tokens consumed)
  /// - It returns `Err(e)` - parse error
  ///
  /// ## Key Behavior
  ///
  /// Since this parser implements [`TryParseInput`], when it returns `Ok(None)`:
  /// - The parser **peeked ahead** and saw tokens it doesn't match
  /// - **No tokens were consumed** - input position unchanged
  /// - Repetition **stops cleanly**
  ///
  /// ## When to Use This
  ///
  /// Use `.repeated()` when:
  /// - You want to parse zero or more occurrences
  /// - The parser can look ahead and decide if it matches
  /// - You want automatic stopping based on parser's lookahead
  ///
  /// ## See Also
  ///
  /// - [`repeated_while`](crate::ParseInput::repeated_while) - When you want to provide explicit stopping condition
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn repeated(self) -> Repeated<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Repeated::new(self)
  }

  /// Creates a `Separated` combinator that parses separated elements, where **this parser
  /// handles the lookahead**.
  ///
  /// This parser (the element parser) will be called repeatedly to parse elements separated
  /// by the given separator. Since this parser implements [`TryParseInput`], it can peek ahead
  /// and return `Ok(None)` when it doesn't match, **without consuming any tokens**.
  ///
  /// ## Key Behavior
  ///
  /// The combinator stops when this element parser returns `Ok(None)`:
  /// - Parser **peeked ahead** and saw tokens it doesn't match
  /// - **No tokens consumed** - separator or closing delimiter left in input
  /// - Parsing **stops cleanly**
  ///
  /// ## When to Use This
  ///
  /// Use `.separated()` when:
  /// - Your element parser has built-in lookahead (implements `TryParseInput`)
  /// - You want the element parser to decide when to stop
  /// - The parser returns `Ok(None)` for non-matching tokens
  ///
  /// ## See Also
  ///
  /// - [`separated_while`](crate::ParseInput::separated_while) - When you want to provide the lookahead/stopping logic externally
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
