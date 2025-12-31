use crate::lexer::InputRef;

use super::*;

/// Parser will not consume any valid token if failed to parse the value.
///
/// Currently, this trait is sealed and cannot be implemented outside of this crate.
pub trait ParseOptional<'inp, L, O, Ctx, Lang: ?Sized = ()>: sealed::Sealed {
  /// Attempts to parse a `O` from the input.
  ///
  /// If the function returns `Ok(None)`, it means the next token does not match,
  /// and promises no valid token is consumed.
  fn parse_optional(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Option<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

pub(crate) mod sealed {
  pub trait Sealed {}
}
