use crate::emitter::UnexpectedTrailingSeparatorEmitter;

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowLeading<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>, Delim>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [AllowLeading],
  emitters = { + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
