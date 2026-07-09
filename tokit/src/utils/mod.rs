pub use delimited::*;
pub use escaped::*;
pub use expected::*;
pub use generic_arraydeque::GenericArrayDeque;
pub use lexeme::*;

pub use mayber::{Maybe, MaybeMut, MaybeRef, Owned, Ref};
pub use message::CowStr;
pub use oneof::OneOf;
pub use positioned_char::*;

pub use to_equivalent::*;

/// Re-export of generic-arraydeque for direct access.
pub use generic_arraydeque::{self, typenum};

/// A module for custom comparing traits.
pub mod cmp;
/// A module for displaying in a human-friendly way.
pub mod human_display;
/// A module for displaying in SDL.
pub mod sdl_display;
/// A module for displaying in syntax trees.
pub mod syntax_tree_display;

/// Common knowledge types for lexing and parsing.
pub mod knowledge;

/// Marker types used in various utilities.
pub mod marker;

mod delimited;
mod escaped;
mod expected;
mod lexeme;
mod message;
mod oneof;
mod positioned_char;
mod to_equivalent;

/// Enables destructuring a parsed element into its constituent components.
///
/// This trait provides a way to break down complex parsed elements into their
/// individual parts, taking ownership of each component. This is particularly
/// useful for transformation, analysis, or when building different representations
/// of the parsed data.
///
/// ## Design Philosophy
///
/// The trait uses an associated type rather than generic parameters to ensure
/// that each implementing type has exactly one way to be decomposed. This provides
/// type safety and makes the interface predictable for consumers.
///
/// ## Usage Patterns
///
/// Common scenarios for using this trait:
/// - **AST transformation**: Converting parsed elements into different AST representations
/// - **Analysis**: Extracting specific components for validation or processing
/// - **Serialization**: Breaking down elements for custom serialization formats
/// - **Testing**: Accessing individual components for detailed assertions
///
/// ## Examples
///
/// ```rust,ignore
/// // Extracting components for transformation
/// let float_value: FloatValue<&str, SimpleSpan> = parse_float("3.14e-2")?;
/// let (span, int_part, frac_part, exp_part) = float_value.into_components();
///
/// // Building a custom representation
/// let custom_float = CustomFloat {
///     location: span,
///     integer: int_part,
///     fractional: frac_part,
///     exponent: exp_part,
/// };
///
/// // Component analysis
/// let int_literal: IntValue<&str, SimpleSpan> = parse_int("-42")?;
/// let (span, sign, digits) = int_literal.into_components();
///
/// if sign.is_some() {
///     println!("Found negative integer at {:?}", span);
/// }
/// ```
///
/// ## Implementation Guidelines
///
/// When implementing this trait:
/// - Include all meaningful components of the parsed element
/// - Order components logically (typically: span first, then sub-components in source order)
/// - Use tuples for simple decomposition, custom structs for complex cases
/// - Ensure the decomposition is complete (no information loss)
/// - Document the component structure clearly
///
/// ## Component Ordering Convention
///
/// To maintain consistency across implementations, follow this ordering:
/// 1. **Overall span**: The span covering the entire element
/// 2. **Required components**: Core parts that are always present
/// 3. **Optional components**: Parts that may or may not be present
/// 4. **Sub-elements**: Nested parsed elements in source order
pub trait IntoComponents {
  /// The tuple or struct type containing the decomposed components.
  ///
  /// This associated type defines the structure returned by `into_components()`.
  /// It should include all meaningful parts of the parsed element in a logical
  /// order that makes sense for the specific element type.
  type Components;

  /// Consumes this element and returns its constituent components.
  ///
  /// This method breaks down the parsed element into its individual parts,
  /// providing owned access to each component. The exact structure of the
  /// returned components is defined by the `Components` associated type.
  fn into_components(self) -> Self::Components;
}

/// A trait for checking if a token is an ASCII character.
pub trait IsAsciiChar {
  /// Returns `true` if self is equal to the given ASCII character.
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool;

  /// Checks if the value is an ASCII decimal digit:
  /// U+0030 '0' ..= U+0039 '9'.
  fn is_ascii_digit(&self) -> bool;

  /// Returns `true` if self is one of the given ASCII characters.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn one_of(&self, choices: &[ascii::AsciiChar]) -> bool {
    choices.iter().any(|&ch| self.is_ascii_char(ch))
  }
}

