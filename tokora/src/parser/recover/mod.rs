use crate::{
  error::{MaybeIncomplete, MaybeTerminal},
  input::Cursor,
};

use super::{
  recovery_gate::{Attempted, inplace_attempt, speculated_attempt},
  *,
};

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
/// use tokora::parser::{ParseInput, RecoverInput};
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
pub trait RecoverInput<'inp, L, O, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
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
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;
}

impl<'inp, L, O, Ctx, Lang: ?Sized, Cmpl, F> RecoverInput<'inp, L, O, Ctx, Lang, Cmpl> for F
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
  F: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[inline(always)]
  fn recover_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (self)(input, err)
  }
}

/// A trait for recovery parsers that continue from the error position without backtracking.
///
/// This trait defines the interface for recovery parsers used by [`InplaceRecover`]. When the
/// primary parser fails, implementors of this trait receive the error and a [`Cursor`] marking
/// where the primary parser started, and attempt to produce a valid output by parsing from the
/// current (error) position.
///
/// Unlike [`RecoverInput`], the input position is **not** restored - recovery continues from
/// where the primary parser stopped. The recovery handler receives a [`Cursor`] rather than a
/// [`Checkpoint`](crate::input::Checkpoint) because checkpoints are single-use backtracking
/// capabilities (restoring one consumes it); a recovery handler only needs the position facts
/// the cursor carries, e.g. to span from the start of the failed parse.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for closures with the signature:
/// ```ignore
/// FnMut(&mut InputRef, Cursor, Error) -> Result<O, Error>
/// ```
///
/// # Example
///
/// ```ignore
/// use tokora::parser::{ParseInput, InplaceRecoverInput};
///
/// // Manual implementation
/// struct SkipToSemicolon;
///
/// impl InplaceRecoverInput<'_, MyLexer, Stmt, MyContext> for SkipToSemicolon {
///     fn inplace_recover_input(&mut self, input, _cursor, _err) -> Result<Stmt, Error> {
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
/// parser.inplace_recover(|input, _cursor, _err| {
///     // Skip to semicolon from error position
///     skip_to_semicolon(input)?;
///     Ok(Stmt::Error)
/// })
/// ```
///
/// See [`InplaceRecover`] for usage examples.
pub trait InplaceRecoverInput<'inp, L, O, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  /// Try to recover from a parsing error without backtracking.
  ///
  /// This method is called when the primary parser fails. Unlike [`RecoverInput::recover_input`],
  /// the input position has **not** been restored - it remains at the error position.
  ///
  /// # Parameters
  ///
  /// - `input`: Input reference at the current (error) position
  /// - `cursor`: A [`Cursor`] marking where the primary parser started — a position
  ///   view, not a restorable checkpoint (recovery cannot backtrack)
  /// - `err`: The error produced by the failed primary parser
  ///
  /// # Returns
  ///
  /// - `Ok(output)`: Successfully recovered with a value
  /// - `Err(error)`: Recovery failed
  fn inplace_recover_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    cursor: Cursor<'inp, '_, L>,
    err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;
}

