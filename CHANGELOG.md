# Unreleased (0.3.0)

## Changed (breaking)

- **Completeness-generic parser traits.** `ParseInput`/`TryParseInput` gain a trailing,
  defaulted completeness parameter (`Cmpl = Complete`) mirroring `InputRef`; the
  `parse_input`/`try_parse_input` methods take `&mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>`.
  Every default-spelled bound, impl, closure, and fn keeps compiling and keeps its behavior —
  the parameter is inference-carried through every builder-returned adapter (each adapter
  struct gains the same trailing defaulted parameter plus a phantom, including `With`), so
  0.2.0 chains infer exactly as before at a `Complete` drive.
- **`parse_partial` takes trait-bound parsers.** The bare-`FnOnce` bypass is gone:
  `parse_partial` now drives any `P: ParseInput<'inp, L, O, Ctx, Lang, Partial>` — a typed fn
  item, a named combinator chain, or a parser written generic over `Cmpl`
  (write-once-run-both). `FnOnce`-only closures (moving out of a capture) must hoist the
  moved capture into an `Option` + `take()`.
- **`Partial: SurfaceIncomplete` additionally requires `MaybeIncomplete`.** The input layer
  *constructs* incompletes (`From<Incomplete<L::Offset>>`, as before); the atom layer now
  also *recognizes* them, so partial mode's error type needs both one-liners. A correct
  refill loop already called `is_incomplete()`.
- **The parser vocabulary reserves the completeness position.** `ParseState`,
  `RecoverInput`/`InplaceRecoverInput` (generalized), `ParseChoice`/`ParseTokenChoice`,
  `ParsePrattLHS`/`ParsePrattRHS`, `PrattFoldPrefix`/`PrattFoldInfix`/`PrattFoldPostfix`, and
  `PrattCst` gain the defaulted `Cmpl = Complete` parameter (their `_with` callback
  signatures thread `ParseState<…, Cmpl>`).

## Added

- **Write-once-run-both parsers.** One parser item — a fn generic over
  `Cmpl: SurfaceIncomplete<…>` or a combinator chain assembled inside one — runs under the
  complete drivers (`Parse`/`Parser`) and the Sans-I/O partial driver (`parse_partial`),
  with chunked-equivalence proven by test oracles over every cut point.
- **Partial-mode support across the try/consume atom families.** The leaf atoms (`expect`,
  `any`, `Ident`, the `keyword!`/`punct!` vocabulary and their `parse`/`try_parse` entry
  points), the pass-through adapters (`map*`/`filter*`/`filter_map*`/`validate*`/`then*`/
  `ignored`/`spanned`/`sliced`/`located`/`padded*`/`recover`/`inplace_recover`/
  `skip_then_retry`/`labelled`/`unwrapped`/`accepted`/`opt`), the try-driven collections
  (`repeated`, `separated*` incl. the delim variants, `fold`/`try_fold*`/`rfold`,
  `collect`), the delimited shapes and their `try_` twins, and `try_ident_list*` are generic
  over the completeness parameter. The scanner drivers beneath them (`try_expect*`,
  `skip_while`, `sync_to`/`sync_through`/`sync_balanced`, `fold`/`foldn`/`foldr_within`/
  `foldrn`, `consume_cached_*`) generalize with them. The decision-window class
  (`*_while`, `peek_*`, `dispatch_*`, pratt) and the CST `node` family stay Complete-only,
  each impl carrying its recorded reason; driving one at `Partial` is a compile-time wall.
- **`SurfaceIncomplete::is_incomplete_error`** — the error-interrogation twin of
  `surface_incomplete`: a constant, bound-free `false` for `Complete` (the check const-folds
  away) and `MaybeIncomplete`-routed for `Partial`.
- **The atom-layer never-recoverable gate.** The resilient collection loops
  (`repeated`/`separated` families) re-raise a frontier `Incomplete` from their element
  parser untouched instead of spending it as a diagnostic and looping — locked by a source
  census over the four gated loop bodies plus behavior tests in both modes.

## Migration (0.2 → 0.3)

- **Complete-mode users: nothing.** Every default-spelled bound, impl, closure, fn item, and
  builder chain compiles unchanged with identical behavior and codegen (the frontier rules
  remain compiled out of `Complete` monomorphizations).
- **`parse_partial` callers:** typed-fn parsers (the only pattern that ever inferred) are
  unchanged. The error type must now implement `MaybeIncomplete` alongside
  `From<Incomplete<L::Offset>>` (usually one line; see `tokora::error::MaybeIncomplete`).
  A `FnOnce`-only closure must be restructured (hoist the moved capture into an
  `Option` + `take()`).
