use super::*;

impl_separated_parse! {
  owned_type = [AllowLeading<AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang>>>],
  ref_type = [AllowLeading<AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang>>>],
  wrapper_type = [AllowLeading<AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [AllowLeading, AllowTrailing],
  emitters = {},
  block3_inline = true,
  block4_inline = false,
}
