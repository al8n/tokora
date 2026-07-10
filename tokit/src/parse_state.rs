use super::{input::Cursor, *};

/// A parsing state passed to parser functions.
pub struct ParseState<'a, 'inp, 'closure, L, Ctx, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  inp: &'a mut InputRef<'inp, 'closure, L, Ctx, Lang>,
  start: Cursor<'inp, 'closure, L>,
}

impl<'a, 'inp, 'closure, L, Ctx, Lang: ?Sized> ParseState<'a, 'inp, 'closure, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Create a new `ParseState`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(
    inp: &'a mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    start: Cursor<'inp, 'closure, L>,
  ) -> Self {
    Self { inp, start }
  }

  /// Returns the span covering the output being parsed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span(&self) -> L::Span {
    self.inp.span_since(&self.start)
  }

  /// Returns a mutable reference to an emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn emitter(&mut self) -> &mut Ctx::Emitter {
    self.inp.emitter()
  }

  /// Returns the state of the lexer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state(&self) -> &L::State {
    self.inp.state()
  }

  /// Returns a mutable reference to the state of the lexer.
  ///
  /// # Checkpoint invalidation
  ///
  /// Replacing the lexer state re-keys every offset-dependent fact the input tracks
  /// (cache spans, dedup watermark, poison boundary). All outstanding checkpoints are
  /// invalidated; restoring one afterwards is a contract violation (debug builds
  /// panic).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn state_mut(&mut self) -> &mut L::State {
    self.inp.state_mut()
  }

  /// Returns the source slice covering the output being parsed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice(&self) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    self.inp.slice_since(&self.start)
  }
}
