use crate::{TryParseInput, try_parse_input::ParseAttempt};

use super::*;

/// A combinator that peeks ahead before applying a parser, enabling conditional parsing.
///
/// This combinator looks ahead at the input using a fixed window size, then:
/// 1. **Calls a handler** with the peeked tokens
/// 2. **If handler returns `Ok(())`**: applies the inner parser
/// 3. **If handler returns `Err(e)`**: stops without parsing and returns the error
///
/// Unlike [`PeekThenChoice`] which chooses between multiple parsers, `PeekThen` makes a
/// **binary decision**: parse or don't parse. It's useful for:
/// - **Validation**: Check conditions before committing to parsing
/// - **Early rejection**: Fail fast if lookahead shows incompatible input
/// - **Contextual parsing**: Parse only when specific tokens are present
///
/// # Type Parameters
///
/// - `P`: The inner parser to apply if the condition passes
/// - `D`: Handler function that inspects lookahead and returns `Ok(())` or `Err(...)`
/// - `T`: Token type from the lexer
/// - `Window`: Lookahead window size (e.g., `typenum::U1`, `U2`, etc.)
///
/// # Examples
///
/// ## Basic Conditional Parsing
///
/// ```ignore
/// use tokit::parser::{ParseInput, Action};
/// use generic_arraydeque::typenum::U1;
///
/// // Only parse identifier if it doesn't start with underscore
/// let parser = identifier_parser()
///     .peek_then::<_, U1>(|mut peeked, _emitter| {
///         match peeked.front() {
///             Some(Token::Identifier(name)) if !name.starts_with('_') => Ok(()),
///             _ => Err(InvalidIdentifierError::new()),
///         }
///     });
/// ```
///
/// ## Multi-Token Validation
///
/// ```ignore
/// use generic_arraydeque::typenum::U2;
///
/// // Parse function only if next two tokens are "fn" and an identifier
/// let parser = function_parser()
///     .peek_then::<_, U2>(|mut peeked, _| {
///         let tok1 = peeked.get(0);
///         let tok2 = peeked.get(1);
///
///         match (tok1, tok2) {
///             (Some(Token::Fn), Some(Token::Identifier(_))) => Ok(()),
///             _ => Err(ExpectedFunctionError::new()),
///         }
///     });
/// ```
///
/// ## Optional Parsing with `or_not`
///
/// ```ignore
/// use generic_arraydeque::typenum::U1;
///
/// // Parse optional visibility modifier
/// let parser = visibility_parser()
///     .peek_then::<_, U1>(|mut peeked, _| {
///         match peeked.front() {
///             Some(Token::Pub) => Ok(()),
///             _ => Err(()),  // Not an error, just skip
///         }
///     })
///     .or_not();  // Converts to Option<Visibility>
///
/// // Input: "pub fn" → Ok(Some(Visibility::Public))
/// // Input: "fn"     → Ok(None)
/// ```
///
/// ## Context-Aware Parsing
///
/// ```ignore
/// // In a context where we only want numbers
/// let parser = value_parser()
///     .peek_then::<_, U1>(|mut peeked, _| {
///         match peeked.front() {
///             Some(Token::Number(_)) => Ok(()),
///             Some(other) => Err(UnexpectedToken::new(other.kind())),
///             None => Err(UnexpectedEot::new()),
///         }
///     });
/// ```
///
/// # Difference from `PeekThenChoice`
///
/// | Feature | `PeekThen` | `PeekThenChoice` |
/// |---------|-----------|------------------|
/// | **Decision** | Binary (parse or error) | N-way (which parser to use) |
/// | **Input Parser** | Single parser | Tuple of parsers |
/// | **Handler Returns** | `Result<(), E>` | `Result<Id, E>` |
/// | **Use Case** | Validation, filtering | Alternative parsers |
///
/// # Performance
///
/// - **Lookahead cost**: O(W) where W is the window size
/// - **No backtracking**: Parser runs at most once
/// - **Stack allocation**: Lookahead window lives on the stack
///
/// # See Also
///
/// - [`PeekThenChoice`] - Choose between multiple parsers based on lookahead
/// - [`or_not`](PeekThen::or_not) - Convert to optional parsing (returns `Option<T>`)
/// - [`filter`](crate::parser::Filter) - Validate after parsing (no lookahead)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeekThen<P, D, T, Window> {
  parser: P,
  handler: D,
  _token: PhantomData<T>,
  _capacity: PhantomData<Window>,
}

impl<P, D, T, Window> Apply<OrNot<Self>> for PeekThen<P, D, T, Window> {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, _: Self::Options) -> OrNot<Self> {
    OrNot::new(self)
  }
}

impl<P, D, T, W: Window> PeekThen<P, D, T, W> {
  /// Creates a new `PeekThen` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn of<'inp, L, O, Ctx, Lang>(parser: P, condition: D) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    Lang: ?Sized,
  {
    Self {
      parser,
      handler: condition,
      _token: PhantomData,
      _capacity: PhantomData,
    }
  }
}

impl<'inp, P, D, L, O, Ctx, Lang, W> ParseInput<'inp, L, O, Ctx, Lang>
  for PeekThen<P, D, L::Token, W>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  D: FnMut(
    Peeked<'_, 'inp, L, W>,
    &mut Ctx::Emitter,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  W: Window,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let (output, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;
    (self.handler)(output, emitter).and_then(|_| self.parser.parse_input(inp))
  }
}

impl<'inp, P, D, L, O, Ctx, Lang, W> TryParseInput<'inp, L, O, Ctx, Lang>
  for PeekThen<P, D, L::Token, W>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  D: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  W: Window,
{
  fn try_parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let (output, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

    if output.is_empty() {
      return Ok(ParseAttempt::Decline);
    }

    self
      .handler
      .decide(output, emitter)
      .and_then(|val| match val {
        Action::Continue => self.parser.parse_input(inp).map(ParseAttempt::Accept),
        Action::Stop => Ok(ParseAttempt::Decline),
      })
  }
}
