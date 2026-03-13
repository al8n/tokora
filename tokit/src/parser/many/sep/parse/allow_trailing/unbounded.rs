use crate::emitter::UnexpectedLeadingSeparatorEmitter;

use super::*;

impl_separated_parse! {
  owned_type = [AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [AllowTrailing],
  emitters = { + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = false,
}
