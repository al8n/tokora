use crate::{
  Check, Span,
  emitter::SeparatedByEmitter, error::parser::{UnexpectedLeadingOf, UnexpectedTrailingOf},
};

use super::*;

// No trailing, no leading, unbounded
impl<'inp, L, F, Sep, O, Container, E, C>
  ParseInput<'inp, L, ParseResult<'inp, Container, L, E>, E, C>
  for SeqSep<F, Sep, O, Container, Init>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, ParseResult<'inp, O, L, E>, E, C>,
  Sep: Check<L::Token, SeqSepHint>,
  E: SeparatedByEmitter<'inp, L, Sep>,
  C: Cache<'inp, L>,
  Container: Default + super::Container<Spanned<O, L::Span>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, E, C>,
  ) -> ParseResult<'inp, Container, L, E>
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
    let mut container = Container::default();
    let mut state = State::Start;
    let mut leading_seps = None;
    let ckp = inp.save();

    loop {
      // peek two tokens ahead
      let peeked = inp.peek_one();

      match peeked {
        None => {
          let trailings: L::Span = match state {
            State::Separator(span) => span,
            State::RepeatedSeparator(span) => span,
            _ => return Ok(Spanned::new(inp.span_since(ckp.cursor()), container)),
          };

          if let Some(leadings) = leading_seps.take() {
            let first = container.first().map(|t| t.span()).unwrap_or_else(|| {
            
            });
            inp.emitter().emit_leading_separator(UnexpectedLeadingPunctuatorFor::from_prefix(first, leadings))?;
          }

          // if the emitter treat trailing separator error as a non-fatal error, emit it
          // otherwise, return an error
          let span = inp.span_since(ckp.cursor());
          // TODO(al8n): improve the trailing error, add info about the separator
          inp.emitter().emit_trailing_separator(trailings.clone())?;

          return Ok(Spanned::new(span, container));
        }
        Some(tok) => {
          let tok = tok.as_ref();
          let peek_span = tok.token().span_ref();
          match tok.token().data() {
            Lexed::Error(_) => {
              // if the next token is an error token, emit the error.
              let nxt = inp
                .next()
                .expect("peeked token already confirmed there must be a token");
              inp.emit_lexer_error(nxt.map_data(|s| s.unwrap_error()))?;
              continue;
            }
            Lexed::Token(tok) => {
              match self.sep.check(tok) {
                SeqSepHint::End => {
                  let trailings = match state {
                    State::Separator(span) => span,
                    State::RepeatedSeparator(span) => span,
                    _ => return Ok(Spanned::new(inp.span_since(ckp.cursor()), container)),
                  };

                  if let Some(leadings) = leading_seps.take() {
                    inp.emitter().emit_leading_separator(leadings)?;
                  }

                  let span = inp.span_since(ckp.cursor());
                  // TODO(al8n): improve the trailing error, add info about the separator
                  inp.emitter().emit_trailing_separator(trailings)?;
                  return Ok(Spanned::new(span, container));
                }
                SeqSepHint::Separator => {
                  let sep_tok = inp
                    .next()
                    .expect("peeked token already confirmed there must be a token");
                  match &mut state {
                    State::Start => {
                      let leadings = sep_tok.into_components().0;
                      match leading_seps.as_mut() {
                        None => {
                          leading_seps = Some(leadings);
                        }
                        Some(span) => {
                          // Do not emit at this moment, just extend the span
                          *span.end_mut() = leadings.end();
                        }
                      }
                    }
                    State::Separator(span) => {
                      // one more repeated separator
                      state = State::RepeatedSeparator(L::Span::new(
                        span.start(),
                        sep_tok.into_span().into_range().end,
                      ));
                    }
                    State::RepeatedSeparator(span) => {
                      // one more repeated separator
                      *span.end_mut() = sep_tok.span_ref().end();
                    }
                    State::Element => {
                      // Change the current state to Separator.
                      state = State::Separator(sep_tok.into_span());
                    }
                  }
                }
                SeqSepHint::Continue => {
                  // if the next token belongs to an element, check the current state
                  match state {
                    // parse the first element
                    State::Start => {
                      // If we have leading separators, let the emitter decide whether to return early
                      if let Some(leadings) = leading_seps.take() {
                        inp.emitter().emit_leading_separator(leadings)?;
                      }

                      // parse the first element
                      let element = self.f.parse_input(inp)?;
                      container.push(element);
                      state = State::Element;
                    }
                    // we are in element state, so the next token should be a separator,
                    // so missing separator case, let's construct a missing separator error,
                    // and emit it via the emitter, and let the emitter decide whether to return early
                    State::Element => {
                      let off = peek_span.start();
                      inp.emitter().emit_missing_separator(off)?;

                      // parse the next element
                      let element = self.f.parse_input(inp)?;
                      container.push(element);
                      state = State::Element;
                    }
                    State::Separator(_) => {
                      // parse the next element
                      let element = self.f.parse_input(inp)?;
                      container.push(element.map_data(|d| d));
                      state = State::Element;
                    }
                    // before finding an element, there are repeated separators
                    // so emit repeated separators error, and let the emitter decide whether to return early
                    State::RepeatedSeparator(span) => {
                      inp.emitter().emit_repeated_separators(span.clone())?;

                      // parse the next element
                      let element = self.f.parse_input(inp)?;
                      container.push(element.map_data(|d| d));
                      state = State::Element;
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
