# Unreleased (0.2.0)

## Added

- **`Token::SURFACES_TRIVIA` and `Lexer::SURFACES_TRIVIA`** — new defaulted associated
  `const bool` (default `false`). Declares whether a lexer *surfaces trivia as real tokens*
  (every source byte reaches the sink as a token or a reported lexer error) instead of
  skipping it at the lexer level. `Lexer::SURFACES_TRIVIA` defaults to the token
  vocabulary's `Token::SURFACES_TRIVIA`, so a `LogosLexer`-backed dialect declares it once
  on its `Token` impl. Every existing `Token`/`Lexer` impl keeps compiling unchanged — the
  default preserves prior behavior.

## Changed

- **Lossless (`gap_kind`) `CstSink` construction is now compile-time restricted to
  trivia-surfacing lexers** (`rowan` feature). `CstSink::new` carries an inline-`const`
  guard requiring `Lexer::SURFACES_TRIVIA == true`. Pairing a syntactic (trivia-skipping)
  lexer with a lossless sink is now a compile error (a post-monomorphization `error[E0080]`
  at build/test/doc time) instead of a runtime `CstFinishError::UncoveredGap` on the first
  skipped-whitespace gap. The guard does not fire under `cargo check`.

## Migration

- A lexer paired with a lossless `CstSink` must declare `const SURFACES_TRIVIA: bool = true;`
  on its `Token` impl (or override it on the `Lexer` impl). Forgetting it is a loud build
  error naming the exact const. Syntactic lexers that never construct a lossless sink are
  unaffected — the default `false` is honest for them.

---

_Entries below predate the `tokora` crate and use the pre-rename version line._

# 0.3.0 (Nov 3rd, 2025)

## Breaking Changes

- **Unicode escape error types renamed for clarity**: Variable-length unicode escape types now use "Variable" prefix instead of "Braced"
  - `BracedUnicodeEscapeError` → `VariableUnicodeEscapeError`
  - `EmptyBracedUnicodeEscape` → `EmptyVariableUnicodeEscape`
  - `TooManyDigitsInBracedUnicodeEscape` → `TooManyDigitsInVariableUnicodeEscape`
  - `MalformedBracedUnicodeSequence` → `MalformedVariableUnicodeSequence`
  - This change better reflects the variable-length nature of `\u{...}` escapes (1-6 hex digits)
  - **Migration guide**:

    ```rust
    // Before
    let error = BracedUnicodeEscapeError::empty(span);

    // After
    let error = VariableUnicodeEscapeError::empty(span);
    ```

- **Escape sequence utility types renamed for clarity**: Types in `utils::escaped` now have more descriptive names
  - `EscapedCharacter` → `SingleCharEscape` (represents single-char escapes like `\n`, `\t`)
  - `EscapedSequence` → `MultiCharEscape` (represents multi-char escapes like `\xXX`, `\u{...}`)
  - `Escaped` → `EscapedLexeme` (wrapper enum for both escape types)
  - Field renames for consistency:
    - `char` → `character` (in `SingleCharEscape`)
    - `seq` → `content` (in `MultiCharEscape`)
    - `escaped` → `lexeme` (in `EscapedLexeme`)
  - **Migration guide**:

    ```rust
    // Before
    let escaped = EscapedCharacter::new(positioned_char, span);
    let c = escaped.char();

    // After
    let escaped = SingleCharEscape::new(positioned_char, span);
    let c = escaped.character();
    ```

- **Error types now have lifetime parameters**: `Expected`, `UnexpectedToken`, and `UnexpectedKeyword` now include a lifetime parameter `'a`
  - `Expected<T>` → `Expected<'a, T>` (to support `OneOf(&'a [T])`)
  - `UnexpectedToken<T, TK>` → `UnexpectedToken<'a, T, TK>`
  - `UnexpectedKeyword<S>` → `UnexpectedKeyword<'a, S>`
  - This change enables zero-copy error construction when referencing static slices
  - **Migration guide**:

    ```rust
    // Before
    let error: UnexpectedToken<&str, TokenKind> = ...;
    let expected: Expected<TokenKind> = Expected::one_of(&[...]);

    // After
    let error: UnexpectedToken<'_, &str, TokenKind> = ...;
    let expected: Expected<'_, TokenKind> = Expected::one_of(&[...]);
    ```

