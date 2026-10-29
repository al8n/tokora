use crate::emitter::{TooFewEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_while_parse! {
  owned_type = [AllowLeading<AtLeast<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  ref_type = [AllowLeading<AtLeast<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  wrapper_type = [AllowLeading<AtLeast<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = at_least,
  policy = [AllowLeading],
  emitters = {
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