- **Fully-explicit spellings:** if you name *every* parameter of the traits, the adapter
  structs, or the free-fn constructors (turbofish like `delimited::<Paren, _, _, _, _, _>`),
  append the completeness argument — `Complete`, or just `_` at a drive site. Default
  spellings need nothing.

## Added

- **Delimited shape parsers** (`tokora::parser`). `delimited::<D>` — commits a `Delimiter`
  pair's opener, runs an inner parser, commits the closer, and returns the three as a
  span-carrying `Delimited` covering the whole construct; plus the named conveniences
  `parens`/`braces`/`brackets`/`angles` and the result aliases
  `DelimitedOf`/`ParensOf`/`BracesOf`/`BracketsOf`/`AnglesOf`. A missing closer is a hard
  error — the closer's unexpected-token or end-of-input error propagates; the
  `Unclosed`/`Unopened`/`Undelimited` vocabulary is deliberately not fired by this family.
- **`TypedDelimiter`** (`tokora::delimiter`) — additive `Delimiter` subtrait materializing
  span-carrying opening/closing punctuator values; implemented for `Paren`/`Brace`/
  `Bracket`/`Angle`.
- **Attempt twins for the delimited shapes** (`tokora::parser`).
  `try_delimited::<D>`/`try_parens`/`try_braces`/`try_brackets`/`try_angles` and the result
  aliases `TryDelimitedOf`/`TryParensOf`/`TryBracesOf`/`TryBracketsOf`/`TryAnglesOf` —
  decline (`Ok(None)`, zero consumption) iff the opener is absent (wrong token or end of
  input at entry); the moment the opener is consumed the parse is committed and inner/closer
  errors propagate exactly as the committed forms' (deliberately not `opt(parens(…))`, which
  would swallow an unclosed group into a decline).

## Changed

- **Free-atom bounds converge on the house traits** (`tokora::parser`). The inner-parser
  parameters are trait bounds instead of bare closures: `delimited`/`parens`/`braces`/
  `brackets`/`angles`, their `try_` twins, and `separated1`/`list_of` take
  `impl ParseInput`; `opt` takes `impl TryParseInput`. The closure blanket impls make this a
  strict relaxation — every existing closure and fn-item call site compiles unchanged — while
  named implementors no longer need `|inp| p.parse_input(inp)` adapter closures. Predicate
  parameters (`peek`, `until`) are functions, not parsers, and stay plain `FnMut` closures.
  Returned parsers are unchanged `impl FnMut` closures and implement `ParseInput` through the
  same blankets.

## Fixed

- **Try shapes decline only on definite absence.** The five attempt shapes
  (`try_delimited`/`try_parens`/`try_braces`/`try_brackets`/`try_angles`) derived "opener
  absent" from `try_expect`'s `Ok(None)`, which also covers a **terminal scanner stop** — a
  fresh resource-limit trip whose diagnostic a recovering emitter accepted, or an
  already-latched poison boundary — so an EOI-tolerant grammar could complete with output
  shaped as if the optional construct were absent. The shapes (and the punct vocabulary's
  `try_parse`/`try_parse_of` forms, whose machinery the named twins share) now build on the
  new attempt primitive `InputRef::try_expect_or_stop`: `Ok(None)` means the opener is
  **definitely absent** (the next valid token is not the opener and stays unconsumed, or the
  input has genuinely ended), while a terminal stop fails with the same end-of-input error
  the committed forms raise there — the trip's own diagnostic having already reached the
  emitter, and a fatal emitter's rejection still propagating from the scan itself.
  `try_expect` keeps its documented fold (now with an explicit warning and a cross-reference
  to the new primitive).

# 0.2.0 (2026-07-17)

## Added

- **Event-stream CST (feature `rowan`).** A rewindable, lossless concrete-syntax-tree channel
  that rides the parser's own backtracking: the tree is a flat log of events on the emitter's
  rewindable buffer, so a parser rollback truncates the log and rewinds the half-built tree for
  free — no separate tree-side undo.
  - **`CstEmitter`** — the tree-structuring capability subtrait of `Emitter` (`cst_start`,
    `cst_finish`, `cst_token`, `cst_mark`, `cst_start_at`), defaulted to no-ops on the
    diagnostics emitters. One parser assembly therefore builds a tree or runs tree-less by
    emitter choice alone; driving a `node`-bearing parser with an emitter that does not forward
    `CstEmitter` is a compile error, never a silently empty tree.
  - **`Sink`** — the rewindable recording sink: wraps an inner emitter, records every
    committed token into a lossless green tree through a dialect `fn(&Token) -> u16` kind mapper,
    and forwards diagnostics to the inner emitter. `finish(root_kind, source)` materializes the
    tree and gap-tiles untokenized bytes (lexer-skipped trivia, covered lexer-error spans) so
    `tree.text() == source` structurally for every input; an unexplained gap is a typed
    `FinishError`, never a wrong tree. Adds `TriviaPolicy` / `with_trivia_policy`,
    `finish_partial`, and the `FinishError` enum.
  - **`node` / `node_at` / `node_opt`** — the CST bracketing combinators: a sub-parse's
    committed tokens, trivia, and nested nodes become the children of one syntax node. The
    bracket is append-only, so a decline or an error-path unwind leaves no node and no dangling
    open node — speculation and recovery cannot corrupt the tree. Gated on
    `Ctx::Emitter: CstEmitter`; AST-level Pratt gains an additive `with_cst_kinds` classifier so
    operator expressions nest into the tree.
  - **Event vocabulary and branded marks** — `EventMark`, a backtracking-safe handle carrying
    buffer index, truncation era, and issuing-sink identity (a stale or foreign spend panics in
    every build); the single-use `Marker` / `CompletedMarker` typestate; and the reserved
    `TOMBSTONE` kind.

