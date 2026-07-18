use crate::emitter::{TooFewEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [AllowLeading<AtLeast<Separated<F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  ref_type = [AllowLeading<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  wrapper_type = [AllowLeading<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang, Cmpl>>>],
  map_depth = 2,
  cardinality = at_least,
  policy = [AllowLeading],
  emitters = {
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
