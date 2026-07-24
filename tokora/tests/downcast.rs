use tokora::utils::{Downcast, DowncastRef};

#[derive(Debug, Eq, PartialEq)]
enum Value {
  Number(u8),
  Text(String),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Text;

impl Downcast<String> for Value {
  fn downcast(self) -> Option<String> {
    match self {
      Self::Text(value) => Some(value),
      Self::Number(_) => None,
    }
  }
}

impl DowncastRef<Text> for Value {
  fn downcast_ref(&self) -> Option<Text> {
    match self {
      Self::Text(_) => Some(Text),
      Self::Number(_) => None,
    }
  }
}

#[test]
fn owned_downcast_projects_or_declines() {
  let value = Value::Text(String::from("text"));
  let text = value.downcast().expect("text projects to String");
  assert_eq!(text, "text");

  let declined: Option<String> = Value::Number(7).downcast();
  assert_eq!(declined, None);
}

#[test]
fn borrowed_downcast_ref_projects_or_declines_without_consuming() {
  let value = Value::Text(String::from("text"));
  assert_eq!(value.downcast_ref(), Some(Text));
  assert_eq!(value, Value::Text(String::from("text")));

  let number = Value::Number(7);
  let declined: Option<Text> = number.downcast_ref();
  assert_eq!(declined, None);
  assert_eq!(number, Value::Number(7));
}
