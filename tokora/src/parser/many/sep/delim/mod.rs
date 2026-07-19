use core::mem;

use crate::{
  TryParseInput,
  container::Container as ContainerT,
  delimiter::Delimiter,
  emitter::{FullContainerEmitter, SeparatedEmitter, UnclosedEmitter},
  error::Unclosed,
  punct::Punctuator,
  try_parse_input::{Accept, Decline},
};

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

mod allow_leading;
mod allow_leading_require_trailing;
mod allow_surrounded;
mod allow_trailing;

mod require_leading;
mod require_leading_allow_trailing;
mod require_surrounded;
mod require_trailing;

impl<'inp, L, P, Sep, O, Ctx, Delim, Lang: ?Sized, Cmpl>
  DelimitedBy<Separated<&mut P, Sep, O, L, Ctx, Lang, Cmpl>, Delim>
{
  fn parse_separated<'closure, Container, CH, SP, EH>(
    &mut self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
    container: &mut Container,
    continue_state_handler: &CH,
    separator_state_handler: &SP,
    end_state_handler: &EH,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: crate::input::SurfaceIncomplete<'inp, L, Ctx, Lang>,
    Delim: Delimiter<'inp, L, Lang>,
    Sep: Punctuator<'inp, L, Lang>,
    L: Lexer<'inp>,
    P: TryParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
    Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
      + FullContainerEmitter<'inp, L, Lang>
      + UnclosedEmitter<'inp, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: crate::input::SurfaceIncomplete<'inp, L, Ctx, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
      From<UnexpectedEot<L::Offset, Lang>> + From<Unclosed<(), L::Span, Lang>>,
    Container: DelimiterHandler<'inp, L> + SeparatorHandler<'inp, L> + ContainerT<O>,
    EH: EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang, Cmpl>,
    CH: ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang, Cmpl>,
    SP: SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang, Cmpl>,
  {
    trace_event!(inp, "separated");
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

    let mut state: State<L::Token, L::Span> = State::Start;
    let parser = &mut self.parser;
    let mut num_elems = 0;

    let elems_start = inp.cursor().clone();
    let mut cursor = elems_start.clone();
    let (elems_span, right) = loop {
      let mut ps = None;
      let peek_span = match inp.try_expect_map(|t| {
        if Sep::eval(&t.data.kind()) {
          Some(false)
        } else {
          match Delim::is_close(&t.data.kind()) {
            true => Some(true),
            false => {
              ps = Some(t.span().clone());
              None
            }
          }
        }
      })? {
        None => match ps {
          None => {
            break (
              parser.handle_end(state, inp, &anchor, num_elems, end_state_handler)?,
              None,
            );
          }
          Some(span) => span,
        },
        Some((is_closed, tok)) => {
          if is_closed {
            break (inp.span_since(&elems_start), Some(tok));
          } else {
            state = parser.handle_separator(state, inp, container, separator_state_handler, tok)?;
            cursor = inp.cursor().clone();
            continue;
          }
        }
      };

      match parser.f.try_parse_input(inp) {
        // The never-recoverable gate (0.3.0): a frontier `Incomplete` from the element
        // parser re-raises untouched — never spent as a diagnostic. Constant-false under
        // `Complete`.
        Err(e) if Cmpl::is_incomplete_error(&e) => return Err(e),
        Err(e) => {
          let span = inp.span_since(&cursor);
          inp.emitter().emit_error(Spanned::new(span, e))?;
        }
        Ok(Decline) => {
          break (
            parser.handle_end(state, inp, &anchor, num_elems, end_state_handler)?,
            None,
          );
        }
        Ok(Accept(elem)) => {
          // if the peeked token belongs to an element, check the current state
          state = parser.handle_continue(
            state,
            inp,
            &anchor,
            peek_span,
            elem,
            &mut num_elems,
            container,
            continue_state_handler,
          )?;
        }
      }

      let new_cursor = inp.cursor().clone();
      if new_cursor.as_inner() == cursor.as_inner() {
        break (
          parser.handle_end(state, inp, &anchor, num_elems, end_state_handler)?,
          None,
        );
      }
      cursor = new_cursor;
    };

    // A FRESH close-miss token, scanned here rather than carried out of the loop: the loop's
    // last non-sep/non-close token must not masquerade as the wrong closer. `Some` ⇒ a real
    // wrong token sits here; `None` ⇒ end of input.
    let mut close_err = None;
    let right = match right {
      Some(tok) => Some(tok),
      None => inp.try_expect(|tok| match Delim::is_close(&tok.data.kind()) {
        true => true,
        false => {
          close_err = Some(Delim::unexpected_close_token(tok.cloned()));
          false
        }
      })?,
    };

    match right {
      // (b) a wrong token sits where the closer should: unexpected-token, expected-close.
      None if close_err.is_some() => {
        inp.emitter().emit_unexpected_token(close_err.unwrap())?;
      }
      // (a) EOI with the opener still open: the opener was never closed.
      None => {
        if let Some(open_span) = open_span.clone() {
          inp
            .emitter()
            .emit_unclosed(Unclosed::<(), L::Span, Lang>::of(open_span, Delim::name()))?;
        }
      }
      Some(right) => {
        container.on_close_delimiter(right);
      }
    }

    Ok(elems_span)
  }
}
