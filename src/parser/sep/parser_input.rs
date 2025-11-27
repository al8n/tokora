use crate::{
  Check, Checkpoint, Span,
  emitter::{BatchEmitter, SeparatedByEmitter},
  error::{
    syntax::{MissingSyntaxOf, TooFew, TooMany},
    token::{
      MissingLeadingOf, MissingTokenOf, MissingTrailingOf, UnexpectedLeadingOf,
      UnexpectedRepeatedOf, UnexpectedToken, UnexpectedTrailingOf,
    },
  },
};

use super::*;

impl<'inp, L, F, Sep, O, Container, E, C, Trailing, Leading, Max, Min>
  ParseInput<'inp, L, ParseResult<'inp, Container, L, E>, E, C>
  for SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Max, Min>>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, ParseResult<'inp, O, L, E>, E, C>,
  Sep: Check<L::Token, SeqSepHint<'inp, <L::Token as Token<'inp>>::Kind>>,
  E: SeparatedByEmitter<'inp, O, Sep, L>,
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
    let ckp = inp.save();
    let leading_spec = self.leading();

    let mut lexer_errs_id = None;

    loop {
      // peek two tokens ahead
      let peeked = inp.peek_one();

      match peeked {
        None => {
          return self
            .handle_end(state, inp, &ckp, &mut container)
            .map(|span| Spanned::new(span, container));
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
                  return self
                    .handle_end(state, inp, &ckp, &mut container)
                    .map(|span| Spanned::new(span, container));
                }
                SeqSepHint::Skip => {
                  inp.consume_one();
                  continue;
                }
                SeqSepHint::Unexpected(exp) => {
                  let unexpected_tok = inp
                    .next()
                    .expect("peeked token already confirmed there must be a token");
                  let (span, token) = unexpected_tok.into_components();
                  let err = match exp {
                    Some(expected) => UnexpectedToken::with_expected(span, expected)
                      .with_found(token.unwrap_token()),
                    None => UnexpectedToken::new(span).with_found(token.unwrap_token()),
                  };

                  inp.emitter().emit_unexpected_token(err)?;

                  continue;
                }
                SeqSepHint::Separator => {
                  let sep_tok = inp
                    .next()
                    .expect("peeked token already confirmed there must be a token");
                  match state {
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
                          // we are not allowed to have multiple leading separators.
                          let (tok_span, tok_token) = tok.into_components();
                          let (sep_span, sep_tok) = sep_tok.into_components();

                          // we are not allowed to have multiple leading separators.
                          // try to emit leading separator error via the emitter
                          <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::create_batch_with_error(
                            inp.emitter(),
                            "leading separators".into(),
                            Spanned::new(
                              tok_span.clone(),
                              UnexpectedLeadingOf::<'_, Sep, L>::leading(
                                tok_span.clone(),
                                tok_token,
                              ),
                            ),
                          )?;

                          <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::emit_to_batch(
                            inp.emitter(),
                            &tok_span,
                            Spanned::new(
                              sep_span.clone(),
                              UnexpectedLeadingOf::<'_, Sep, L>::leading(
                                sep_span.clone(),
                                sep_tok.unwrap_token(),
                              ),
                            ),
                          )?;

                          // store the first leading sep span as this will be used to identify the batch later
                          state = State::Leadings(tok_span);
                        }
                        SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {
                          let (sep_span, sep_tok) = sep_tok.into_components();

                          // we are not allowed to have multiple leading separators.
                          // try to emit leading separator error via the emitter
                          <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::create_batch_with_error(
                            inp.emitter(),
                            "leading separators".into(),
                            Spanned::new(
                              sep_span.clone(),
                              UnexpectedLeadingOf::<'_, Sep, L>::leading(
                                sep_span.clone(),
                                sep_tok.unwrap_token(),
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
                        &span,
                        Spanned::new(
                          sep_span.clone(),
                          UnexpectedLeadingOf::<'_, Sep, L>::leading(
                            sep_span.clone(),
                            sep_tok.into_data().unwrap_token(),
                          ),
                        ),
                      )?;

                      // no need to change state, still in leadings
                      state = State::Leadings(span);
                    }
                    // first token is a separator
                    State::Start => {
                      // we do not need to check leading spec here, as we cached the leading separator token,
                      // the check will be done when we find the first element or reach the end of input

                      state = State::Leading(sep_tok.map_data(|t| t.unwrap_token()));
                    }
                    // we are in separator state, so the next token should be an element,
                    // so repeated separator case, let's construct a repeated separator error,
                    // and emit it via the emitter, and let the emitter decide whether to return early
                    State::Separator(tok) => {
                      // one more repeated separator
                      let (tok_span, tok_token) = tok.into_components();
                      let (sep_span, sep_token) = sep_tok.into_components();

                      // create a batch for repeated separator errors if not already created
                      <E as BatchEmitter<'_, L, UnexpectedRepeatedOf<'_, Sep, L>>>::create_batch_with_error(
                        inp.emitter(),
                        "repeated separator".into(),
                        Spanned::new(
                          tok_span.clone(),
                          UnexpectedRepeatedOf::<'_, Sep, L>::repeated(
                            tok_span.clone(),
                            tok_token.clone(),
                          ),
                        ),
                      )?;

                      <E as BatchEmitter<'_, L, UnexpectedRepeatedOf<'_, Sep, L>>>::emit_to_batch(
                        inp.emitter(),
                        &tok_span,
                        Spanned::new(
                          sep_span.clone(),
                          UnexpectedRepeatedOf::<'_, Sep, L>::repeated(
                            sep_span,
                            sep_token.unwrap_token(),
                          ),
                        ),
                      )?;

                      // change state to RepeatedSeparator, store the span as the id for the batch
                      state = State::RepeatedSeparator(tok_span);
                    }
                    // we are in repeated separator state,
                    // so just extend the repeated separator span
                    State::RepeatedSeparator(span) => {
                      let (sep_span, sep_token) = sep_tok.into_components();
                      <E as BatchEmitter<'_, L, UnexpectedRepeatedOf<'_, Sep, L>>>::emit_to_batch(
                        inp.emitter(),
                        &span,
                        Spanned::new(
                          sep_span.clone(),
                          UnexpectedRepeatedOf::<'_, Sep, L>::repeated(
                            sep_span.clone(),
                            sep_token.unwrap_token(),
                          ),
                        ),
                      )?;
                      // no need to change state, still in RepeatedSeparator
                      state = State::RepeatedSeparator(span);
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
                            UnexpectedLeadingOf::<'_, Sep, L>::leading(sep_span, sep_token),
                          )?;
                        }
                        SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {}
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
                          let off = peek_span.start();
                          // unhappy, missing the required leading separator
                          inp
                            .emitter()
                            .emit_missing_leading_separator(
                              MissingLeadingOf::<'_, Sep, L>::leading(off),
                            )?;
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
                      inp.emitter().emit_missing_separator(
                        MissingTokenOf::<'_, Sep, L>::new(off).with_knowledge(Default::default()),
                      )?;

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

