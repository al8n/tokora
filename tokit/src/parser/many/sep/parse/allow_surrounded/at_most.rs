use crate::emitter::TooManyEmitter;

use super::*;

impl_separated_parse! {
  owned_type = [AllowLeading<AllowTrailing<AtMost<Separated<F, Sep, O, L, Ctx, Lang>>>>],
  ref_type = [AllowLeading<AllowTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  wrapper_type = [AllowLeading<AllowTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  map_depth = 3,
  cardinality = at_most,
  policy = [AllowLeading, AllowTrailing],
  emitters = { + TooManyEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
