use core::str::Utf8Error;

use crate::cst::CstText;

impl CstText for smol_bytes_0_2::shared::Bytes {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    core::str::from_utf8(self)
  }
}

impl CstText for smol_bytes_0_2::compact::Bytes {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    core::str::from_utf8(self)
  }
}

impl CstText for smol_bytes_0_2::Utf8Bytes {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    Ok(self.as_str())
  }
}

impl CstText for smol_bytes_0_2::compact::Utf8Bytes {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    Ok(self.as_str())
  }
}
