use crate::emitter::{UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>, Delim>],
  ref_type = [DelimitedBy<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>, Delim>],
  wrapper_type = [DelimitedBy<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>, Delim>],
  map_depth = 1,
  cardinality = unbounded,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
