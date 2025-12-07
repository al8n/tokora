// use core::convert::identity;

// use crate::{emitter::DelimiterEmitter, error::Unclosed};

use super::*;

/// A parser that parses a construct delimited by left and right tokens.
///
/// See also: [`DelimSepSeq`]
pub struct Delimited<P, Condition, Open, Close, Delim, O, W, Config = RepeatedOptions> {
  parser: Repeated<P, Condition, O, W, Config>,
  left_classifier: Open,
  right_classifier: Close,
  delimiter: Delim,
  _m: PhantomData<O>,
  _window: PhantomData<W>,
}

impl<P, Condition, Open, Close, Delim, O, W, Options>
  Delimited<P, Condition, Open, Close, Delim, O, W, Options>
{
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn collect<Container>(self) -> Collect<Self, Container>
  where
    Container: Default,
  {
    Collect::new(self, Container::default())
  }

  /// Collects the parsed elements with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn collect_with<Container>(self, container: Container) -> Collect<Self, Container> {
    Collect::new(self, container)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(
    parser: Repeated<P, Condition, O, W, Options>,
    left: Open,
    right: Close,
    delim: Delim,
  ) -> Self {
    Self {
      parser,
      left_classifier: left,
      right_classifier: right,
      delimiter: delim,
      _m: PhantomData,
      _window: PhantomData,
    }
  }
}

impl<F, Condition, Open, Close, Delim, O, Max, Min, W>
  Delimited<F, Condition, Open, Close, Delim, O, W, RepeatedOptions<Max, Min>>
{
  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn minimum(&self) -> usize
  where
    Min: MinSpec,
  {
    Min::minimum(&self.parser.config.secondary.secondary)
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn maximum(&self) -> usize
  where
    Max: MaxSpec,
  {
    Max::maximum(&self.parser.config.secondary.primary)
  }
}

// impl<'inp, L, P, Open, Close, O, Condition, Ctx, Delim, W, Max, Min, Lang: ?Sized>
//   ParseInput<'inp, L, O, Ctx, Lang> for Delimited<P, Condition, Open, Close, Delim, With<With<Spanned<L::Token, L::Span>, Spanned<L::Token, L::Span>>, O>, W, RepeatedOptions<Max, Min>>
// where
//   L: Lexer<'inp>,
//   P: ParseInput<'inp, L, O, Ctx, Lang>,
//   Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
//   Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
//   Delim: Clone,
//   W: Window,
//   Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
//   Ctx: ParseContext<'inp, L, Lang>,
//   Ctx::Emitter: DelimiterEmitter<'inp, Delim, L, Lang>,
//   <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
//     + From<<L::Token as Token<'inp>>::Error>
//     + From<UnexpectedEot<L::Offset, Lang>>,
//   Max: super::MaxSpec,
//   Min: super::MinSpec,
// {
//   fn parse_input(
//     &mut self,
//     inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
//   ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
//   where
//     L: Lexer<'inp>,
//     Ctx: ParseContext<'inp, L, Lang>,
//   {
//     // Sync the input to the next token boundary, any lexer errors will be emitted during this process.
//     let ckp = inp.save();
//     let first = inp.sync_until_token()?;

//     let state = match first {
//       // End of input reached
//       None => return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into()),
//       Some(maybe_tok) => {
//         let ct = maybe_tok.as_maybe_ref().map(identity, |t| t.as_ref());

//         let tok = ct.token().copied().into_data();
//         match self.left_classifier.check(tok) {
//           Err(knd) => {
//             let (span, tok) = maybe_tok
//               .map(|t| t.into_token().cloned(), |t| t.into_token())
//               .into_inner()
//               .into_components();

//             Err(
//               UnexpectedToken::<_, _, _, Lang>::with_expected_of(
//                 span,
//                 Expected::one(knd),
//               )
//               .with_found(tok),
//             )
//           }
//           Ok(_) => {
//             let tok = maybe_tok
//               .map(|t| t.into_token().cloned(), |t| t.into_token())
//               .into_inner();
//             // Skip the opening delimiter
//             inp.skip_one();
//             Ok(tok)
//           }
//         }
//       }
//     };

//     // we already handled the first token above
//     let (open, (peeked, emitter)) = match state {
//       Ok(left) => (Some(left), inp.sync_until_token_then_peek_with_emitter()?),
//       Err(err) => {
//         inp.emitter().emit_unexpected_token(err)?;
//         (None, inp.sync_until_token_then_peek_with_emitter()?)
//       }
//     };

//     let elem = match self.condition.decide(peeked, emitter)? {
//       Action::End => {
//         let span = inp.span_since(ckp.cursor());
//         inp.emitter().emit_unclosed(Unclosed::of(
//           span,
//           self.delimiter.clone(),
//         ))?;
//         return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into());
//       },
//       Action::Continue => self.parser.parse_input(inp)?,
//     };

//     let close = inp.sync_until_token()?;
//     match (open, close) {
//       (None, None) => todo!(),
//       (None, Some(_)) => todo!(),
//       (Some(_), None) => {
//         let span = inp.span_since(ckp.cursor());
//         inp.emitter().emit_unclosed(Unclosed::of(
//           span,
//           self.delimiter.clone(),
//         ))?;
//         Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into())
//       },
//       (Some(open), Some(close)) => todo!(),
//     }
//   }
// }
