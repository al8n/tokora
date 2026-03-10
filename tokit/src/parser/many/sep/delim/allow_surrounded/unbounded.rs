use super::*;

impl_separated_delim! {
  owned_type = [DelimitedBy<AllowLeading<AllowTrailing<Separated<F, Sep, O, L, Ctx, Lang>>>, Delim>],
  ref_type = [DelimitedBy<AllowLeading<AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  wrapper_type = [DelimitedBy<AllowLeading<AllowTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>],
  map_depth = 3,
  cardinality = unbounded,
  policy = [AllowLeading, AllowTrailing],
  emitters = {},
  block3_inline = true,
  block4_inline = true,
}
