/// Generates the 4 `ParseInput` impl blocks for `sep/parse/` leaf files.
///
/// Each leaf file in `sep/parse/` implements the same pattern with 4 impl blocks
/// that differ only in the wrapping type, emitter bounds, inline attributes, and
/// constructor details.
///
/// The `@map` and `@body` internal rules handle the parts that vary, dispatched
/// by a variant identifier (`bare`, `at_least`, `at_most`, `bounded`).
macro_rules! impl_separated_parse {
  // --- Internal dispatch for map_self (block 1) ---
  (@map_self bare $self:ident) => {
    $self.as_mut().map_parser(|p| p.as_mut())
  };
  (@map_self $variant:ident $self:ident) => {
    $self.as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut()))
  };

  // --- Internal dispatch for map_primary (block 2) ---
  (@map_primary bare $self:ident) => {
    $self.primary_mut().as_mut().map_parser(|p| p.as_mut())
  };
  (@map_primary $variant:ident $self:ident) => {
    $self.primary_mut().as_mut().map_parser(|p| p.map_parser_mut(|p| p.as_mut()))
  };

  // --- Internal dispatch for block 3 body ---
  (@block3 bare $self:ident $inp:ident) => {{
    let (parser, container) = $self.parts_mut();
    let f = parser.fn_mut();
    let parser = Collect::new(Separated::new::<Sep>(&mut **f), &mut **container);
    Wrapper(parser).parse_input($inp)
  }};
  (@block3 at_least $self:ident $inp:ident) => {{
    let (parser, container) = $self.parts_mut();
    let minimum = parser.minimum();
    let f = parser.parser_mut().fn_mut();
    let parser = AtLeast::new(Separated::new::<Sep>(&mut **f), minimum.get());
    Wrapper(Collect::new(parser, &mut **container)).parse_input($inp)
  }};
  (@block3 at_most $self:ident $inp:ident) => {{
    let (parser, container) = $self.parts_mut();
    let maximum = parser.maximum();
    let f = parser.parser_mut().fn_mut();
    let parser = AtMost::new(Separated::new::<Sep>(&mut **f), maximum.get());
    Wrapper(Collect::new(parser, &mut **container)).parse_input($inp)
  }};
  (@block3 bounded $self:ident $inp:ident) => {{
    let (parser, container) = $self.parts_mut();
    let maximum = parser.maximum();
    let minimum = parser.minimum();
    let f = parser.parser_mut().fn_mut();
    let parser = Bounded::new(Separated::new::<Sep>(&mut **f), maximum.get(), minimum.get());
    Wrapper(Collect::new(parser, &mut **container)).parse_input($inp)
  }};

  // --- Internal dispatch for block 4 body ---
  (@block4 bare $self:ident $inp:ident) => {{
    const HANDLER: &Unbounded = &Unbounded;
    let (parser, container) = $self.0.parts_mut();
    parser.parse($inp, container, HANDLER, HANDLER, HANDLER)
  }};
  (@block4 at_least $self:ident $inp:ident) => {{
    let (parser, container) = $self.0.parts_mut();
    let minimum = parser.minimum();
    parser.parser_mut().parse($inp, container, &minimum, &minimum, &minimum)
  }};
  (@block4 at_most $self:ident $inp:ident) => {{
    let (parser, container) = $self.0.parts_mut();
    let limitation = parser.maximum();
    parser.parser_mut().parse($inp, container, &limitation, &limitation, &limitation)
  }};
  (@block4 bounded $self:ident $inp:ident) => {{
    let (parser, container) = $self.0.parts_mut();
    let limitation = parser.to_with();
    parser.parser_mut().parse($inp, container, &limitation, &limitation, &limitation)
  }};

  // --- Inline attribute helper ---
  (@inline true $($item:tt)*) => { #[cfg_attr(not(tarpaulin), inline(always))] $($item)* };
  (@inline false $($item:tt)*) => { $($item)* };

  // --- Main entry point ---
  (
    variant = $variant:ident,
    owned_type = [$($owned:tt)*],
    ref_type = [$($reft:tt)*],
    wrapper_type = [$($wt:tt)*],
    emitters = {$($emitters:tt)*},
    block3_inline = $b3i:ident,
    block4_inline = $b4i:ident $(,)?
  ) => {
    // Block 1: owned -> Container
    impl<'inp, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, Container, Ctx, Lang>
      for Collect<$($owned)*, Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper(impl_separated_parse!(@map_self $variant self))
          .parse_input(inp)
          .map(|_| mem::take(&mut self.container))
      }
    }

    // Block 2: owned -> Spanned<Container>
    impl<'inp, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
      for With<Collect<$($owned)*, Container, Ctx, Lang>, PhantomSpan>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn parse_input(
        &mut self,
        inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
        Wrapper(impl_separated_parse!(@map_primary $variant self))
          .parse_input(inp)
          .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
      }
    }

    // Block 3: &mut ref -> L::Span
    impl<'inp, 'c, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Collect<&'c mut $($reft)*, &'c mut Container, Ctx, Lang>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L>,
    {
      impl_separated_parse!(@inline $b3i
        fn parse_input(
          &mut self,
          input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L: Lexer<'inp>,
          Ctx: ParseContext<'inp, L, Lang>,
        {
          impl_separated_parse!(@block3 $variant self input)
        }
      );
    }

    struct Wrapper<T>(T);

    // Block 4: Wrapper -> L::Span
    impl<'inp, 'c, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
      ParseInput<'inp, L, L::Span, Ctx, Lang>
      for Wrapper<Collect<$($wt)*, &'c mut Container, Ctx, Lang>>
    where
      L: Lexer<'inp>,
      F: TryParseInput<'inp, L, O, Ctx, Lang>,
      Sep: Punctuator<'inp, L, Lang>,
      Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
        + FullContainerEmitter<'inp, L, Lang>
        $($emitters)*,
      Ctx: ParseContext<'inp, L, Lang>,
      Container: ContainerT<O> + SeparatorHandler<'inp, L>,
    {
      impl_separated_parse!(@inline $b4i
        fn parse_input(
          &mut self,
          inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
        ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
          impl_separated_parse!(@block4 $variant self inp)
        }
      );
    }
  };
}
