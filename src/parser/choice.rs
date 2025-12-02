use super::*;

/// A choice of multiple parsers.
pub trait ParseChoice<'inp, L, O, Ctx, Lang: ?Sized = ()> {
  /// The id of the parser branch.
  type Id;

  /// Parses using branch identified by `id`.
  fn parse_choice(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
    id: &Self::Id,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

macro_rules! tuple_choice {
  (@output $end:literal; $($param:literal),+ $(,)?) => {
    ::paste::paste! {
      impl<'inp, L, O, Ctx, Lang: ?Sized, $([< P $param >]),+>
        ParseChoice<'inp, L, O, Ctx, Lang>
        for ($([< P $param >],)+)
      where
        L: Lexer<'inp>,
        Ctx: ParseContext<'inp, L, Lang>,
        $([< P $param >]: ParseInput<'inp, L, O, Ctx, Lang>),+
      {
        type Id = deranged::RangedU8<0, $end>;

        fn parse_choice(
          &mut self,
          inp: &mut InputRef<'inp, '_, L, <Ctx>::Emitter, <Ctx>::Cache, Lang>,
          id: &Self::Id,
        ) -> Result<O, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error> {
          match id.get() {
            $($param => self.$param.parse_input(inp),)+
            _ => unreachable!("deranged::RangedU8 guarantees in-bounds"),
          }
        }
      }
    }
  };
  (@mid $end:literal) => {
    seq_macro::seq!(N in 0..=$end {
      tuple_choice!(@output $end; #(N,)*);
    });
  };
  ($end:literal) => {
    seq_macro::seq!(E in 0..=$end {
      tuple_choice!(@mid E);
    });
  };
}

tuple_choice!(32);

impl<'inp, L, O, Ctx, Lang: ?Sized, P, const N: usize> ParseChoice<'inp, L, O, Ctx, Lang> for [P; N]
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
{
  type Id = deranged::RangedUsize<0, N>;

  fn parse_choice(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, <Ctx>::Emitter, <Ctx>::Cache, Lang>,
    id: &Self::Id,
  ) -> Result<O, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self[id.get()].parse_input(inp)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::boxed::Box;

  impl<'inp, L, O, Ctx, T, Lang: ?Sized> ParseChoice<'inp, L, O, Ctx, Lang> for Box<T>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    T: ParseChoice<'inp, L, O, Ctx, Lang>,
  {
    type Id = T::Id;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn parse_choice(
      &mut self,
      inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
      id: &Self::Id,
    ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
      (**self).parse_choice(inp, id)
    }
  }
};
