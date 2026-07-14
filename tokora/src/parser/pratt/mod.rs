use crate::{Emitter, InputRef, Lexer, ParseContext, ParseInput, span::Spanned};

pub use expr::*;
mod expr;

/// The power level of an operator, used to determine the order of operations in Pratt parsing.
///
/// Implemented by tokora for every standard integer type (saturating at the type's bounds), so a
/// plain `i64` — the default `Power` of [`Precedenced`] — works out of the box:
///
/// ```
/// use tokora::parser::PrattPower;
///
/// assert_eq!(3i64.next(), 4);
/// assert_eq!(3i64.prev(), 2);
/// // Saturating at the representable bounds — never wraps, never panics.
/// assert_eq!(u8::MAX.next(), u8::MAX);
/// assert_eq!(i8::MIN.prev(), i8::MIN);
/// ```
///
/// Custom implementations (a domain-specific precedence ladder, a newtype with named levels)
/// remain first-class; the integer impls only cover the common numeric case that previously
/// forced every consumer into a newtype (the orphan rule bars downstream
/// `impl PrattPower for i64`).
pub trait PrattPower: Default + Clone + Ord {
  /// Returns the next higher power level.
  fn next(&self) -> Self;

  /// Returns the previous lower power level.
  ///
  /// # Important
  ///
  /// This is called for right-associative operators to compute the minimum
  /// precedence for the recursive parse. If your implementation uses numeric
  /// types, ensure `prev()` uses saturating subtraction to avoid
  /// underflow/panic when called on the minimum representable value.
  fn prev(&self) -> Self;
}

macro_rules! impl_pratt_power_for_int {
  ($($int:ty),+ $(,)?) => {
    $(
      /// Saturating integer binding power: `next` adds one and `prev` subtracts one, both
      /// clamped at the type's representable bounds, honoring the trait's no-underflow
      /// requirement on [`prev`](PrattPower::prev).
      impl PrattPower for $int {
        #[inline(always)]
        fn next(&self) -> Self {
          self.saturating_add(1)
        }

        #[inline(always)]
        fn prev(&self) -> Self {
          self.saturating_sub(1)
        }
      }
    )+
  };
}

