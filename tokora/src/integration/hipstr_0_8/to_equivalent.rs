use crate::utils::{IntoEquivalent, ToEquivalent, to_equivalent::sealed};

const _: () = {
  use hipstr_0_8::{HipByt, HipStr};
  use sealed::Sealed;

  impl Sealed<HipStr<'_>> for str {}

  impl<'a> ToEquivalent<HipStr<'a>> for str {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn to_equivalent(&self) -> HipStr<'a> {
      HipStr::from(self)
    }
  }

  impl<'a> IntoEquivalent<HipStr<'a>> for &str {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn into_equivalent(self) -> HipStr<'a> {
      HipStr::from(self)
    }
  }

  impl Sealed<HipByt<'_>> for [u8] {}

  impl<'a> ToEquivalent<HipByt<'a>> for [u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn to_equivalent(&self) -> HipByt<'a> {
      HipByt::from(self)
    }
  }

  impl<'a> IntoEquivalent<HipByt<'a>> for &[u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn into_equivalent(self) -> HipByt<'a> {
      HipByt::from(self)
    }
  }
};
