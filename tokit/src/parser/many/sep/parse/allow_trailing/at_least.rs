use crate::emitter::{TooFewEmitter, UnexpectedLeadingSeparatorEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [AllowTrailing<AtLeast<Separated<F, Sep, O, L, Ctx, Lang>>>],
  ref_type = [AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  wrapper_type = [AllowTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = at_least,
  policy = [AllowTrailing],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
