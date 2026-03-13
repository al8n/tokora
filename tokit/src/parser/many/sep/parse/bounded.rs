use crate::emitter::{
  TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
  UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  variant = bounded,
  owned_type = [Bounded<Separated<F, Sep, O, L, Ctx, Lang>>],
  ref_type = [Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  wrapper_type = [Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
