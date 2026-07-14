use crate::emitter::{TooFewEmitter, TooManyEmitter};

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<AllowLeading<AllowTrailing<Bounded<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<AllowTrailing<Bounded<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<AllowTrailing<Bounded<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  map_depth = 4,
  cardinality = bounded,
  policy = [AllowLeading, AllowTrailing],
  emitters = {
    + TooManyEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
