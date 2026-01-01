// use crate::parser::sep::{LeadingSpec, TrailingSpec};

// use super::*;

// mod parse_input;

// /// A parser that parses separator-delimited elements enclosed in delimiter tokens.
// ///
// /// This combinator wraps a [`SeparatedBy`] parser with **opening and closing delimiters**,
// /// parsing constructs like `[a, b, c]` or `{x; y; z}` where elements are separated by
// /// delimiters (commas, semicolons, etc.).
// ///
// /// Unlike [`DelimitedBy`] which parses consecutive elements without separators,
// /// `DelimitedSeparatedBy` expects **separators between elements**.
// ///
// /// # Type Parameters
// ///
// /// - `P`: The element parser
// /// - `SepClassifier`: Classifier for the separator token (e.g., comma, semicolon)
// /// - `Condition`: Decision function for when to stop parsing
// /// - `Open`: Classifier for the opening delimiter (e.g., `[`, `{`, `(`)
// /// - `Close`: Classifier for the closing delimiter (e.g., `]`, `}`, `)`)
// /// - `Delim`: Delimiter type/marker
// /// - `O`: Output type of the element parser
// /// - `W`: Lookahead window size
// /// - `L`: Lexer type
// /// - `Ctx`: Parse context
// /// - `Options`: Configuration (trailing/leading separators, min/max bounds)
// /// - `Lang`: Language marker type (default `()`)
// ///
// /// # Examples
// ///
// /// ## Comma-Separated List in Brackets
// ///
// /// ```ignore
// /// use tokit::parser::ParseInput;
// /// use generic_arraydeque::typenum::U1;
// ///
// /// // Parse: [a, b, c]
// /// let parser = parse_element()
// ///     .separated_by(
// ///         |tok| matches!(tok, Token::Comma),
// ///         |mut peeked, _| {
// ///             match peeked.front() {
// ///                 Some(Token::RBracket) | None => Ok(Action::Stop),
// ///                 _ => Ok(Action::Continue),
// ///             }
// ///         }
// ///     )
// ///     .delimited_by(
// ///         |tok| matches!(tok, Token::LBracket),
// ///         |tok| matches!(tok, Token::RBracket),
// ///         Delimiter::Bracket
// ///     )
// ///     .collect::<Vec<_>>();
// ///
// /// // Input: "[a, b, c]"    → Ok(vec![a, b, c])
// /// // Input: "[x]"          → Ok(vec![x])
// /// // Input: "[]"           → Ok(vec![])
// /// ```
// ///
// /// ## With Trailing Comma
// ///
// /// ```ignore
// /// // Parse: [a, b, c,]  (trailing comma allowed)
// /// let parser = parse_element()
// ///     .separated_by(
// ///         |tok| matches!(tok, Token::Comma),
// ///         stop_condition
// ///     )
// ///     .allow_trailing()
// ///     .delimited_by(
// ///         |tok| matches!(tok, Token::LBracket),
// ///         |tok| matches!(tok, Token::RBracket),
// ///         Delimiter::Bracket
// ///     )
// ///     .collect::<Vec<_>>();
// ///
// /// // Input: "[a, b, c,]" → Ok(vec![a, b, c])
// /// // Input: "[a, b, c]"  → Ok(vec![a, b, c])
// /// ```
// ///
// /// ## Semicolon-Separated in Braces
// ///
// /// ```ignore
// /// // Parse: {stmt; stmt; stmt}
// /// let parser = parse_statement()
// ///     .separated_by(
// ///         |tok| matches!(tok, Token::Semicolon),
// ///         |mut peeked, _| {
// ///             match peeked.front() {
// ///                 Some(Token::RBrace) | None => Ok(Action::Stop),
// ///                 _ => Ok(Action::Continue),
// ///             }
// ///         }
// ///     )
// ///     .delimited_by(
// ///         |tok| matches!(tok, Token::LBrace),
// ///         |tok| matches!(tok, Token::RBrace),
// ///         Delimiter::Brace
// ///     )
// ///     .collect::<Vec<_>>();
// /// ```
// ///
// /// ## Function Arguments
// ///
// /// ```ignore
// /// // Parse: (arg, arg, arg)
// /// let parser = parse_argument()
// ///     .separated_by(
// ///         |tok| matches!(tok, Token::Comma),
// ///         |mut peeked, _| {
// ///             match peeked.front() {
// ///                 Some(Token::RParen) | None => Ok(Action::Stop),
// ///                 _ => Ok(Action::Continue),
// ///             }
// ///         }
// ///     )
// ///     .delimited_by(
// ///         |tok| matches!(tok, Token::LParen),
// ///         |tok| matches!(tok, Token::RParen),
// ///         Delimiter::Paren
// ///     )
// ///     .collect::<Vec<_>>();
// ///
// /// // Input: "(a, b, c)" → Ok(vec![a, b, c])
// /// // Input: "(x)"       → Ok(vec![x])
// /// // Input: "()"        → Ok(vec![])
// /// ```
// ///
// /// ## With Leading Separator
// ///
// /// ```ignore
// /// // Parse: [,a,b,c]  (leading comma allowed - unusual but possible)
// /// let parser = parse_element()
// ///     .separated_by(sep, stop)
// ///     .allow_leading()
// ///     .delimited_by(left, right, delim)
// ///     .collect::<Vec<_>>();
// /// ```
// ///
// /// # How It Works
// ///
// /// 1. **Parse opening delimiter**: Consume the left delimiter token
// /// 2. **Parse separated elements**: Use the SeparatedBy parser
// /// 3. **Parse closing delimiter**: Consume the right delimiter token
// /// 4. **Return**: Return the collected elements
// ///
// /// # Comparison with DelimitedBy
// ///
// /// | Feature | `DelimitedBy` | `DelimitedSeparatedBy` |
// /// |---------|---------------|------------------------|
// /// | **Separators** | ❌ No separators | ✅ Elements separated |
// /// | **Base Parser** | [`RepeatedOnCondition`] | [`SeparatedBy`] |
// /// | **Example** | `[a b c]` | `[a, b, c]` |
// /// | **Config** | Min/max only | Trailing/leading + min/max |
// ///
// /// **When to use**:
// /// - `DelimitedBy`: Parse consecutive elements (e.g., `[a b c]`)
// /// - `DelimitedSeparatedBy`: Parse separated lists (e.g., `[a, b, c]`)
// ///
// /// # Performance
// ///
// /// - **Memory**: O(1) for the parser structure
// /// - **Runtime**: O(n) where n is the number of elements
// /// - **Separator matching**: O(1) per separator
// /// - **Delimiter matching**: O(1) per delimiter
// ///
// /// # See Also
// ///
// /// - [`DelimitedBy`] - Delimited lists without separators
// /// - [`SeparatedBy`] - The underlying separator parser
// /// - [`delimited_by`](SeparatedBy::delimited_by) - How to create this combinator
// /// - [`collect`](DelimitedSeparatedBy::collect) - Collect elements into a container
// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
// pub struct DelimitedSeparatedBy<
//   P,
//   SepClassifier,
//   Condition,
//   Open,
//   Close,
//   Delim,
//   O,
//   W,
//   L,
//   Ctx,
//   Lang: ?Sized = (),
// > {
//   parser: SeparatedBy<P, SepClassifier, Condition, O, W, L, Ctx, Lang>,
//   left_classifier: Open,
//   right_classifier: Close,
//   delimiter: Delim,
//   _m: PhantomData<O>,
//   _window: PhantomData<W>,
// }

