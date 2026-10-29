use crate::emitter::{MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [RequireLeading<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  ref_type = [RequireLeading<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  wrapper_type = [RequireLeading<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [RequireLeading, RequireTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = false,
}
