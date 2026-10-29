use crate::{error::ErrorNode, span::SimpleSpan};

impl ErrorNode for bytes_1::Bytes {
  #[inline]
  fn error(_span: SimpleSpan) -> Self {
    bytes_1::Bytes::from_static(b"<error>")
  }

  #[inline]
  fn missing(_span: SimpleSpan) -> Self {
    bytes_1::Bytes::from_static(b"<missing>")
  }
}
