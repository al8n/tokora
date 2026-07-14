use super::*;

#[test]
fn unit_state_check_ok() {
  let state = ();
  assert!(state.check().is_ok());
}

#[test]
fn unit_state_clone_and_debug() {
  let state = ();
  let cloned = state.clone();
  let _ = format!("{:?}", cloned);
}
