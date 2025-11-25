use crate::punct::*;

/// A trait for checking
pub trait Check<T: ?Sized> {
  /// Check against the target.
  fn check(&mut self, target: &T) -> bool;
}

impl<F, T> Check<T> for F
where
  F: ?Sized + FnMut(&T) -> bool,
  T: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&mut self, target: &T) -> bool {
    (self)(target)
  }
}

macro_rules! check_punct {
  ($(
    $name:ident::$trait:ident::$is_fn:ident
  ),+$(,)?) => {
    $(
      impl<T, S, C, Lang> $crate::__private::Check<T> for $name<S, C, Lang>
      where
        T: for<'a> $crate::__private::$trait<'a> + ?::core::marker::Sized,
      {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn check(&mut self, target: &T) -> ::core::primitive::bool {
          target.$is_fn()
        }
      }
    )*
  };
}

check_punct!(
  Comma::PunctuatorToken::is_comma,
  Semicolon::PunctuatorToken::is_semicolon,
  Colon::PunctuatorToken::is_colon,
  Dot::PunctuatorToken::is_dot,
  Dash::PunctuatorToken::is_minus,
  Underscore::PunctuatorToken::is_underscore,
  Pipe::PunctuatorToken::is_pipe,
  Ampersand::PunctuatorToken::is_ampersand,
  Space::PunctuatorToken::is_space,
  Tab::PunctuatorToken::is_tab,
  Newline::PunctuatorToken::is_newline,
  CarriageReturn::PunctuatorToken::is_carriage_return,
  CarriageReturnNewline::PunctuatorToken::is_crlf,
  Trivia::TriviaToken::is_trivia,
  OpenAngle::PunctuatorToken::is_open_angle,
  CloseAngle::PunctuatorToken::is_close_angle,
  OpenBrace::PunctuatorToken::is_open_brace,
  CloseBrace::PunctuatorToken::is_close_brace,
  OpenParen::PunctuatorToken::is_open_paren,
  CloseParen::PunctuatorToken::is_close_paren,
  OpenBracket::PunctuatorToken::is_open_bracket,
  CloseBracket::PunctuatorToken::is_close_bracket,
  Equal::PunctuatorToken::is_equal,
  FatArrow::OperatorToken::is_fat_arrow,
  Arrow::OperatorToken::is_arrow,
);
