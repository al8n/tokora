use crate::emitter::{
  TooManyEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AtMost<Separated<F, Sep, O, L, Ctx, Lang>>, Delim>],
  ref_type = [DelimitedBy<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  wrapper_type = [DelimitedBy<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  map_depth = 2,
  cardinality = at_most,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
