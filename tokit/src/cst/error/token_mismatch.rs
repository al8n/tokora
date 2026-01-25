use super::super::{CstElement, Language};
use derive_more::{From, Into};
use rowan::SyntaxToken;

/// An error indicating a mismatch between expected and actual syntax token kinds.
///
/// This error occurs when attempting to cast a [`SyntaxToken`] to a typed [`CstToken`](crate::cst::CstToken)
/// type, but the token's kind doesn't match the expected kind for that type. This is the
/// token-equivalent of [`CstNodeMismatch`](super::CstNodeMismatch).
///
/// # Design
///
/// `CstTokenMismatch` provides:
/// - **Type information**: The expected kind that was requested
/// - **Actual token**: The original token that failed to cast
/// - **Error recovery**: Methods to extract the original token for retry
/// - **Debugging**: Clear error messages with both expected and found kinds
///
/// # Type Parameters
///
/// - `N`: The typed [`CstToken`] type that was expected
///
/// # Common Scenarios
///
/// This error typically occurs when:
/// 1. **Dynamic casting**: Attempting to cast without checking [`can_cast()`](crate::cst::CstElement::can_cast) first
/// 2. **Malformed input**: The parser produced an unexpected token sequence
/// 3. **Grammar changes**: Code expects an old token kind after grammar updates
/// 4. **Enum casting**: An enum token variant doesn't match any expected kinds
///
/// # Examples
///
/// ## Basic Error Handling
///
/// ```rust,ignore
/// use tokit::cst::{CstToken, error};
///
/// let result = Colon::try_cast_token(syntax_token);
///
/// match result {
///     Ok(colon) => {
///         // Successfully cast
///         println!("Found colon at: {:?}", colon.syntax().text_range());
///     }
///     Err(mismatch) => {
///         // Cast failed - log the error
///         eprintln!("Type mismatch: {}", mismatch);
///         eprintln!("Expected: {:?}", mismatch.expected());
///         eprintln!("Found: {:?}", mismatch.found().kind());
///         eprintln!("Text: {}", mismatch.found().text());
///     }
/// }
/// ```
///
/// ## Recovering from Errors and Retrying
///
/// ```rust,ignore
/// use tokit::cst::error::CstTokenMismatch;
///
/// // Try to cast to a comma first
/// let result = Comma::try_cast_token(syntax_token);
///
/// let separator = match result {
///     Ok(comma) => Separator::Comma(comma),
///     Err(mismatch) => {
///         // Recover the original syntax token
///         let (expected_kind, original_token) = mismatch.into_components();
///
///         // Try casting to a semicolon instead
///         match Semicolon::try_cast_token(original_token) {
///             Ok(semicolon) => Separator::Semicolon(semicolon),
///             Err(e) => return Err(e.into()),
///         }
///     }
/// };
/// ```
///
/// ## Safe Casting with Validation
///
/// ```rust,ignore
/// use tokit::cst::{CstToken, SyntaxTreeElement};
///
/// // Check before casting to avoid errors
/// let token = if Comma::can_cast(syntax_token.kind()) {
///     Comma::try_cast_token(syntax_token).unwrap()
/// } else {
///     // Handle unexpected token gracefully
///     return Err(ParseError::ExpectedComma {
///         found: syntax_token.kind(),
///         position: syntax_token.text_range(),
///     });
/// };
/// ```
///
/// ## Using in Error Propagation
///
/// ```rust,ignore
/// use tokit::cst::{CstToken, error};
///
/// fn parse_punctuation(
///     token: SyntaxToken<MyLanguage>
/// ) -> Result<Punctuation, error::CstTokenMismatch<Punctuation>> {
///     // Try casting - error automatically propagates with ?
///     let punct = Punctuation::try_cast_token(token)?;
///     Ok(punct)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct CstTokenMismatch<N, Lang: Language> {
  found: SyntaxToken<Lang>,
  _m: core::marker::PhantomData<N>,
}

impl<N, Lang: Language> core::fmt::Display for CstTokenMismatch<N, Lang>
where
  N: CstElement<Lang>,
  Lang::Kind: core::fmt::Display,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "syntax token mismatch: expected syntax token of kind {}, but found syntax token of kind {}",
      N::KIND,
      self.found.kind()
    )
  }
}

impl<N, Lang: Language> core::error::Error for CstTokenMismatch<N, Lang>
where
  N: CstElement<Lang> + core::fmt::Debug,
  Lang::Kind: core::fmt::Display,
{
}

impl<N, Lang: Language> CstTokenMismatch<N, Lang> {
  /// Creates a new syntax token mismatch error.
  ///
  /// This constructor is typically called by [`CstToken::try_cast_token()`](crate::cst::CstToken::try_cast_token)
  /// implementations when a cast fails. You rarely need to call this directly.
  ///
  /// # Arguments
  ///
  /// - `expected`: The syntax kind that was expected for type `N`
  /// - `found`: The actual syntax token that couldn't be cast
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::error::CstTokenMismatch;
  ///
  /// // Typically used in try_cast_token implementations
  /// impl CstToken for Comma {
  ///     fn try_cast_token(syntax: SyntaxToken<Self::Language>)
  ///         -> Result<Self, CstTokenMismatch<Self>>
  ///     {
  ///         if Self::can_cast(syntax.kind()) {
  ///             Ok(Self { syntax })
  ///         } else {
  ///             Err(CstTokenMismatch::new(Self::KIND, syntax))
  ///         }
  ///     }
  ///     // ...
  /// }
  /// ```
  #[inline]
  pub const fn new(found: SyntaxToken<Lang>) -> Self {
    Self {
      found,
      _m: core::marker::PhantomData,
    }
  }

