use crate::emitter::{
  TooManyEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  variant = at_most,
  owned_type = [AtMost<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
