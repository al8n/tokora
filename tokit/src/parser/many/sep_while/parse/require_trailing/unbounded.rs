use crate::emitter::{MissingTrailingSeparatorEmitter, UnexpectedLeadingSeparatorEmitter};

use super::*;

impl_separated_while_parse! {
  owned_type = [RequireTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  ref_type = [RequireTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  wrapper_type = [RequireTrailing<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [RequireTrailing],
  emitters = {
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = false,
  block4_inline = false,
}
