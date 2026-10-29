use crate::emitter::MissingLeadingSeparatorEmitter;

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<RequireLeading<AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  map_depth = 3,
  cardinality = unbounded,
  policy = [RequireLeading, AllowTrailing],
  emitters = { + MissingLeadingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
