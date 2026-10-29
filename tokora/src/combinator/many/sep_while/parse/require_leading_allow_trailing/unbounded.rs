use crate::emitter::MissingLeadingSeparatorEmitter;

use super::*;

impl_separated_while_parse! {
  owned_type = [RequireLeading<AllowTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  ref_type = [RequireLeading<AllowTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  wrapper_type = [RequireLeading<AllowTrailing<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [RequireLeading, AllowTrailing],
  emitters = { + MissingLeadingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = false,
  block4_inline = false,
}
