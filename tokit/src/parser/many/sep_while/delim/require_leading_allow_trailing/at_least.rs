use crate::emitter::{MissingLeadingSeparatorEmitter, TooFewEmitter};

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<RequireLeading<AllowTrailing<AtLeast<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  ref_type = [DelimitedBy<RequireLeading<AllowTrailing<AtLeast<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  wrapper_type = [DelimitedBy<RequireLeading<AllowTrailing<AtLeast<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>>, Delim>],
  map_depth = 4,
  cardinality = at_least,
  policy = [RequireLeading, AllowTrailing],
  emitters = {
    + MissingLeadingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
