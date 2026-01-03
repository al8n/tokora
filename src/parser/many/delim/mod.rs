use super::*;

mod repeated;
mod repeated_while;

/// A handler for delimiter events during parsing.
pub trait DelimiterHandler<'inp, L> {
  /// Called when a delimiter is encountered.
  fn on_open_delimiter(&mut self, open: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>;

  /// Called when a closing delimiter is encountered.
  fn on_close_delimiter(&mut self, close: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>;
}

impl<'inp, L, T> DelimiterHandler<'inp, L> for &mut T
where
  T: ?Sized + DelimiterHandler<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_open_delimiter(&mut self, open: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>,
  {
    (**self).on_open_delimiter(open);
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_close_delimiter(&mut self, close: Spanned<L::Token, L::Span>)
  where
    L: Lexer<'inp>,
  {
    (**self).on_close_delimiter(close);
  }
}

macro_rules! blackhole {
  ($ty:ty) => {
    impl<'inp, L> DelimiterHandler<'inp, L> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
      where
        L: Lexer<'inp>,
      {
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
      where
        L: Lexer<'inp>,
      {
      }
    }
  };
  (@generic $ty:ty) => {
    impl<'inp, L, T> DelimiterHandler<'inp, L> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
      where
        L: Lexer<'inp>,
      {
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
      where
        L: Lexer<'inp>,
      {
      }
    }
  };
}

blackhole!(());
blackhole!(@generic core::marker::PhantomData<T>);
blackhole!(@generic crate::utils::marker::Ignored<T>);

#[cfg(any(feature = "alloc", feature = "std"))]
const _: () = {
  use std::{collections::vec_deque::VecDeque, vec::Vec};

  impl<'inp, L, T> DelimiterHandler<'inp, L> for Vec<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  impl<'inp, L, T> DelimiterHandler<'inp, L> for VecDeque<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }

  #[cfg(feature = "smallvec")]
  impl<'inp, L, T, N> DelimiterHandler<'inp, L> for smallvec::SmallVec<N>
  where
    N: smallvec::Array<Item = T>,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_open_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn on_close_delimiter(&mut self, _: Spanned<<L>::Token, <L>::Span>)
    where
      L: Lexer<'inp>,
    {
    }
  }
};

/// A parser that parses repeated elements enclosed in delimiter tokens (without separators).
///
/// This combinator wraps a [`RepeatedWhile`] parser with **opening and closing delimiters**,
/// parsing constructs like `[element element element]` or `{item item item}`.
///
/// Unlike separated sequences which expect separators between elements (e.g., commas),
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
/// # Comparison with DelimitedSeparatedWhile
///
/// | Feature | `DelimitedBy` | `DelimitedSeparatedWhile` |
/// |---------|---------------|------------------------|
/// | **Separators** | ❌ No separators | ✅ Elements separated (e.g., commas) |
/// | **Base Parser** | [`RepeatedWhile`] | [`SeparatedWhile`] |
/// | **Example** | `[a b c]` | `[a, b, c]` |
/// | **Use Case** | Consecutive items | Separated lists |
///
/// **When to use**:
/// - `DelimitedBy`: Parse lists of consecutive elements (no separators)
/// - `DelimitedSeparatedWhile`: Parse comma/semicolon-separated lists
///
/// # Performance
///
/// - **Memory**: O(1) for the parser structure
/// - **Runtime**: O(n) where n is the number of elements
/// - **Delimiter matching**: O(1) per delimiter
///
/// # See Also
///
/// - [`RepeatedWhile`] - The underlying repetition parser
/// - [`delimited_by`](RepeatedWhile::delimited_by) - How to create this combinator
/// - [`collect`](DelimitedBy::collect) - Collect elements into a container
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DelimitedBy<P, Open, Close, Delim> {
  pub(crate) parser: P,
  pub(crate) left_classifier: Open,
  pub(crate) right_classifier: Close,
  pub(crate) delimiter: Delim,
}

impl<P, Open, Close, Delim> DelimitedBy<P, Open, Close, Delim> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(parser: P, left: Open, right: Close, delim: Delim) -> Self {
    Self {
      parser,
      left_classifier: left,
      right_classifier: right,
      delimiter: delim,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) fn map_parser_mut<'a, Q, F>(
    &'a mut self,
    f: F,
  ) -> DelimitedBy<Q, &'a Open, &'a Close, &'a Delim>
  where
    F: FnOnce(&'a mut P) -> Q,
    Q: 'a,
  {
    DelimitedBy {
      parser: f(&mut self.parser),
      left_classifier: &self.left_classifier,
      right_classifier: &self.right_classifier,
      delimiter: &self.delimiter,
    }
  }
}
