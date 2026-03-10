use crate::emitter::{MissingLeadingSeparatorEmitter, TooManyEmitter};

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireLeading<AllowTrailing<AtMost<Separated<F, Sep, O, L, Ctx, Lang>>>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<AllowTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<AllowTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>, Delim>],
  map_depth = 4,
  cardinality = at_most,
  policy = [RequireLeading, AllowTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
