use crate::emitter::{MissingLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireLeading<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>, Delim>],
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
