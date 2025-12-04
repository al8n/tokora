use crate::{emitter::DelimiterEmitter, error::Unclosed};

use super::*;

/// A parser that parses a construct delimited by left and right tokens.
///
/// See also: [`DelimSepSeq`]
pub struct Delimiter<P, Open, Close, Delim, Window> {
  parser: P,
  left_classifier: Open,
  right_classifier: Close,
  delimiter: Delim,
  _window: PhantomData<Window>,
}

impl<P, Open, Close, Delim, Window> Delimiter<P, Open, Close, Delim, Window> {
  /// Creates a new `Delim` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(parser: P, left: Open, right: Close, delim: Delim) -> Self {
    Self {
      parser,
      left_classifier: left,
      right_classifier: right,
      delimiter: delim,
      _window: PhantomData,
    }
  }
}

impl<'inp, L, P, Open, Close, O, Ctx, Delim, Window, Lang: ?Sized> ParseInput<'inp, L, O, Ctx, Lang>
  for Delimiter<P, Open, Close, Delim, Window>
where
  L: Lexer<'inp>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Delim: Clone,
  Window: Capacity,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: DelimiterEmitter<'inp, Delim, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let output = inp.peek::<Window::CAPACITY>();
    if output.is_empty() {
      return Err(UnexpectedEot::eot_of(inp.cursor().into_inner()).into());
    }

    let mut iter = output.iter();
    let first = iter.next();
    let second = iter.next();

    match (first, second) {
      (None, _) => Err(UnexpectedEot::eot_of(inp.cursor().into_inner()).into()),
      (Some(first), None) => {
        let ct = first.as_ref();
        let (spanned, tok) = ct.token().as_ref().into_components();

        match tok {
          Lexed::Error(err) => {
            let nxt = inp
              .next()
              .expect("peek gurantees there is a next token")
              .map_data(|t| t.unwrap_error());
            inp.emitter().emit_lexer_error(nxt)?;

            Err(UnexpectedEot::eot_of(inp.cursor().into_inner()).into())
          }
          Lexed::Token(tok) => match self.left_classifier.check(tok) {
            Err(knd) => {
              inp.emitter().emit_unexpected_token(
                UnexpectedToken::with_expected(spanned.clone(), Expected::one(knd))
                  .with_found(tok.clone())
                  .into(),
              )?;
              Err(UnexpectedEot::eot_of(inp.cursor().into_inner()).into())
            }
            Ok(_) => {
              let (span, nxt) = inp
                .next()
                .expect("peek gurantees there is a next token")
                .map_data(|t| t.unwrap_token())
                .into_components();
              inp
                .emitter()
                .emit_unclosed(Unclosed::of(span, self.delimiter.clone()))?;
              Err(UnexpectedEot::eot_of(inp.cursor().into_inner()).into())
            }
          },
        }
      }
      (Some(first), Some(second)) => {
        let buf = [(); generic_arraydeque::typenum::U10::USIZE];
        Ok(())
      }
    }
  }
}
