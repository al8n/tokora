//! Parse context trait and implementations.
//!
//! A parse context bundles the error emitter and cache configuration
//! used during parsing. This provides better type inference and a
//! simpler API compared to configuring emitter and cache separately.

use core::marker::PhantomData;

use crate::{
  Cache, Emitter, Lexer,
  cache::DefaultCache,
  emitter::{ComposableEmitter, Fatal},
  input::InputContext,
  lexer::SliceOf,
};

/// A context that provides emitter and cache configuration for parsing.
pub trait ParseContext<'inp, L, Lang: ?Sized = ()> {
  /// The emitter type used for error handling.
  type Emitter: Emitter<'inp, L, Lang>
  where
    L: Lexer<'inp>;

  /// The cache type used for lookahead.
  type Cache: Cache<'inp, L, Lang>
  where
    L: Lexer<'inp>;

  /// Provides the emitter and cache instances for parsing.
  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>;
}

impl<'inp, L, Lang: ?Sized> ParseContext<'inp, L, Lang> for ()
where
  L: Lexer<'inp>,
  Fatal<(), Lang>: Emitter<'inp, L, Lang>,
{
  type Emitter = Fatal<(), Lang>;
  type Cache = DefaultCache<'inp, L>;

  #[inline(always)]
  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>,
  {
    InputContext::new(Fatal::of(), DefaultCache::<'inp, L>::new())
  }
}

/// Custom context: use a custom emitter and cache pair.
impl<'inp, L, E, C, Lang: ?Sized> ParseContext<'inp, L, Lang> for (E, C)
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  C: Cache<'inp, L, Lang>,
{
  type Emitter = E;
  type Cache = C;

  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>,
  {
    InputContext::new(self.0, self.1)
  }
}

/// Convenient type alias for the default parse context.
pub type FatalContext<'inp, L, Error, Lang = ()> =
  ParserContext<'inp, L, Fatal<Error, Lang>, DefaultCache<'inp, L>, Lang>;

/// A concrete [`ParseContext`] implementation that holds an emitter and optional cache options.
pub struct ParserContext<'inp, L, E, C = DefaultCache<'inp, L>, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  C: Cache<'inp, L, Lang>,
{
  emitter: E,
  cache: Option<C::Options>,
  _marker: PhantomData<&'inp L>,
  _lang: PhantomData<Lang>,
}

impl<'inp, L, E, C> ParserContext<'inp, L, E, C>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  /// Creates a new parser context with the given emitter.
  #[inline(always)]
  pub const fn new(emitter: E) -> Self {
    Self::of(emitter)
  }

  /// Creates a new parser context with the given emitter and cache options.
  #[inline(always)]
  pub const fn with_cache_options(emitter: E, options: C::Options) -> Self {
    Self::with_cache_options_of(emitter, options)
  }
}

impl<'inp, L, E, C, Lang: ?Sized> ParserContext<'inp, L, E, C, Lang>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  C: Cache<'inp, L, Lang>,
{
  const fn new_in(emitter: E, opts: Option<C::Options>) -> Self {
    Self {
      emitter,
      cache: opts,
      _marker: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a new parser context with the given emitter for a specific language.
  #[inline(always)]
  pub const fn of(emitter: E) -> Self {
    Self::new_in(emitter, None)
  }

  /// Creates a new parser context with the given emitter and cache options for a specific language.
  #[inline(always)]
  pub const fn with_cache_options_of(emitter: E, options: C::Options) -> Self {
    Self::new_in(emitter, Some(options))
  }
}

impl<'inp, L, E, C, Lang: ?Sized> ParseContext<'inp, L, Lang> for ParserContext<'inp, L, E, C, Lang>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  C: Cache<'inp, L, Lang>,
{
  type Emitter = E;
  type Cache = C;

  #[inline(always)]
  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>,
  {
    match self.cache {
      Some(options) => InputContext::new(self.emitter, C::with_options(options)),
      None => InputContext::new(self.emitter, C::new()),
    }
  }
}

/// The error type context `Ctx`'s emitter produces.
///
/// Generic parser signatures name this projection in every return type, and spelling it
/// out means chaining [`ParseContext::Emitter`] through [`Emitter::Error`]. `ErrorOf`
/// names that path once, so `Result<T, ErrorOf<'inp, L, Ctx, Lang>>` stays legible.
///
/// # Examples
///
/// The alias is definitionally the nested projection — this identity function compiles
/// precisely because the two spellings are the same type:
///
/// ```rust
/// use tokora::{Emitter, ErrorOf, Lexer, ParseContext};
///
/// fn same_type<'inp, L, Ctx>(
///   err: <Ctx::Emitter as Emitter<'inp, L>>::Error,
/// ) -> ErrorOf<'inp, L, Ctx, ()>
/// where
///   L: Lexer<'inp>,
///   Ctx: ParseContext<'inp, L>,
/// {
///   err
/// }
/// ```
pub type ErrorOf<'inp, L, Ctx, Lang> =
  <<Ctx as ParseContext<'inp, L, Lang>>::Emitter as Emitter<'inp, L, Lang>>::Error;

/// The context bundle a generic parser atom takes.
///
/// Implemented for every [`ParseContext`] whose emitter is a
/// [`ComposableEmitter`](crate::emitter::ComposableEmitter) and whose source slice is
/// [`Clone`], so an atom needs only `Ctx: ParseCtx<'inp, L>` to unlock the entire
/// emitter surface. The emitter requirement rides on the [`ParseContext`] supertrait as
/// an associated-type bound so it elaborates to callers of the bundle rather than having
/// to be restated at every use site; the `SliceOf<'inp, L>: Clone` requirement lives on
/// the blanket impl (a projection bound cannot be a supertrait), so it gates which
/// contexts qualify without forcing that clause onto every mention of the bound. Atoms
/// that clone a slice restate that one bound locally.
///
/// # Examples
///
/// One `ParseCtx` bound stands in for the context ladder — the bundled function can call
/// into code demanding individual emitter capabilities of `Ctx::Emitter`:
///
/// ```rust
/// use tokora::{Lexer, ParseCtx};
/// use tokora::emitter::{
///   SeparatedEmitter, TooFewEmitter, UnclosedEmitter, UnexpectedTrailingSeparatorEmitter,
/// };
///
/// fn needs_diagnostics<'inp, L, E>()
/// where
///   L: Lexer<'inp>,
///   E: SeparatedEmitter<'inp, L>
///     + TooFewEmitter<'inp, L>
///     + UnexpectedTrailingSeparatorEmitter<'inp, L>
///     + UnclosedEmitter<'inp, L>,
/// {
/// }
///
/// // The single bound elaborates: the whole family is available on `Ctx::Emitter`.
/// fn atom_shaped<'inp, L, Ctx>()
/// where
///   L: Lexer<'inp>,
///   Ctx: ParseCtx<'inp, L>,
/// {
///   needs_diagnostics::<L, Ctx::Emitter>()
/// }
/// ```
pub trait ParseCtx<'inp, L, Lang: ?Sized = ()>:
  ParseContext<'inp, L, Lang, Emitter: ComposableEmitter<'inp, L, Lang>>
where
  L: Lexer<'inp>,
{
}

impl<'inp, L, Lang: ?Sized, T> ParseCtx<'inp, L, Lang> for T
where
  L: Lexer<'inp>,
  SliceOf<'inp, L>: Clone,
  T: ParseContext<'inp, L, Lang, Emitter: ComposableEmitter<'inp, L, Lang>>,
{
}

#[cfg(test)]
mod tests;