  /// Returns the expected syntax token kind.
  ///
  /// This is the kind that was expected when attempting to cast to type `N`.
  /// For simple tokens, this is typically `N::KIND`. For enum tokens, this
  /// may be a marker kind representing the enum itself.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// if let Err(mismatch) = Comma::try_cast_token(token) {
  ///     println!("Expected kind: {:?}", mismatch.expected());
  ///     // Output: Expected kind: Comma
  /// }
  /// ```
  ///
  /// ## Using for Error Messages
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// match Colon::try_cast_token(token) {
  ///     Ok(colon) => { /* ... */ }
  ///     Err(e) => {
  ///         eprintln!(
  ///             "Expected {:?} but found {:?} at {:?}",
  ///             e.expected(),
  ///             e.found().kind(),
  ///             e.found().text_range()
  ///         );
  ///     }
  /// }
  /// ```
  #[inline]
  pub const fn expected(&self) -> Lang::Kind
  where
    N: CstElement<Lang>,
  {
    N::KIND
  }

  /// Returns a reference to the syntax token that was found.
  ///
  /// This provides access to the original token that failed to cast,
  /// allowing you to inspect its kind, text, position, and other properties.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// if let Err(mismatch) = Semicolon::try_cast_token(token) {
  ///     let found = mismatch.found();
  ///     println!("Found kind: {:?}", found.kind());
  ///     println!("Found text: {}", found.text());
  ///     println!("Position: {:?}", found.text_range());
  /// }
  /// ```
  ///
  /// ## Inspecting Token Context
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// match RBrace::try_cast_token(token) {
  ///     Ok(brace) => { /* ... */ }
  ///     Err(e) => {
  ///         let found = e.found();
  ///         eprintln!("Expected closing brace");
  ///         eprintln!("Found: {} at line {}", found.text(), /* line calc */);
  ///
  ///         // Show surrounding context
  ///         if let Some(parent) = found.parent() {
  ///             eprintln!("Context: {}", parent.text());
  ///         }
  ///     }
  /// }
  /// ```
  #[inline]
  pub const fn found(&self) -> &SyntaxToken<Lang> {
    &self.found
  }

  /// Consumes the error and returns the expected kind and found token.
  ///
  /// This is useful for recovering the original syntax token after a failed cast,
  /// allowing you to try casting to a different type or perform other operations
  /// on the token.
  ///
  /// # Returns
  ///
  /// A tuple of `(expected_kind, found_token)`:
  /// - `expected_kind`: The kind that was expected for type `N`
  /// - `found_token`: The original token that failed to cast
  ///
  /// # Examples
  ///
  /// ## Fallback Casting
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// // Try to cast to a comma first, then try semicolon
  /// let result = Comma::try_cast_token(syntax_token);
  ///
  /// let separator = match result {
  ///     Ok(comma) => comma,
  ///     Err(mismatch) => {
  ///         let (_, token) = mismatch.into_components();
  ///         Semicolon::try_cast_token(token)?
  ///     }
  /// };
  /// ```
  ///
  /// ## Custom Error Handling
  ///
  /// ```rust,ignore
  /// use tokit::cst::error::CstTokenMismatch;
  ///
  /// let result = Colon::try_cast_token(token);
  ///
  /// if let Err(mismatch) = result {
  ///     let (expected, found) = mismatch.into_components();
  ///
  ///     // Create a custom error with more context
  ///     return Err(ParseError::UnexpectedToken {
  ///         expected,
  ///         found_kind: found.kind(),
  ///         found_text: found.text().to_string(),
  ///         position: found.text_range(),
  ///     });
  /// }
  /// ```
  ///
  /// ## Retry Logic
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// enum Punctuation {
  ///     Comma(Comma),
  ///     Semicolon(Semicolon),
  ///     Colon(Colon),
  /// }
  ///
  /// fn try_parse_punctuation(
  ///     token: SyntaxToken<MyLanguage>
  /// ) -> Result<Punctuation, ParseError> {
  ///     // Try each variant in order
  ///     Comma::try_cast_token(token.clone())
  ///         .map(Punctuation::Comma)
  ///         .or_else(|e| {
  ///             let (_, token) = e.into_components();
  ///             Semicolon::try_cast_token(token)
  ///                 .map(Punctuation::Semicolon)
  ///         })
  ///         .or_else(|e| {
  ///             let (_, token) = e.into_components();
  ///             Colon::try_cast_token(token)
  ///                 .map(Punctuation::Colon)
  ///         })
  ///         .map_err(|e| ParseError::from(e))
  /// }
  /// ```
  #[inline]
  pub fn into_components(self) -> (Lang::Kind, SyntaxToken<Lang>)
  where
    N: CstElement<Lang>,
  {
    (N::KIND, self.found)
  }
}
