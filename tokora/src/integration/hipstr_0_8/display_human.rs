use core::fmt;

use crate::utils::DisplayHuman;

const _: () = {
  use hipstr_0_8::{HipByt, HipStr};

  impl DisplayHuman for HipStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      fmt::Display::fmt(self, f)
    }
  }

  impl DisplayHuman for HipByt<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      DisplayHuman::fmt(self.as_ref(), f)
    }
  }
};