- **Parser error types now include span tracking**: `UnexpectedToken` and `UnexpectedKeyword` now include a `span` field to track error locations
  - All constructors now require a `Span` parameter as the first argument
  - `into_components()` now returns `(Span, ...)` instead of just the token/keyword components
  - This change improves error reporting by providing precise source locations for parser errors
  - **Migration guide**:

    ```rust
    // Before
    let error = UnexpectedToken::with_found("}", Expected::one("{"));
    let (found, expected) = error.into_components();

    // After
    let error = UnexpectedToken::with_found(span, "}", Expected::one("{"));
    let (span, found, expected) = error.into_components();
    ```

- **Chumsky is now optional**: The `chumsky` dependency is now behind the `chumsky` feature flag (enabled by default with `std` feature)
  - Use `default-features = false` if you only need the lexer functionality
  - Renamed `parseable.rs` → `chumsky.rs` to better reflect the optional nature

## New Features

### Hexadecimal Escape Sequence Error Handling

- **`HexEscapeError<Char>`**: New error type for `\xXX` hex escape sequences
  - `Incomplete(IncompleteHexEscape)` - For sequences with fewer than 2 hex digits (e.g., `\x`, `\xA`)
  - `Malformed(MalformedHexEscape<Char>)` - For invalid hexadecimal characters (e.g., `\xGG`, `\xZ9`)
  - Follows the same design patterns as `UnicodeEscapeError`
  - Provides precise error reporting with span tracking and invalid character positions

- **`IncompleteHexEscape`**: Error for incomplete hex escape sequences
  - Tracks the span of the incomplete sequence
  - Occurs when hex escape has fewer than 2 hex digits
  - Methods: `span()`, `bump(n)` for position adjustment

- **`MalformedHexEscape<Char>`**: Error for malformed hex escape sequences
  - Contains `InvalidHexDigits<Char, 2>` with the invalid characters and their positions
  - Tracks both the span and the specific invalid hex digits encountered
  - Methods: `digits()`, `digits_ref()`, `span()`, `is_incomplete()`, `bump(n)`
  - Type alias: `InvalidHexEscapeDigits<Char>` for `InvalidHexDigits<Char, 2>`

### Enhanced Error Position Tracking

- **`span()` getter**: Added `span()` method to `UnexpectedToken` and `UnexpectedKeyword` for retrieving error locations
- **`bump()` method**: Added `bump(offset)` method to all span-containing error types for adjusting error positions:
  - `UnexpectedToken::bump(offset)` - Adjust token error positions
  - `UnexpectedKeyword::bump(offset)` - Adjust keyword error positions
  - `UnexpectedPrefix::bump(offset)` - Adjust prefix error positions
  - `UnexpectedSuffix::bump(offset)` - Adjust suffix error positions
  - `MalformedLiteral::bump(offset)` - Adjust malformed literal positions
  - `IncompleteToken::bump(offset)` - Adjust incomplete token positions
  - `Unclosed::bump(offset)` - Adjust unclosed delimiter positions
  - Useful for adjusting error positions when combining spans from different contexts

### Documentation Improvements

- **Comprehensive doc tests**: Added detailed documentation with working examples for all error type methods
  - `Expected<T>` - Complete documentation for expected value types with 3 doc tests
  - `UnexpectedToken<T, TK>` - Full API documentation with 14 doc tests covering all methods and use cases
  - `UnexpectedKeyword<S>` - Complete method documentation with 8 doc tests
  - **Module-level documentation**: Added detailed explanations of design philosophy and common patterns
  - All 25 examples are tested and guaranteed to compile

- **Enhanced unicode escape error documentation**: Complete rewrite of `error::unicode_escape` module
  - Added 100+ lines of module-level documentation explaining:
    - Design philosophy for unicode escape error handling
    - Format specifications for `\uXXXX` (fixed, 4 digits) vs `\u{...}` (variable, 1-6 digits)
    - Error type hierarchy and when each error occurs
    - Common error patterns and examples
  - Added comprehensive examples to all public types and methods
  - Improved naming clarity (Variable prefix for variable-length escapes)

