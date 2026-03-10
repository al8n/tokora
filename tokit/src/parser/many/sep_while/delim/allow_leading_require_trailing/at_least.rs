use crate::emitter::{MissingTrailingSeparatorEmitter, TooFewEmitter};

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<AllowLeading<RequireTrailing<AtLeast<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<RequireTrailing<AtLeast<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<RequireTrailing<AtLeast<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  map_depth = 4,
  cardinality = at_least,
  policy = [AllowLeading, RequireTrailing],
  emitters = {
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
