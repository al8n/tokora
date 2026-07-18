use crate::emitter::MissingLeadingSeparatorEmitter;

use super::*;

impl_separated_parse! {
  owned_type = [RequireLeading<AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  ref_type = [RequireLeading<AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  wrapper_type = [RequireLeading<AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [RequireLeading, AllowTrailing],
  emitters = { + MissingLeadingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = false,
}
