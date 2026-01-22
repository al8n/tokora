use core::mem::MaybeUninit;

use generic_arraydeque::{ArrayLength, GenericArrayDeque, array::GenericArray, typenum};

use crate::{
  cache::Peeked,
  input::InputRef,
  located::Located,
  parser::*,
  punct::*,
  slice::Sliced,
  span::Spanned,
  token::PunctuatorToken,
  utils::marker::{PhantomLocated, PhantomSliced, PhantomSpan},
};

use super::*;

mod sealed {
  pub trait Sealed {}
}

/// A trait for parsers that specify the capacity of their peek buffer.
pub trait Window: sealed::Sealed {
  /// The capacity of the peek buffer.
  type CAPACITY: ArrayLength;

  /// Create an uninitialized array of the specified capacity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn array<T>() -> GenericArray<MaybeUninit<T>, Self::CAPACITY> {
    GenericArray::uninit()
  }

  /// Create a deque of the specified capacity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deque<T>() -> GenericArrayDeque<MaybeUninit<T>, Self::CAPACITY> {
    GenericArrayDeque::new()
  }
}

macro_rules! peek_buf_capacity_impl_for_typenum {
  ($($size:literal), + $(,)?) => {
    paste::paste! {
      $(
        impl sealed::Sealed for typenum::[< U $size >] {}

        impl Window for typenum::[< U $size >] {
          type CAPACITY = typenum::[< U $size >];
        }
      )*
    }
  };
}

seq_macro::seq!(N in 1..=32 {
  peek_buf_capacity_impl_for_typenum! {
    #(N,)*
  }
});

/// Decision action for conditional parsing.
pub trait Decision<'inp, L, E, W, Lang: ?Sized = ()> {
  /// Decide the next action based on the peeked tokens.
  fn decide(&mut self, toks: Peeked<'_, 'inp, L, W>, emitter: &mut E) -> Result<Action, E::Error>
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L, Lang>,
    W: Window;
}

impl<'inp, F, L, E, W, Lang: ?Sized> Decision<'inp, L, E, W, Lang> for F
where
  F: FnMut(Peeked<'_, 'inp, L, W>, &mut E) -> Result<Action, E::Error>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  W: Window,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn decide(&mut self, toks: Peeked<'_, 'inp, L, W>, emitter: &mut E) -> Result<Action, E::Error>
  where
    W: Window,
  {
    (self)(toks, emitter)
  }
}

/// A trait for parsers that accumulate their results into a container.
pub trait Accumulator<'inp, L, Container, Ctx, Lang: ?Sized = ()> {
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn collect(self) -> Collect<Self, Container, Ctx, Lang>
  where
    Self: Sized,
    Container: Default,
    Collect<Self, Container, Ctx, Lang>: ParseInput<'inp, L, Container, Ctx, Lang>,
  {
    Collect::new(self, Container::default())
  }

  /// Collects the parsed elements with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn collect_with(self, container: Container) -> Collect<Self, Container, Ctx, Lang>
  where
    Self: Sized,
    Collect<Self, Container, Ctx, Lang>: ParseInput<'inp, L, Container, Ctx, Lang>,
  {
    Collect::new(self, container)
  }
}

impl<'inp, P, Container, L, Ctx, Lang: ?Sized> Accumulator<'inp, L, Container, Ctx, Lang> for P where
  Collect<P, Container, Ctx, Lang>: ParseInput<'inp, L, Container, Ctx, Lang>
{
}

macro_rules! define_separated_by {
  ($($name:ident),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Creates a `SeparatedWhile` combinator which separates elements by the `" $name:snake "` separator and applies this parser repeatedly."]
        ///
        /// See [`separated_while`](crate::ParseInput::separated_while) for details.
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn [< separated_by_ $name:snake _while>]<Condition, W>(
          self,
          condition: Condition,
        ) -> SeparatedWhile<Self, $name, Condition, O, W, L, Ctx, Lang>
        where
          Self: Sized,
          L: Lexer<'inp>,
          L::Token: PunctuatorToken<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
          Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
          W: Window,
        {
          SeparatedWhile::new(self, condition)
        }
      )*
    }
  };
}

