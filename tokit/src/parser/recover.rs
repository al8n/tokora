use crate::input::Checkpoint;

use super::*;

/// A trait for recovery parsers that start from the original position after backtracking.
///
/// This trait defines the interface for recovery parsers used by [`Recover`]. When the primary
/// parser fails, implementors of this trait receive the error and attempt to produce a valid
/// output by parsing from the restored checkpoint position.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for closures with the signature:
/// ```ignore
/// FnMut(&mut InputRef, Error) -> Result<O, Error>
/// ```
///
/// # Example
///
/// ```ignore
/// use tokit::parser::{ParseInput, RecoverInput};
///
/// // Manual implementation
/// struct ErrorNodeRecovery;
///
/// impl RecoverInput<'_, MyLexer, Node, MyContext> for ErrorNodeRecovery {
///     fn recover_input(&mut self, input, err) -> Result<Node, Error> {
///         // Create error node with span from error
///         Ok(Node::Error(err.span()))
///     }
/// }
///
/// // Or use a closure (automatic implementation)
/// parser.recover(|_input, err| {
///     Ok(Node::Error(err.span()))
/// })
/// ```
///
/// See [`Recover`] for usage examples.
pub trait RecoverInput<'inp, L, O, Ctx, Lang: ?Sized = ()> {
  /// Try to recover from a parsing error.
  ///
  /// This method is called when the primary parser fails. The input position has been
  /// restored to where it was before the primary parser started.
  ///
  /// # Parameters
  ///
  /// - `input`: Input reference at the original starting position (after backtracking)
  /// - `err`: The error produced by the failed primary parser
  ///
  /// # Returns
  ///
  /// - `Ok(output)`: Successfully recovered with a value
  /// - `Err(error)`: Recovery failed
  fn recover_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, L, O, Ctx, Lang: ?Sized, F> RecoverInput<'inp, L, O, Ctx, Lang> for F
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  F: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn recover_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (self)(input, err)
  }
}

/// A trait for recovery parsers that continue from the error position without backtracking.
///
/// This trait defines the interface for recovery parsers used by [`InplaceRecover`]. When the
/// primary parser fails, implementors of this trait receive the error and the checkpoint, and
/// attempt to produce a valid output by parsing from the current (error) position.
///
/// Unlike [`RecoverInput`], the input position is **not** restored - recovery continues from
/// where the primary parser stopped.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for closures with the signature:
/// ```ignore
/// FnMut(&mut InputRef, Checkpoint, Error) -> Result<O, Error>
/// ```
///
/// # Example
///
/// ```ignore
/// use tokit::parser::{ParseInput, InplaceRecoverInput};
///
/// // Manual implementation
/// struct SkipToSemicolon;
///
/// impl InplaceRecoverInput<'_, MyLexer, Stmt, MyContext> for SkipToSemicolon {
///     fn inplace_recover_input(&mut self, input, _ckp, _err) -> Result<Stmt, Error> {
///         // Skip tokens until semicolon from current position
///         while !input.peek().is_semicolon() {
///             input.next();
///         }
///         input.next(); // consume semicolon
///         Ok(Stmt::Error)
///     }
/// }
///
/// // Or use a closure (automatic implementation)
/// parser.inplace_recover(|input, _ckp, _err| {
///     // Skip to semicolon from error position
///     skip_to_semicolon(input)?;
///     Ok(Stmt::Error)
/// })
/// ```
///
/// See [`InplaceRecover`] for usage examples.
pub trait InplaceRecoverInput<'inp, L, O, Ctx, Lang: ?Sized = ()> {
  /// Try to recover from a parsing error without backtracking.
  ///
  /// This method is called when the primary parser fails. Unlike [`RecoverInput::recover_input`],
  /// the input position has **not** been restored - it remains at the error position.
  ///
  /// # Parameters
  ///
  /// - `input`: Input reference at the current (error) position
  /// - `ckp`: Checkpoint saved before the primary parser started (for reference only)
  /// - `err`: The error produced by the failed primary parser
  ///
  /// # Returns
  ///
  /// - `Ok(output)`: Successfully recovered with a value
  /// - `Err(error)`: Recovery failed
  fn inplace_recover_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    ckp: Checkpoint<'inp, '_, L>,
    err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, L, O, Ctx, Lang: ?Sized, F> InplaceRecoverInput<'inp, L, O, Ctx, Lang> for F
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  F: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang>,
    Checkpoint<'inp, '_, L>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn inplace_recover_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    ckp: Checkpoint<'inp, '_, L>,
    err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (self)(input, ckp, err)
  }
}

