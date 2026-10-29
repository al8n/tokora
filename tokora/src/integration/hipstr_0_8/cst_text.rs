use core::str::Utf8Error;

use crate::cst::CstText;

impl CstText for hipstr_0_8::HipStr<'_> {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    Ok(self.as_str())
  }
}

impl CstText for hipstr_0_8::HipByt<'_> {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    core::str::from_utf8(self)
  }
}
