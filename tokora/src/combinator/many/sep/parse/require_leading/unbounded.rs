use crate::emitter::{MissingLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [RequireLeading<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>],
  ref_type = [RequireLeading<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>],
  wrapper_type = [RequireLeading<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [RequireLeading],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = false,
}
