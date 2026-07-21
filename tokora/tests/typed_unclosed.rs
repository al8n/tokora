#![cfg(all(feature = "std", feature = "logos"))]

mod common;

use common::{TestLexer, Token, TokenKind};
use generic_arraydeque::typenum::U1;
use tokora::{
  Accumulator, FatalContext, InputRef, Parse, ParseContext, ParseInput, Parser, SimpleSpan,
  cache::Peeked,
  error::{
    Unclosed, UnexpectedEot,
    syntax::{FullContainer, MissingSyntax, TooFew},
    token::{MissingToken, SeparatedError, UnexpectedToken},
  },
  parser::{Action, delimited},
  punct::{Brace, Bracket, CloseBrace, CloseBracket, OpenBrace, OpenBracket},
};

#[derive(Debug)]
enum CustomLang {}

#[derive(Debug)]
enum TypedError {
  Bracket(Unclosed<Bracket, SimpleSpan, CustomLang>),
  Brace(Unclosed<Brace, SimpleSpan, CustomLang>),
  Other,
}

impl From<()> for TypedError {
  fn from(_: ()) -> Self {
    Self::Other
  }
}

impl From<Unclosed<Bracket, SimpleSpan, CustomLang>> for TypedError {
  fn from(err: Unclosed<Bracket, SimpleSpan, CustomLang>) -> Self {
    Self::Bracket(err)
  }
}

impl From<Unclosed<Brace, SimpleSpan, CustomLang>> for TypedError {
  fn from(err: Unclosed<Brace, SimpleSpan, CustomLang>) -> Self {
    Self::Brace(err)
  }
}

impl<'inp, T, K: Clone, S> From<UnexpectedToken<'inp, T, K, S, CustomLang>> for TypedError {
  fn from(_: UnexpectedToken<'inp, T, K, S, CustomLang>) -> Self {
    Self::Other
  }
}

impl<O> From<UnexpectedEot<O, CustomLang>> for TypedError {
  fn from(_: UnexpectedEot<O, CustomLang>) -> Self {
    Self::Other
  }
}

impl<S> From<FullContainer<S, CustomLang>> for TypedError {
  fn from(_: FullContainer<S, CustomLang>) -> Self {
    Self::Other
  }
}

impl<S> From<TooFew<S, CustomLang>> for TypedError {
  fn from(_: TooFew<S, CustomLang>) -> Self {
    Self::Other
  }
}

impl<'inp, Kind: Clone, O> From<MissingToken<'inp, Kind, O, CustomLang>> for TypedError {
  fn from(_: MissingToken<'inp, Kind, O, CustomLang>) -> Self {
    Self::Other
  }
}

impl<'inp, T, Kind: Clone, S> From<SeparatedError<'inp, T, Kind, S, CustomLang>> for TypedError {
  fn from(_: SeparatedError<'inp, T, Kind, S, CustomLang>) -> Self {
    Self::Other
  }
}

impl<O> From<MissingSyntax<O, CustomLang>> for TypedError {
  fn from(_: MissingSyntax<O, CustomLang>) -> Self {
    Self::Other
  }
}

impl From<OpenBracket<(), (), CustomLang>> for TokenKind {
  fn from(_: OpenBracket<(), (), CustomLang>) -> Self {
    Self::LBracket
  }
}

impl From<CloseBracket<(), (), CustomLang>> for TokenKind {
  fn from(_: CloseBracket<(), (), CustomLang>) -> Self {
    Self::RBracket
  }
}

impl From<OpenBrace<(), (), CustomLang>> for TokenKind {
  fn from(_: OpenBrace<(), (), CustomLang>) -> Self {
    Self::LBrace
  }
}

impl From<CloseBrace<(), (), CustomLang>> for TokenKind {
  fn from(_: CloseBrace<(), (), CustomLang>) -> Self {
    Self::RBrace
  }
}

type Ctx<'inp> = FatalContext<'inp, TestLexer<'inp>, TypedError, CustomLang>;

fn number<'inp>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx<'inp>, CustomLang>,
) -> Result<i64, TypedError> {
  match inp.next()? {
    Some(token) => match token.into_data() {
      Token::Num(number) => Ok(number),
      _ => Err(TypedError::Other),
    },
    None => Err(TypedError::Other),
  }
}

fn decide_number<'inp>(
  mut peeked: Peeked<'_, 'inp, TestLexer<'inp>, U1>,
  _: &mut <Ctx<'inp> as ParseContext<'inp, TestLexer<'inp>, CustomLang>>::Emitter,
) -> Result<Action, TypedError> {
  Ok(match peeked.pop_front() {
    None => Action::Stop,
    Some(token) => {
      let token = token
        .as_maybe_ref()
        .map(|token| token.token().copied(), |token| token.token())
        .into_inner();
      if matches!(**token.data(), Token::Num(_)) {
        Action::Continue
      } else {
        Action::Stop
      }
    }
  })
}

fn bare_bracket_many<'inp>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx<'inp>, CustomLang>,
) -> Result<Vec<i64>, TypedError> {
  number
    .repeated_while::<_, U1>(decide_number)
    .delimited::<Bracket>()
    .collect()
    .parse_input(inp)
}

fn bare_bracket<'inp>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx<'inp>, CustomLang>,
) -> Result<i64, TypedError> {
  delimited::<Bracket, _, _, CustomLang, _, _, _>(number)(inp).map(|group| *group.data())
}

fn bare_brace<'inp>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx<'inp>, CustomLang>,
) -> Result<i64, TypedError> {
  delimited::<Brace, _, _, CustomLang, _, _, _>(number)(inp).map(|group| *group.data())
}

#[test]
fn bare_builtin_markers_preserve_typed_unclosed_conversions_under_custom_language() {
  let bracket =
    Parser::with_parser_of::<'_, TestLexer<'_>, i64, TypedError, _, CustomLang>(bare_bracket)
      .parse_str("[1");
  match bracket {
    Err(TypedError::Bracket(err)) => assert_eq!(err.name_ref(), "[]"),
    Err(other) => panic!("expected Unclosed<Bracket, _, CustomLang>, got {other:?}"),
    Ok(value) => panic!("expected unclosed bracket, parsed {value}"),
  }

  let brace =
    Parser::with_parser_of::<'_, TestLexer<'_>, i64, TypedError, _, CustomLang>(bare_brace)
      .parse_str("{1");
  match brace {
    Err(TypedError::Brace(err)) => assert_eq!(err.name_ref(), "{}"),
    Err(other) => panic!("expected Unclosed<Brace, _, CustomLang>, got {other:?}"),
    Ok(value) => panic!("expected unclosed brace, parsed {value}"),
  }
}

#[test]
fn bare_bracket_many_builder_preserves_typed_unclosed_under_custom_language() {
  let result = Parser::with_parser_of::<'_, TestLexer<'_>, Vec<i64>, TypedError, _, CustomLang>(
    bare_bracket_many,
  )
  .parse_str("[1 2");

  match result {
    Err(TypedError::Bracket(err)) => assert_eq!(err.name_ref(), "[]"),
    Err(other) => panic!("expected Unclosed<Bracket, _, CustomLang>, got {other:?}"),
    Ok(value) => panic!("expected unclosed bracket, parsed {value:?}"),
  }
}
