use crate::emitter::{MissingTrailingSeparatorEmitter, TooFewEmitter, TooManyEmitter};

use super::*;

impl_separated_parse! {
  owned_type = [AllowLeading<RequireTrailing<Bounded<Separated<F, Sep, O, L, Ctx, Lang>>>>],
  ref_type = [AllowLeading<RequireTrailing<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  wrapper_type = [AllowLeading<RequireTrailing<Bounded<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>>],
  map_depth = 3,
  cardinality = bounded,
  policy = [AllowLeading, RequireTrailing],
  emitters = {
    + TooFewEmitter<'inp, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = true,
  block4_inline = true,
}
