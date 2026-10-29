use crate::emitter::MissingTrailingSeparatorEmitter;

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowLeading<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>, Delim>],
  map_depth = 3,
  cardinality = unbounded,
  policy = [AllowLeading, RequireTrailing],
  emitters = { + MissingTrailingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
