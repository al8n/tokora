use crate::utils::cmp::{__assert_equivalent_impl, Equivalent};

const _: () = {
  use hipstr_0_8::{HipByt, HipStr};

  // `HipStr` implements both `AsRef<str>` and `AsRef<[u8]>`, so `self.as_ref()`
  // is ambiguous; the byte view is selected explicitly to mirror the `Bytes` impls in
  // `integration/bytes_1/equivalent.rs`.
  impl Equivalent<str> for HipStr<'_> {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &str) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other.as_bytes())
    }
  }

  impl Equivalent<[u8]> for HipStr<'_> {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &[u8]) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other)
    }
  }

  impl Equivalent<HipStr<'_>> for HipStr<'_> {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &HipStr<'_>) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(AsRef::<[u8]>::as_ref(other))
    }
  }

  impl Equivalent<str> for HipByt<'_> {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &str) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other.as_bytes())
    }
  }

  impl Equivalent<[u8]> for HipByt<'_> {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &[u8]) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other)
    }
  }

  impl Equivalent<HipByt<'_>> for HipByt<'_> {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &HipByt<'_>) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(AsRef::<[u8]>::as_ref(other))
    }
  }

  __assert_equivalent_impl::<HipStr<'_>, str>();
  __assert_equivalent_impl::<HipStr<'_>, [u8]>();
  __assert_equivalent_impl::<str, HipStr<'_>>();
  __assert_equivalent_impl::<[u8], HipStr<'_>>();

  __assert_equivalent_impl::<HipByt<'_>, str>();
  __assert_equivalent_impl::<HipByt<'_>, [u8]>();
  __assert_equivalent_impl::<str, HipByt<'_>>();
  __assert_equivalent_impl::<[u8], HipByt<'_>>();
};
