use crate::emitter::{TooManyEmitter, UnexpectedLeadingSeparatorEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [AllowTrailing<AtMost<Separated<F, Sep, O, L, Ctx, Lang>>>],
  ref_type = [AllowTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  wrapper_type = [AllowTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = at_most,
  policy = [AllowTrailing],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
