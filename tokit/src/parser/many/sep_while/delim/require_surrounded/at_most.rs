use crate::emitter::{MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter, TooManyEmitter};

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<RequireLeading<RequireTrailing<AtMost<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<RequireTrailing<AtMost<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<RequireTrailing<AtMost<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  map_depth = 4,
  cardinality = at_most,
  policy = [RequireLeading, RequireTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
