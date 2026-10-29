use core::fmt;

use crate::utils::DisplayHuman;

impl DisplayHuman for bstr_1::BStr {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    fmt::Display::fmt(self, f)
  }
}
