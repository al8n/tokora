use crate::emitter::TooFewEmitter;

use super::*;

impl_separated_parse! {
  owned_type = [AllowLeading<AllowTrailing<AtLeast<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>>],
  ref_type = [AllowLeading<AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>],
  wrapper_type = [AllowLeading<AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>],
  map_depth = 3,
  cardinality = at_least,
  policy = [AllowLeading, AllowTrailing],
  emitters = { + TooFewEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
