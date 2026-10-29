use crate::utils::{Equivalent, cmp::__assert_equivalent_impl};

const _: () = {
  use bstr_1::BStr;

  impl Equivalent<str> for BStr {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &str) -> bool {
      <BStr as AsRef<[u8]>>::as_ref(self).eq(other.as_bytes())
    }
  }

  impl Equivalent<[u8]> for BStr {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &[u8]) -> bool {
      <BStr as AsRef<[u8]>>::as_ref(self).eq(other)
    }
  }

  impl Equivalent<BStr> for BStr {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &BStr) -> bool {
      self.eq(other)
    }
  }

  // The lattice is the anti-drift mechanism, not a macro: these rows fail to compile if either
  // half of the two-file family goes missing, which is teeth by construction. The reverse
  // directions (`str`/`[u8]` against `BStr`) come from the blanket `impl<T: AsRef<[u8]>>` in
  // `utils::cmp` and are asserted here so a narrowing of that blanket is caught at this backing
  // too.
  __assert_equivalent_impl::<BStr, str>();
  __assert_equivalent_impl::<BStr, [u8]>();
  __assert_equivalent_impl::<BStr, BStr>();
  __assert_equivalent_impl::<str, BStr>();
  __assert_equivalent_impl::<[u8], BStr>();
};
