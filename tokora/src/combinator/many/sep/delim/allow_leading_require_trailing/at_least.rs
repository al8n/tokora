use crate::emitter::{MissingTrailingSeparatorEmitter, TooFewEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowLeading<RequireTrailing<AtLeast<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<RequireTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<RequireTrailing<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  map_depth = 4,
  cardinality = at_least,
  policy = [AllowLeading, RequireTrailing],
  emitters = {
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