- **Enhanced escape sequence utility documentation**: Complete rewrite of `utils::escaped` module
  - Added detailed module-level documentation explaining:
    - Design philosophy for escape sequence representation
    - Distinction between single-char (`\n`) and multi-char (`\xXX`) escapes
    - Zero-copy design principles
    - Usage patterns in lexer implementations
  - Improved type names for better clarity
  - Added examples demonstrating typical usage patterns
  - Added missing `lexeme()` getter method to `EscapedLexeme`

- **Hexadecimal escape error documentation**: New `error::hex_escape` module with comprehensive documentation
  - Module-level docs explaining hex escape format (`\xXX`) and error types
  - Detailed examples for all error variants
  - Clear distinction between incomplete (fewer digits) and malformed (invalid hex) errors

- **Enhanced examples with custom error wrappers**: Both `simple_calculator` and `custom_parser` examples now demonstrate:
  - Custom error types compatible with logosky's `Span` type
  - Rich error reporting with detailed location information
  - Implementation of Chumsky's `Error` and `LabelError` traits for custom error types
  - User-friendly error messages showing precise source locations
  - Example error format: `at 10..15: unexpected token Number, expected Plus`
  - **Run examples**: `cargo run --example simple_calculator --features chumsky`

### No-alloc Support

- **Zero-allocation mode**: LogoSky now supports `no_std` + `no_alloc` environments
  - Core lexer and utility types work without heap allocation
  - `alloc` feature enables collection-based utilities
  - `std` feature enables full error handling and format support

### Macros for Token Types

- **`keyword!` macro**: Define keyword tokens with zero boilerplate

  ```rust
  keyword! {
    (Let, "LET", "let"),
    (Const, "CONST", "const"),
  }
  ```

  - Automatically implements common traits (`Debug`, `Clone`, `PartialEq`, etc.)
  - Generic over span type `S` and optional content `C`
  - Provides `AsRef<str>` and `Borrow<str>` implementations

- **`punctuator!` macro**: Define punctuation tokens with minimal code

  ```rust
  punctuator! {
    (LParen, "L_PAREN", "("),
    (RParen, "R_PAREN", ")"),
  }
  ```

  - Similar ergonomics to `keyword!` macro
  - Includes methods for accessing raw string literals

### CST (Concrete Syntax Tree) Support

- **CST module**: Added new `cst` module with rowan integration (requires `rowan` feature)
  - `SyntaxTreeBuilder<Lang>`: A builder for constructing syntax trees using rowan's GreenNodeBuilder
  - `Parseable<'a, I, T, Error>` trait: Enables types to be parsed from a tokenizer and produce CST nodes
  - `CstElement`: Base trait for all typed CST elements (nodes and tokens)
  - `CstNode`: Trait for typed CST nodes with zero-cost conversions from untyped `SyntaxNode`
  - `CstToken`: Trait for typed CST tokens (terminal elements)
  - `SyntaxNodeChildren<N>` iterator: Iterate over children of a particular CST node type

- **CST cast utilities** (in `cst::cast` module):
  - `child<N>()`: Get the first child of a specific type
  - `children<N>()`: Get all children of a specific type
  - `token<L>()`: Get a token child with a specific kind
  - Type-safe casting with proper error handling

- **CST error types** (in `cst::error` module):
  - `Incomplete`: Error for incomplete CST constructions
  - `CstNodeMismatch`: Type mismatch for CST nodes
  - `CstTokenMismatch`: Type mismatch for CST tokens
  - All error types include rich context for better diagnostics

### New Utility Types

