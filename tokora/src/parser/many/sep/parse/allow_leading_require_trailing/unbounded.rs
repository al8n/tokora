use crate::emitter::MissingTrailingSeparatorEmitter;

use super::*;

impl_separated_parse! {
  owned_type = [AllowLeading<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  ref_type = [AllowLeading<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  wrapper_type = [AllowLeading<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [AllowLeading, RequireTrailing],
  emitters = { + MissingTrailingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = false,
}
