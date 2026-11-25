//! What the flat driver's suites write down of a parse: a handler that records each item it is
//! told, and a machine that resolves those items with no input.

// Each suite uses a part of it.
#![allow(dead_code)]

use tokora::{
  Emitter, InputRef, ParseContext, SimpleSpan,
  pratt::{
    PrattHandler, PrattInfix, PrattPoll, PrattRHS, PrattReduction, PrattResolver, Precedenced,
  },
  span::Spanned,
};

use crate::flat_fixture::{ByteLexer, Tok};

/// One item of an expression, as the handler was told it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
  /// A prefix operator: its byte, its offset and its power.
  Prefix(u8, usize, u8),
  /// An operand, as the operand parser accepted it.
  Operand(String),
  /// An infix operator: its byte, its offset, its associativity and its power.
  Infix(u8, usize, PrattInfix<(), (), ()>, u8),
  /// A postfix operator: its byte, its offset and its power.
  Postfix(u8, usize, u8),
}

impl Item {
  /// The byte of the token this item is, an operand's first.
  pub fn byte(&self) -> u8 {
    match self {
      Item::Prefix(byte, ..) | Item::Infix(byte, ..) | Item::Postfix(byte, ..) => *byte,
      Item::Operand(text) => text.as_bytes()[0],
    }
  }
}

/// Writes down each item it is told.
#[derive(Debug, Default)]
pub struct Items(pub Vec<Item>);

/// The error of a parse over `ByteLexer<'inp, SKIP>` in `Ctx`.
type ErrorIn<'inp, const SKIP: bool, Ctx> =
  <<Ctx as ParseContext<'inp, ByteLexer<'inp, SKIP>>>::Emitter as Emitter<
    'inp,
    ByteLexer<'inp, SKIP>,
  >>::Error;

impl<'inp, const SKIP: bool, Ctx> PrattHandler<'inp, u8, ByteLexer<'inp, SKIP>, String, Ctx>
  for Items
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
{
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), ErrorIn<'inp, SKIP, Ctx>> {
    let (tok, power) = operator.into_components();
    self
      .0
      .push(Item::Prefix(tok.data().0, tok.span().start(), power));
    Ok(())
  }

  fn on_operand(
    &mut self,
    _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    operand: String,
  ) -> Result<(), ErrorIn<'inp, SKIP, Ctx>> {
    self.0.push(Item::Operand(operand));
    Ok(())
  }

  fn on_infix(
    &mut self,
    _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    operator: Precedenced<
      PrattInfix<Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>>,
      u8,
    >,
  ) -> Result<(), ErrorIn<'inp, SKIP, Ctx>> {
    let (operator, power) = operator.into_components();
    let (associativity, tok) = match operator {
      PrattInfix::Left(tok) => (PrattInfix::Left(()), tok),
      PrattInfix::Right(tok) => (PrattInfix::Right(()), tok),
      PrattInfix::Neither(tok) => (PrattInfix::Neither(()), tok),
    };
    self.0.push(Item::Infix(
      tok.data().0,
      tok.span().start(),
      associativity,
      power,
    ));
    Ok(())
  }

  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), ErrorIn<'inp, SKIP, Ctx>> {
    let (tok, power) = operator.into_components();
    self
      .0
      .push(Item::Postfix(tok.data().0, tok.span().start(), power));
    Ok(())
  }
}

type Machine = PrattResolver<String, u8, u8, u8, u8, u8, u8>;

/// The fold the typed driver's folds write, for one reduction.
fn fold(reduction: PrattReduction<String, u8, u8, u8, u8, u8, u8>) -> String {
  match reduction {
    PrattReduction::Prefix(prefix) => {
      let (operand, operator) = prefix.into_components();
      format!("({}{operand})", char::from(operator.into_data()))
    }
    PrattReduction::Infix(infix) => {
      let (left, right, operator) = infix.into_components();
      let (PrattInfix::Left(byte) | PrattInfix::Right(byte) | PrattInfix::Neither(byte)) =
        operator.into_data();
      format!("({left}{}{right})", char::from(byte))
    }
    PrattReduction::Postfix(postfix) => {
      let (operand, operator) = postfix.into_components();
      format!("({operand}{})", char::from(operator.into_data()))
    }
  }
}

/// Performs the one reduction the machine hands out.
fn reduce(machine: &mut Machine) {
  match machine.poll() {
    PrattPoll::Reduce(reduction) => machine
      .handle_reduced(fold(reduction))
      .expect("the machine takes the value of its own reduction"),
    PrattPoll::Term | PrattPoll::Operator | PrattPoll::Reducing | PrattPoll::Done(_) => {
      panic!("the machine owed a reduction")
    }
  }
}

/// Reduces what binds tighter than the operator the machine is offered.
fn settle(machine: &mut Machine, offered: &PrattRHS<(), (), (), (), u8>) {
  while machine.offer(offered).is_reduce() {
    machine
      .close()
      .expect("an operand ends where a reduction is owed");
    reduce(machine);
  }
}

/// Resolves `items`, fed in written order to a machine with `floor` and no input, into the tree
/// the typed driver's folds write.
pub fn replay(items: &[Item], floor: u8) -> String {
  let mut machine = Machine::new().min_precedence(floor);
  for item in items {
    match item {
      Item::Prefix(byte, _, power) => machine
        .handle_prefix(Precedenced::new(*byte, *power))
        .expect("a prefix operator is taken in a term"),
      Item::Operand(text) => machine
        .handle_operand(text.clone())
        .expect("an operand is taken in a term"),
      Item::Infix(byte, _, associativity, power) => {
        settle(
          &mut machine,
          &PrattRHS::Infix(Precedenced::new(*associativity, *power)),
        );
        let operator = match associativity {
          PrattInfix::Left(()) => PrattInfix::Left(*byte),
          PrattInfix::Right(()) => PrattInfix::Right(*byte),
          PrattInfix::Neither(()) => PrattInfix::Neither(*byte),
        };
        machine
          .handle_infix(Precedenced::new(operator, *power))
          .expect("the flat driver told only operators the machine takes");
      }
      Item::Postfix(byte, _, power) => {
        settle(
          &mut machine,
          &PrattRHS::Postfix(Precedenced::new((), *power)),
        );
        machine
          .handle_postfix(Precedenced::new(*byte, *power))
          .expect("the flat driver told only operators the machine takes");
        reduce(&mut machine);
      }
    }
  }
  machine
    .finish()
    .expect("an expression ends after an operand");
  loop {
    match machine.poll() {
      PrattPoll::Reduce(reduction) => machine
        .handle_reduced(fold(reduction))
        .expect("the machine takes the value of its own reduction"),
      PrattPoll::Done(tree) => return tree,
      PrattPoll::Term | PrattPoll::Operator | PrattPoll::Reducing => {
        panic!("after `finish` the machine reduces, and then it is done")
      }
    }
  }
}
