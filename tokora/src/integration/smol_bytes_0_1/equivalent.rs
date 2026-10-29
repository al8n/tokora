use crate::utils::{Equivalent, cmp::__assert_equivalent_impl};

const _: () = {
  use smol_bytes_0_1::{Utf8Bytes, compact, shared};

  // `Utf8Bytes` implements both `AsRef<str>` and `AsRef<[u8]>`, so `self.as_ref()`
  // is ambiguous; the byte view is selected explicitly to mirror the `HipStr` impls in
  // `integration/hipstr_0_8/equivalent.rs`.
  impl Equivalent<str> for shared::Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &str) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other.as_bytes())
    }
  }

  impl Equivalent<[u8]> for shared::Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &[u8]) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other)
    }
  }

  impl Equivalent<shared::Bytes> for shared::Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &shared::Bytes) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(AsRef::<[u8]>::as_ref(other))
    }
  }

  impl Equivalent<str> for compact::Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &str) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other.as_bytes())
    }
  }

  impl Equivalent<[u8]> for compact::Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &[u8]) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other)
    }
  }

  impl Equivalent<compact::Bytes> for compact::Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &compact::Bytes) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(AsRef::<[u8]>::as_ref(other))
    }
  }

  impl Equivalent<str> for Utf8Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &str) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other.as_bytes())
    }
  }

  impl Equivalent<[u8]> for Utf8Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &[u8]) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other)
    }
  }

  impl Equivalent<Utf8Bytes> for Utf8Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &Utf8Bytes) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(AsRef::<[u8]>::as_ref(other))
    }
  }

  impl Equivalent<str> for compact::Utf8Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &str) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other.as_bytes())
    }
  }

  impl Equivalent<[u8]> for compact::Utf8Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &[u8]) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(other)
    }
  }

  impl Equivalent<compact::Utf8Bytes> for compact::Utf8Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &compact::Utf8Bytes) -> bool {
      AsRef::<[u8]>::as_ref(self).eq(AsRef::<[u8]>::as_ref(other))
    }
  }

  __assert_equivalent_impl::<shared::Bytes, str>();
  __assert_equivalent_impl::<shared::Bytes, [u8]>();
  __assert_equivalent_impl::<str, shared::Bytes>();
  __assert_equivalent_impl::<[u8], shared::Bytes>();

  __assert_equivalent_impl::<compact::Bytes, str>();
  __assert_equivalent_impl::<compact::Bytes, [u8]>();
  __assert_equivalent_impl::<str, compact::Bytes>();
  __assert_equivalent_impl::<[u8], compact::Bytes>();

  __assert_equivalent_impl::<Utf8Bytes, str>();
  __assert_equivalent_impl::<Utf8Bytes, [u8]>();
  __assert_equivalent_impl::<str, Utf8Bytes>();
  __assert_equivalent_impl::<[u8], Utf8Bytes>();

  __assert_equivalent_impl::<compact::Utf8Bytes, str>();
  __assert_equivalent_impl::<compact::Utf8Bytes, [u8]>();
  __assert_equivalent_impl::<str, compact::Utf8Bytes>();
  __assert_equivalent_impl::<[u8], compact::Utf8Bytes>();
};
