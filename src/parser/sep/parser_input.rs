use crate::{
  Check,
  emitter::{BatchEmitter, RepeatedEmitter, SeparatedByEmitter},
  error::{
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{
      MissingLeadingOf, MissingSeparatorOf, MissingTrailingOf, UnexpectedLeadingOf,
      UnexpectedRepeatedOf, UnexpectedTrailingOf,
    },
  },
  lexer::{Checkpoint, Span},
};

use super::*;

use core::mem;

impl<
  'inp,
  L,
  F,
  SepClassifier,
  Condition,
  O,
  Container,
  Ctx,
  Trailing,
  Leading,
  Max,
  Min,
  Lang: ?Sized,
  W,
> ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    SeparatedBy<F, SepClassifier, Condition, O, W, SeparatedByOptions<Trailing, Leading, Max, Min>>,
    Container,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + crate::container::Container<O>,
  W: Window,
  Trailing: super::TrailingSpec,
  Leading: super::LeadingSpec,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self
      .as_mut()
      .map_parser(|p| p.as_mut())
      .parse_input(inp)
      .map(|_| mem::take(&mut self.container))
  }
}

impl<
  'inp,
  L,
  F,
  SepClassifier,
  Condition,
  O,
  Container,
  Ctx,
  Trailing,
  Leading,
  Max,
  Min,
  Lang: ?Sized,
  W,
> ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      SeparatedBy<
        F,
        SepClassifier,
        Condition,
        O,
        W,
        SeparatedByOptions<Trailing, Leading, Max, Min>,
      >,
      Container,
    >,
    PhantomSpan,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + crate::container::Container<O>,
  W: Window,
  Trailing: super::TrailingSpec,
  Leading: super::LeadingSpec,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self
      .primary_mut()
      .as_mut()
      .map_parser(|p| p.as_mut())
      .parse_input(inp)
      .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
  }
}

impl<
  'inp,
  'c,
  L,
  F,
  SepClassifier,
  Condition,
  O,
  Container,
  Ctx,
  Trailing,
  Leading,
  Max,
  Min,
  Lang: ?Sized,
  W,
> ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut SeparatedBy<
      &'c mut F,
      &'c mut SepClassifier,
      Condition,
      O,
      W,
      &'c mut SeparatedByOptions<Trailing, Leading, Max, Min>,
    >,
    &'c mut Container,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: crate::container::Container<O>,
  W: Window,
  Trailing: super::TrailingSpec,
  Leading: super::LeadingSpec,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, <Ctx>::Emitter, <Ctx>::Cache, Lang>,
  ) -> Result<L::Span, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let Self {
      parser:
        SeparatedBy {
          f,
          sep,
          condition,
          config,
          ..
        },
      container,
    } = self;
    let mut parser = SeparatedBy {
      f: &mut **f,
      sep: &mut **sep,
      condition: &mut *condition,
      config: &mut **config,
      _m: PhantomData,
      _decision_window: PhantomData,
    }
    .collect_with(&mut *container);

    parser.parse_input(input)
  }
}

impl<
  'inp,
  'c,
  L,
  F,
  SepClassifier,
  Condition,
  O,
  Container,
  Ctx,
  Trailing,
  Leading,
  Max,
  Min,
  Lang: ?Sized,
  W,
> ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    SeparatedBy<
      &'c mut F,
      &'c mut SepClassifier,
      &'c mut Condition,
      O,
      W,
      &'c mut SeparatedByOptions<Trailing, Leading, Max, Min>,
    >,
    &'c mut Container,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: crate::container::Container<O>,
  W: Window,
  Trailing: super::TrailingSpec,
  Leading: super::LeadingSpec,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let Self { parser, container } = self;

    let mut state: State<L::Token, L::Span> = State::Start;
    let ckp = inp.save();
    let leading_spec = parser.config.leading();
    let mut num_elems = 0;

    loop {
      let (peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      let peek_span = match peeked.front() {
        None => {
          drop(peeked);
          return parser.handle_end(state, inp, &ckp, num_elems, container);
        }
        Some(tok) => {
          // the sync_until_token_then_peek_with_emitter guarantees the first token is not a `Lexed::Error`
          let tok = tok
            .as_maybe_ref()
            .map(
              |t| t.token().map(|t| *t, |t| t.unwrap_token_ref()),
              |t| t.token().map_data(|t| t.unwrap_token_ref()),
            )
            .into_inner();
          let peek_span = tok.span();
          match tok.data() {
            // TODO(al8n): move the batching logic into the lexer sync function
            // Lexed::Error(_) => {
            //   drop(peeked);

            //   // if the next token is an error token, emit the error.
            //   let nxt = inp
            //     .next()
            //     .expect("peeked token already confirmed there must be a token");

            //   // try to batch lexer errors
            //   if let Some(lexer_errs_id) = &mut lexer_errs_id {
            //     inp
            //       .emitter()
            //       .emit_to_batch(lexer_errs_id, nxt.map_data(|s| s.unwrap_error()))?;
            //   } else {
            //     let nxt_span = nxt.span_ref().clone();
            //     inp.emitter().create_batch_with_error(
            //       "lexer errors".into(),
            //       nxt.map_data(|s| s.unwrap_error()),
            //     )?;
            //     lexer_errs_id = Some(nxt_span);
            //   }
            //   continue;
            // }
            tok if parser.sep.check(tok) => {
              drop(peeked);
              state = parser.handle_separator::<_, Ctx, Lang>(state, inp, leading_spec)?;

              continue;
            }
            _ => peek_span.clone(),
          }
        }
      };

      match parser.condition.decide(peeked, emitter)? {
        Action::End => {
          return parser.handle_end(state, inp, &ckp, num_elems, container);
        }
        Action::Continue => {
          // if the peeked token belongs to an element, check the current state
          state = parser.handle_continue(
            state,
            inp,
            &ckp,
            &peek_span,
            &mut num_elems,
            leading_spec,
            container,
          )?;
        }
      }
    }
  }
}