impl_pratt_power_for_int!(
  i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

/// A type with an associated precedence level, used in Pratt parsing.
#[derive(Debug, Clone, Copy)]
pub struct Precedenced<T, Power = i64> {
  token: T,
  precedence: Power,
}

impl<T, Power> Precedenced<T, Power> {
  /// Creates a new `Precedenced` with the given token and precedence.
  #[inline(always)]
  pub const fn new(token: T, precedence: Power) -> Self {
    Self { token, precedence }
  }

  /// Returns a new `Precedenced` with the given token but a different precedence level.
  #[inline(always)]
  pub const fn with_precedence(token: T, precedence: Power) -> Self {
    Self { token, precedence }
  }

  /// Returns the reference to the token contained in this `Precedenced`.
  #[inline(always)]
  pub const fn token_ref(&self) -> &T {
    &self.token
  }

  /// Returns the mutable reference to the token contained in this `Precedenced`.
  #[inline(always)]
  pub const fn token_mut(&mut self) -> &mut T {
    &mut self.token
  }

  /// Returns the precedence level of this `Precedenced`.
  #[inline(always)]
  pub const fn precedence(&self) -> &Power {
    &self.precedence
  }

  /// Decomposes this `Precedenced` into its precedence.
  #[inline(always)]
  pub fn into_precedence(self) -> Power {
    self.precedence
  }

  /// Decomposes this `Precedenced` into its data.
  #[inline(always)]
  pub fn into_data(self) -> T {
    self.token
  }

  /// Decomposes this `Precedenced` into its token and precedence components.
  #[inline(always)]
  pub fn into_components(self) -> (T, Power) {
    (self.token, self.precedence)
  }
}

/// A left-hand side for Pratt parsing, which can be either an operand or a prefix operator with its precedence level.
#[derive(Debug, Clone, Copy)]
pub enum PrattLHS<Op, Pre, Power = i64> {
  /// A left-hand side that is an operand (not an operator).
  Operand(Op),
  /// A left-hand side that is a prefix operator with its precedence level.
  Prefix(Precedenced<Pre, Power>),
}

/// An infix operator for Pratt parsing, which can be left-associative, right-associative, or non-associative with its precedence level.
#[derive(Debug, Clone, Copy)]
pub enum PrattInfix<L, R, N> {
  /// A left-associative infix operator with its precedence level and operator type.
  Left(L),
  /// A right-associative infix operator with its precedence level and operator type.
  Right(R),
  /// A non-associative infix operator with its precedence level and operator type.
  Neither(N),
}

/// A right-hand side for Pratt parsing, which can be a left-associative, right-associative, or non-associative infix operator with its precedence level,
/// or a postfix operator with its precedence level.
#[derive(Debug, Clone, Copy)]
pub enum PrattRHS<L, R, N, Post, Power = i64> {
  /// An infix operator with its precedence level and associativity.
  Infix(Precedenced<PrattInfix<L, R, N>, Power>),
  /// Postfix operator with its precedence level and operator type.
  Postfix(Precedenced<Post, Power>),
}

/// A trait for parsing left-hand sides in Pratt parsing, which can be either operands or operators with precedence levels.
pub trait ParsePrattLHS<'inp, Power, Op, Pre, L, Ctx, Lang: ?Sized = ()> {
  /// Try to parse a pratt lhs from the lexer input, returning with its precedence level if successful.
  fn parse_pratt_lhs(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<PrattLHS<Op, Pre, Power>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, P, Power, L, Op, Pre, Ctx, Lang: ?Sized>
  ParsePrattLHS<'inp, Power, Op, Pre, L, Ctx, Lang> for P
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: ParseInput<'inp, L, PrattLHS<Op, Pre, Power>, Ctx, Lang>,
{
  #[inline(always)]
  fn parse_pratt_lhs(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<PrattLHS<Op, Pre, Power>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parse_input(input)
  }
}

/// A trait for parsing right-hand sides in Pratt parsing, which can be infix operators with precedence levels and associativity,
/// or postfix operators with precedence levels.
pub trait ParsePrattRHS<
  'inp,
  Power,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  Post,
  L,
  Ctx,
  Lang: ?Sized = (),
>
{
  /// Try to parse a pratt rhs from the lexer input, returning with its precedence level if successful.
  fn parse_pratt_rhs(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    PrattRHS<LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, P, Power, LeftAssoc, RightAssoc, NeitherAssoc, Post, L, Ctx, Lang: ?Sized>
  ParsePrattRHS<'inp, Power, LeftAssoc, RightAssoc, NeitherAssoc, Post, L, Ctx, Lang> for P
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: ParseInput<'inp, L, PrattRHS<LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>, Ctx, Lang>,
{
  #[inline(always)]
  fn parse_pratt_rhs(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    PrattRHS<LeftAssoc, RightAssoc, NeitherAssoc, Post, Power>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parse_input(input)
  }
}

/// A trait for postfix fold dispatch
pub trait PrattFoldPostfix<'inp, Power, Operator, L, O, Ctx, Lang: ?Sized = ()> {
  /// Apply the postfix fold to the operand.
  fn fold_postfix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    operand: O,
    operator: Precedenced<Operator, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, P, Power, Operator, L, O, Ctx, Lang: ?Sized>
  PrattFoldPostfix<'inp, Power, Operator, L, O, Ctx, Lang> for P
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang>,
    O,
    Precedenced<Operator, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[inline(always)]
  fn fold_postfix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    operand: O,
    operator: Precedenced<Operator, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self(input, operand, operator)
  }
}

/// A trait for infix fold dispatch
pub trait PrattFoldInfix<
  'inp,
  Power,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  L,
  O,
  Ctx,
  Lang: ?Sized = (),
>
{
  /// Apply the infix fold to the operand.
  fn fold_infix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    left: O,
    right: O,
    operator: Precedenced<PrattInfix<LeftAssoc, RightAssoc, NeitherAssoc>, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, P, Power, LO, RO, NO, L, O, Ctx, Lang: ?Sized>
  PrattFoldInfix<'inp, Power, LO, RO, NO, L, O, Ctx, Lang> for P
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang>,
    O,
    O,
    Precedenced<PrattInfix<LO, RO, NO>, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[inline(always)]
  fn fold_infix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    left: O,
    right: O,
    operator: Precedenced<PrattInfix<LO, RO, NO>, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self(input, left, right, operator)
  }
}

/// A trait for prefix fold dispatch
pub trait PrattFoldPrefix<'inp, Power, Operator, L, O, Ctx, Lang: ?Sized = ()> {
  /// Apply the prefix fold to the operand.
  fn fold_prefix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    operand: O,
    operator: Precedenced<Operator, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, P, Power, Operator, L, O, Ctx, Lang: ?Sized>
  PrattFoldPrefix<'inp, Power, Operator, L, O, Ctx, Lang> for P
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang>,
    O,
    Precedenced<Operator, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[inline(always)]
  fn fold_prefix(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    operand: O,
    operator: Precedenced<Operator, Power>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self(input, operand, operator)
  }
}

