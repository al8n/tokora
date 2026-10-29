use crate::emitter::UnexpectedTrailingSeparatorEmitter;

use super::*;

impl_separated_while_parse! {
  owned_type = [AllowLeading<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  ref_type = [AllowLeading<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>],
  wrapper_type = [AllowLeading<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [AllowLeading],
  emitters = { + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = false,
  block4_inline = false,
}