- **`InvalidHexDigits<Char, N>`**: Generic, zero-copy container for storing invalid hex digit characters
  - Stack-allocated container for collecting invalid hex digits during escape sequence parsing
  - Two implementations via feature flags:
    - **Without `generic-array`** (default): Uses `const N: usize` (const-generic implementation)
    - **With `generic-array`**: Uses `N: ArrayLength` for type-level capacity specification
  - Internally uses `GenericVec<PositionedChar<Char>, N>` for efficient storage
  - Methods: `from_positioned_char()`, `from_char()`, `from_array()`, `try_from_iter()`, `push()`, `push_char()`, `len()`, `is_full()`, `bump(n)`
  - Implements `Deref<Target = [PositionedChar<Char>]>` for convenient access
  - Specialized for different escape formats:
    - `InvalidHexDigits<Char, 2>` for hex escapes (`\xXX`)
    - `InvalidHexDigits<Char, 4>` for fixed unicode escapes (`\uXXXX`)
    - `InvalidHexDigits<Char, 6>` for variable unicode escapes (`\u{XXXXXX}`)
  - Enables precise error reporting by tracking both invalid characters and their positions
  - Zero heap allocation design for no-alloc environments

- **`Message`**: Feature-aware message container that seamlessly adapts between `&'static str`
  in `no_std` + `no_alloc` builds and `Cow<'static, str>` when `alloc` or `std` is enabled
  - Unified API with `new`, `from_static`, `from_string`, and zero-copy accessors
  - Implements common conversion traits (`From`, `Into`, `AsRef`, `Borrow`, `AsMut`, `Deref`)
    for ergonomic integration with existing error types
  - Re-exported from `logosky::utils` for downstream use and backed by doctested examples

- **`Errors<E, C>`**: Environment-adaptive error collection container
  - Automatically adapts to allocation environment:
    - **no_std (no alloc)**: Uses `GenericVec<E, 2>` with fixed capacity of 2 errors
    - **alloc/std**: Uses `Vec<E>` for unlimited error collection
  - Generic over error type `E` and container `C` for custom containers
  - Provides consistent API across all environments
  - Methods: `new()`, `push()`, `try_push()`, `len()`, `is_empty()`, `clear()`, `iter()`, etc.
  - Alloc-specific methods: `with_capacity()`, `reserve()`, `pop()`, `retain()`, `truncate()`
  - Implements `Display`, `IntoIterator`, `FromIterator`, and standard traits
  - Perfect for collecting multiple parsing errors in no-alloc environments

