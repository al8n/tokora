use crate::emitter::{MissingLeadingSeparatorEmitter, TooFewEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [RequireLeading<AllowTrailing<AtLeast<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>>],
  ref_type = [RequireLeading<AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>],
  wrapper_type = [RequireLeading<AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>],
  map_depth = 3,
  cardinality = at_least,
  policy = [RequireLeading, AllowTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
