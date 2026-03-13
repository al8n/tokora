use crate::emitter::UnexpectedLeadingSeparatorEmitter;

use super::*;

impl_separated_while_parse! {
  owned_type = [AllowTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  ref_type = [AllowTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  wrapper_type = [AllowTrailing<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [AllowTrailing],
  emitters = { + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = false,
}
