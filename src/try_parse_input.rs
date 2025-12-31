use crate::lexer::InputRef;

use super::*;

/// Tentative parsing trait for optional token consumption with automatic backtracking.
///
/// Unlike [`ParseInput`] which must produce a value or error, `TryParseInput` allows parsers
/// to inspect the input and decide whether to consume it based on lookahead. If the parser
/// returns `Ok(None)`, **no valid tokens are consumed** - the input position only advances
/// past any error tokens that were consumed by the emitter.
/// 
/// **IMPORTANT:**
///
/// Implementations **must** uphold this contract:
/// - ✅ `Ok(Some(value))` - Parser succeeded, tokens consumed, value produced
/// - ✅ `Ok(None)` - Parser declined, **no valid tokens consumed** (error tokens may be consumed by emitter)
/// - ✅ `Err(error)` - Parser encountered an error (may have consumed tokens)
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
}

