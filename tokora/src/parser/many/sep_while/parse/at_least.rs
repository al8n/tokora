use crate::emitter::{
  TooFewEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_while_parse! {
  owned_type = [AtLeast<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  ref_type = [AtLeast<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>],
  wrapper_type = [AtLeast<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>],
  map_depth = 1,
  cardinality = at_least,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
