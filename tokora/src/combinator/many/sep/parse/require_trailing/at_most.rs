use crate::emitter::{
  MissingTrailingSeparatorEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  owned_type = [RequireTrailing<AtMost<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  ref_type = [RequireTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  wrapper_type = [RequireTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  map_depth = 2,
  cardinality = at_most,
  policy = [RequireTrailing],
  emitters = {
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