impl<'inp, L, O, Ctx, Lang: ?Sized, Cmpl, F> InplaceRecoverInput<'inp, L, O, Ctx, Lang, Cmpl> for F
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
  F: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    Cursor<'inp, '_, L>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[inline(always)]
  fn inplace_recover_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
    cursor: Cursor<'inp, '_, L>,
    err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (self)(input, cursor, err)
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
/// use tokora::parser::ParseInput;
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
/// # The never-recoverable law, and its terminal dual
///
/// Two failures are never handed to the recoverer, whatever it would have produced from them:
///
/// - an [`Incomplete`](crate::error::Incomplete) — *more input may fix this*, so synthesizing over
///   it fabricates output from input that has not finished arriving;
/// - a **terminal stop** — a resource limit the parse tripped, which *no input ever clears*.
///   Re-entering the parser from a recoverer only re-trips the same limit.
///
/// Terminality is read from **all four** places this crate stores it, not from the error value
/// alone: [`MaybeTerminal::is_terminal`](crate::error::MaybeTerminal::is_terminal) for a stop the
/// grammar's error type carries; the input's session trip counter for a **descent** budget; and the
/// input's poison latch for a **scanner** trip. The last two exist because a conversion into the
/// grammar's error type may discard the marker — `()` does — and because a *rejecting* emitter
/// reports a scanner trip by returning the value its `From<<L::Token as Token>::Error>` builds,
/// with no [`UnexpectedEnd`](crate::error::UnexpectedEnd) anywhere on that path to mark.
///
/// All four are attempt-relative and all four are *read* on the failure path only. What a
/// successful attempt pays is the **three** baselines taken before it, and it is three rather than
/// two because the two counters are separate cells with separate granularities:
///
/// * `InputRef::trip_snapshot` — a `u64` load plus the issuing input's address, which is the
///   nonce that makes a cross-fed baseline a panic rather than an answer;
/// * `InputRef::scanner_trip_snapshot` — a second `u64` load;
/// * `InputRef::latch_snapshot` — one clone of the latch, the same `Option<L::Offset>` the
///   checkpoint this combinator already saves clones for its own rollback set.
///
/// So: two `u64` loads — one machine word each on a 64-bit target, two each on a 32-bit one — one
/// address read, and one `Option<L::Offset>` clone, per attempt.
///
/// # See Also
///
/// - [`InplaceRecover`] - Error recovery without backtracking
#[cfg_attr(
  feature = "peek",
  doc = " - [`PeekThenChoice`] - Deterministic choice (no error recovery)"
)]
#[cfg_attr(
  not(feature = "peek"),
  doc = " - `PeekThenChoice` - Deterministic choice (no error recovery)"
)]
///
/// # Caveats
///
/// When the primary parser fails, `Recover` rolls the input back to the
/// pre-attempt state — position, lexer state, **and** diagnostics: emissions
/// made by the failed attempt are rewound from the
/// [`Emitter`](crate::Emitter)'s log, so a successful recovery leaves no
/// spurious errors behind. The failure itself reaches the [`RecoverInput`]
/// implementation as the error value; whether to emit a diagnostic for it is
/// the recoverer's decision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Recover<P, R, O, L, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: P,
  recoverer: R,
  _m: PhantomData<O>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _l: PhantomData<L>,
  _cmpl: PhantomData<Cmpl>,
}

impl<P, R, O, L, Ctx, Lang: ?Sized, Cmpl> Recover<P, R, O, L, Ctx, Lang, Cmpl> {
  /// Creates a new `Recover` parser.
  #[inline(always)]
  pub(crate) const fn new(parser: P, recoverer: R) -> Self {
    Self {
      parser,
      recoverer,
      _m: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _l: PhantomData,
      _cmpl: PhantomData,
    }
  }
}

impl<'inp, P, R, L, O, Ctx, Lang, Cmpl> ParseInput<'inp, L, O, Ctx, Lang, Cmpl>
  for Recover<P, R, O, L, Ctx, Lang, Cmpl>