impl<T> IsAsciiChar for &T
where
  T: IsAsciiChar + ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <T as IsAsciiChar>::is_ascii_char(*self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <T as IsAsciiChar>::is_ascii_digit(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn one_of(&self, choices: &[ascii::AsciiChar]) -> bool {
    <T as IsAsciiChar>::one_of(*self, choices)
  }
}

impl<T> IsAsciiChar for &mut T
where
  T: IsAsciiChar + ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <T as IsAsciiChar>::is_ascii_char(*self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <T as IsAsciiChar>::is_ascii_digit(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn one_of(&self, choices: &[ascii::AsciiChar]) -> bool {
    <T as IsAsciiChar>::one_of(*self, choices)
  }
}

impl IsAsciiChar for char {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    if self.is_ascii() {
      *self as u8 == ch as u8
    } else {
      false
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    char::is_ascii_digit(self)
  }
}

impl IsAsciiChar for u8 {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    *self == ch as u8
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    u8::is_ascii_digit(self)
  }
}

impl IsAsciiChar for str {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    self.len() == 1 && self.as_bytes()[0] == ch as u8
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    self.len() == 1 && self.as_bytes()[0].is_ascii_digit()
  }
}

impl IsAsciiChar for [u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    self.len() == 1 && self[0] == ch as u8
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    self.len() == 1 && self[0].is_ascii_digit()
  }
}

#[cfg(feature = "bstr_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bstr_1")))]
impl IsAsciiChar for bstr_1::BStr {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_digit(self)
  }
}

#[cfg(feature = "bytes_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl IsAsciiChar for bytes_1::Bytes {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_digit(self)
  }
}

#[cfg(feature = "hipstr_0_8")]
#[cfg_attr(docsrs, doc(cfg(feature = "hipstr_0_8")))]
impl IsAsciiChar for hipstr_0_8::HipByt<'_> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_digit(self)
  }
}

#[cfg(feature = "hipstr_0_8")]
#[cfg_attr(docsrs, doc(cfg(feature = "hipstr_0_8")))]
impl IsAsciiChar for hipstr_0_8::HipStr<'_> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <str as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <str as IsAsciiChar>::is_ascii_digit(self)
  }
}

/// A trait for character-like types that can report their encoded length in bytes.
///
/// `CharLen` provides a uniform way to query the byte length of different character
/// types, which is essential for converting positioned characters into byte spans.
///
/// # Implementations
///
/// tokit provides implementations for:
/// - **`u8`**: Always returns `1` (single byte)
/// - **`char`**: Returns `len_utf8()` (1-4 bytes depending on the character)
/// - **`&T`**: Delegates to `T::len()` for any `T: CharLen`
///
/// # Design Note
///
/// This trait is **sealed** and cannot be implemented outside of tokit. If you need
/// to work with a custom character type, use [`Lexeme::span_with`] or
/// [`UnknownLexeme::from_range`](crate::error::UnknownLexeme::from_range) and provide your own length function.
///
/// # Use Cases
///
/// - **Span calculation**: Convert positioned characters to byte spans automatically
/// - **UTF-8 handling**: Properly account for multi-byte characters
/// - **Error reporting**: Determine the exact byte range of an unexpected character
///
/// # Examples
///
/// ## Automatic Length Detection
///
/// ```rust
/// use tokit::utils::{Lexeme, PositionedChar};
///
/// // ASCII character (1 byte)
/// let ascii = Lexeme::from(PositionedChar::with_position('a', 10));
/// let span = ascii.span();
/// assert_eq!(span.len(), 1);
///
/// // Multi-byte UTF-8 character (3 bytes)
/// let emoji = Lexeme::from(PositionedChar::with_position('€', 20));
/// let span = emoji.span();
/// assert_eq!(span.len(), 3);
/// ```
///
/// ## With Custom Length Function
///
/// ```rust
/// use tokit::utils::{Lexeme, PositionedChar};
///
/// // For types that don't implement CharLen, use span_with
/// struct CustomChar(char);
///
/// let lexeme = Lexeme::from(PositionedChar::with_position(CustomChar('€'), 5));
/// let span = lexeme.span_with(|c| c.0.len_utf8());
///
/// assert_eq!(span.start(), 5);
/// assert_eq!(span.end(), 8);
/// ```
#[allow(clippy::len_without_is_empty)]
pub trait CharLen: sealed::Sealed {
  /// Returns the length of this character in bytes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tokit::utils::{Lexeme, PositionedChar};
  ///
  /// // The trait is used internally by span()
  /// let ascii = Lexeme::from(PositionedChar::with_position('A', 0));
  /// assert_eq!(ascii.span().len(), 1);
  ///
  /// let euro = Lexeme::from(PositionedChar::with_position('€', 0));
  /// assert_eq!(euro.span().len(), 3);
  ///
  /// let crab = Lexeme::from(PositionedChar::with_position('🦀', 0));
  /// assert_eq!(crab.span().len(), 4);
  /// ```
  fn char_len(&self) -> usize;
}

