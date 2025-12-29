/// A combinator that makes a parser optional, returning `Option<T>` instead of `T`.
///
/// This wraps a parser that returns `T` and converts it to return `Option<T>`, where:
/// - `Ok(Some(value))` if the inner parser succeeds
/// - `Ok(None)` if the inner parser fails or condition returns `None`
///
/// Unlike traditional `.or()` combinators that backtrack, `OrNot` uses **lookahead-based
/// decisions** via [`peek_then_choice_or_not`](crate::parser::ParseChoice::peek_then_choice_or_not)
/// to determine whether to parse or skip.
///
/// # Type Parameters
///
/// - `P`: The inner parser
///
/// # Examples
///
/// ## Basic Optional Parsing
///
/// ```ignore
/// use tokit::parser::ParseInput;
///
/// // Parse an optional sign before a number
/// let parser = parse_sign()
///     .or_not()           // Returns Option<Sign>
///     .then(parse_number());
///
/// // Input: "+123"  → Ok((Some(Plus), 123))
/// // Input: "123"   → Ok((None, 123))
/// ```
///
/// ## With Lookahead Decision
///
/// ```ignore
/// use generic_arraydeque::typenum::U1;
///
/// // Parse optional 'mut' keyword
/// let parser = (parse_mut_keyword(),)
///     .peek_then_choice_or_not::<_, U1>(|mut peeked, _| {
///         match peeked.front() {
///             Some(Token::Mut) => Ok(Some(0)),  // Parse it
///             _ => Ok(None),                    // Skip it
///         }
///     });
///
/// // Returns OrNot<...> which outputs Option<MutKeyword>
/// ```
///
/// ## Default Values
///
/// ```ignore
/// // Parse optional delimiter, default to comma
/// let delimiter = parse_delimiter()
///     .or_not()
///     .map(|opt| opt.unwrap_or(Delimiter::Comma));
/// ```
///
/// ## Chaining with Other Combinators
///
/// ```ignore
/// // Parse: `pub`? `fn` name
/// let parser = parse_pub()
///     .or_not()                        // Option<Pub>
///     .then_ignore(parse_fn_keyword())
///     .then(parse_identifier())
///     .map(|(pub_kw, name)| FnDecl {
///         is_public: pub_kw.is_some(),
///         name,
///     });
/// ```
///
/// # When to Use
///
/// - **Optional elements**: Elements that may or may not appear
/// - **Default values**: Elements with fallback values when absent
/// - **Conditional parsing**: Parse different structures based on presence
///
/// **vs `.or()`**: Traditional backtracking `.or()` tries each alternative sequentially.
/// `OrNot` uses lookahead to make a single upfront decision.
///
/// # See Also
///
/// - [`unwrap`](crate::parser::ParseInputUnwrapExt::unwrap) - Convert `Option<T>` back to `T` (panics on None)
/// - [`peek_then_choice_or_not`](crate::parser::ParseChoice::peek_then_choice_or_not) - Creates OrNot with lookahead
/// - [`filter_map`](crate::parser::FilterMap) - Transform and optionally filter
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrNot<P>(pub(super) P);

impl<P> OrNot<P> {
  /// Creates a new `OrNot` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: P) -> Self {
    Self(parser)
  }
}