where
  P: ParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  R: RecoverInput<'inp, L, O, Ctx, Lang, Cmpl>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: MaybeIncomplete + MaybeTerminal,
  Lang: ?Sized,
  Cmpl: Completeness,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    // Speculate through the recovery chokepoint's `speculated_attempt`, which wraps the primary in
    // `try_attempt`: it saves a checkpoint before the primary runs, and on `Ok` keeps the progress
    // while dropping the checkpoint's lineage id — closing the success-path leak where a bare
    // `save` left an orphan id on the live stack for every valid parse. On `Err` it restores that
    // checkpoint (rewinding to the pre-parse state, see [`restore`](InputRef::restore)) and hands
    // the error back, so the recoverer runs from the restored position exactly as the raw
    // save/restore pair did.
    //
    // The never-recoverable law and its terminal dual are the chokepoint's, and so are the two
    // baselines they are read against: an `Incomplete` (more input may fix this) and a terminal
    // stop (no input ever will) both ride the `Err` channel untouched, because recovery fabricates
    // a value from a *malformed* construct and neither an unfinished construct nor a tripped budget
    // is that. FIVE readings, in the five places the two conditions are stored — see
    // [`recovery_gate`](super::recovery_gate) for each of them, for why the error value alone can
    // never answer for a scanner stop a rejecting emitter reported, and for why the input-side
    // readings have to happen inside the attempt rather than after it.
    match speculated_attempt(inp, |input| self.parser.parse_input(input)) {
      Attempted::Done(output) => Ok(output),
      Attempted::Reraise(e) => Err(e),
      Attempted::Recoverable(e) => self.recoverer.recover_input(inp, e),
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
/// use tokora::parser::ParseInput;
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
/// # The never-recoverable law, and its terminal dual
///
/// An [`Incomplete`](crate::error::Incomplete) error, and a **terminal stop**, are re-raised
/// untouched — before the recovery parser runs — exactly as [`Recover`] and
/// [`skip_then_retry`](crate::ParseInput::skip_then_retry) do, off the same five witnesses
/// [`Recover`] documents. Recovery synthesizes progress over a *malformed* construct; an incomplete
/// one is merely unfinished and a tripped limit is not a construct at all. See
/// [`MaybeIncomplete`](crate::error::MaybeIncomplete) and
/// [`MaybeTerminal`](crate::error::MaybeTerminal).
///
/// Checking before the handler runs matters more here than anywhere else: this path never
/// backtracks, so it cannot undo a recovery it should never have begun.
///
/// # See Also
///
/// - [`Recover`] - Error recovery with backtracking
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InplaceRecover<P, R, O, L, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: P,
  recoverer: R,
  _m: PhantomData<O>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _l: PhantomData<L>,
  _cmpl: PhantomData<Cmpl>,
}

impl<P, R, O, L, Ctx, Lang: ?Sized, Cmpl> InplaceRecover<P, R, O, L, Ctx, Lang, Cmpl> {
  /// Creates a new `InplaceRecover` parser.
  #[inline(always)]
  pub(crate) const fn new(parser: P, recoverer: R) -> Self {
    Self {
      parser,
      recoverer,
      _m: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _l: PhantomData,
      _cmpl: PhantomData,
    }
  }
}

impl<'inp, P, R, L, O, Ctx, Lang, Cmpl> ParseInput<'inp, L, O, Ctx, Lang, Cmpl>
  for InplaceRecover<P, R, O, L, Ctx, Lang, Cmpl>
where
  P: ParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  R: InplaceRecoverInput<'inp, L, O, Ctx, Lang, Cmpl>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: MaybeIncomplete + MaybeTerminal,
  Lang: ?Sized,
  Cmpl: Completeness,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    // The in-place path never backtracks: hand the recovery handler a position view
    // (the cursor where the primary parser started), not a restorable checkpoint.
    let cursor = inp.cursor().clone();
    // The never-recoverable law and its terminal dual, through the same chokepoint [`Recover`]
    // uses — `inplace_attempt` differs from it in exactly one respect, that there is no checkpoint
    // and so no rollback, and in none of the five witnesses it reads. Checking the failure BEFORE
    // the handler runs is the only correct order here: this path cannot undo a recovery it should
    // never have begun.
    match inplace_attempt(inp, |input| self.parser.parse_input(input)) {
      Attempted::Done(output) => Ok(output),
      Attempted::Reraise(e) => Err(e),
      Attempted::Recoverable(e) => self.recoverer.inplace_recover_input(inp, cursor, e),
    }
  }
}

// ── RECOVERY_GATE_CENSUS_END — production above, tests below ──────────────────
//
// `RECOVERY_GATE_CENSUS` (in `parser/recovery_gate.rs`) reads this file's source and counts
// needles that its own test fixtures also spell. The split is this marker rather than the first
// `#[cfg(test)]`, because the suites below are feature-gated and their attribute is not that
// literal; the census `expect()`s the marker, so deleting it fails loudly instead of silently
// widening the scan over test code.

// The no-growth regression needs a lexer that actually runs (so `save`/`next` push and
// commit real checkpoints), which pins it to `logos` + `std` — the same set the
// `live_checkpoints_len` accessor is gated to.
#[cfg(all(test, feature = "logos_0_16", feature = "std"))]
mod tests;
