use crate::{
  Window, cache::Peeked, input::InputRef, parser::PeekThenChoice, try_parse_input::ParseAttempt,
};

use super::*;

/// A choice of multiple parsers.
pub trait ParseChoice<'inp, L, O, Ctx, Lang: ?Sized = ()> {
  /// The id of the parser branch.
  type Id;

  /// Parses using branch identified by `id`.
  fn parse_choice(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    id: &Self::Id,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  /// Parses using branch identified by `id`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_parse_choice(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    id: Option<&Self::Id>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    match id {
      Some(id) => self.parse_choice(inp, id).map(ParseAttempt::Accept),
      None => Ok(ParseAttempt::Decline),
    }
  }

  /// Creates a `PeekThenChoice` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `H` returns `Ok(id)`, the inner choice parser is applied with the given id, otherwise,
  /// parsing is stopped and return the error from the handler.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn peek_then_choice<H, W: Window>(self, condition: H) -> PeekThenChoice<Self, H, L, Ctx, W, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    H: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<Self::Id, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    PeekThenChoice::of(self, condition)
  }

  /// Creates a `PeekThenChoice` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `H` returns `Ok(id)`, the inner choice parser is applied with the given id, otherwise,
  /// parsing is stopped and return the error from the handler.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn peek_then_try_choice<H, W: Window>(
    self,
    condition: H,
  ) -> PeekThenChoice<Self, H, L, Ctx, W, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    H: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<Option<Self::Id>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    PeekThenChoice::of(self, condition)
  }
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
        type Id = Branch<$end>;

        fn parse_choice(
          &mut self,
          inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
          id: &Self::Id,
        ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
          match id.id() {
            $($param => self.$param.parse_input(inp),)+
            _ => unreachable!(concat!("Branch<", stringify!($end), "> guarantees in-bounds")),
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

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_choice(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    id: &Self::Id,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self[id.get()].parse_input(inp)
  }
}

impl<'inp, L, O, Ctx, Lang: ?Sized, P> ParseChoice<'inp, L, O, Ctx, Lang> for [P]
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
{
  type Id = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_choice(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    id: &Self::Id,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self[*id].parse_input(inp)
  }
}

impl<'inp, L, O, Ctx, Lang: ?Sized, P> ParseChoice<'inp, L, O, Ctx, Lang> for &mut [P]
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
{
  type Id = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_choice(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    id: &Self::Id,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self[*id].parse_input(inp)
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
      inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
      id: &Self::Id,
    ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
      (**self).parse_choice(inp, id)
    }
  }
};

/// Branch identifier for choice parsers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Branch<const N: usize>(usize);

impl<const N: usize> Branch<N> {
  /// Returns the matched branch id.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn id(&self) -> usize {
    self.0
  }
}

#[allow(non_upper_case_globals)]
mod sealed {
  use super::Branch;

  macro_rules! bound {
    ($($param: literal),+$(,)?) => {
      paste::paste! {
        $(
          #[doc(hidden)]
          pub trait [< _ $param >] {}
        )*
      }
    };
  }

  seq_macro::seq!(N in 1..=32 {
    bound!(#(N,)*);
  });

  impl<const N: usize> Branch<N> {
    /// The zeroth branch.
    pub const B0: Self = Branch(0);
  }

  macro_rules! const_value {
    ($(
      $(#[$meta:meta])*
      $id:literal
    ),+$(,)?) => {
      paste::paste! {
        $(
          impl<const N: usize> Branch<N>
          where
            Self: [< _ $id >],
          {
            $(#[$meta])*
            pub const [<B $id>]: Self = Branch($id);
          }
        )*
      }
    };
  }

  macro_rules! impl_bound {
    (@inner $end:literal; $($param:literal),+ $(,)?) => {
      ::paste::paste! {
        $(
          impl [< _ $param >] for Branch<$end>
          {}
        )*
      }
    };
    ($end:literal) => {
      paste::paste! {
        seq_macro::seq!(P in 1..=$end {
          impl_bound!(@inner $end; P);
        });
      }
    };
  }

  seq_macro::seq!(E in 1..=32 {
    impl_bound!(E);
  });

  const_value!(
    /// The first branch.
    1,
    /// The second branch.
    2,
    /// The third branch.
    3,
    /// The fourth branch.
    4,
    /// The fifth branch.
    5,
    /// The sixth branch.
    6,
    /// The seventh branch.
    7,
    /// The eighth branch.
    8,
    /// The ninth branch.
    9,
    /// The tenth branch.
    10,
    /// The eleventh branch.
    11,
    /// The twelfth branch.
    12,
    /// The thirteenth branch.
    13,
    /// The fourteenth branch.
    14,
    /// The fifteenth branch.
    15,
    /// The sixteenth branch.
    16,
    /// The seventeenth branch.
    17,
    /// The eighteenth branch.
    18,
    /// The nineteenth branch.
    19,
    /// The twentieth branch.
    20,
    /// The twenty-first branch.
    21,
    /// The twenty-second branch.
    22,
    /// The twenty-third branch.
    23,
    /// The twenty-fourth branch.
    24,
    /// The twenty-fifth branch.
    25,
    /// The twenty-sixth branch.
    26,
    /// The twenty-seventh branch.
    27,
    /// The twenty-eighth branch.
    28,
    /// The twenty-ninth branch.
    29,
    /// The thirtieth branch.
    30,
    /// The thirty-first branch.
    31,
    /// The thirty-second branch.
    32,
  );
}
