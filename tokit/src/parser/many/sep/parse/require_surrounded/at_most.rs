use crate::emitter::{
  MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter, TooManyEmitter,
};

use super::*;

impl_separated_parse! {
  owned_type = [RequireLeading<RequireTrailing<AtMost<Separated<F, Sep, O, L, Ctx, Lang>>>>],
  ref_type = [RequireLeading<RequireTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  wrapper_type = [RequireLeading<RequireTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  map_depth = 3,
  cardinality = at_most,
  policy = [RequireLeading, RequireTrailing],
  emitters = {
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
