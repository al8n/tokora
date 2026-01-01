use core::{convert::identity, mem};

use mayber::Maybe::{Owned, Ref};

use crate::{
  container::{DelimiterContainer, SeparatorsContainer},
  emitter::{DelimitedEmitter, SeparatedEmitter},
  error::{Unclosed, Undelimited},
  parser::{collect::Collect, sep::State},
};

use super::*;

impl<
  'inp,
  L,
  P,
  SepClassifier,
  Condition,
  O,
  Container,
  Ctx,
  Open,
  Close,
  Delim,
  Trailing,
  Leading,
  Max,
  Min,
  Lang: ?Sized,
  W,
> ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    DelimitedSeparatedOnCondition<
      P,
      SepClassifier,
      Condition,
      Open,
      Close,
      Delim,
      O,
      W,
      L,
      Ctx,
      SeparatedOnConditionOptions<Trailing, Leading, Max, Min>,
      Lang,
    >,
    Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  Delim: Clone,
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter:
    SeparatedEmitter<'inp, O, SepClassifier, L, Lang> + DelimitedEmitter<'inp, Delim, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default
    + DelimiterContainer<Spanned<L::Token, L::Span>, Spanned<L::Token, L::Span>, O>
    + SeparatorsContainer<Spanned<L::Token, L::Span>, O>,
  W: Window,
  Trailing: super::TrailingSpec,
  Leading: super::LeadingSpec,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.derive(inp).map(|s| s.into_data())
  }
}

impl<
  'inp,
  L,
  P,
  SepClassifier,
  Condition,
  O,
  Container,
  Ctx,
  Open,
  Close,
  Delim,
  Trailing,
  Leading,
  Max,
  Min,
  Lang: ?Sized,
  W,
> ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      DelimitedSeparatedOnCondition<
        P,
        SepClassifier,
        Condition,
        Open,
        Close,
        Delim,
        O,
        W,
        L,
        Ctx,
        SeparatedOnConditionOptions<Trailing, Leading, Max, Min>,
        Lang,
      >,
      Container,
      Ctx,
      Lang,
    >,
    PhantomSpan,
  >
where
  L: Lexer<'inp>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  Delim: Clone,
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter:
    SeparatedEmitter<'inp, O, SepClassifier, L, Lang> + DelimitedEmitter<'inp, Delim, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default
    + crate::container::Container<O>
    + DelimiterContainer<Spanned<L::Token, L::Span>, Spanned<L::Token, L::Span>, O>
    + SeparatorsContainer<Spanned<L::Token, L::Span>, O>,
  W: Window,
  Trailing: super::TrailingSpec,
  Leading: super::LeadingSpec,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self.primary.derive(inp)
  }
}

impl<
  'inp,
  P,
  SepClassifier,
  Condition,
  O,
  Container,
  Open,
  Close,
  Delim,
  Trailing,
  Leading,
  Max,
  Min,
  W,
  L,
  Ctx,
  Lang: ?Sized,