/// A parser that provides error recovery by trying an alternative parser with backtracking.
///
/// This combinator implements **error recovery with checkpoint restoration**. If the main
/// parser fails, the input position is reset to where it was before parsing, and a recovery
/// parser is tried instead.
///
/// This is useful for:
/// - **Resilient parsing**: Continue parsing after errors to find more issues
/// - **Fallback strategies**: Try alternative interpretations when the primary fails
/// - **Error correction**: Insert synthetic nodes to maintain parse tree structure
///
/// # Type Parameters
///
/// - `P`: The primary parser to try first
/// - `R`: The recovery parser to use if primary fails
/// - `O`: Output type (both parsers must produce the same type)
/// - `L`: Lexer type
/// - `Ctx`: Parse context
/// - `Lang`: Language marker type (default `()`)
///
/// # Examples
///
/// ## Basic Recovery
///
/// ```ignore
/// use tokit::parser::ParseInput;
///
/// // Try to parse a valid expression, fall back to error node
/// let parser = parse_expression()
///     .recover(parse_error_node());
///
/// // Input: "1 + 2"      → Ok(BinaryOp(Add, 1, 2))
/// // Input: "@ invalid" → Ok(ErrorNode(...))  // recovery parser used
/// ```
///
/// ## Statement-Level Recovery
///
/// ```ignore
/// // Parse statement, recover by skipping to next semicolon
/// let parser = parse_statement()
///     .recover(
///         skip_to(|tok| matches!(tok, Token::Semicolon))
///             .then_ignore(any())  // consume semicolon
///             .map(|_| Statement::Error)
///     );
///
/// // Input: "let x = 1;"      → Ok(LetStmt { .. })
/// // Input: "### bad ;"       → Ok(Statement::Error)
/// //                               ^ skips to semicolon
/// ```
///
/// ## Multiple Recovery Strategies
///
/// ```ignore
/// // Try multiple parsers with recovery fallbacks
/// let parser = parse_function()
///     .recover(parse_struct())
///     .recover(parse_error_item());
///
/// // Tries: function → struct → error item
/// ```
///
/// ## Collecting Errors While Recovering
///
/// ```ignore
/// // Use with greedy emitter to collect all errors
/// let parser = parse_item()
///     .recover_with(|_err, state| {
///         // Error already emitted by failed parser
///         // Return placeholder and continue
///         Ok(Item::Error(state.span()))
///     });
/// ```
///
/// # How It Works
///
/// 1. **Save checkpoint**: Record current input position and state
/// 2. **Try primary parser**: Attempt to parse with the main parser
/// 3. **On success**: Return the parsed value
/// 4. **On failure**:
///    - **Restore checkpoint**: Reset input to saved position
///    - **Try recovery parser**: Attempt recovery from the original position
///    - **Return recovery result**: May succeed or fail
///
/// # Comparison with InplaceRecover
///
/// | Feature | `Recover` | `InplaceRecover` |
/// |---------|-----------|------------------|
/// | **Backtracking** | ✅ Restores position on error | ❌ Continues from error position |
/// | **Use Case** | Try alternative from same position | Skip ahead to resynchronize |
/// | **Performance** | Saves/restores checkpoint | No checkpoint overhead |
/// | **Example** | Parse expr or error node | Skip to next statement boundary |
///
/// **When to use**:
/// - `Recover`: Try completely different parsers from the same starting position
/// - `InplaceRecover`: Continue parsing from where the error occurred (e.g., skip tokens)
///
/// # Performance
///
/// - **Memory**: O(1) for checkpoint (just a position marker)
/// - **Runtime**: Two parser attempts in worst case
/// - **Backtracking**: Resets to saved position, no token re-buffering needed
///
/// # Error Handling
///
/// - Errors from the **primary parser are discarded** (recovery masks them)
/// - Errors from the **recovery parser are propagated**
/// - For error collection, use an emitter that accumulates errors
///
/// # See Also
///
/// - [`InplaceRecover`] - Error recovery without backtracking
/// - [`PeekThenChoice`] - Deterministic choice (no error recovery)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Recover<P, R, O, L, Ctx, Lang: ?Sized = ()> {
  parser: P,
  recoverer: R,
  _m: PhantomData<O>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _l: PhantomData<L>,
}

impl<P, R, O, L, Ctx, Lang: ?Sized> Recover<P, R, O, L, Ctx, Lang> {
  /// Creates a new `Recover` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(parser: P, recoverer: R) -> Self {
    Self {
      parser,
      recoverer,
      _m: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _l: PhantomData,
    }
  }
}

