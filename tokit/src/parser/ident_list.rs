use crate::{
  Accumulator, Emitter, InputRef, Lexer, ParseContext, ParseInput, Source, TryParseInput,
  emitter::{
    SeparatedEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  error::UnexpectedEot,
  parser::SeparatorHandler,
  punct::Punctuator,
  span::Spanned,
  token::IdentifierToken,
  try_parse_input::Accept,
  types::{Ident, IdentList},
};

/// Returns a parser for the a list of identifiers separated by the given separator.
///
/// The parser will not consume any valid token if it is not a valid ident list.
pub fn try_ident_list<'inp, Sep, L, Container, Ctx>() -> impl TryParseInput<
  'inp,
  L,
  IdentList<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Container>,
  Ctx,
> + 'inp
where
  L: Lexer<'inp>,
  L::Source: Source<L::Offset>,
  L::Token: IdentifierToken<'inp>,
  Sep: Punctuator<'inp, L>,
  Ctx: ParseContext<'inp, L>,
  Ctx::Emitter: SeparatedEmitter<'inp, L>
    + UnexpectedLeadingSeparatorEmitter<'inp, L>
    + UnexpectedTrailingSeparatorEmitter<'inp, L>,
  <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
  Container: Default
    + crate::container::Container<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>>
    + SeparatorHandler<'inp, L>
    + 'inp,
{
  try_ident_list_of::<Sep, _, _, _, _>()
}

/// Returns a parser for the a list of identifiers separated by the given separator for the specified language.
///
/// The parser will not consume any valid token if it is not a valid ident list.
pub fn try_ident_list_of<'inp, Sep, L, Container, Ctx, Lang>() -> impl TryParseInput<
  'inp,
  L,
  IdentList<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Container, Lang>,
  Ctx,
  Lang,
> + 'inp
where
  L: Lexer<'inp>,
  L::Source: Source<L::Offset>,
  L::Token: IdentifierToken<'inp>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default
    + crate::container::Container<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>>
    + SeparatorHandler<'inp, L>
    + 'inp,
{
  move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
    Ident::try_parse_of
      .separated::<Sep>()
      .collect()
      .spanned()
      .parse_input(inp)
      .map(|seg: Spanned<Container, _>| {
        let (span, container) = seg.into_components();
        Accept(IdentList::new(span, container))
      })
  }
}
