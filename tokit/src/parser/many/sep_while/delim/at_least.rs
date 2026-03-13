use crate::emitter::{TooFewEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter};

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<AtLeast<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>, Delim>],
  ref_type = [DelimitedBy<AtLeast<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>, Delim>],
  wrapper_type = [DelimitedBy<AtLeast<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>, Delim>],
  map_depth = 2,
  cardinality = at_least,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
