use crate::emitter::TooFewEmitter;

use super::*;

impl_separated_while_parse! {
  owned_type = [AllowLeading<AllowTrailing<AtLeast<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>>],
  ref_type = [AllowLeading<AllowTrailing<AtLeast<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>>],
  wrapper_type = [AllowLeading<AllowTrailing<AtLeast<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>>],
  map_depth = 3,
  cardinality = at_least,
  policy = [AllowLeading, AllowTrailing],
  emitters = { + TooFewEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
