use crate::emitter::TooFewEmitter;

use super::*;

impl_separated_parse! {
  owned_type = [AllowLeading<AllowTrailing<AtLeast<Separated<F, Sep, O, L, Ctx, Lang>>>>],
  ref_type = [AllowLeading<AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  wrapper_type = [AllowLeading<AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  map_depth = 3,
  cardinality = at_least,
  policy = [AllowLeading, AllowTrailing],
  emitters = { + TooFewEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
