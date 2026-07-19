use core::mem;

use crate::{
  container::Container as ContainerT,
  delimiter::Delimiter,
  emitter::{FullContainerEmitter, UnclosedEmitter},
  error::{Unclosed, syntax::FullContainer},
};

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

impl<'inp, L, P, O, Condition, Ctx, Delim, W, Lang: ?Sized>
  DelimitedBy<&mut RepeatedWhile<P, Condition, O, W, L, Ctx, Lang>, Delim>
{
  fn parse_repeated<Container>(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    container: &mut Container,
    on_stop: impl FnOnce(
      usize,
      &mut InputRef<'inp, '_, L, Ctx, Lang>,
      &L::Span,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Delim: Delimiter<'inp, L, Lang>,
    L: Lexer<'inp>,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: FullContainerEmitter<'inp, L, Lang> + UnclosedEmitter<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
      From<UnexpectedEot<L::Offset, Lang>> + From<Unclosed<(), L::Span, Lang>>,
    Container: Default + ContainerT<O> + DelimiterHandler<'inp, L>,
  {
    // Sync the input to the next token boundary, any lexer errors will be emitted during this process.
    let anchor = inp.cursor().clone();
    let mut first_kind = None;
    let left_delimiter = inp.try_expect(|tok| {
      let (span, tok) = tok.into_components();
      match Delim::is_open(&tok.kind()) {
        false => {
          first_kind = Some(Delim::unexpected_open_token(Spanned::new(
            span.clone(),
            tok.clone(),
          )));
          false
        }
        true => true,
      }
    })?;

    // The opener's span, captured iff an opener is actually committed. It is the anchor of
    // the `Unclosed` diagnostic below: no opener, no unclosed.
    let mut open_span: Option<L::Span> = None;
    match left_delimiter {
      None if inp.is_eoi() => {
        return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into());
      }
      None => {
        // safe unwrap as we know when left_delimiter is None, first_kind is Some
        inp.emitter().emit_unexpected_token(first_kind.unwrap())?;
      }
      Some(open) => {
        open_span = Some(open.span_ref().clone());
        container.on_open_delimiter(open);
      }
    }

    let mut nums = 0;
    let mut elem_cur = inp.cursor().clone();

    loop {
      // Probe the close position WITHOUT consuming, so a terminal scanner stop is not
      // misread as EOF (finding 1). `Close` short-circuits before the stop condition is
      // consulted, exactly as the consuming `try_expect` did.
      match inp.probe_close(|tok| Delim::is_close(&tok.data.kind()))? {
        CloseStatus::Close => {
          // Commit the closer (the probe left it unconsumed) and run the end handler.
          if let Some(closed) = inp.try_expect(|tok| Delim::is_close(&tok.data.kind()))? {
            container.on_close_delimiter(closed);
          }
          let span = inp.span_since(&anchor);
          return on_stop(nums, inp, &span).map(|_| mem::take(container));
        }
        // A terminal scanner stop: its own diagnostic already explains the halt —
        // propagate it and add no `Unclosed`.
        CloseStatus::Tripped => {
          return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into());
        }
        // The closer is absent (a wrong token or genuine EOF) — consult the stop
        // condition to decide whether another element is expected.
        close => {
          let (peeked, emitter) = inp.peek_with_emitter::<W>()?;
          match self.parser.condition.decide(peeked, emitter)? {
            // missing ending delimiter
            Action::Stop => {
              // PRIMARY — the close-miss diagnostic first: under a fail-fast emitter
              // this short-circuits, so `Unclosed` (not the secondary bounds) surfaces.
              match close {
                // (b) a wrong token sits where the closer should: unexpected-token,
                // expected-close (the existing vocabulary).
                CloseStatus::WrongToken(tok) => inp
                  .emitter()
                  .emit_unexpected_token(Delim::unexpected_close_token(tok))?,
                // (a) end of input with the opener still open: the opener was never
                // closed.
                _ => {
                  if let Some(open_span) = open_span.clone() {
                    inp
                      .emitter()
                      .emit_unclosed(Unclosed::<(), L::Span, Lang>::of(open_span, Delim::name()))?;
                  }
                }
              }
              // SECONDARY (finding 2) — the delimited driver used to return here WITHOUT
              // running the repeated end handler, silently dropping the `TooFew`/bounds
              // diagnostic under a recovering emitter (the plain `Repeated` driver runs
              // it). Run it after the primary, matching the primary-then-secondary order
              // the separated drivers established.
              let span = inp.span_since(&anchor);
              return on_stop(nums, inp, &span).map(|_| mem::take(container));
            }
            Action::Continue => {
              // TODO(al8n): tracing dropped element
              if let Err(_e) = container.push(self.parser.f.parse_input(inp)?) {
                let span = inp.span_since(&anchor);
                inp.emitter().emit_full_container(FullContainer::of(
                  span,
                  nums,
                  container.max_capacity(),
                ))?;
              }
              nums += 1;
            }
          }
        }
      }

      // The progress guard (parity with `DelimitedBy<Repeated>`): a `Continue` cycle whose
      // element parser consumed nothing would fail the same close-delimiter check and see the
      // same lookahead forever. No progress means no more elements — break to the close-
      // delimiter epilogue below, exactly as the plain `Repeated` driver does.
      let new_cursor = inp.cursor().clone();
      if new_cursor.as_inner() == elem_cur.as_inner() {
        break;
      }
      elem_cur = new_cursor;
    }

    // No progress was made — treat as end of elements (the same epilogue as
    // `DelimitedBy<Repeated>`): accept a close delimiter if it is at hand, report it
    // otherwise, then run the stop handler on the delimited span. The four-way probe
    // keeps a terminal scanner stop out of the `Unclosed` path.
    match inp.probe_close(|t| Delim::is_close(&t.data.kind()))? {
      CloseStatus::Close => {
        if let Some(close) = inp.try_expect(|t| Delim::is_close(&t.data.kind()))? {
          container.on_close_delimiter(close);
        }
      }
      CloseStatus::WrongToken(tok) => inp
        .emitter()
        .emit_unexpected_token(Delim::unexpected_close_token(tok))?,
      CloseStatus::Eof => {
        // EOI — no tokens left, no close delimiter: the opener was never closed.
        if let Some(open_span) = open_span.clone() {
          inp
            .emitter()
            .emit_unclosed(Unclosed::<(), L::Span, Lang>::of(open_span, Delim::name()))?;
        }
      }
      CloseStatus::Tripped => {
        return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into());
      }
    }

    let span = inp.span_since(&anchor);
    on_stop(nums, inp, &span).map(|_| mem::take(container))
  }
}
