use crate::{
  container::Container as ContainerT,
  emitter::{UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter},
};

use super::*;

impl_separated_parse! {
  owned_type = [Separated<F, Sep, O, L, Ctx, Lang>],
  ref_type = [Separated<&'c mut F, Sep, O, L, Ctx, Lang>],
  wrapper_type = [Separated<&'c mut F, Sep, O, L, Ctx, Lang>],
  map_depth = 0,
  cardinality = unbounded,
  policy = [],
  emitters = {
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  },
  block3_inline = false,
  block4_inline = false,
}
