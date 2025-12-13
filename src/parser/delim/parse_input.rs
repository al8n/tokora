use core::{convert::identity, mem};

use mayber::Maybe::{Owned, Ref};

use crate::{
  container::DelimiterContainer,
  emitter::{DelimiterEmitter, RepeatedEmitter},
  error::{
    Unclosed, Undelimited,
    syntax::{FullContainer, TooFew, TooMany},
  },
};

use super::*;

impl<'inp, L, P, Open, Close, O, Condition, Container, Ctx, Delim, W, Max, Min, Lang: ?Sized>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    DelimitedBy<P, Condition, Open, Close, Delim, O, W, RepeatedOptions<Max, Min>>,
    Container,
    Ctx,
    Lang,
  >
where
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Delim: Clone,
  L: Lexer<'inp>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: DelimiterEmitter<'inp, Delim, L, Lang> + RepeatedEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container:
    Default + DelimiterContainer<Spanned<L::Token, L::Span>, Spanned<L::Token, L::Span>, O>,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
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
    let has_open = match state {
      Ok(left) => {
        self.container.push_open(left);
        true
      }
      Err(err) => {
        inp.emitter().emit_unexpected_token(err)?;
        false
      }
    };

    let mut nums = 0;
    let max = self.parser.maximum();
    let min = self.parser.minimum();

    loop {
      let (mut peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      if let Some(front) = peeked.front() {
        let tok = front
          .as_maybe_ref()
          .map(
            |t| t.token().data().unwrap_token_ref(),
            |t| t.as_ref().token().data().unwrap_token_ref(),
          )
          .into_inner();

        // find the ending delimiter
        if self.parser.right_classifier.check(tok).is_ok() {
          let front = peeked.pop_front().expect("just checked there is a front");
          drop(peeked);
          let close = match front {
            Ref(_) => inp
              .next()
              .expect("peeked guarantee there is a next token")
              .map_data(|t| t.unwrap_token()),
            Owned(ct) => ct.into_token().map_data(|t| t.unwrap_token()),
          };
          self.container.push_close(close);

          if min > nums {
            let span = inp.span_since(ckp.cursor());
            inp.emitter().emit_too_few(TooFew::of(span, nums, min))?;
          }

          if nums > max {
            let span = inp.span_since(ckp.cursor());
            inp.emitter().emit_too_many(TooMany::of(span, nums, max))?;
          }

          return Ok(mem::take(&mut self.container));
        }
      }

      match self.parser.parser.condition.decide(peeked, emitter) {
        Err(err) => return Err(err),
        Ok(action) => match action {
          // missing ending delimiter
          Action::Stop => {
            if has_open {
              let span = inp.span_since(ckp.cursor());
              inp
                .emitter()
                .emit_unclosed(Unclosed::of(span, self.parser.delimiter.clone()))?;
            } else {
              let span = inp.span_since(ckp.cursor());
              inp
                .emitter()
                .emit_undelimited(Undelimited::of(span, self.parser.delimiter.clone()))?;
            }

            return Ok(mem::take(&mut self.container));
          }
          Action::Continue => {
            if self
              .container
              .push(self.parser.parser.f.parse_input(inp)?)
              .is_some()
            {
              let span = inp.span_since(ckp.cursor());
              inp.emitter().emit_full_container(FullContainer::of(
                span,
                nums,
                Container::capacity(),
              ))?;
            }
            nums += 1;
          }
        },
      }
    }
  }
}
