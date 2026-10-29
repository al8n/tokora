use crate::emitter::{
  MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter, TooFewEmitter,
};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireLeading<RequireTrailing<AtLeast<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<RequireTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<RequireTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  map_depth = 4,
  cardinality = at_least,
  policy = [RequireLeading, RequireTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
