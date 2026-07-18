use crate::emitter::{
  TooFewEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  owned_type = [AtLeast<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>],
  ref_type = [AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>],
  wrapper_type = [AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>],
  map_depth = 1,
  cardinality = at_least,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
