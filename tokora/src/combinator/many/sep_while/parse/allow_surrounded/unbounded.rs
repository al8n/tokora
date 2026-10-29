use super::*;

impl_separated_while_parse! {
  owned_type = [AllowLeading<AllowTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  ref_type = [AllowLeading<AllowTrailing<SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>>>],
  wrapper_type = [AllowLeading<AllowTrailing<SeparatedWhile<&'c mut F, Sep, &'c mut Condition, O, W, L, Ctx, Lang>>>],
  map_depth = 2,
  cardinality = unbounded,
  policy = [AllowLeading, AllowTrailing],
  emitters = {},
  block3_inline = false,
  block4_inline = false,
}
