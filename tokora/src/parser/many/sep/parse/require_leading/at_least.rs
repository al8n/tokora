use crate::emitter::{
  MissingLeadingSeparatorEmitter, TooFewEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  owned_type = [RequireLeading<AtLeast<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  ref_type = [RequireLeading<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  wrapper_type = [RequireLeading<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  map_depth = 2,
  cardinality = at_least,
  policy = [RequireLeading],
  emitters = {
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
