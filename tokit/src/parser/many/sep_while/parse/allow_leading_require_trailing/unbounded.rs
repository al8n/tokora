use crate::emitter::MissingTrailingSeparatorEmitter;

use super::*;

impl_separated_while_parse! {
  owned_type = [AllowLeading<RequireTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  ref_type = [AllowLeading<RequireTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  wrapper_type = [AllowLeading<RequireTrailing<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [AllowLeading, RequireTrailing],
  emitters = { + MissingTrailingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = false,
  block4_inline = false,
}