- **`Token::SURFACES_TRIVIA` and `Lexer::SURFACES_TRIVIA`** — new defaulted associated
  `const bool` (default `false`). Declares whether a lexer *surfaces trivia as real tokens*
  (every source byte reaches the sink as a token or a reported lexer error) instead of
  skipping it at the lexer level. `Lexer::SURFACES_TRIVIA` defaults to the token
  vocabulary's `Token::SURFACES_TRIVIA`, so a `LogosLexer`-backed dialect declares it once
  on its `Token` impl. Every existing `Token`/`Lexer` impl keeps compiling unchanged — the
  default preserves prior behavior.

- **Dialect-author atoms** (`tokora::parser`). `separated1` — one-or-more `item`s separated by
  a punctuator, permitting an optional leading separator, collected into a `Vec`; `list_of` —
  zero-or-more elements up to an `Until` stopper that is left in place for the caller;
  `peek_kind` — reports the next token's kind without consuming it, the dispatch primitive for
  sum-type composites; and `opt` — adapts a declining `try_`-parser to yield `Option`, honoring
  the decline-consumes-nothing contract.

- **Capability bundles.** `ComposableEmitter` — one bound standing in for the full emitter
  capability ladder — and `ParseCtx`, the parse-context bundle implemented for every
  `ParseContext` whose emitter is a `ComposableEmitter` and whose source slice is `Clone`, so an
  atom needs only `Ctx: ParseCtx<'inp, L>` to unlock the ladder. Adds the `ErrorOf` context
  alias.

- **`Emitter::release`** — the dual of `rewind`: settles a *kept* checkpoint (a commit) so an
  emitter can reclaim per-checkpoint bookkeeping instead of stranding one dead row per committed
  guard. Advisory and observably pure; the stateless and value-keyed emitters (`Fatal`,
  `Silent`, `Ignored`, `Verbose`) inherit the no-op default.

- **smol-bytes source support** (feature `smol_bytes_0_1`, alias `smol_bytes`).
  `Source` and `Slice` implementations for the `smol-bytes` crate's `shared::Bytes`,
  `compact::Bytes`, and `Utf8Bytes` buffers. Sits on the same tier as the other byte
  backends — it implies neither `std` nor `alloc` in tokora and works on `no_std`; it
  enables smol-bytes' own `alloc` feature and requires smol-bytes ≥ 0.1.2 (the first
  `rlib`-only release with an `alloc` tier).

- **`SliceOf<'inp, L>`** — a type alias naming a lexer's source-slice path once, so bounds and
  return types carrying the slice stay legible.

## Changed

- **Lossless (`gap_kind`) `Sink` construction is now compile-time restricted to
  trivia-surfacing lexers** (`rowan` feature). `Sink::new` carries an inline-`const`
  guard requiring `Lexer::SURFACES_TRIVIA == true`. Pairing a syntactic (trivia-skipping)
  lexer with a lossless sink is now a compile error (a post-monomorphization `error[E0080]`
  at build/test/doc time) instead of a runtime `FinishError::UncoveredGap` on the first
  skipped-whitespace gap. The guard does not fire under `cargo check`.

- **Every committed token now settles through one input-layer primitive.** The input layer
  funnels each token — consumed, or skipped behind a scan frontier — through a single settle
  point and the new defaulted `Emitter::commit_token` hook, exactly once per token; peeks,
  declines, and unconsumed stoppers never settle. This is the auto-emission chokepoint the
  recording `Sink` overrides to record tokens, which is what makes every consuming atom
  tree-producing with no per-atom code. Diagnostics emitters keep the no-op default, so their
  observable behavior is unchanged.

## Migration

- A lexer paired with a lossless `Sink` must declare `const SURFACES_TRIVIA: bool = true;`
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
