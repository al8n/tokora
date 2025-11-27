use crate::{
  Check, Span,
  emitter::{BatchEmitter, SeparatedByEmitter},
  error::token::{MissingLeadingOf, MissingTokenOf, UnexpectedLeadingOf, UnexpectedRepeatedOf, UnexpectedTrailingOf},
};

use super::*;

// No trailing, no leading, unbounded
impl<'inp, L, F, Sep, O, Container, E, C, Trailing, Leading, Max, Min>
  ParseInput<'inp, L, ParseResult<'inp, Container, L, E>, E, C>
  for SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Max, Min>>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, ParseResult<'inp, O, L, E>, E, C>,
  Sep: Check<L::Token, SeqSepHint>,
  E: SeparatedByEmitter<'inp, L, Sep>,
  C: Cache<'inp, L>,
  Container: Default + super::Container<Spanned<O, L::Span>>,
  Trailing: super::TrailingSpec,
  Leading: super::LeadingSpec,
  Max: super::MaxSpec,
  Min: super::MinSpec,
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
    let mut state: State<L::Token, L::Span> = State::Start;
    let mut leading_seps = None;
    let ckp = inp.save();
    let mut num = 0;
    let minimum = self.minimum();
    let maximum = self.maximum();

    let mut leadings = 0;
    let mut trailings = 0;
    let leading_spec = self.leading();
    let trailing_spec = self.trailing();

    let mut lexer_errs_id = None;
    let mut leading_seps_errs_id = None;
    let mut trailing_seps_errs_id = None;
    let mut repeated_seps_errs_id = None;

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
            let first = container.first().map(|t| t.span()).unwrap_or_else(|| {});
            inp
              .emitter()
              .emit_leading_separator(UnexpectedLeadingOf::leading(first, leadings))?;
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

              // try to batch lexer errors
              if let Some(lexer_errs_id) = &mut lexer_errs_id {
                inp
                  .emitter()
                  .emit_to_batch(lexer_errs_id, nxt.map_data(|s| s.unwrap_error()))?;
              } else {
                let nxt_span = nxt.span_ref().clone();
                inp.emitter().create_batch_with_error(
                  "lexer errors".into(),
                  nxt.map_data(|s| s.unwrap_error()),
                )?;
                lexer_errs_id = Some(nxt_span);
              }
              continue;
            }
            Lexed::Token(tok) => {
              match self.sep.check(tok) {
                SeqSepHint::End => {
                  let trailings = match state {
                    State::Separator(span) => span,
                    _ => return Ok(Spanned::new(inp.span_since(ckp.cursor()), container)),
                  };

                  if let Some(leadings) = leading_seps.take() {
                    inp.emitter().emit_leading_separator(leadings)?;
                  }

                  let span = inp.span_since(ckp.cursor());
                  // TODO(al8n): improve the trailing error, add info about the separator
                  inp
                    .emitter()
                    .emit_trailing_separator(UnexpectedTrailingOf::trailing(trailings, found))?;
                  return Ok(Spanned::new(span, container));
                }
                SeqSepHint::Separator => {
                  let sep_tok = inp
                    .next()
                    .expect("peeked token already confirmed there must be a token");
                  match &mut state {
                    // happy path, we found a separator after an element
                    State::Element => {
                      // Change the current state to Separator.
                      state = State::Separator(sep_tok.map_data(|t| t.unwrap_token()));
                    }
                    // First token is a separator, we found another leading separator
                    State::Leading(tok) => {
                      // whatever the leading spec is, multiple leading separators are not allowed
                      // we should start a leading separator error batch and emit the newly found leading separator
                      // to the batch
                      match leading_spec {
                        SepFixSpec::Deny(_) => {
                          let sep_span = sep_tok.span_ref().clone();
                          // if leading sep is denied, we must have an existing leading sep error batch
                          <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::emit_to_batch(
                            inp.emitter(),
                            tok.span_ref(),
                            Spanned::new(
                              sep_span.clone(),
                              UnexpectedLeadingOf::leading(
                                sep_span.clone(),
                                sep_tok.into_data().unwrap_token()
                              ),
                            ),
                          )?;

                          state = State::Leadings(tok.span_ref().clone());
                        }
                        SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {
                          let sep_span = sep_tok.span_ref().clone();
                          // we are not allowed to have multiple leading separators.
                          // try to emit leading separator error via the emitter
                          <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::create_batch_with_error(
                            inp.emitter(),
                            "leading separator".into(),
                            Spanned::new(
                              sep_span.clone(),
                              UnexpectedLeadingOf::leading(
                                sep_span.clone(),
                                sep_tok.into_data().unwrap_token()
                              ),
                            ),
                          )?;

                          // store the first leading sep span as this will be used to identify the batch later
                          state = State::Leadings(sep_span);
                        }
                      }
                    }
                    State::Leadings(span) => {
                      // we already have multiple leading separators, just emit the newly found leading separator
                      let sep_span = sep_tok.span_ref().clone();
                      <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::emit_to_batch(
                        inp.emitter(),
                        span,
                        Spanned::new(
                          sep_span.clone(),
                          UnexpectedLeadingOf::leading(
                            sep_span.clone(),
                            sep_tok.into_data().unwrap_token(),
                          ),
                        ),
                      )?;

                      // no need to change state, still in leadings
                    }
                    // first token is a separator
                    State::Start => {
                      match leading_spec {
                        SepFixSpec::Deny(deny) => {
                          let sep_span = sep_tok.span_ref().clone();
                          // we are not allowed to have leading separator.
                          // try to emit leading separator error via the emitter
                          <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::create_batch_with_error(
                            inp.emitter(),
                            "leading separator".into(),
                            Spanned::new(
                              sep_span.clone(),
                              UnexpectedLeadingOf::leading(
                                sep_span.clone(),
                                sep_tok.into_data().unwrap_token()
                              ),
                            ),
                          )?;
                        }
                        SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {}
                      }

                      state = State::Leading(sep_tok.map_data(|t| t.unwrap_token()));
                    }
                    // we are in separator state, so the next token should be an element,
                    // so repeated separator case, let's construct a repeated separator error,
                    // and emit it via the emitter, and let the emitter decide whether to return early
                    State::Separator(tok) => {
                      // one more repeated separator
                      let (sep_span, sep_token) = sep_tok.into_components();

                      // create a batch for repeated separator errors if not already created
                      <E as BatchEmitter<'_, L, UnexpectedRepeatedOf<'_, Sep, L>>>::create_batch_with_error(
                        inp.emitter(),
                        "repeated separator".into(),
                        Spanned::new(
                          sep_span.clone(),
                          UnexpectedRepeatedOf::repeated(
                            sep_span.clone(),
                            sep_token.unwrap_token(),
                          ),
                        ),
                      )?;

                      // change state to RepeatedSeparator, store the span as the id for the batch
                      state = State::RepeatedSeparator(sep_span);
                    }
                    // we are in repeated separator state,
                    // so just extend the repeated separator span
                    State::RepeatedSeparator(span) => {
                      let (sep_span, sep_token) = sep_tok.into_components();
                      <E as BatchEmitter<'_, L, UnexpectedRepeatedOf<'_, Sep, L>>>::emit_to_batch(
                        inp.emitter(),
                        span,
                        Spanned::new(
                          sep_span.clone(),
                          UnexpectedRepeatedOf::repeated(
                            sep_span.clone(),
                            sep_token.unwrap_token(),
                          ),
                        ),
                      )?;
                      // no need to change state, still in RepeatedSeparator
                    }
                  }
                }
                SeqSepHint::Continue => {
                  // if the next token belongs to an element, check the current state
                  match state {
                    State::Separator(_) => {
                      // parse the next element
                      let element = self.f.parse_input(inp)?;
                      container.push(element.map_data(|d| d));
                      state = State::Element;
                    }
                    // we have only one leading separator before
                    State::Leading(leading_tok) => {
                      match leading_spec {
                        // no leading separators allowed
                        SepFixSpec::Deny(_) => {
                          let (sep_span, sep_token) = leading_tok.into_components();
                          inp.emitter().emit_unexpected_leading_separator(
                            UnexpectedLeadingOf::leading(sep_span, sep_token),
                          )?;
                        }
                        SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {},
                      }

                      // parse the first element
                      let element = self.f.parse_input(inp)?;
                      container.push(element);
                      state = State::Element;
                    }
                    State::Leadings(span) => {
                      // we have multiple leading separators before
                      // emit the batch via the emitter
                      <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::emit_batch(
                        inp.emitter(),
                        &span,
                      )?;
                      // parse the first element
                      let element = self.f.parse_input(inp)?;
                      container.push(element);
                      state = State::Element;
                    }
                    // parse the first element
                    State::Start => {
                      match leading_spec {
                        SepFixSpec::Require(_) => {
                          // unhappy, missing the required leading separator
                          inp.emitter().emit_missing_leading_separator(MissingLeadingOf::leading(peek_span.start()))?;
                        }
                        SepFixSpec::Deny(_) | SepFixSpec::Allow(_) => {
                          // so happyyyyy, no leading separators, just parse the first element
                        }
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
                      inp.emitter().emit_missing_separator(MissingTokenOf::new(off).with_knowledge(Default::default()))?;

                      // parse the next element
                      let element = self.f.parse_input(inp)?;
                      container.push(element);
                      state = State::Element;
                    }
                    // before finding an element, there are repeated separators
                    // so emit repeated separators error, and let the emitter decide whether to return early
                    State::RepeatedSeparator(span) => {
                      // before parsing the next element, emit the repeated separator errors
                      <E as BatchEmitter<'_, L, UnexpectedRepeatedOf<'_, Sep, L>>>::emit_batch(
                        inp.emitter(),
                        &span,
                      )?;

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