>
  Collect<
    DelimitedSeparatedOnCondition<
      P,
      SepClassifier,
      Condition,
      Open,
      Close,
      Delim,
      O,
      W,
      L,
      Ctx,
      SeparatedOnConditionOptions<Trailing, Leading, Max, Min>,
      Lang,
    >,
    Container,
    Ctx,
    Lang,
  >
{
  fn derive(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter:
      SeparatedEmitter<'inp, O, SepClassifier, L, Lang> + DelimitedEmitter<'inp, Delim, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    Delim: Clone,
    Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
    Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    SepClassifier: Check<L::Token>,
    Container: Default
      + crate::container::Container<O>
      + DelimiterContainer<Spanned<L::Token, L::Span>, Spanned<L::Token, L::Span>, O>
      + SeparatorsContainer<Spanned<L::Token, L::Span>, O>,
    W: Window,
    Trailing: super::TrailingSpec,
    Leading: super::LeadingSpec,
    Max: super::MaxSpec,
    Min: super::MinSpec,
  {
    // Sync the input to the next token boundary, any lexer errors will be emitted during this process.
    let ckp = inp.save();
    let first = inp.sync_until_token()?;

    let state = match first {
      // End of input reached
      None => return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into()),
      Some(maybe_tok) => {
        let ct = maybe_tok.as_maybe_ref().map(identity, |t| t.as_ref());

        let tok = ct.token().copied().into_data();
        match self.parser.left_classifier.check(tok) {
          Err(knd) => {
            let (span, tok) = maybe_tok
              .map(|t| t.into_token().cloned(), |t| t.into_token())
              .into_inner()
              .into_components();

            Err(
              UnexpectedToken::<_, _, _, Lang>::with_expected_of(span, Expected::one(knd))
                .with_found(tok),
            )
          }
          Ok(_) => {
            // consume the opening delimiter token
            let tok = match maybe_tok {
              Ref(_) => inp
                .next()
                .expect("peeked guarantee there is a next token")
                .map_data(|t| t.unwrap_token()),
              Owned(ct) => ct.into_token(),
            };
            Ok(tok)
          }
        }
      }
    };

    // we already handled the first token above
    let mut left_span = None;
    let has_open = match state {
      Ok(left) => {
        left_span = Some(left.span_ref().clone());
        self.container.push_open(left);
        true
      }
      Err(err) => {
        inp.emitter().emit_unexpected_token(err)?;
        false
      }
    };

    let mut state: State<L::Token, L::Span> = State::Start;
    let leading_spec = self.parser.leading();
    let mut parser = self.parser.parser.as_mut();
    let container = &mut self.container;
    let mut num_elems = 0;

    let elems_start = inp.cursor().clone();
    let (elems_span, right) = loop {
      let (peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      let peek_span = match peeked.front() {
        None => {
          drop(peeked);
          break (
            parser.handle_end(state, inp, &ckp, num_elems, container)?,
            None,
          );
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
            t if parser.sep.check(t) => {
              drop(peeked);
              state = parser.handle_separator(state, inp, leading_spec)?;

              continue;
            }
            t => match self.parser.right_classifier.check(t) {
              Ok(_) => {
                drop(peeked);

                let Ok(Some(tok)) = inp.next_token() else {
                  unreachable!("peeked guarantee there is a next token")
                };

                break (inp.span_since(&elems_start), Some(tok));
              }
              Err(_) => peek_span.clone(),
            },
          }
        }
      };

      match parser.condition.decide(peeked, emitter)? {
        Action::Stop => {
          break (
            parser.handle_end(state, inp, &ckp, num_elems, container)?,
            None,
          );
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
    };

    match right.or_else(|| match inp.peek_one() {
      None => None,
      Some(tok) => {
        let t = tok
          .as_maybe_ref()
          .map(
            |t| t.token().map(|t| *t, |t| t.unwrap_token_ref()),
            |t| t.token().map_data(|t| t.unwrap_token_ref()),
          )
          .into_inner();

        if self.parser.right_classifier.check(t.data()).is_ok() {
          inp.next().map(|t| t.map_data(|t| t.unwrap_token()))
        } else {
          None
        }
      }
    }) {
      // missing closing delimiter
      None if has_open => {
        let span = inp.span_since(ckp.cursor());
        inp
          .emitter()
          .emit_unclosed(Unclosed::of(span, self.parser.delimiter.clone()))?;
      }
      // no open and close delimiters
      None => {
        let span = inp.span_since(ckp.cursor());
        inp
          .emitter()
          .emit_undelimited(Undelimited::of(span, self.parser.delimiter.clone()))?;
      }
      Some(right) => {
        let _ = self.container.push_close(right);
      }
    }

    Ok(Spanned::new(elems_span, mem::take(&mut self.container)))
  }
}
