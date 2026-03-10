use crate::emitter::{MissingLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<RequireLeading<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>, Delim>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [RequireLeading],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
