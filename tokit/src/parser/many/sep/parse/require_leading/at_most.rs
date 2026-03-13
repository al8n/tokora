use crate::emitter::{
  MissingLeadingSeparatorEmitter, TooManyEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_parse! {
  owned_type = [RequireLeading<AtMost<Separated<F, Sep, O, L, Ctx, Lang>>>],
  ref_type = [RequireLeading<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  wrapper_type = [RequireLeading<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = at_most,
  policy = [RequireLeading],
  emitters = {
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
