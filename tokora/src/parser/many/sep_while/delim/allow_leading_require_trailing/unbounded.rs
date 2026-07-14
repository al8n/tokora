use crate::emitter::MissingTrailingSeparatorEmitter;

use super::*;

impl_separated_while_delim! {
  owned_type = [DelimitedBy<AllowLeading<RequireTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<RequireTrailing<SeparatedWhile<&'c mut F, Sep, Condition, O, W, L, Ctx, Lang>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<RequireTrailing<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>, Delim>],
  map_depth = 3,
  cardinality = unbounded,
  policy = [AllowLeading, RequireTrailing],
  emitters = {
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
