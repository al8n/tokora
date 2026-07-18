use crate::emitter::UnexpectedTrailingSeparatorEmitter;

use super::*;

impl_separated_parse! {
  owned_type = [AllowLeading<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>],
  ref_type = [AllowLeading<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>],
  wrapper_type = [AllowLeading<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [AllowLeading],
  emitters = { + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = false,
}
