use super::*;

// Use DummyToken defined below

#[test]
fn lexed_from_ok_result() {
  let result: Result<DummyToken, ()> = Ok(DummyToken);
  let lexed: Lexed<'_, DummyToken> = result.into();
  assert!(lexed.is_token());
  assert!(!lexed.is_error());
}

#[test]
fn lexed_from_err_result() {
  let result: Result<DummyToken, ()> = Err(());
  let lexed: Lexed<'_, DummyToken> = result.into();
  assert!(!lexed.is_token());
  assert!(lexed.is_error());
}

#[test]
fn lexed_into_result_ok() {
  let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  let result: Result<DummyToken, ()> = lexed.into();
  assert!(result.is_ok());
}

#[test]
fn lexed_into_result_err() {
  let lexed = Lexed::<'_, DummyToken>::Error(());
  let result: Result<DummyToken, ()> = lexed.into();
  assert!(result.is_err());
}

#[test]
fn lexed_expect_token() {
  let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  let tok = lexed.expect_token("should be a token");
  assert_eq!(tok, DummyToken);
}

#[test]
#[should_panic(expected = "not a token")]
fn lexed_expect_token_panics_on_error() {
  let lexed = Lexed::<'_, DummyToken>::Error(());
  lexed.expect_token("not a token");
}

#[test]
fn lexed_expect_error() {
  let lexed = Lexed::<'_, DummyToken>::Error(());
  lexed.expect_error("should be an error");
}

#[test]
#[should_panic(expected = "not an error")]
fn lexed_expect_error_panics_on_token() {
  let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  lexed.expect_error("not an error");
}

#[test]
fn lexed_expect_token_ref() {
  let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  let tok = lexed.expect_token_ref("should be a token");
  assert_eq!(tok, &DummyToken);
}

#[test]
#[should_panic(expected = "not a token")]
fn lexed_expect_token_ref_panics_on_error() {
  let lexed = Lexed::<'_, DummyToken>::Error(());
  lexed.expect_token_ref("not a token");
}

#[test]
fn lexed_expect_error_ref() {
  let lexed = Lexed::<'_, DummyToken>::Error(());
  let _err = lexed.expect_error_ref("should be an error");
}

#[test]
#[should_panic(expected = "not an error")]
fn lexed_expect_error_ref_panics_on_token() {
  let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  lexed.expect_error_ref("not an error");
}

#[test]
fn lexed_expect_token_mut() {
  let mut lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  let tok = lexed.expect_token_mut("should be a token");
  assert_eq!(tok, &mut DummyToken);
}

#[test]
#[should_panic(expected = "not a token")]
fn lexed_expect_token_mut_panics_on_error() {
  let mut lexed = Lexed::<'_, DummyToken>::Error(());
  lexed.expect_token_mut("not a token");
}

#[test]
fn lexed_expect_error_mut() {
  let mut lexed = Lexed::<'_, DummyToken>::Error(());
  let _err = lexed.expect_error_mut("should be an error");
}

#[test]
#[should_panic(expected = "not an error")]
fn lexed_expect_error_mut_panics_on_token() {
  let mut lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  lexed.expect_error_mut("not an error");
}

// Display test removed: DummyToken::Error = () which doesn't impl Display

#[test]
fn lexed_clone() {
  let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  let cloned = lexed.clone();
  assert_eq!(lexed, cloned);
}

#[test]
fn lexed_try_unwrap() {
  let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  assert!(lexed.try_unwrap_token().is_ok());

  let lexed = Lexed::<'_, DummyToken>::Error(());
  assert!(lexed.try_unwrap_error().is_ok());
}

#[test]
fn lexed_unwrap_ref() {
  let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
  assert_eq!(lexed.unwrap_token_ref(), &DummyToken);

  let lexed = Lexed::<'_, DummyToken>::Error(());
  assert_eq!(lexed.unwrap_error_ref(), &());
}
