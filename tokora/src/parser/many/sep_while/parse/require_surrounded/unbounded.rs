use crate::emitter::{MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter};

use super::*;

impl_separated_while_parse! {
  owned_type = [RequireLeading<RequireTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  ref_type = [RequireLeading<RequireTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  wrapper_type = [RequireLeading<RequireTrailing<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [RequireLeading, RequireTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = false,
  block4_inline = false,
}
