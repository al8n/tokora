use crate::emitter::{TooManyEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<AllowLeading<AtMost<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<AtMost<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<AtMost<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>, Delim>],
  map_depth = 3,
  cardinality = at_most,
  policy = [AllowLeading],
  emitters = {
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
