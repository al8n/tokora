use crate::{Cache, DefaultCache, Emitter, Lexer, Noop, Token, lexer::{Input, InputRef}};

/// A trait for parsers
pub trait Parser<'source, T, L, O, E, C = DefaultCache<'source, T, L>> {
  /// Parses input and produces an output or an error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse(&mut self, input: &'source L::Source, emitter: &mut E) -> Result<O, E::Error>
  where
    L: Lexer<'source, T>,
    L::State: Default,
    T: Token<'source>,
    C: Cache<'source, T, L>,
    E: Emitter<'source, T, L::Span>
  {
    self.parse_with_state(input, Default::default(), emitter)
  }

  /// Parses input and produces an output or an error.
  fn parse_with_state(&mut self, input: &'source L::Source, state: L::State, emitter: &mut E) -> Result<O, E::Error>
  where
    L: Lexer<'source, T>,
    T: Token<'source>,
    C: Cache<'source, T, L>,
    E: Emitter<'source, T, L::Span>;
}

impl<'source, T, L, O, E, C, F> Parser<'source, T, L, O, E, C> for F
where
  F: FnMut(InputRef<'source, '_, T, L, E, C>) -> Result<O, E::Error>,
  T: Token<'source>,
  E: Emitter<'source, T, L::Span>,
  L: Lexer<'source, T>,
{
  fn parse_with_state(
    &mut self,
    input: &'source L::Source,
    state: L::State,
    emitter: &mut E,
  ) -> Result<O, E::Error>
  where
    L: Lexer<'source, T>,
    T: Token<'source>,
    C: Cache<'source, T, L>,
  {
    let mut inp = Input::with_state_and_cache(input, state, C::new());
    let inp_ref = inp.as_ref(emitter);

    (self)(inp_ref)
  }
}

/// a
pub fn any<'source, T, L, C, Error>(_inp: InputRef<'source, '_, T, L, Noop<Error>, C>) -> Result<T, Error>
where
  T: Token<'source>,
  L: Lexer<'source, T>,
  C: Cache<'source, T, L>,
{
  // match inp.next() {
  //   Some(Ok(spanned)) => Ok(spanned.into_data()),
  //   Some(Err(err)) => Err(err.into_data()),
  //   None => Err(inp.emit_error(Spanned::new(
  //     inp.span_at_cursor(),
  //     E::Error::EndOfInput,
  //   ))?),
  // }
  todo!()
}

#[test]
fn t() {
  any.parse("", emitter);
}

// impl<'source, T, L, O, Error, F, C> Parser<'source, T, L, O, Error, C> for F
// where
//   F: FnMut(InputRef<'source, '_, T, L, C, Noop<Error>>) -> Result<O, Spanned<Error, L::Span>>,
//   T: Token<'source>,
//   Error: From<T::Error>,
//   L: Lexer<'source, T>,
// {
//   fn parse(
//     &mut self,
//     input: &'source L::Source,
//   ) -> Result<O, Spanned<Error, L::Span>>
//   where
//     L: Lexer<'source, T>,
//     L::State: Default,
//     T: Token<'source>,
//     C: Cache<'source, T, L>,
//   {
//     let mut inp = Input::with_state_and_cache(input, Default::default(), C::new());
//     let mut emitter = Noop::default();
//     let inp_ref = inp.as_ref(&mut emitter);

//     match (self)(inp_ref) {
//       Ok(output) => Ok(output),
//       Err(err) => match <Noop<Error> as Emitter<'source, T, L::Span>>::emit_error(&mut emitter, err) {
//         Ok(_) => unreachable!("no-op emitter should always return an error"),
//         Err(err) => Err(err),
//       },
//     }
//   }
// }

// impl<'source, T, L, O, E, F, C> ParserWithEmitter<'source, T, L, O, E, C> for F
// where
//   F: FnMut(InputRef<'source, '_, T, L, E, C>) -> Result<O, Spanned<E::Error, L::Span>>,
//   T: Token<'source>,
//   E: Emitter<'source, T, L::Span>,
//   L: Lexer<'source, T>,
// {
//   fn parse_with_emitter(&mut self, input: &'source <L>::Source, emitter: &mut E) -> Result<O, Spanned<<E as Emitter<'source, T, L::Span>>::Error, L::Span>>
//   where
//     L: Lexer<'source, T>,
//     L::State: Default,
//     T: Token<'source>,
//     C: Cache<'source, T, L>,
//     E: Emitter<'source, T, L::Span>,
//   {
//     let mut inp = Input::with_state_and_cache(input, Default::default(), C::new());
//     let inp_ref = inp.as_ref(emitter);

//     (self)(inp_ref)  
//   }
// }