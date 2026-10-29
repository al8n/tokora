use crate::utils::{IntoEquivalent, ToEquivalent, to_equivalent::sealed};

const _: () = {
  use bytes_1::Bytes;
  use sealed::Sealed;

  impl Sealed<Bytes> for [u8] {}

  impl ToEquivalent<Bytes> for [u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn to_equivalent(&self) -> Bytes {
      Bytes::copy_from_slice(self)
    }
  }

  impl IntoEquivalent<Bytes> for &[u8] {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn into_equivalent(self) -> Bytes {
      Bytes::copy_from_slice(self)
    }
  }

  fn __assert_bytes_to_equivalent_impl() {
    fn _assert<T: ToEquivalent<Bytes> + ?Sized>() {}

    _assert::<[u8]>();
    _assert::<&[u8]>();
  }

  fn __assert_bytes_into_equivalent_impl() {
    fn _assert<T: IntoEquivalent<Bytes> + ?Sized>() {}

    _assert::<&[u8]>();
  }
};
