use crate::emitter::UnexpectedLeadingSeparatorEmitter;

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang>>, Delim>],
  ref_type = [DelimitedBy<AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  wrapper_type = [DelimitedBy<AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, Delim>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [AllowTrailing],
  emitters = { + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