impl<'inp, F, Sep, O, Container, Trailing, Leading, Max, Min>
  SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Max, Min>>
{
  fn handle_end<'closure, L, E, C>(
    &mut self,
    state: State<L::Token, L::Span>,
    inp: &mut InputRef<'inp, 'closure, L, E, C>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    container: &mut Container,
  ) -> Result<L::Span, Spanned<E::Error, L::Span>>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
    F: ParseInput<'inp, L, ParseResult<'inp, O, L, E>, E, C>,
    Sep: Check<L::Token, SeqSepHint<'inp, <L::Token as Token<'inp>>::Kind>>,
    E: SeparatedByEmitter<'inp, O, Sep, L>,
    C: Cache<'inp, L>,
    Container: super::Container<Spanned<O, L::Span>>,
    Trailing: super::TrailingSpec,
    Leading: super::LeadingSpec,
    Max: super::MaxSpec,
    Min: super::MinSpec,
  {
    let minimum = self.minimum();
    let maximum = self.maximum();
    let leading_spec = self.leading();
    let trailing_spec = self.trailing();

    Ok(match state {
      // we are in the start state, so no elements were found
      State::Start => {
        let span = inp.span_since(ckp.cursor());
        if minimum > 0 {
          inp
            .emitter()
            .emit_too_few(TooFew::new(span.clone(), container.len(), minimum))?;
        }
        span
      }
      // we are in element state, so all good, check for trailing separator, and the minimum, maximum constraints
      State::Element => {
        let full_span = inp.span_since(ckp.cursor());
        let nums = container.len();
        if nums < minimum {
          inp
            .emitter()
            .emit_too_few(TooFew::new(full_span.clone(), nums, minimum))?;
        }

        if nums > maximum {
          inp
            .emitter()
            .emit_too_many(TooMany::new(full_span.clone(), nums, maximum))?;
        }

        if trailing_spec.is_require() {
          let off = inp.span().end();
          inp
            .emitter()
            .emit_missing_trailing_separator(MissingTrailingOf::<'_, Sep, L>::trailing(off))?;
        }
        full_span
      }
      State::Leading(spanned) => {
        // only find leading separators, no element
        let (sep_span, sep_token) = spanned.into_components();
        match leading_spec {
          SepFixSpec::Deny(_) => {
            // we are not allowed to have leading separators
            inp.emitter().emit_unexpected_leading_separator(
              UnexpectedLeadingOf::<'_, Sep, L>::leading(sep_span, sep_token),
            )?;
          }
          SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {
            // we should emit an error as we are missing the element followed the leading separator
            inp
              .emitter()
              .emit_missing_element(MissingSyntaxOf::<'_, O, L>::new(sep_span.end()))?;
          }
        }
        inp.span_since(ckp.cursor())
      }
      State::Leadings(leadings) => {
        // only find leading separators, no element
        // emit the batch via the emitter
        <E as BatchEmitter<'_, L, UnexpectedLeadingOf<'_, Sep, L>>>::emit_batch(
          inp.emitter(),
          &leadings,
        )?;

        let full_span = inp.span_since(ckp.cursor());
        if !leading_spec.is_deny() {
          // we should emit an error as we are missing the element followed the leading separator
          inp
            .emitter()
            .emit_missing_element(MissingSyntaxOf::<'_, O, L>::new(full_span.end()))?;
        }

        full_span
      }
      // we have a trailing separator
      State::Separator(spanned) => {
        let (sep_span, sep_token) = spanned.into_components();

        // we have a trailing separator, but the spec says no trailing separators allowed
        if trailing_spec.is_deny() {
          inp.emitter().emit_unexpected_trailing_separator(
            UnexpectedTrailingOf::<'_, Sep, L>::trailing(sep_span, sep_token),
          )?;
        }

        let full_span = inp.span_since(ckp.cursor());
        let nums = container.len();
        if nums < minimum {
          inp
            .emitter()
            .emit_too_few(TooFew::new(full_span.clone(), nums, minimum))?;
        }

        if nums > maximum {
          inp
            .emitter()
            .emit_too_many(TooMany::new(full_span.clone(), nums, maximum))?;
        }

        full_span
      }
      State::RepeatedSeparator(trailings) => {
        // we have more than one trailing separator
        // drop the repeated separator errors batch.
        <E as BatchEmitter<'_, L, UnexpectedRepeatedOf<'_, Sep, L>>>::drop_batch(
          inp.emitter(),
          &trailings,
        );

        // rewind to the end of the last element
        let mut lxr = inp.lexer_at(trailings.start_ref());

        // create a new batch for unexpected trailing separators
        <E as BatchEmitter<'_, L, UnexpectedTrailingOf<'_, Sep, L>>>::create_batch(
          inp.emitter(),
          trailings.clone(),
          "trailing separators".into(),
        );

        while let Some(tok) = lxr.lex() {
          let span = lxr.span();

          if span.end_ref().ge(trailings.end_ref()) {
            break;
          }

          match tok {
            Err(_) => {}
            Ok(tok) => {
              if self.sep.check(&tok) == SeqSepHint::Separator {
                <E as BatchEmitter<'_, L, UnexpectedTrailingOf<'_, Sep, L>>>::emit_to_batch(
                  inp.emitter(),
                  &trailings,
                  Spanned::new(
                    span.clone(),
                    UnexpectedTrailingOf::<'_, Sep, L>::trailing(span, tok),
                  ),
                )?;
              }
            }
          }
        }

        <E as BatchEmitter<'_, L, UnexpectedTrailingOf<'_, Sep, L>>>::emit_batch(
          inp.emitter(),
          &trailings,
        )?;

        inp.span_since(ckp.cursor())
      }
    })
  }
}