impl<'inp, P, R, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for Recover<P, R, O, L, Ctx, Lang>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  R: RecoverInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let ckp = inp.save();
    match self.parser.parse_input(inp) {
      Ok(output) => Ok(output),
      Err(e) => {
        inp.restore(ckp);
        self.recoverer.recover_input(inp, e)
      }
    }
  }
}

/// A parser that provides error recovery without backtracking, continuing from error position.
///
/// This combinator implements **error recovery without checkpoint restoration**. If the main
/// parser fails, the recovery parser starts from **where the error occurred**, not from the
/// original starting position.
///
/// This is useful for:
/// - **Panic mode recovery**: Skip tokens until a synchronization point
/// - **Resynchronization**: Find the next safe point to continue parsing
/// - **Performance**: Avoid checkpoint overhead when you don't need backtracking
///
/// # Type Parameters
///
/// - `P`: The primary parser to try first
/// - `R`: The recovery parser starting from the error position
/// - `O`: Output type (both parsers must produce the same type)
/// - `L`: Lexer type
/// - `Ctx`: Parse context
/// - `Lang`: Language marker type (default `()`)
///
/// # Examples
///
/// ## Skip to Synchronization Point
///
/// ```ignore
/// use tokit::parser::ParseInput;
///
/// // Parse statement, skip to semicolon on error
/// let parser = parse_statement()
///     .inplace_recover(
///         skip_to(|tok| matches!(tok, Token::Semicolon))
///             .then_ignore(any())
///             .map(|_| Statement::Error)
///     );
///
/// // Input: "let x = 1;"     → Ok(LetStmt { .. })
/// // Input: "bad ### ; ok"   → Ok(Statement::Error)
/// //             ^^^ ^
/// //        error here, skip to semicolon from error position
/// ```
///
/// ## Block-Level Recovery
///
/// ```ignore
/// // Parse block item, skip to next closing brace on error
/// let parser = parse_block_item()
///     .inplace_recover(
///         skip_to(|tok| matches!(tok, Token::RBrace | Token::Semicolon))
///             .map(|_| Item::Error)
///     );
///
/// // Continues parsing from where error occurred
/// ```
///
/// ## Forward-Only Error Recovery
///
/// ```ignore
/// // Parser that never backtracks, only moves forward
/// let parser = parse_token()
///     .inplace_recover(
///         any()  // Consume whatever token caused the error
///             .map(|tok| Value::Error(tok))
///     );
///
/// // Always makes progress, never goes back
/// ```
///
/// # How It Works
///
/// 1. **Try primary parser**: Attempt to parse with the main parser
/// 2. **On success**: Return the parsed value
/// 3. **On failure**:
///    - **Keep current position**: Don't restore any checkpoint
///    - **Try recovery parser**: Start recovery from current (error) position
///    - **Return recovery result**: May succeed or fail
///
/// # Comparison with Recover
///
/// | Feature | `Recover` | `InplaceRecover` |
/// |---------|-----------|------------------|
/// | **Backtracking** | ✅ Restores position | ❌ No backtracking |
/// | **Recovery starts from** | Original position | Error position |
/// | **Checkpoint overhead** | Yes (save/restore) | No |
/// | **Use Case** | Try alternatives | Skip to sync point |
///
/// **When to use**:
/// - `Recover`: When recovery needs to start from the beginning (alternative interpretation)
/// - `InplaceRecover`: When recovery should continue from error point (skip ahead)
///
/// # Performance
///
/// - **Memory**: O(1) - no checkpoint storage
/// - **Runtime**: Two parser attempts in worst case (like `Recover`)
/// - **No backtracking overhead**: More efficient when you don't need to reset position
///
/// # Error Handling
///
/// - Errors from the **primary parser are discarded**
/// - Errors from the **recovery parser are propagated**
/// - Recovery parser sees input from where the primary parser stopped
///
/// # See Also
///
/// - [`Recover`] - Error recovery with backtracking
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InplaceRecover<P, R, O, L, Ctx, Lang: ?Sized = ()> {
  parser: P,
  recoverer: R,
  _m: PhantomData<O>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _l: PhantomData<L>,
}

impl<P, R, O, L, Ctx, Lang: ?Sized> InplaceRecover<P, R, O, L, Ctx, Lang> {
  /// Creates a new `InplaceRecover` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(parser: P, recoverer: R) -> Self {
    Self {
      parser,
      recoverer,
      _m: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _l: PhantomData,
    }
  }
}

impl<'inp, P, R, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for InplaceRecover<P, R, O, L, Ctx, Lang>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  R: InplaceRecoverInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let ckp = inp.save();
    match self.parser.parse_input(inp) {
      Ok(output) => Ok(output),
      Err(e) => self.recoverer.inplace_recover_input(inp, ckp, e),
    }
  }
}
