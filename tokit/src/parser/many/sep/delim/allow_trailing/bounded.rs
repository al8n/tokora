use crate::emitter::{TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowTrailing<Bounded<Separated<F, Sep, O, L, Ctx, Lang>>>, Delim>],
  ref_type = [DelimitedBy<AllowTrailing<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowTrailing<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  map_depth = 3,
  cardinality = bounded,
  policy = [AllowTrailing],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
