use crate::utils::{IntoEquivalent, ToEquivalent, to_equivalent::sealed};

const _: () = {
  use sealed::Sealed;
  use smol_bytes_0_1::{Utf8Bytes, compact, shared};

  impl Sealed<shared::Bytes> for [u8] {}

  impl ToEquivalent<shared::Bytes> for [u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn to_equivalent(&self) -> shared::Bytes {
      shared::Bytes::copy_from_slice(self)
    }
  }

  impl IntoEquivalent<shared::Bytes> for &[u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn into_equivalent(self) -> shared::Bytes {
      shared::Bytes::copy_from_slice(self)
    }
  }

  impl Sealed<compact::Bytes> for [u8] {}

  impl ToEquivalent<compact::Bytes> for [u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn to_equivalent(&self) -> compact::Bytes {
      compact::Bytes::copy_from_slice(self)
    }
  }

  impl IntoEquivalent<compact::Bytes> for &[u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn into_equivalent(self) -> compact::Bytes {
      compact::Bytes::copy_from_slice(self)
    }
  }

  impl Sealed<Utf8Bytes> for str {}

  impl ToEquivalent<Utf8Bytes> for str {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn to_equivalent(&self) -> Utf8Bytes {
      Utf8Bytes::from(self)
    }
  }

  impl IntoEquivalent<Utf8Bytes> for &str {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn into_equivalent(self) -> Utf8Bytes {
      Utf8Bytes::from(self)
    }
  }

  impl Sealed<compact::Utf8Bytes> for str {}

  impl ToEquivalent<compact::Utf8Bytes> for str {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn to_equivalent(&self) -> compact::Utf8Bytes {
      compact::Utf8Bytes::from(self)
    }
  }

  impl IntoEquivalent<compact::Utf8Bytes> for &str {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn into_equivalent(self) -> compact::Utf8Bytes {
      compact::Utf8Bytes::from(self)
    }
  }

  fn __assert_smol_bytes_to_equivalent_impl() {
    fn _assert_shared<T: ToEquivalent<shared::Bytes> + ?Sized>() {}
    fn _assert_compact<T: ToEquivalent<compact::Bytes> + ?Sized>() {}
    fn _assert_utf8<T: ToEquivalent<Utf8Bytes> + ?Sized>() {}
    fn _assert_compact_utf8<T: ToEquivalent<compact::Utf8Bytes> + ?Sized>() {}

    _assert_shared::<[u8]>();
    _assert_shared::<&[u8]>();
    _assert_compact::<[u8]>();
    _assert_compact::<&[u8]>();
    _assert_utf8::<str>();
    _assert_utf8::<&str>();
    _assert_compact_utf8::<str>();
    _assert_compact_utf8::<&str>();
  }

  fn __assert_smol_bytes_into_equivalent_impl() {
    fn _assert_shared<T: IntoEquivalent<shared::Bytes> + ?Sized>() {}
    fn _assert_compact<T: IntoEquivalent<compact::Bytes> + ?Sized>() {}
    fn _assert_utf8<T: IntoEquivalent<Utf8Bytes> + ?Sized>() {}
    fn _assert_compact_utf8<T: IntoEquivalent<compact::Utf8Bytes> + ?Sized>() {}

    _assert_shared::<&[u8]>();
    _assert_compact::<&[u8]>();
    _assert_utf8::<&str>();
    _assert_compact_utf8::<&str>();
  }
};