/// A trait for postfix fold dispatch
pub trait PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang: ?Sized = ()> {
  /// Apply the postfix fold to the operand.
  fn fold_postfix(
    &mut self,
    operand: Spanned<L::Token, L::Span>,
    operator: Spanned<L::Token, L::Span>,
    emitter: &mut Ctx::Emitter,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, P, Power, L, Ctx, Lang: ?Sized> PrattFoldTokenPostfix<'inp, Power, L, Ctx, Lang> for P
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: FnMut(
    Spanned<L::Token, L::Span>,
    Spanned<L::Token, L::Span>,
    &mut Ctx::Emitter,
  )
    -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[inline(always)]
  fn fold_postfix(
    &mut self,
    operand: Spanned<L::Token, L::Span>,
    operator: Spanned<L::Token, L::Span>,
    emitter: &mut Ctx::Emitter,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self(operand, operator, emitter)
  }
}

/// A trait for infix fold dispatch
pub trait PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang: ?Sized = ()> {
  /// Apply the infix fold to the operand.
  fn fold_infix(
    &mut self,
    left: Spanned<L::Token, L::Span>,
    right: Spanned<L::Token, L::Span>,
    infix: Spanned<PrattInfix<L::Token, L::Token, L::Token>, L::Span>,
    emitter: &mut Ctx::Emitter,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, P, Power, L, Ctx, Lang: ?Sized> PrattFoldTokenInfix<'inp, Power, L, Ctx, Lang> for P
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: FnMut(
    Spanned<L::Token, L::Span>,
    Spanned<L::Token, L::Span>,
    Spanned<PrattInfix<L::Token, L::Token, L::Token>, L::Span>,
    &mut Ctx::Emitter,
  )
    -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[inline(always)]
  fn fold_infix(
    &mut self,
    left: Spanned<L::Token, L::Span>,
    right: Spanned<L::Token, L::Span>,
    infix: Spanned<PrattInfix<L::Token, L::Token, L::Token>, L::Span>,
    emitter: &mut <Ctx>::Emitter,
  ) -> Result<Spanned<L::Token, L::Span>, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self(left, right, infix, emitter)
  }
}

/// A trait for prefix fold dispatch
pub trait PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang: ?Sized = ()> {
  /// Apply the prefix fold to the operand.
  fn fold_prefix(
    &mut self,
    operator: Spanned<L::Token, L::Span>,
    operand: Spanned<L::Token, L::Span>,
    emitter: &mut Ctx::Emitter,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'inp, P, Power, L, Ctx, Lang: ?Sized> PrattFoldTokenPrefix<'inp, Power, L, Ctx, Lang> for P
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: FnMut(
    Spanned<L::Token, L::Span>,
    Spanned<L::Token, L::Span>,
    &mut Ctx::Emitter,
  )
    -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
{
  #[inline(always)]
  fn fold_prefix(
    &mut self,
    operator: Spanned<L::Token, L::Span>,
    operand: Spanned<L::Token, L::Span>,
    emitter: &mut Ctx::Emitter,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self(operator, operand, emitter)
  }
}
