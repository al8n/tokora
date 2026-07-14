use crate::emitter::{
  TooManyEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_while_parse! {
  owned_type = [AtMost<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  ref_type = [AtMost<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>],
  wrapper_type = [AtMost<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>],
  map_depth = 1,
  cardinality = at_most,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
