use crate::emitter::{MissingLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireLeading<Separated<F, Sep, O, L, Ctx, Lang>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [RequireLeading],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
