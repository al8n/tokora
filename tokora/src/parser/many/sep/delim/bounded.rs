use crate::emitter::{
  TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
  UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<Bounded<Separated<F, Sep, O, L, Ctx, Lang>>, Delim>],
  ref_type = [DelimitedBy<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  wrapper_type = [DelimitedBy<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  map_depth = 2,
  cardinality = bounded,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
