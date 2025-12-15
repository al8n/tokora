use super::*;

mod parse;

/// A parser that parses repeated elements enclosed in delimiter tokens (without separators).
///
/// This combinator wraps a [`Repeated`] parser with **opening and closing delimiters**,
/// parsing constructs like `[element element element]` or `{item item item}`.
///
/// Unlike [`DelimitedSeparatedBy`] which expects separators between elements (e.g., commas),
/// `DelimitedBy` parses **consecutive elements** with no separators between them.
///
/// # Type Parameters
///
/// - `P`: The element parser
/// - `Condition`: Decision function to determine when to stop parsing elements
/// - `Open`: Classifier for the opening delimiter (e.g., `[`, `{`, `(`)
/// - `Close`: Classifier for the closing delimiter (e.g., `]`, `}`, `)`)
/// - `Delim`: Delimiter type/marker
/// - `O`: Output type of the element parser
/// - `W`: Lookahead window size for the condition
/// - `L`: Lexer type
/// - `Ctx`: Parse context
/// - `Config`: Configuration (min/max bounds)
/// - `Lang`: Language marker type (default `()`)
///
/// # Examples
///
/// ## Basic Bracketed List
///
/// ```ignore
/// use tokit::parser::ParseInput;
/// use generic_arraydeque::typenum::U1;
///
/// // Parse: [element element element]
/// let parser = parse_element()
///     .repeated::<_, U1>(|mut peeked, _| {
///         match peeked.front() {
///             Some(Token::RBracket) | None => Ok(Action::Stop),
///             _ => Ok(Action::Continue),
///         }
///     })
///     .delimited_by(
///         |tok| matches!(tok, Token::LBracket),
///         |tok| matches!(tok, Token::RBracket),
///         Delimiter::Bracket
///     )
///     .collect::<Vec<_>>();
///
/// // Input: "[a b c]"    → Ok(vec![a, b, c])
/// // Input: "[x]"        → Ok(vec![x])
/// // Input: "[]"         → Ok(vec![])
/// ```
///
/// ## Generic Delimiters
///
/// ```ignore
/// // Parse: {token token token}
/// let parser = parse_token()
///     .repeated(stop_condition)
///     .delimited_by(
///         |tok| matches!(tok, Token::LBrace),
///         |tok| matches!(tok, Token::RBrace),
///         Delimiter::Brace
///     )
///     .collect::<Vec<_>>();
///
/// // Input: "{foo bar baz}" → Ok(vec![foo, bar, baz])
/// ```
///
/// ## Parenthesized Expressions
///
/// ```ignore
/// // Parse: (expr expr expr)
/// let parser = parse_expression()
///     .repeated(|mut peeked, _| {
///         match peeked.front() {
///             Some(Token::RParen) | None => Ok(Action::Stop),
///             _ => Ok(Action::Continue),
///         }
///     })
///     .delimited_by(
///         |tok| matches!(tok, Token::LParen),
///         |tok| matches!(tok, Token::RParen),
///         Delimiter::Paren
///     )
///     .collect::<Vec<_>>();
/// ```
///
/// ## With Bounds
///
/// ```ignore
/// // Parse 1-10 elements in brackets
/// let parser = parse_element()
///     .repeated(stop_condition)
///     .at_least(Minimum::new(1))
///     .at_most(Maximum::new(10))
///     .delimited_by(
///         |tok| matches!(tok, Token::LBracket),
///         |tok| matches!(tok, Token::RBracket),
///         Delimiter::Bracket
///     )
///     .collect::<Vec<_>>();
///
/// // Input: "[]"        → Err (too few elements)
/// // Input: "[a]"       → Ok(vec![a])
/// // Input: "[a b ... (11 total)]" → Err (too many elements)
/// ```
///
/// # How It Works
///
/// 1. **Parse opening delimiter**: Consume the left delimiter token
/// 2. **Parse elements**: Use the repeated parser to parse elements
/// 3. **Parse closing delimiter**: Consume the right delimiter token
/// 4. **Return**: Return the collected elements
///
/// # Comparison with DelimitedSeparatedBy
///
/// | Feature | `DelimitedBy` | `DelimitedSeparatedBy` |
/// |---------|---------------|------------------------|
/// | **Separators** | ❌ No separators | ✅ Elements separated (e.g., commas) |
/// | **Base Parser** | [`Repeated`] | [`SeparatedBy`] |
/// | **Example** | `[a b c]` | `[a, b, c]` |
/// | **Use Case** | Consecutive items | Separated lists |
///
/// **When to use**:
/// - `DelimitedBy`: Parse lists of consecutive elements (no separators)
/// - `DelimitedSeparatedBy`: Parse comma/semicolon-separated lists
///
/// # Performance
///
/// - **Memory**: O(1) for the parser structure
/// - **Runtime**: O(n) where n is the number of elements
/// - **Delimiter matching**: O(1) per delimiter
///
/// # See Also
///
/// - [`DelimitedSeparatedBy`] - Delimited lists with separators
/// - [`Repeated`] - The underlying repetition parser
/// - [`delimited_by`](Repeated::delimited_by) - How to create this combinator
/// - [`collect`](DelimitedBy::collect) - Collect elements into a container
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DelimitedBy<P, Open, Close, Delim, O, W, L, Ctx, Lang: ?Sized = ()> {
  parser: P,
  left_classifier: Open,
  right_classifier: Close,
  delimiter: Delim,
  _m: PhantomData<O>,
  _window: PhantomData<W>,
  _lang: PhantomData<Lang>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
}

impl<P, Open, Close, Delim, O, W, L, Ctx, Lang: ?Sized>
  DelimitedBy<P, Open, Close, Delim, O, W, L, Ctx, Lang>
{
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn collect<Container>(self) -> Collect<Self, Container, (), ()>
  where
    Container: Default,
  {
    Collect::new(self, Container::default())
  }

  /// Collects the parsed elements with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn collect_with<Container>(
    self,
    container: Container,
  ) -> Collect<Self, Container, (), ()> {
    Collect::new(self, container)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(parser: P, left: Open, right: Close, delim: Delim) -> Self {
    Self {
      parser,
      left_classifier: left,
      right_classifier: right,
      delimiter: delim,
      _m: PhantomData,
      _window: PhantomData,
      _lang: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
    }
  }
}
