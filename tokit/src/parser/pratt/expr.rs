use core::marker::PhantomData;

use super::*;

/// Creates a pratt parser for a specific language
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn pratt<
  'inp,
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
>(
  parse_lhs: Lhs,
  parse_rhs: Rhs,
  fold_prefix: FoldPrefix,
  fold_infix: FoldInfix,
  fold_postfix: FoldPostfix,
) -> Pratt<
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
>
where
  Lhs: ParsePrattLHS<'inp, Power, O, PreOp, L, Ctx>,
  Rhs: ParsePrattRHS<'inp, Power, LeftAssoc, RightAssoc, NeitherAssoc, PostOp, L, Ctx>,
  Power: PrattPower,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L>,
{
  pratt_of(parse_lhs, parse_rhs, fold_prefix, fold_infix, fold_postfix)
}

/// Creates a pratt parser for a specific language
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn pratt_of<
  'inp,
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
  Lang,
>(
  parse_lhs: Lhs,
  parse_rhs: Rhs,
  fold_prefix: FoldPrefix,
  fold_infix: FoldInfix,
  fold_postfix: FoldPostfix,
) -> Pratt<
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
  Lang,
>
where
  Lhs: ParsePrattLHS<'inp, Power, O, PreOp, L, Ctx, Lang>,
  Rhs: ParsePrattRHS<'inp, Power, LeftAssoc, RightAssoc, NeitherAssoc, PostOp, L, Ctx, Lang>,
  Power: PrattPower,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  Pratt::new(parse_lhs, parse_rhs, fold_prefix, fold_infix, fold_postfix)
}

/// A Pratt parser combinator.
///
/// Built via [`pratt(lhs, rhs, fold_prefix, fold_infix, fold_postfix)`](pratt)
/// or [`pratt_of(lhs, rhs, fold_prefix, fold_infix, fold_postfix)`](pratt_of) and configured with
/// `.prefix(...)`, `.postfix(...)`, `.infix(...)`, and `.min_precedence(...)` methods.
pub struct Pratt<
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
  Lang: ?Sized = (),