impl<'c, 'inp, F, SepClassifier, Condition, O, Trailing, Leading, Max, Min, W>
  SeparatedBy<
    &'c mut F,
    &'c mut SepClassifier,
    &'c mut Condition,
    O,
    W,
    &'c mut SeparatedByOptions<Trailing, Leading, Max, Min>,
  >
{
  pub(in crate::parser) fn handle_separator<'closure, L, Ctx, Lang>(
    &mut self,
    mut state: State<L::Token, L::Span>,
    inp: &mut InputRef<'inp, 'closure, L, Ctx::Emitter, Ctx::Cache, Lang>,
    leading_spec: SepFixSpec,
  ) -> Result<State<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
    Lang: ?Sized,
  {
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
        state = State::Leadings(self.handle_leading_state::<_, Ctx, _>(
          inp,
          tok,
          sep_tok,
          leading_spec,
        )?);
      }
      State::Leadings(span) => {
        state = State::Leadings(self.handle_leadings_state::<_, Ctx, _>(inp, span, sep_tok)?);
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
        // change state to RepeatedSeparator, store the span as the id for the batch
        state =
          State::RepeatedSeparator(self.handle_separator_state::<_, Ctx, _>(inp, tok, sep_tok)?);
      }
      // we are in repeated separator state,
      // so just extend the repeated separator span
      State::RepeatedSeparator(span) => {
        state = State::RepeatedSeparator(
          self.handle_repeated_separators_state::<_, Ctx, _>(inp, span, sep_tok)?,
        );
      }
    }
    Ok(state)
  }

  #[allow(clippy::too_many_arguments)]
  pub(in crate::parser) fn handle_continue<'closure, L, Ctx, Lang, Container>(
    &mut self,
    mut state: State<L::Token, L::Span>,
    inp: &mut InputRef<'inp, 'closure, L, Ctx::Emitter, Ctx::Cache, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    peek_span: &L::Span,
    num_elems: &mut usize,
    leading_spec: SepFixSpec,
    container: &mut Container,
  ) -> Result<State<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    W: Window,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    SepClassifier: Check<L::Token>,
    Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
    Container: crate::container::Container<O>,
    Trailing: super::TrailingSpec,
    Leading: super::LeadingSpec,
    Max: super::MaxSpec,
    Min: super::MinSpec,
    Lang: ?Sized,
  {
    match state {
      State::Separator(_) => {
        // parse the next element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            container.len(),
            Container::capacity(),
          ))?;
        }
        state = State::Element;
      }
      // we have only one leading separator before
      State::Leading(leading_tok) => {
        match leading_spec {
          // no leading separators allowed
          SepFixSpec::Deny(_) => {
            let (sep_span, sep_token) = leading_tok.into_components();
            inp
              .emitter()
              .emit_unexpected_leading_separator(
                UnexpectedLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(sep_span, sep_token),
              )?;
          }
          SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {}
        }

        // parse the first element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            *num_elems,
            Container::capacity(),
          ))?;
        }
        state = State::Element;
      }
      State::Leadings(span) => {
        // we have multiple leading separators before
        // emit the batch via the emitter
        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedLeadingOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::emit_batch(inp.emitter(), &span)?;
        // parse the first element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            *num_elems,
            Container::capacity(),
          ))?;
        }
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
                MissingLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(off),
              )?;
          }
          SepFixSpec::Deny(_) | SepFixSpec::Allow(_) => {
            // so happyyyyy, no leading separators, just parse the first element
          }
        }

        // parse the first element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            *num_elems,
            Container::capacity(),
          ))?;
        }

        state = State::Element;
      }
      // we are in element state, so the next token should be a separator,
      // so missing separator case, let's construct a missing separator error,
      // and emit it via the emitter, and let the emitter decide whether to return early
      State::Element => {
        let off = peek_span.start();
        inp
          .emitter()
          .emit_missing_separator(MissingSeparatorOf::<'_, SepClassifier, L, Lang>::of(off))?;

        // parse the next element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            *num_elems,
            Container::capacity(),
          ))?;
        }
        state = State::Element;
      }
      // before finding an element, there are repeated separators
      // so emit repeated separators error, and let the emitter decide whether to return early
      State::RepeatedSeparator(span) => {
        // before parsing the next element, emit the repeated separator errors
        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedRepeatedOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::emit_batch(inp.emitter(), &span)?;

        // parse the next element
        if push(num_elems, container, self.f.parse_input(inp)?).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            *num_elems,
            Container::capacity(),
          ))?;
        }
        state = State::Element;
      }
    }

    Ok(state)
  }

  pub(in crate::parser) fn handle_end<'closure, L, Ctx, Lang, Container>(
    &mut self,
    state: State<L::Token, L::Span>,
    inp: &mut InputRef<'inp, 'closure, L, Ctx::Emitter, Ctx::Cache, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    num_elems: usize,
    container: &mut Container,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    W: Window,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    SepClassifier: Check<L::Token>,
    Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
    Container: crate::container::Container<O>,
    Trailing: super::TrailingSpec,
    Leading: super::LeadingSpec,
    Max: super::MaxSpec,
    Min: super::MinSpec,
    Lang: ?Sized,
  {
    let minimum = self.config.minimum();
    let maximum = self.config.maximum();
    let leading_spec = self.config.leading();
    let trailing_spec = self.config.trailing();

    Ok(match state {
      // we are in the start state, so no elements were found
      State::Start => {
        let span = inp.span_since(ckp.cursor());
        if minimum > 0 {
          inp
            .emitter()
            .emit_too_few(TooFew::of(span.clone(), num_elems, minimum))?;
        }
        span
      }
      // we are in element state, so all good, check for trailing separator, and the minimum, maximum constraints
      State::Element => {
        let full_span = inp.span_since(ckp.cursor());
        if num_elems < minimum {
          inp
            .emitter()
            .emit_too_few(TooFew::of(full_span.clone(), num_elems, minimum))?;
        }

        if num_elems > maximum {
          inp
            .emitter()
            .emit_too_many(TooMany::of(full_span.clone(), num_elems, maximum))?;
        }

        if trailing_spec.is_require() {
          let off = inp.span().end();
          inp
            .emitter()
            .emit_missing_trailing_separator(
              MissingTrailingOf::<'_, SepClassifier, L, Lang>::trailing_of(off),
            )?;
        }
        full_span
      }
      State::Leading(spanned) => {
        // only find leading separators, no element
        let (sep_span, sep_token) = spanned.into_components();
        match leading_spec {
          SepFixSpec::Deny(_) => {
            // we are not allowed to have leading separators
            inp
              .emitter()
              .emit_unexpected_leading_separator(
                UnexpectedLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(sep_span, sep_token),
              )?;
          }
          SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {
            // we should emit an error as we are missing the element followed the leading separator
            inp
              .emitter()
              .emit_missing_element(MissingSyntaxOf::<'_, O, L, Lang>::of(sep_span.end()))?;
          }
        }
        inp.span_since(ckp.cursor())
      }
      State::Leadings(leadings) => {
        // only find leading separators, no element
        // emit the batch via the emitter
        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedLeadingOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::emit_batch(inp.emitter(), &leadings)?;

        let full_span = inp.span_since(ckp.cursor());
        if !leading_spec.is_deny() {
          // we should emit an error as we are missing the element followed the leading separator
          inp
            .emitter()
            .emit_missing_element(MissingSyntaxOf::<'_, O, L, Lang>::of(full_span.end()))?;
        }

        full_span
      }
      // we have a trailing separator
      State::Separator(spanned) => {
        let (sep_span, sep_token) = spanned.into_components();

        // we have a trailing separator, but the spec says no trailing separators allowed
        if trailing_spec.is_deny() {
          inp
            .emitter()
            .emit_unexpected_trailing_separator(
              UnexpectedTrailingOf::<'_, SepClassifier, L, Lang>::trailing_of(sep_span, sep_token),
            )?;
        }

        let full_span = inp.span_since(ckp.cursor());
        let nums = container.len();
        if nums < minimum {
          inp
            .emitter()
            .emit_too_few(TooFew::of(full_span.clone(), nums, minimum))?;
        }

        if nums > maximum {
          inp
            .emitter()
            .emit_too_many(TooMany::of(full_span.clone(), nums, maximum))?;
        }

        full_span
      }
      State::RepeatedSeparator(trailings) => {
        // we have more than one trailing separator
        // drop the repeated separator errors batch.
        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedRepeatedOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::drop_batch(inp.emitter(), &trailings);

        // rewind to the end of the last element
        let mut lxr = inp.lexer_at(trailings.start_ref());

        // create a new batch for unexpected trailing separators
        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedTrailingOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::create_batch(
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
              if self.sep.check(&tok) {
                <Ctx::Emitter as BatchEmitter<
                  '_,
                  L,
                  UnexpectedTrailingOf<'_, SepClassifier, L, Lang>,
                  Lang,
                >>::emit_to_batch(
                  inp.emitter(),
                  &trailings,
                  Spanned::new(
                    span.clone(),
                    UnexpectedTrailingOf::<'_, SepClassifier, L, Lang>::trailing_of(span, tok),
                  ),
                )?;
              }
            }
          }
        }

        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedTrailingOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::emit_batch(inp.emitter(), &trailings)?;

        inp.span_since(ckp.cursor())
      }
    })
  }

  pub(in crate::parser) fn handle_repeated_separators_state<'closure, L, Ctx, Lang>(
    &mut self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx::Emitter, Ctx::Cache, Lang>,
    repeated_separators_errs_id: L::Span,
    sep_tok: Spanned<Lexed<'inp, L::Token>, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
    Lang: ?Sized,
  {
    let (sep_span, sep_token) = sep_tok.into_components();
    <Ctx::Emitter as BatchEmitter<
      '_,
      L,
      UnexpectedRepeatedOf<'_, SepClassifier, L, Lang>,
      Lang,
    >>::emit_to_batch(
      inp.emitter(),
      &repeated_separators_errs_id,
      Spanned::new(
        sep_span.clone(),
        UnexpectedRepeatedOf::<'_, SepClassifier, L, Lang>::repeated_of(
          sep_span.clone(),
          sep_token.unwrap_token(),
        ),
      ),
    )?;
    // no need to change state, still in RepeatedSeparator
    Ok(repeated_separators_errs_id)
  }

  pub(in crate::parser) fn handle_leadings_state<'closure, L, Ctx, Lang>(
    &mut self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx::Emitter, Ctx::Cache, Lang>,
    leading_errs_id: L::Span,
    sep_tok: Spanned<Lexed<'inp, L::Token>, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
    Lang: ?Sized,
  {
    // we already have multiple leading separators, just emit the newly found leading separator
    let (sep_span, sep_tok) = sep_tok.into_components();
    <Ctx::Emitter as BatchEmitter<
      '_,
      L,
      UnexpectedLeadingOf<'_, SepClassifier, L, Lang>,
      Lang,
    >>::emit_to_batch(
      inp.emitter(),
      &leading_errs_id,
      Spanned::new(
        sep_span.clone(),
        UnexpectedLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(
          sep_span.clone(),
          sep_tok.unwrap_token(),
        ),
      ),
    )?;

    // no need to change state, still in leadings
    Ok(leading_errs_id)
  }

  pub(in crate::parser) fn handle_leading_state<'closure, L, Ctx, Lang>(
    &mut self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx::Emitter, Ctx::Cache, Lang>,
    tok: Spanned<L::Token, L::Span>,
    sep_tok: Spanned<Lexed<'inp, L::Token>, L::Span>,
    leading_spec: SepFixSpec,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
    Lang: ?Sized,
  {
    Ok(match leading_spec {
      SepFixSpec::Deny(_) => {
        // we are not allowed to have multiple leading separators.
        let (tok_span, tok_token) = tok.into_components();
        let (sep_span, sep_tok) = sep_tok.into_components();

        // we are not allowed to have multiple leading separators.
        // try to emit leading separator error via the emitter
        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedLeadingOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::create_batch_with_error(
          inp.emitter(),
          "leading separators".into(),
          Spanned::new(
            tok_span.clone(),
            UnexpectedLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(
              tok_span.clone(),
              tok_token,
            ),
          ),
        )?;

        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedLeadingOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::emit_to_batch(
          inp.emitter(),
          &tok_span,
          Spanned::new(
            sep_span.clone(),
            UnexpectedLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(
              sep_span.clone(),
              sep_tok.unwrap_token(),
            ),
          ),
        )?;

        // store the first leading sep span as this will be used to identify the batch later
        tok_span
      }
      SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {
        let (sep_span, sep_tok) = sep_tok.into_components();

        // we are not allowed to have multiple leading separators.
        // try to emit leading separator error via the emitter
        <Ctx::Emitter as BatchEmitter<
          '_,
          L,
          UnexpectedLeadingOf<'_, SepClassifier, L, Lang>,
          Lang,
        >>::create_batch_with_error(
          inp.emitter(),
          "leading separators".into(),
          Spanned::new(
            sep_span.clone(),
            UnexpectedLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(
              sep_span.clone(),
              sep_tok.unwrap_token(),
            ),
          ),
        )?;

        // store the first leading sep span as this will be used to identify the batch later
        sep_span
      }
    })
  }

  pub(in crate::parser) fn handle_separator_state<'closure, L, Ctx, Lang>(
    &mut self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx::Emitter, Ctx::Cache, Lang>,
    tok: Spanned<L::Token, L::Span>,
    sep_tok: Spanned<Lexed<'inp, L::Token>, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedByEmitter<'inp, O, SepClassifier, L, Lang>,
    Lang: ?Sized,
  {
    // one more repeated separator
    let (tok_span, tok_token) = tok.into_components();
    let (sep_span, sep_token) = sep_tok.into_components();

    // create a batch for repeated separator errors if not already created
    <Ctx::Emitter as BatchEmitter<
      '_,
      L,
      UnexpectedRepeatedOf<'_, SepClassifier, L, Lang>,
      Lang,
    >>::create_batch_with_error(
      inp.emitter(),
      "repeated separator".into(),
      Spanned::new(
        tok_span.clone(),
        UnexpectedRepeatedOf::<'_, SepClassifier, L, Lang>::repeated_of(
          tok_span.clone(),
          tok_token.clone(),
        ),
      ),
    )?;

    <Ctx::Emitter as BatchEmitter<
      '_,
      L,
      UnexpectedRepeatedOf<'_, SepClassifier, L, Lang>,
      Lang,
    >>::emit_to_batch(
      inp.emitter(),
      &tok_span,
      Spanned::new(
        sep_span.clone(),
        UnexpectedRepeatedOf::<'_, SepClassifier, L, Lang>::repeated_of(
          sep_span,
          sep_token.unwrap_token(),
        ),
      ),
    )?;

    Ok(tok_span)
  }
}

#[cfg_attr(not(tarpaulin), inline(always))]
fn push<C, T>(nums: &mut usize, container: &mut C, item: T) -> Option<T>
where
  C: crate::container::Container<T>,
{
  match container.push(item) {
    None => {
      *nums += 1;
      None
    }
    Some(item) => Some(item),
  }
}
