//! Flexible message container that adapts to `no_std` and `alloc` environments.
//!
//! `Message` provides a single abstraction for short, human-readable strings that may be
//! either static literals or owned allocations depending on the available features. In
//! `no_std` + `no_alloc` builds the type degenerates to a lightweight wrapper around a
//! `&'static str`, while in `alloc`/`std` builds it stores a `Cow<'static, str>` to balance
//! zero-copy ergonomics with configurability.

#[cfg(any(feature = "std", feature = "alloc"))]
use std::{borrow::Cow, string::String};

#[cfg(not(any(feature = "std", feature = "alloc")))]
type Inner = &'static str;

#[cfg(any(feature = "std", feature = "alloc"))]
type Inner = Cow<'static, str>;

/// Feature-aware message container with a unified API.
///
/// - **`no_std` + `no_alloc`**: stores a `&'static str`
/// - **`alloc` / `std`**: stores a `Cow<'static, str>`
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  derive_more::Display,
  derive_more::AsMut,
  derive_more::AsRef,
  derive_more::Deref,
  derive_more::DerefMut,
)]
#[cfg_attr(not(any(feature = "std", feature = "alloc")), derive(Copy))]
#[repr(transparent)]
#[display("{inner}")]
pub struct Message {
  #[deref]
  #[deref_mut]
  #[as_ref]
  #[as_mut]
  inner: Inner,
}

impl Message {
  /// Creates a new message from the provided representation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::Message;
  ///
  /// let msg = Message::new("greeting");
  /// assert_eq!(msg.as_str(), "greeting");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(inner: impl Into<Inner>) -> Self {
    Self {
      inner: inner.into(),
    }
  }

  /// Creates a message from a `'static` string without allocation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::Message;
  ///
  /// const MSG: Message = Message::from_static("hello");
  /// assert_eq!(MSG.as_str(), "hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_static(value: &'static str) -> Self {
    Self {
      inner: Self::borrow_const(value),
    }
  }

  /// Returns the message as a string slice.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::Message;
  ///
  /// let msg = Message::from_static("world");
  /// assert_eq!(msg.as_str(), "world");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &str {
    Self::as_str_inner(&self.inner)
  }

  /// Borrows the inner representation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::Message;
  /// # #[cfg(any(feature = "std", feature = "alloc"))]
  /// # {
  /// use std::borrow::Cow;
  ///
  /// let msg = Message::from_static("inner");
  /// assert!(matches!(msg.as_inner(), &Cow::Borrowed("inner")));
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_inner(&self) -> &Inner {
    &self.inner
  }

  /// Consumes the message and returns the inner representation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::Message;
  /// # #[cfg(any(feature = "std", feature = "alloc"))]
  /// # {
  /// use std::borrow::Cow;
  ///
  /// let msg = Message::from_static("consume");
  /// let inner = msg.into_inner();
  /// assert_eq!(inner, Cow::Borrowed("consume"));
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_inner(self) -> Inner {
    self.inner
  }
}

impl From<&'static str> for Message {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: &'static str) -> Self {
    Self::from_static(value)
  }
}

impl AsRef<str> for Message {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl core::borrow::Borrow<str> for Message {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
const _: () = {
  impl Message {
    #[cfg_attr(not(tarpaulin), inline(always))]
    const fn borrow_const(value: &'static str) -> Inner {
      value
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    const fn as_str_inner(inner: &Inner) -> &str {
      inner
    }
  }

  impl From<Message> for &'static str {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: Message) -> Self {
      value.inner
    }
  }

  impl From<&Message> for &'static str {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: &Message) -> Self {
      value.inner
    }
  }
};

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  impl Message {
    #[cfg_attr(not(tarpaulin), inline(always))]
    const fn borrow_const(value: &'static str) -> Inner {
      Cow::Borrowed(value)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    const fn as_str_inner(inner: &Inner) -> &str {
      match inner {
        Cow::Borrowed(value) => value,
        Cow::Owned(value) => value.as_str(),
      }
    }

    /// Creates a message by taking ownership of a string.
    ///
    /// ## Examples
    ///
    /// ```
    /// use tokit::utils::Message;
    ///
    /// let msg = Message::from_string(std::string::String::from("owned"));
    /// assert_eq!(msg.as_str(), "owned");
    /// ```
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn from_string(value: String) -> Self {
      Self {
        inner: Cow::Owned(value),
      }
    }

    /// Ensures the message is owned and returns a mutable reference.
    ///
    /// ## Examples
    ///
    /// ```
    /// use tokit::utils::Message;
    ///
    /// let mut msg = Message::from_static("grow");
    /// msg.to_mut().push('!');
    /// assert_eq!(msg.as_str(), "grow!");
    /// ```
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn to_mut(&mut self) -> &mut String {
      self.inner.to_mut()
    }
  }

  impl From<String> for Message {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: String) -> Self {
      Message::from_string(value)
    }
  }

  impl From<Cow<'static, str>> for Message {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: Cow<'static, str>) -> Self {
      Self { inner: value }
    }
  }

  impl From<Message> for Cow<'static, str> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: Message) -> Self {
      value.inner
    }
  }

  impl From<&Message> for Cow<'static, str> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: &Message) -> Self {
      value.inner.clone()
    }
  }

  impl AsMut<str> for Message {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn as_mut(&mut self) -> &mut str {
      self.inner.to_mut()
    }
  }
};
