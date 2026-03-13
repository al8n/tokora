use crate::emitter::{
  TooFewEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  variant = at_least,
  owned_type = [AtLeast<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
