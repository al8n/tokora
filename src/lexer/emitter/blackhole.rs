// use crate::utils::Spanned;

// use super::{super::BlackHole, Emitter, Token, Lexer};

// impl<'a, L> Emitter<'a, L> for BlackHole
// where
//   L: Lexer<'a>,
// {
//   type Error = core::convert::Infallible;

//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn emit_token_error(
//     &mut self,
//     _err: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
//   ) -> Result<(), Spanned<Self::Error, L::Span>> {
//     Ok(())
//   }

//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn emit_error(&mut self, _err: Spanned<Self::Error, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>> {
//     Ok(())
//   }
// }