/// Core trait implemented by every parser combinator.
///
/// This mirrors the ergonomics of libraries like `winnow`: a parser is
/// simply something that can mutate an [`InputRef`] and either produce
/// a value or a spanned error using the configured `Emitter`.
pub trait ParseInput<'inp, L, O, Ctx, Lang: ?Sized = ()> {
  /// Try to parse from the given input.
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  /// Wraps the output of this parser in a `Spanned` with the span of the parsed input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn spanned(self) -> With<PhantomSpan, Self>
  where
    Self: Sized,
    L: Lexer<'inp>,
  {
    With::new(PhantomSpan::phantom(), self)
  }

  /// Wraps the output of this parser in a `Sliced` with the source slice of the parsed input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn sliced(self) -> With<PhantomSliced, Self>
  where
    Self: Sized,
    L: Lexer<'inp>,
  {
    With::new(PhantomSliced::phantom(), self)
  }

  /// Wraps the output of this parser in a `Located` with the span and source slice of the parsed input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn located(self) -> With<PhantomLocated, Self>
  where
    Self: Sized,
    L: Lexer<'inp>,
  {
    With::new(PhantomLocated::phantom(), self)
  }

  /// Ignores the output of this parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn ignored(self) -> Ignore<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ignore<Self, O, L, Ctx, Lang>: ParseInput<'inp, L, (), Ctx, Lang>,
  {
    Ignore::new(self)
  }

  /// Creates a parser over a mutable reference to this parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn by_ref(&mut self) -> &mut ByRef<Self> {
    ByRef::from_ref_mut(self)
  }

  /// Creates a `RepeatedWhile` combinator that applies this parser repeatedly, where **you
  /// provide the lookahead logic**.
  ///
  /// The parser will be called repeatedly until:
  /// - Your condition function returns `Action::Stop` - you decided to stop based on lookahead
  /// - It returns `Err(e)` - fatal error
  ///
  /// ## Key Behavior
  ///
  /// Unlike [`repeated()`](TryParseInput::repeated), this parser doesn't need built-in lookahead:
  /// - **You provide** a condition function that peeks ahead at tokens
  /// - Condition decides `Continue` or `Stop` based on what it sees
  /// - Element parser is only called when condition says `Continue`
  ///
  /// ## Type Parameters
  ///
  /// - `W`: Window size for lookahead (e.g., `U1` for 1 token, `U2` for 2 tokens)
  ///
  /// ## See Also
  ///
  /// - [`repeated`](TryParseInput::repeated) - Parser has lookahead, no separator
  /// - [`Action`] - The decision type (`Continue` or `Stop`)
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn repeated_while<Condition, W>(
    self,
    condition: Condition,
  ) -> RepeatedWhile<Self, Condition, O, W, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W::CAPACITY>,
    W: Window,
  {
    RepeatedWhile::new(self, condition)
  }

  /// Creates a `SeparatedWhile` combinator that parses separated elements, where **you
  /// provide the lookahead logic**.
  ///
  /// The parser will be called repeatedly to parse elements separated by the given separator,
  /// until:
  /// - Your condition function returns `Action::Stop` - you decided to stop based on lookahead
  /// - It returns `Err(e)` - fatal error
  ///
  /// ## Key Behavior
  ///
  /// Unlike [`separated()`](TryParseInput::separated), this parser doesn't need built-in lookahead:
  /// - **You provide** a condition function that peeks ahead at tokens
  /// - Condition decides `Continue` or `Stop` based on what it sees
  /// - Element parser is only called when condition says `Continue`
  /// - Separator is parsed between elements
  ///
  /// ## Type Parameters
  ///
  /// - `W`: Window size for lookahead (e.g., `U1` for 1 token, `U2` for 2 tokens)
  ///
  /// ## See Also
  ///
  /// - [`separated`](TryParseInput::separated) - Parser has lookahead, with separator
  /// - [`Action`] - The decision type (`Continue` or `Stop`)
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn separated_while<SepClassifier, Condition, W>(
    self,
    condition: Condition,
  ) -> SeparatedWhile<Self, SepClassifier, Condition, O, W, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    SepClassifier: Punctuator<'inp, L, Lang>,
    W: Window,
  {
    SeparatedWhile::new(self, condition)
  }

  define_separated_by!(
    Comma,
    Semicolon,
    Dot,
    Colon,
    Pipe,
    Ampersand,
    Hyphen,
    Underscore,
    DoubleColon,
    Arrow,
    FatArrow,
    Tilde,
    Slash,
    Backslash,
    Percent,
    Dollar,
    Hash,
    At,
  );

  /// Creates a `PeekThen` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `C` returns `Ok(())`, the inner parser is applied, otherwise,
  /// parsing is stopped and return the error from the handler.
  fn peek_then<C, W>(self, condition: C) -> PeekThen<Self, C, L::Token, W>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    C: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    W: Window,
    PeekThen<Self, C, L::Token, W>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    PeekThen::of(self, condition)
  }

  /// Creates a `PeekThen` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `C` returns `Ok(Action::Continue)`, the inner parser is applied,
  /// otherwise returns `None`.
  #[doc(alias = "or_not")]
  fn peek_then_try<C, W>(self, condition: C) -> PeekThen<Self, C, L::Token, W>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    C: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
    PeekThen<Self, C, L::Token, W>: TryParseInput<'inp, L, O, Ctx, Lang>,
  {
    PeekThen::of(self, condition)
  }

  /// Map the output of this parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn map<U, F>(self, f: F) -> Map<Self, F, L, Ctx, O, U, Lang>
  where
    Self: Sized,
    F: FnMut(O) -> U,
    L: Lexer<'inp>,
    Map<Self, F, L, Ctx, O, U, Lang>: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    Map::new(self, f)
  }

  /// Map the output of this parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn map_with<U, F>(self, f: F) -> MapWith<Self, F, L, Ctx, O, U, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(O, ParseState<'_, 'inp, '_, L, Ctx, Lang>) -> U,
    MapWith<Self, F, L, Ctx, O, U, Lang>: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    MapWith::new(self, f)
  }

  /// Filter the output of this parser using a validation function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn filter<F>(self, validator: F) -> Filter<Self, F, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
    Filter<Self, F, O, L, Ctx, Lang>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    Filter::of(self, validator)
  }

  /// Filter the output of this parser using a validation function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn filter_with<F>(self, validator: F) -> FilterWith<Self, F, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(
      &O,
      ParseState<'_, 'inp, '_, L, Ctx, Lang>,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
    FilterWith<Self, F, O, L, Ctx, Lang>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    FilterWith::of(self, validator)
  }

  /// Filter and map the output of this parser using a validation/transformation function.
  ///
  /// The parser must produce a `Spanned<O>` value. The mapper receives
  /// the data and span, and returns `Ok(new_value)` or an error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn filter_map<U, F>(self, mapper: F) -> FilterMap<Self, F, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(O) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
    FilterMap<Self, F, O, U, L, Ctx, Lang>: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    FilterMap::of(self, mapper)
  }

  /// Filter and map the output of this parser using a validation/transformation function.
  ///
  /// The parser must produce a `Spanned<O>` value. The mapper receives
  /// the data and span, and returns `Ok(new_value)` or an error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn filter_map_with<U, F>(self, mapper: F) -> FilterMapWith<Self, F, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(
      O,
      ParseState<'_, 'inp, '_, L, Ctx, Lang>,
    ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
    FilterMapWith<Self, F, O, U, L, Ctx, Lang>: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    FilterMapWith::of(self, mapper)
  }

  /// Validate the output of this parser with full location context.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn validate<F>(self, validator: F) -> Validate<Self, F, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Validate::of(self, validator)
  }

  /// Validate the output of this parser with full location context.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn validate_with<F>(self, validator: F) -> ValidateWith<Self, F, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(
      &O,
      ParseState<'_, 'inp, '_, L, Ctx, Lang>,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    ValidateWith::of(self, validator)
  }

  /// Sequence this parser with another, ignoring the output of the second.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn then_ignore<G, U>(self, second: G) -> ThenIgnore<Self, G, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    G: ParseInput<'inp, L, U, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    ThenIgnore<Self, G, O, U, L, Ctx, Lang>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    ThenIgnore::new(self, second)
  }

  /// Sequence this parser with another, using the first result to determine the second parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn and_then<T, U>(self, then: T) -> AndThen<Self, T, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    T: FnMut(O) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    AndThen<Self, T, O, U, L, Ctx, Lang>: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    AndThen::new(self, then)
  }

  /// Sequence this parser with another, using the first result to determine the second parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn and_then_with<T, U>(self, then: T) -> AndThenWith<Self, T, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    T: FnMut(
      O,
      ParseState<'_, 'inp, '_, L, Ctx, Lang>,
    ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
    L: Lexer<'inp>,
    AndThenWith<Self, T, O, U, L, Ctx, Lang>: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    AndThenWith::new(self, then)
  }

  /// Sequence this parser with another, keeping both outputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn then<T, U>(self, then: T) -> Then<Self, T, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    T: ParseInput<'inp, L, U, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    Then<Self, T, O, U, L, Ctx, Lang>: ParseInput<'inp, L, (O, U), Ctx, Lang>,
  {
    Then::new(self, then)
  }

  /// Sequence this parser with another, ignoring the output of the first.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn ignore_then<G, U>(self, second: G) -> IgnoreThen<Self, G, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    G: ParseInput<'inp, L, U, Ctx, Lang>,
    IgnoreThen<Self, G, O, U, L, Ctx, Lang>: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    IgnoreThen::new(self, second)
  }

  /// Recover from errors by trying an alternative parser with backtracking.
  ///
  /// If this parser fails, the input position is reset to where it was before parsing,
  /// and the recovery parser is tried from the original position. This enables trying
  /// completely different parsing strategies when the primary approach fails.
  ///
  /// # Use Cases
  ///
  /// - **Alternative interpretations**: Try parsing as different constructs
  /// - **Fallback values**: Return error/placeholder nodes on failure
  /// - **Resilient parsing**: Continue parsing to find more issues
  ///
  /// # Example
  ///
  /// ```ignore
  /// // Parse expression or fallback to error node
  /// let parser = parse_expression()
  ///     .recover(parse_error_node());
  ///
  /// // Input: "1 + 2"      → Ok(BinaryOp(Add, 1, 2))
  /// // Input: "@ invalid" → Ok(ErrorNode(...))
  /// ```
  ///
  /// # Comparison with inplace_recover
  ///
  /// - `recover()`: Resets to starting position, tries alternative from beginning
  /// - `inplace_recover()`: Continues from error position, typically skips ahead
  ///
  /// See [`Recover`] for detailed documentation and more examples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn recover<R>(self, recovery: R) -> Recover<Self, R, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    R: RecoverInput<'inp, L, O, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    Recover<Self, R, O, L, Ctx, Lang>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    Recover::new(self, recovery)
  }

  /// Recover from errors without backtracking, continuing from the error position.
  ///
  /// If this parser fails, the recovery parser starts from where the error occurred,
  /// not from the original starting position. This is typically used to skip ahead to
  /// a synchronization point (like a semicolon or brace) to resume parsing.
  ///
  /// # Use Cases
  ///
  /// - **Panic mode recovery**: Skip tokens until reaching a safe point
  /// - **Resynchronization**: Find the next statement/block boundary
  /// - **Performance**: Avoid checkpoint overhead when backtracking isn't needed
  ///
  /// # Example
  ///
  /// ```ignore
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
  /// //        error, skip to semicolon from here
  /// ```
  ///
  /// # Comparison with recover
  ///
  /// - `recover()`: Resets to starting position, tries alternative from beginning
  /// - `inplace_recover()`: Continues from error position, typically skips ahead
  ///
  /// See [`InplaceRecover`] for detailed documentation and more examples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn inplace_recover<R>(self, recovery: R) -> InplaceRecover<Self, R, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    R: InplaceRecoverInput<'inp, L, O, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    InplaceRecover<Self, R, O, L, Ctx, Lang>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    InplaceRecover::new(self, recovery)
  }

  /// Creates a parser that accepts any token with optional padding.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn padded(self) -> Padded<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Padded<Self, O, L, Ctx, Lang>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    Padded::new(self)
  }

  /// Creates a parser that accepts any token with optional padding.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn padded_left(self) -> PaddedLeft<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    PaddedLeft<Self, O, L, Ctx, Lang>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    PaddedLeft::new(self)
  }

  /// Creates a parser that accepts any token with optional padding.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn padded_right(self) -> PaddedRight<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    PaddedRight<Self, O, L, Ctx, Lang>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    PaddedRight::new(self)
  }
}

