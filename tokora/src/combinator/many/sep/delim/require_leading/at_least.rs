use crate::emitter::{
  MissingLeadingSeparatorEmitter, TooFewEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireLeading<AtLeast<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  map_depth = 3,
  cardinality = at_least,
  policy = [RequireLeading],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
