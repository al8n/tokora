use crate::emitter::TooManyEmitter;

use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowLeading<AllowTrailing<AtMost<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<AllowTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<AllowTrailing<AtMost<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>>, Delim>],
  map_depth = 4,
  cardinality = at_most,
  policy = [AllowLeading, AllowTrailing],
  emitters = { + TooManyEmitter<'inp, L, Lang> },
  block3_inline = true,
  block4_inline = true,
}
