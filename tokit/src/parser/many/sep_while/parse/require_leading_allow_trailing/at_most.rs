use crate::emitter::{MissingLeadingSeparatorEmitter, TooManyEmitter};

use super::*;

impl_separated_while_parse! {
  owned_type = [RequireLeading<AllowTrailing<AtMost<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>>],
  ref_type = [RequireLeading<AllowTrailing<AtMost<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>>],
  wrapper_type = [RequireLeading<AllowTrailing<AtMost<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>>],
  map_depth = 3,
  cardinality = at_most,
  policy = [RequireLeading, AllowTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
