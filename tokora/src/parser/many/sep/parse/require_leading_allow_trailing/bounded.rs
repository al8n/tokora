use crate::emitter::{MissingLeadingSeparatorEmitter, TooFewEmitter, TooManyEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [RequireLeading<AllowTrailing<Bounded<Separated<F, Sep, O, L, Ctx, Lang>>>>],
  ref_type = [RequireLeading<AllowTrailing<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  wrapper_type = [RequireLeading<AllowTrailing<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  map_depth = 3,
  cardinality = bounded,
  policy = [RequireLeading, AllowTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
