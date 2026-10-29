use crate::utils::human_display::DisplayHuman;

impl DisplayHuman for bytes_1::Bytes {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.as_ref().fmt(f)
  }
}
