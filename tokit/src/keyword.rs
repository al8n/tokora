/// Defines a keyword.
///
/// # Examples
/// ```rust
/// use tokit::keyword;
///
/// keyword! {
///   (MyKeyword, "MY_KEYWORD", "my_keyword"),
///   (AnotherKeyword, "ANOTHER_KEYWORD", "another_keyword"),
/// }
/// ```
#[macro_export]
macro_rules! keyword {
  ($(
    $(#[$meta:meta])*
    (
      $name:ident, $syntax_tree_display: literal, $kw:literal
    )
  ),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "The `" $kw "` keyword"]
        $(#[$meta])*
        #[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq, ::core::hash::Hash)]
        pub struct $name<S = $crate::__private::span::SimpleSpan, C = ()> {
          span: S,
          source: C,
        }

        impl<S, C> ::core::convert::AsRef<::core::primitive::str> for $name<S, C> {
          #[inline]
          fn as_ref(&self) -> &str {
            $kw
          }
        }

        impl<S, C> ::core::borrow::Borrow<str> for $name<S, C> {
          #[inline]
          fn borrow(&self) -> &str {
            ::core::convert::AsRef::<str>::as_ref(self)
          }
        }

        impl<S> $name<S> {
          /// Creates a new keyword.
          #[doc = "Creates a new `" $kw "` keyword."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn new(span: S) -> Self {
            Self { span, source: () }
          }
        }

        impl<S, C> $name<S, C> {
          #[doc = "Creates a new `" $kw "` keyword with the given content."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn with_content(span: S, content: C) -> Self {
            Self { span, source: content }
          }

          #[doc = "Returns the raw string literal of the `" $kw "` keyword."]
          #[inline]
          pub const fn raw() -> &'static ::core::primitive::str {
            $kw
          }

          #[doc = "Returns the span of the `" $kw "` keyword."]
          #[inline]
          pub const fn span(&self) -> &S {
            &self.span
          }

          #[doc = "Returns a reference to the content of the `" $kw "` keyword."]
          #[inline]
          pub const fn content(&self) -> &C {
            &self.source
          }
        }

        impl<S, C> $crate::__private::span::AsSpan<S> for $name<S, C> {
          #[inline]
          fn as_span(&self) -> &S {
            self.span()
          }
        }

       impl<S, C> $crate::__private::span::IntoSpan<S> for $name<S, C> {
          #[inline]
          fn into_span(self) -> S {
            self.span
          }
        }

        impl<S, C> $crate::__private::utils::IntoComponents for $name<S, C> {
          type Components = (S, C);

          #[inline]
          fn into_components(self) -> Self::Components {
            (self.span, self.source)
          }
        }

        impl<S, C> ::core::fmt::Display for $name<S, C> {
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            ::core::fmt::Write::write_str(f, $kw)
          }
        }

        impl<S, C> $crate::__private::utils::human_display::DisplayHuman for $name<S, C> {
          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }

        impl<S, C> $crate::__private::utils::sdl_display::DisplayCompact for $name<S, C> {
          type Options = ();

          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>, _: &Self::Options) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }

        impl<S, C> $crate::__private::utils::sdl_display::DisplayPretty for $name<S, C> {
          type Options = ();

          #[inline]
          fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>, _: &Self::Options) -> ::core::fmt::Result {
            ::core::fmt::Display::fmt(self, f)
          }
        }
      )*
    }
  };
}
