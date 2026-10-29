use core::str::Utf8Error;

use crate::cst::CstText;

impl CstText for smol_bytes_0_1::shared::Bytes {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    core::str::from_utf8(self)
  }
}

impl CstText for smol_bytes_0_1::compact::Bytes {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    core::str::from_utf8(self)
  }
}

impl CstText for smol_bytes_0_1::Utf8Bytes {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    Ok(self.as_str())
  }
}

impl CstText for smol_bytes_0_1::compact::Utf8Bytes {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    Ok(self.as_str())
  }
}
