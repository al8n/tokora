use crate::emitter::{
  MissingTrailingSeparatorEmitter, TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
};

use super::*;

impl_separated_while_parse! {
  owned_type = [RequireTrailing<Bounded<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  ref_type = [RequireTrailing<Bounded<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  wrapper_type = [RequireTrailing<Bounded<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = bounded,
  policy = [RequireTrailing],
  emitters = {
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
