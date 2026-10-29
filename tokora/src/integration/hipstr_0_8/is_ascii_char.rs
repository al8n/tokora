use crate::utils::IsAsciiChar;

impl IsAsciiChar for hipstr_0_8::HipByt<'_> {
  #[inline(always)]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[inline(always)]
  fn is_ascii_digit(&self) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_digit(self)
  }
}

impl IsAsciiChar for hipstr_0_8::HipStr<'_> {
  #[inline(always)]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <str as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[inline(always)]
  fn is_ascii_digit(&self) -> bool {
    <str as IsAsciiChar>::is_ascii_digit(self)
  }
}
