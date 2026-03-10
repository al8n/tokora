use crate::emitter::{MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireLeading<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang>>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  map_depth = 3,
  cardinality = unbounded,
  policy = [RequireLeading, RequireTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
