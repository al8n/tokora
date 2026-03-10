use crate::emitter::{TooFewEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowLeading<AtLeast<Separated<F, Sep, O, L, Ctx, Lang>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  map_depth = 3,
  cardinality = at_least,
  policy = [AllowLeading],
  emitters = {
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
