use crate::{
  container::Container as ContainerT,
  emitter::{UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter},
};

use super::*;

impl_separated_while_parse! {
  owned_type = [SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>],
  ref_type = [SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>],
  wrapper_type = [SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>],
  map_depth = 0,
  cardinality = unbounded,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = false,
  block4_inline = false,
}
