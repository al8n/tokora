use crate::emitter::{MissingTrailingSeparatorEmitter, UnexpectedLeadingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang>>, Delim>],
  ref_type = [DelimitedBy<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  wrapper_type = [DelimitedBy<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [RequireTrailing],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
