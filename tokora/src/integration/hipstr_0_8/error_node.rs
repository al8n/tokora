use crate::{error::ErrorNode, span::SimpleSpan};

const _: () = {
  use hipstr_0_8::{HipByt, HipStr};

  impl ErrorNode for HipStr<'_> {
    #[inline]
    fn error(_span: SimpleSpan) -> Self {
      HipStr::borrowed("<error>")
    }

    #[inline]
    fn missing(_span: SimpleSpan) -> Self {
      HipStr::borrowed("<missing>")
    }
  }

  impl ErrorNode for HipByt<'_> {
    #[inline]
    fn error(_span: SimpleSpan) -> Self {
      HipByt::borrowed(b"<error>")
    }

    #[inline]
    fn missing(_span: SimpleSpan) -> Self {
      HipByt::borrowed(b"<missing>")
    }
  }
};
