#![allow(single_use_lifetimes)]

use crate::utils::{IntoEquivalent, ToEquivalent, to_equivalent::sealed};

const _: () = {
  use bstr_1::BStr;
  use sealed::Sealed;

  // `BStr` is a `repr(transparent)` view over `[u8]`, so the conversion is a reborrow rather
  // than a copy — which is why this mirrors the `&'de str -> &'a str` identity idiom of
  // `utils::to_equivalent` rather than the owned-handle idiom `bytes_1`/`hipstr` use. `[u8]`
  // cannot implement `ToEquivalent<&'a BStr>` directly: `to_equivalent(&self)` has only an
  // anonymous borrow to hand back, and the reference form is where the source lifetime is
  // nameable.
  impl<'de: 'a, 'a> Sealed<&'a BStr> for &'de [u8] {}

  impl<'de: 'a, 'a> ToEquivalent<&'a BStr> for &'de [u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn to_equivalent(&self) -> &'a BStr {
      BStr::new(*self)
    }
  }

  impl<'de: 'a, 'a> IntoEquivalent<&'a BStr> for &'de [u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn into_equivalent(self) -> &'a BStr {
      BStr::new(self)
    }
  }

  // The lattice row. It fails to compile if either half of this family's two files loses its
  // BStr block, which is the anti-drift mechanism — the reason the two files are deliberately
  // not collapsed into one macro.
  fn __assert_bstr_equivalence_family() {
    fn _to<'a, T: ToEquivalent<&'a BStr>>() {}
    fn _into<'a, T: IntoEquivalent<&'a BStr>>() {}

    _to::<&[u8]>();
    _into::<&[u8]>();
  }
};