> {
  min_precedence: Power,
  parse_lhs: Lhs,
  parse_rhs: Rhs,
  fold_prefix: FoldPrefix,
  fold_infix: FoldInfix,
  fold_postfix: FoldPostfix,
  _pre_op: PhantomData<PreOp>,
  _post_op: PhantomData<PostOp>,
  _left_assoc: PhantomData<LeftAssoc>,
  _right_assoc: PhantomData<RightAssoc>,
  _neither_assoc: PhantomData<NeitherAssoc>,
  _o: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
  Lang: ?Sized,
>
  Pratt<
    Power,
    Lhs,
    Rhs,
    FoldPrefix,
    FoldInfix,
    FoldPostfix,
    PreOp,
    LeftAssoc,
    RightAssoc,
    NeitherAssoc,
    PostOp,
    L,
    O,
    Ctx,
    Lang,
  >
{
  pub(crate) fn new<'a>(
    parse_lhs: Lhs,
    parse_rhs: Rhs,
    fold_prefix: FoldPrefix,
    fold_infix: FoldInfix,
    fold_postfix: FoldPostfix,
  ) -> Self
  where
    Lhs: ParsePrattLHS<'a, Power, O, PreOp, L, Ctx, Lang>,
    Rhs: ParsePrattRHS<'a, Power, LeftAssoc, RightAssoc, NeitherAssoc, PostOp, L, Ctx, Lang>,
    Power: PrattPower,
  {
    Self {
      parse_lhs,
      parse_rhs,
      min_precedence: Power::default(),
      fold_prefix,
      fold_infix,
      fold_postfix,
      _pre_op: PhantomData,
      _post_op: PhantomData,
      _left_assoc: PhantomData,
      _right_assoc: PhantomData,
      _neither_assoc: PhantomData,
      _o: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
  Lang: ?Sized,
>
  Pratt<
    Power,
    Lhs,
    Rhs,
    FoldPrefix,
    FoldInfix,
    FoldPostfix,
    PreOp,
    LeftAssoc,
    RightAssoc,
    NeitherAssoc,
    PostOp,
    L,
    O,
    Ctx,
    Lang,
  >
{
  /// Configure the prefix fold for this Pratt parser.
  pub fn prefix<'inp, F>(
    self,
    folder: F,
  ) -> Pratt<
    Power,
    Lhs,
    Rhs,
    F,
    FoldInfix,
    FoldPostfix,
    PreOp,
    LeftAssoc,
    RightAssoc,
    NeitherAssoc,
    PostOp,
    L,
    O,
    Ctx,
    Lang,
  >
  where
    F: PrattFoldPrefix<'inp, Power, PreOp, L, O, Ctx, Lang>,
  {
    Pratt {
      parse_lhs: self.parse_lhs,
      parse_rhs: self.parse_rhs,
      min_precedence: self.min_precedence,
      fold_prefix: folder,
      fold_infix: self.fold_infix,
      fold_postfix: self.fold_postfix,
      _pre_op: PhantomData,
      _post_op: PhantomData,
      _left_assoc: PhantomData,
      _right_assoc: PhantomData,
      _neither_assoc: PhantomData,
      _o: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Configure the infix fold for this Pratt parser.
  pub fn infix<'inp, F>(
    self,
    folder: F,
  ) -> Pratt<
    Power,
    Lhs,
    Rhs,
    FoldPrefix,
    F,
    FoldPostfix,
    PreOp,
    LeftAssoc,
    RightAssoc,
    NeitherAssoc,
    PostOp,
    L,
    O,
    Ctx,
    Lang,
  >
  where
    F: PrattFoldInfix<'inp, Power, LeftAssoc, RightAssoc, NeitherAssoc, L, O, Ctx, Lang>,
  {
    Pratt {
      parse_lhs: self.parse_lhs,
      parse_rhs: self.parse_rhs,
      min_precedence: self.min_precedence,
      fold_prefix: self.fold_prefix,
      fold_infix: folder,
      fold_postfix: self.fold_postfix,
      _pre_op: PhantomData,
      _post_op: PhantomData,
      _left_assoc: PhantomData,
      _right_assoc: PhantomData,
      _neither_assoc: PhantomData,
      _o: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Configure the postfix fold for this Pratt parser.
  pub fn postfix<'inp, F>(
    self,
    folder: F,
  ) -> Pratt<
    Power,
    Lhs,
    Rhs,
    FoldPrefix,
    FoldInfix,
    F,
    PreOp,
    LeftAssoc,
    RightAssoc,
    NeitherAssoc,
    PostOp,
    L,
    O,
    Ctx,
    Lang,
  >
  where
    F: PrattFoldPostfix<'inp, Power, PostOp, L, O, Ctx, Lang>,
  {
    Pratt {
      parse_lhs: self.parse_lhs,
      parse_rhs: self.parse_rhs,
      min_precedence: self.min_precedence,
      fold_prefix: self.fold_prefix,
      fold_infix: self.fold_infix,
      fold_postfix: folder,
      _pre_op: PhantomData,
      _post_op: PhantomData,
      _left_assoc: PhantomData,
      _right_assoc: PhantomData,
      _neither_assoc: PhantomData,
      _o: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Configure the minimum precedence level for this Pratt parser.
  pub fn min_precedence(
    self,
    min_precedence: Power,
  ) -> Pratt<
    Power,
    Lhs,
    Rhs,
    FoldPrefix,
    FoldInfix,
    FoldPostfix,
    PreOp,
    LeftAssoc,
    RightAssoc,
    NeitherAssoc,
    PostOp,
    L,
    O,
    Ctx,
    Lang,
  > {
    Pratt {
      parse_lhs: self.parse_lhs,
      parse_rhs: self.parse_rhs,
      min_precedence,
      fold_prefix: self.fold_prefix,
      fold_infix: self.fold_infix,
      fold_postfix: self.fold_postfix,
      _pre_op: PhantomData,
      _post_op: PhantomData,
      _left_assoc: PhantomData,
      _right_assoc: PhantomData,
      _neither_assoc: PhantomData,
      _o: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<
  'inp,
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
  Lang: ?Sized,
> ParseInput<'inp, L, O, Ctx, Lang>
  for Pratt<
    Power,
    Lhs,
    Rhs,
    FoldPrefix,
    FoldInfix,
    FoldPostfix,
    PreOp,
    LeftAssoc,
    RightAssoc,
    NeitherAssoc,
    PostOp,
    L,
    O,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lhs: ParsePrattLHS<'inp, Power, O, PreOp, L, Ctx, Lang>,
  Rhs: ParsePrattRHS<'inp, Power, LeftAssoc, RightAssoc, NeitherAssoc, PostOp, L, Ctx, Lang>,
  FoldPrefix: PrattFoldPrefix<'inp, Power, PreOp, L, O, Ctx, Lang>,
  FoldInfix: PrattFoldInfix<'inp, Power, LeftAssoc, RightAssoc, NeitherAssoc, L, O, Ctx, Lang>,
  FoldPostfix: PrattFoldPostfix<'inp, Power, PostOp, L, O, Ctx, Lang>,
  Power: PrattPower,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    parse(
      input,
      &mut self.parse_lhs,
      &mut self.parse_rhs,
      &mut self.fold_prefix,
      &mut self.fold_infix,
      &mut self.fold_postfix,
      self.min_precedence.clone(),
    )
  }
}

fn parse<
  'inp,
  Power,
  Lhs,
  Rhs,
  FoldPrefix,
  FoldInfix,
  FoldPostfix,
  PreOp,
  LeftAssoc,
  RightAssoc,
  NeitherAssoc,
  PostOp,
  L,
  O,
  Ctx,
  Lang: ?Sized,
>(
  inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  parse_lhs: &mut Lhs,
  parse_rhs: &mut Rhs,
  fold_prefix: &mut FoldPrefix,
  fold_infix: &mut FoldInfix,
  fold_postfix: &mut FoldPostfix,
  min_precedence: Power,
) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lhs: ParsePrattLHS<'inp, Power, O, PreOp, L, Ctx, Lang>,
  Rhs: ParsePrattRHS<'inp, Power, LeftAssoc, RightAssoc, NeitherAssoc, PostOp, L, Ctx, Lang>,
  FoldPrefix: PrattFoldPrefix<'inp, Power, PreOp, L, O, Ctx, Lang>,
  FoldInfix: PrattFoldInfix<'inp, Power, LeftAssoc, RightAssoc, NeitherAssoc, L, O, Ctx, Lang>,
  FoldPostfix: PrattFoldPostfix<'inp, Power, PostOp, L, O, Ctx, Lang>,
  Power: PrattPower,
{
  // Step 1: parse lhs -- either a prefix operator or an operand
  let mut lhs = match parse_lhs.parse_pratt_lhs(inp)? {
    PrattLHS::Operand(o) => o,
    PrattLHS::Prefix(precedenced) => {
      let (operator, power) = precedenced.into_components();
      let operand = parse(
        inp,
        parse_lhs,
        parse_rhs,
        fold_prefix,
        fold_infix,
        fold_postfix,
        power.clone(),
      )?;
      fold_prefix.fold_prefix(inp, operand, Precedenced::new(operator, power))?
    }
  };

  // Step 2: parse rhs -- either an infix/postfix operator or the end of this pratt expression
  let mut prev_op_is_neither: Option<Power> = None;
  while !inp.is_eoi() {
    let ckp = inp.save();
    match parse_rhs.parse_pratt_rhs(inp)? {
      PrattRHS::Postfix(precedenced) => {
        let (operator, op_power) = precedenced.into_components();
        if op_power >= min_precedence {
          lhs = fold_postfix.fold_postfix(inp, lhs, Precedenced::new(operator, op_power))?;
        } else {
          inp.restore(ckp);
          break;
        }
      }
      PrattRHS::Infix(infix) => {
        let lpower = infix.precedence();
        let rpower = match infix.token_ref() {
          PrattInfix::Left(_) => lpower.next(),
          PrattInfix::Right(_) => lpower.prev(),
          PrattInfix::Neither(_) => lpower.next(),
        };

        if lpower.lt(&min_precedence) || prev_op_is_neither.as_ref() == Some(lpower) {
          inp.restore(ckp);
          break;
        }

        let next_neither = if matches!(infix.token_ref(), PrattInfix::Neither(_)) {
          Some((*lpower).clone())
        } else {
          None
        };
        let rhs = parse(
          inp,
          parse_lhs,
          parse_rhs,
          fold_prefix,
          fold_infix,
          fold_postfix,
          rpower,
        )?;
        lhs = fold_infix.fold_infix(inp, lhs, rhs, infix)?;
        prev_op_is_neither = next_neither;
      }
    }
  }

  Ok(lhs)
}