// impl<
//   P,
//   SepClassifier,
//   Condition,
//   Open,
//   Close,
//   Delim,
//   O,
//   Trailing,
//   Leading,
//   Max,
//   Min,
//   Window,
//   L,
//   Ctx,
//   Lang: ?Sized,
// >
//   DelimitedSeparatedBy<
//     P,
//     SepClassifier,
//     Condition,
//     Open,
//     Close,
//     Delim,
//     O,
//     Window,
//     L,
//     Ctx,
//     SeparatedByOptions<Trailing, Leading, Max, Min>,
//     Lang,
//   >
// {
//   /// Returns the specification for leading separators.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   #[allow(private_bounds)]
//   pub fn leading(&self) -> SepFixSpec
//   where
//     Leading: LeadingSpec,
//   {
//     self.parser.leading()
//   }

//   /// Returns the specification for trailing separators.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   #[allow(private_bounds)]
//   pub fn trailing(&self) -> SepFixSpec
//   where
//     Trailing: TrailingSpec,
//   {
//     self.parser.trailing()
//   }

//   /// Returns the minimum number of elements required.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   #[allow(private_bounds)]
//   pub fn minimum(&self) -> usize
//   where
//     Min: MinSpec,
//   {
//     self.parser.minimum()
//   }

//   /// Returns the maximum number of elements allowed.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   #[allow(private_bounds)]
//   pub fn maximum(&self) -> usize
//   where
//     Max: MaxSpec,
//   {
//     self.parser.maximum()
//   }
// }

// impl<P, SepClassifier, Condition, Open, Close, Delim, O, W, L, Ctx, Options, Lang: ?Sized>
//   DelimitedSeparatedBy<P, SepClassifier, Condition, Open, Close, Delim, O, W, L, Ctx, Options, Lang>
// {
//   /// Collects the parsed elements into the specified container.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub fn collect<Container>(self) -> Collect<Self, Container, Ctx, Lang>
//   where
//     Container: Default,
//   {
//     Collect::new(self, Container::default())
//   }

//   /// Collects the parsed elements with the given container.
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub const fn collect_with<Container>(
//     self,
//     container: Container,
//   ) -> Collect<Self, Container, Ctx, Lang> {
//     Collect::new(self, container)
//   }

//   #[cfg_attr(not(tarpaulin), inline(always))]
//   pub(super) const fn new_in(
//     parser: SeparatedBy<P, SepClassifier, Condition, O, W, L, Ctx, Options, Lang>,
//     left: Open,
//     right: Close,
//     delim: Delim,
//   ) -> Self {
//     Self {
//       parser,
//       left_classifier: left,
//       right_classifier: right,
//       delimiter: delim,
//       _m: PhantomData,
//       _window: PhantomData,
//     }
//   }
// }