- **`GenericVec<T, N>`**: Bounded, stack-allocated vector for no-alloc environments
  - Two implementations via feature flags:
    - **Without `generic-array`**: Uses `const N: usize` (const-generic)
    - **With `generic-array`**: Uses `typenum` for type-level capacity
  - Designed for error collection in parsers without heap allocation
  - Silently drops elements when capacity is exceeded (by design)
  - Comprehensive API: `capacity()`, `is_full()`, `remaining_capacity()`, `try_push()`, `pop()`, `clear()`, `retain()`, `truncate()`, `iter_mut()`, etc.
  - Implements `Index`, `IndexMut`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`
  - Safe abstraction over `MaybeUninit` arrays

- **`Lexeme<Char>`**: Zero-copy description of a lexeme in source code
  - Represents either a single positioned character or a byte span
  - Designed for error reporting without string allocation
  - Provides `is_char()`, `is_range()`, `unwrap_char()`, `unwrap_range()` helpers

- **`UnknownLexeme<Char, Knowledge>`**: Error structure for unrecognized lexemes
  - Pairs an unrecognized lexeme with diagnostic knowledge
  - Generic `Knowledge` parameter allows custom diagnostic types
  - Implements `Deref` to `Lexeme` for convenient access
  - Methods: `from_char()`, `from_span()`, `map_char()`, `map_hint()`, etc.

- **`UnexpectedSuffix<Char>`**: Error for unexpected suffixes after valid tokens
  - Tracks the valid token span and the unexpected suffix
  - Useful for catching errors like `123abc` (invalid number literal)

- **`Unclosed<Delimiter>`**: Error for unclosed delimiters
  - Tracks the opening delimiter position
  - Generic over delimiter type (char, string, custom enum, etc.)

- **`RecursionLimiter`**: Stack overflow protection for recursive parsing
  - Configurable recursion depth limit
  - Returns `RecursionLimitExceeded` error when limit is reached
  - Integrates with logos lexer via `State` trait

- **`TokenTracker` / `TokenLimiter`**: Track the count of tokens yielded and enforce limits
  - `TokenTracker`: Trait for tracking token counts during lexing/parsing
  - `TokenLimiter`: Implementation that prevents DoS attacks by limiting total token count
  - Configurable token limit with `TokenLimitExceeded` error
  - Can be embedded in lexer state via `Extras`
  - Useful for protecting against maliciously large or deeply nested inputs

### Internal Improvements

- **Module restructuring**:
  - Split `lexer.rs` into `lexer/tokenizer.rs` for better organization
  - Moved tokenizer iterator to `lexer/tokenizer/iter.rs`

- **Enhanced `PositionedChar`**:
  - Added more utility methods for character manipulation

- **Improved `Tracker` utilities**:
  - Enhanced position tracking capabilities
  - Better integration with lexer state

## Dependencies

### Added

- **`paste = "1"`**: For macro metaprogramming (required for `keyword!` and `punctuator!` macros)
- **`rowan = "0.16"`** (optional, behind `rowan` feature flag): For CST support
- **`generic-array = "1"`** (optional, with `rowan` feature): For fixed-size arrays in CST operations

### Changed

- **`chumsky`**: Now optional, enabled by `chumsky` feature (enabled by default with `std`)

## Features

- **New feature flag**: `chumsky` - Enables parser combinator support (enabled by default with `std` feature)
- **New feature flag**: `rowan` - Enables CST support with rowan integration (requires `std` and `generic-array`)
- **Modified feature**: `smallvec` - Now requires `alloc` feature

## Bug Fixes

- Fixed formatting issues in documentation
- Improved error message clarity in various utility types
- Better handling of edge cases in lexer position tracking

## Improvements

- Enabled the `chumsky` feature by default so parser integration and its tests run on a plain `cargo test`. Consumers who only need lexing can still opt out with `default-features = false`.
- Relaxed tokenizer internals to require only `Clone` for Logos `Extras`, allowing rich/non-`Copy` lexer states without extra wrappers.
- Dropped the `State` trait requirement from `LogoStream`, so custom lexer extras no longer need to implement a LogoSky-specific trait to use Chumsky inputs.

# 0.2.0 (Oct 20th, 2025)

## New Features

- **TriviaToken trait**: Introduced `TriviaToken<'a>` trait for tokens that preserve trivia information
  - Provides `is_trivia()` method to identify whitespace, comments, and other non-semantic tokens
  - Enables building parsers that can preserve formatting and comments (important for formatters, linters, and language servers)

- **Trivia handling utilities in Tokenizer**: Added two new parser combinators for working with trivia tokens
  - `skip_trivias<E>()`: Parser that skips over trivia tokens (whitespace, comments, etc.)
    - Useful for parsers that don't need to preserve formatting
    - Returns `()` and consumes trivia tokens from the input stream
  - `collect_trivias<C, E>()`: Parser that collects trivia tokens into a container
    - Useful for formatters, linters, and tools that need to preserve or analyze trivia
    - Generic over any container type implementing `Container<Spanned<T>>`

## Examples

### Using TriviaToken for trivia-aware parsing

```rust
use logosky::{Token, TriviaToken};

#[derive(Debug, Clone, PartialEq)]
struct MyToken {
  kind: MyTokenKind,
}

impl Token<'_> for MyToken {
  type Char = char;
  type Kind = MyTokenKind;
  type Logos = MyTokens;

  fn kind(&self) -> Self::Kind {
    self.kind
  }
}

impl TriviaToken<'_> for MyToken {
  fn is_trivia(&self) -> bool {
    matches!(self.kind, MyTokenKind::Whitespace | MyTokenKind::Comment)
  }
}
```

### Skipping trivia in parsers

```rust
use logosky::Tokenizer;

// Skip all leading trivia before parsing a token
let parser = MyTokenizer::skip_trivias()
  .ignore_then(my_token_parser);
```

### Collecting trivia for formatters

```rust
use logosky::Tokenizer;

// Collect trivia tokens into a Vec
let parser = MyTokenizer::collect_trivias::<Vec<_>, _>()
  .then(my_token_parser);
```