mod sealed {
  use super::{CharLen, PositionedChar};

  pub trait Sealed {}

  impl Sealed for u8 {}
  impl Sealed for char {}
  impl<T: Sealed> Sealed for PositionedChar<T> {}

  impl<T: Sealed> Sealed for &T {}

  impl CharLen for u8 {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn char_len(&self) -> usize {
      1
    }
  }

  impl CharLen for char {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn char_len(&self) -> usize {
      self.len_utf8()
    }
  }

  impl<T: CharLen> CharLen for PositionedChar<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn char_len(&self) -> usize {
      self.char_ref().char_len()
    }
  }

  impl<T: CharLen> CharLen for &T {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn char_len(&self) -> usize {
      (*self).char_len()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // --- IsAsciiChar for char ---

  #[test]
  fn char_is_ascii_char() {
    assert!(IsAsciiChar::is_ascii_char(&'a', ascii::AsciiChar::a));
    assert!(!IsAsciiChar::is_ascii_char(&'b', ascii::AsciiChar::a));
    assert!(!IsAsciiChar::is_ascii_char(
      &'\u{00E9}',
      ascii::AsciiChar::a
    ));
  }

  #[test]
  fn char_is_ascii_digit() {
    assert!(IsAsciiChar::is_ascii_digit(&'0'));
    assert!(IsAsciiChar::is_ascii_digit(&'9'));
    assert!(!IsAsciiChar::is_ascii_digit(&'a'));
  }

  // --- IsAsciiChar for u8 ---

  #[test]
  fn u8_is_ascii_char() {
    assert!(IsAsciiChar::is_ascii_char(&b'a', ascii::AsciiChar::a));
    assert!(!IsAsciiChar::is_ascii_char(&b'b', ascii::AsciiChar::a));
  }

  #[test]
  fn u8_is_ascii_digit() {
    assert!(IsAsciiChar::is_ascii_digit(&b'0'));
    assert!(IsAsciiChar::is_ascii_digit(&b'9'));
    assert!(!IsAsciiChar::is_ascii_digit(&b'a'));
  }

  // --- IsAsciiChar for str ---

  #[test]
  fn str_is_ascii_char() {
    assert!(IsAsciiChar::is_ascii_char("a", ascii::AsciiChar::a));
    assert!(!IsAsciiChar::is_ascii_char("b", ascii::AsciiChar::a));
    assert!(!IsAsciiChar::is_ascii_char("ab", ascii::AsciiChar::a));
    assert!(!IsAsciiChar::is_ascii_char("", ascii::AsciiChar::a));
  }

  #[test]
  fn str_is_ascii_digit() {
    assert!(IsAsciiChar::is_ascii_digit("5"));
    assert!(!IsAsciiChar::is_ascii_digit("a"));
    assert!(!IsAsciiChar::is_ascii_digit("55"));
    assert!(!IsAsciiChar::is_ascii_digit(""));
  }

  // --- IsAsciiChar for [u8] ---

  #[test]
  fn slice_is_ascii_char() {
    assert!(IsAsciiChar::is_ascii_char(
      [b'a'].as_slice(),
      ascii::AsciiChar::a
    ));
    assert!(!IsAsciiChar::is_ascii_char(
      [b'a', b'b'].as_slice(),
      ascii::AsciiChar::a
    ));
    assert!(!IsAsciiChar::is_ascii_char(
      [].as_slice(),
      ascii::AsciiChar::a
    ));
  }

  #[test]
  fn slice_is_ascii_digit() {
    assert!(IsAsciiChar::is_ascii_digit([b'5'].as_slice()));
    assert!(!IsAsciiChar::is_ascii_digit([b'a'].as_slice()));
    assert!(!IsAsciiChar::is_ascii_digit([b'5', b'6'].as_slice()));
    assert!(!IsAsciiChar::is_ascii_digit([].as_slice()));
  }

  // --- IsAsciiChar for references ---

  #[test]
  fn ref_is_ascii_char() {
    let ch = 'a';
    assert!(IsAsciiChar::is_ascii_char(&&ch, ascii::AsciiChar::a));
    assert!(IsAsciiChar::is_ascii_digit(&&'5'));
  }

  #[test]
  fn mut_ref_is_ascii_char() {
    let mut ch = 'a';
    assert!(IsAsciiChar::is_ascii_char(&&mut ch, ascii::AsciiChar::a));
    assert!(IsAsciiChar::is_ascii_digit(&&mut '5'));
  }

  // --- one_of ---

  #[test]
  fn one_of_matches() {
    let choices = &[
      ascii::AsciiChar::a,
      ascii::AsciiChar::b,
      ascii::AsciiChar::c,
    ];
    assert!(IsAsciiChar::one_of(&'a', choices));
    assert!(IsAsciiChar::one_of(&'b', choices));
    assert!(!IsAsciiChar::one_of(&'d', choices));
    assert!(!IsAsciiChar::one_of(&'A', choices));
  }

  #[test]
  fn one_of_ref() {
    let choices = &[ascii::AsciiChar::a];
    assert!(IsAsciiChar::one_of(&&'a', choices));
    let mut ch = 'a';
    assert!(IsAsciiChar::one_of(&&mut ch, choices));
  }

  // --- CharLen ---

  #[test]
  fn char_len_u8() {
    assert_eq!(CharLen::char_len(&42u8), 1);
    assert_eq!(CharLen::char_len(&0u8), 1);
    assert_eq!(CharLen::char_len(&255u8), 1);
  }

  #[test]
  fn char_len_char() {
    assert_eq!(CharLen::char_len(&'a'), 1);
    assert_eq!(CharLen::char_len(&'\u{00E9}'), 2);
    assert_eq!(CharLen::char_len(&'\u{20AC}'), 3); // Euro sign
    assert_eq!(CharLen::char_len(&'\u{1F980}'), 4); // Crab emoji
  }

  #[test]
  fn char_len_ref() {
    let ch = 'a';
    assert_eq!(CharLen::char_len(&&ch), 1);
  }

  #[test]
  fn char_len_positioned_char() {
    let pc = PositionedChar::with_position('a', 0usize);
    assert_eq!(CharLen::char_len(&pc), 1);
    let pc2 = PositionedChar::with_position('\u{20AC}', 0usize);
    assert_eq!(CharLen::char_len(&pc2), 3);
  }

  // --- IntoComponents ---

  #[test]
  fn into_components_trait() {
    // Test via a punctuator which implements IntoComponents
    use crate::punct::Comma;
    let c = Comma::<usize, &str>::with_content(42, "test");
    let (span, content) = IntoComponents::into_components(c);
    assert_eq!(span, 42);
    assert_eq!(content, "test");
  }

  // --- Additional mut ref tests ---

  #[test]
  fn mut_ref_is_ascii_digit() {
    let mut ch = '5';
    assert!(IsAsciiChar::is_ascii_digit(&&mut ch));
    let mut ch2 = 'a';
    assert!(!IsAsciiChar::is_ascii_digit(&&mut ch2));
  }

  #[test]
  fn mut_ref_one_of() {
    let choices = &[ascii::AsciiChar::a, ascii::AsciiChar::b];
    let mut ch = 'a';
    assert!(IsAsciiChar::one_of(&&mut ch, choices));
    let mut ch2 = 'z';
    assert!(!IsAsciiChar::one_of(&&mut ch2, choices));
  }

  #[test]
  fn ref_one_of_empty() {
    let choices: &[ascii::AsciiChar] = &[];
    assert!(!IsAsciiChar::one_of(&'a', choices));
  }

  // --- CharLen for positioned char ref ---

  #[test]
  fn char_len_positioned_char_ref() {
    let pc = PositionedChar::with_position('a', 0usize);
    assert_eq!(CharLen::char_len(&&pc), 1);
  }

  // --- non-ASCII char tests ---

  #[test]
  fn char_non_ascii_is_not_ascii_char() {
    // Multi-byte char should not match any AsciiChar
    assert!(!IsAsciiChar::is_ascii_char(
      &'\u{1F600}',
      ascii::AsciiChar::a
    ));
  }

  #[test]
  fn str_multibyte_not_digit() {
    // Multi-byte string should not be a digit
    assert!(!IsAsciiChar::is_ascii_digit("\u{00E9}"));
  }

  #[test]
  fn slice_multibyte_not_digit() {
    assert!(!IsAsciiChar::is_ascii_digit([0xFF].as_slice()));
  }
}
