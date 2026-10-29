use crate::utils::{Equivalent, cmp::__assert_equivalent_impl};

const _: () = {
  use bytes_1::Bytes;

  impl Equivalent<str> for Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &str) -> bool {
      self.as_ref().eq(other.as_bytes())
    }
  }

  impl Equivalent<[u8]> for Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &[u8]) -> bool {
      self.as_ref().eq(other)
    }
  }

  impl Equivalent<Bytes> for Bytes {
    #[cfg_attr(test, inline)]
    #[cfg_attr(not(test), inline(always))]
    fn equivalent(&self, other: &Bytes) -> bool {
      self.eq(other)
    }
  }

  __assert_equivalent_impl::<Bytes, str>();
  __assert_equivalent_impl::<Bytes, [u8]>();
  __assert_equivalent_impl::<str, Bytes>();
  __assert_equivalent_impl::<[u8], Bytes>();
};