impl<'inp, F, L, O, Ctx, Lang: ?Sized> ParseInput<'inp, L, O, Ctx, Lang> for F
where
  F: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (self)(input)
  }
}

impl<'inp, F, L, O, Ctx, Lang: ?Sized> ParseInput<'inp, L, O, Ctx, Lang> for &mut ByRef<F>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (**self).parse_input(input)
  }
}

impl<'inp, L, O, Ctx, P, Lang: ?Sized> ParseInput<'inp, L, Spanned<O, L::Span>, Ctx, Lang>
  for With<PhantomSpan, P>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<O, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let cursor = inp.cursor().clone();
    self
      .secondary
      .parse_input(inp)
      .map(|output| Spanned::new(inp.span_since(&cursor), output))
  }
}

impl<'inp, L, O, Ctx, P, Lang: ?Sized>
  ParseInput<'inp, L, Sliced<O, <L::Source as Source<L::Offset>>::Slice<'inp>>, Ctx, Lang>
  for With<PhantomSliced, P>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Sliced<O, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    let cursor = inp.cursor().clone();
    self.secondary.parse_input(inp).map(|output| {
      Sliced::new(
        inp
          .slice_since(&cursor)
          .expect("parser should guarantee slice"),
        output,
      )
    })
  }
}

impl<'inp, L, O, Ctx, P, Lang: ?Sized>
  ParseInput<'inp, L, Located<O, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>, Ctx, Lang>
  for With<PhantomLocated, P>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Located<O, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    let cursor = inp.cursor().clone();
    self.secondary.parse_input(inp).map(|output| {
      Located::new(
        inp
          .slice_since(&cursor)
          .expect("parser should guarantee slice"),
        inp.span_since(&cursor),
        output,
      )
    })
  }
}

/// Extension trait for unwrapping `Option` outputs.
pub trait ParseInputUnwrapExt<'inp, L, O, Ctx, Lang: ?Sized> {
  /// Creates an `Unwrapped` parser that unwraps the `Option` result of this parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  fn unwrap(self) -> Unwrapped<Self, O, Ctx, Lang>
  where
    Self: Sized + ParseInput<'inp, L, Option<O>, Ctx, Lang>,
  {
    Unwrapped::new(self)
  }
}

impl<'inp, F, L, O, Ctx, Lang: ?Sized> ParseInputUnwrapExt<'inp, L, O, Ctx, Lang> for F
where
  F: ParseInput<'inp, L, Option<O>, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
}
