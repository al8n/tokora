use core::str::Utf8Error;

use crate::cst::CstText;

impl CstText for bstr_1::BStr {
  #[inline(always)]
  fn cst_text(&self) -> Result<&str, Utf8Error> {
    core::str::from_utf8(self)
  }
}
