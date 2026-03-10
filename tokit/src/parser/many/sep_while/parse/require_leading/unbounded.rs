use crate::emitter::{MissingLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_while_parse! {
  owned_type = [RequireLeading<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  ref_type = [RequireLeading<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  wrapper_type = [RequireLeading<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [RequireLeading],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = false,
  block4_inline = false,
}
