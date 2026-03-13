use crate::emitter::{TooFewEmitter, TooManyEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowLeading<Bounded<Separated<F, Sep, O, L, Ctx, Lang>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  map_depth = 3,
  cardinality = bounded,
  policy = [AllowLeading],
  emitters = {
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
