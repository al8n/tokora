use crate::emitter::{MissingTrailingSeparatorEmitter, UnexpectedLeadingSeparatorEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [RequireTrailing],
  emitters = {
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = false,
}
