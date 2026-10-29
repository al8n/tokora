use crate::emitter::{
  MissingLeadingSeparatorEmitter, TooFewEmitter, TooManyEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireLeading<Bounded<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  map_depth = 3,
  cardinality = bounded,
  policy = [RequireLeading],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
