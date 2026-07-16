use super::*;

use crate::{
  error::UnexpectedEot,
  token::{PunctuatorToken, PunctuatorTokenExt, SpannedPunctuatorToken},
};

macro_rules! try_expect_punct {
  ($($punct:ident $(:$alias:ident)? :$punct_char:literal),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Tries to advance to the next valid token if it is " $punct " (" $punct_char "). Otherwise leaves the input unchanged."]
        #[inline(always)]
        pub fn [< try_expect_ $punct >](
          &mut self,
        ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L::Token: crate::token::PunctuatorToken<'inp>,
        {
          self.try_expect(|t| t.data.[<is_ $punct>]())
        }

        #[doc = "Advances to the next valid token and expects it to be " $punct " (" $punct_char ")."]
        #[inline(always)]
        pub fn [< expect_ $punct >](
          &mut self,
        ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L::Token: PunctuatorToken<'inp>,
          <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
            + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
        {
          match self.next()? {
            Some(spanned) => {
              <Spanned<L::Token, L::Span> as SpannedPunctuatorToken<'inp, L, Lang>>::[< expect_ $punct >](spanned).map_err(Into::into)
            },
            None => Err(UnexpectedEot::eot_of(self.span().end()).into()),
          }
        }

        $(
          #[doc = "Tries to advance to the next valid token if it is " $alias " (" $punct_char "). Otherwise leaves the input unchanged."]
          #[inline(always)]
          pub fn [< try_expect_ $alias >](
            &mut self,
          ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
          where
            L::Token: PunctuatorToken<'inp>,
          {
            self.[< try_expect_ $punct >]()
          }

          #[doc = "Advances to the next valid token and expects it to be " $alias " (" $punct_char ")."]
          #[inline(always)]
          pub fn [< expect_ $alias >](
            &mut self,
          ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
          where
            L::Token: PunctuatorToken<'inp>,
            <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
              + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
          {
            self.[< expect_ $punct >]()
          }
        )?
      )*
    }
  };
}

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  try_expect_punct!(
    // Delimiters
    open_angle:less_than:"<",
    close_angle:greater_than:">",
    open_brace:"{",
    close_brace:"}",
    open_paren:"(",
    close_paren:")",
    open_bracket:"[",
    close_bracket:"]",

    // ASCII Punctuation
    at:"@",
    asterisk:"*",
    ampersand: "&",
    apostrophe:"'",
    backtick:"`",
    backslash:"\\",
    caret:"^",
    comma:",",
    colon:":",
    dot:".",
    dollar:"$",
    double_quote:"\"",
    equal:"=",
    exclamation:bang:"!",
    hash:"#",
    hyphen:minus:"-",
    pipe:"|",
    plus:"+",
    percent:"%",
    question:"?",
    slash:"/",
    semicolon:";",
    tilde:"~",
    underscore:"_",

    // Multi-character Punctuators
    arrow:thin_arrow:"->",
    fat_arrow:"=>",
    pipe_arrow:pipe_forward:"|>",

    // Equal related
    colon_equal:colon_assign:":=",
    logical_equal: "==",
    logical_not_equal: "!=",
    strict_equal: "===",
    strict_not_equal: "!==",
    less_than_or_equal: "<=",
    greater_than_or_equal: ">=",
    strict_less_than_or_equal: "<==",
    strict_greater_than_or_equal: ">==",

    plus_equal:add_assign: "+=",
    hyphen_equal:sub_assign: "-=",
    asterisk_equal:mul_assign: "*=",
    exponentiation_equal:exponentiation_assign: "**=",
    slash_equal:div_assign: "/=",
    backslash_equal: "\\=",

    percent_equal:rem_assign: "%=",

    ampersand_equal:bitand_assign: "&=",
    pipe_equal:bitor_assign: "|=",
    caret_equal:xor_assign: "^=",

    shl_equal:shl_assign: "<<=",
    shr_equal:shr_assign: ">>=",
    sar_equal:sar_assign: ">>>=",

    shl: "<<",
    shr: ">>",
    sar: ">>>",

    increment: "++",
    decrement: "--",
    exponentiation: "**",

    logical_and: "&&",
    logical_or: "||",

    double_colon:"::",
    spread: "...",
    null_coalesce: "??",
    optional_chain: "?.",

    // Trivia
    tab:"\t",
    newline:"\n",
    carriage_return:"\r",
    crlf:"\r\n",
    space:" ",
  );

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `true`, the token is consumed and returned.
  /// Otherwise, the token remains in the cache and `Ok(None)` is returned.
  pub fn try_expect<F>(
    &mut self,
    mut pred: F,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
  {
    trace_event!(self, "try_expect");
    // if cache is empty, directly try expect on input
    if self.cache.is_empty() {
      return self.try_expect_on_input(pred);
    }

    // pop from cache if matching
    Ok(self.cache.pop_front_if(|t| pred(t.token)).map(|tok| {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.commit_token(&tok, &span);
      *self.state = state;

      Spanned::new(span, tok)
    }))
  }

  // /// Advances to the next valid token and expects it to satisfy the predicate.
  // ///
  // /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  // /// If `pred` returns `Ok`, the token is consumed and returned.
  // /// Otherwise, the error is returned and the token remains in the cache.
  // pub fn try_expect_either<F>(
  //   &mut self,
  //   mut pred: F,
  // ) -> Result<Option<Either<Spanned<L::Token, L::Span>, Spanned<L::Token, L::Span>>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  // where
  //   F: FnMut(Spanned<&L::Token, &L::Span>) -> Option<Either<(), ()>>,
  // {
  //   // if cache is empty, directly try expect on input
  //   if self.cache.is_empty() {
  //     return self.try_expect_on_input(pred);
  //   }

  //   // pop from cache if matching
  //   let mut is_left = false;
  //   Ok(self.cache.pop_front_if(|t| {
  //     match pred(t.token) {
  //       Some(Either::Left(_)) => {
  //         is_left = true;
  //         true
  //       }
  //       Some(Either::Right(_)) => {
  //         true
  //       }
  //       None => false,
  //     }
  //   }).map(|tok| {
  //     let (lexed, state) = tok.into_components();
  //     let (span, tok) = lexed.into_components();
  //     self.set_span_after_consume((&span).into());
  //     *self.state = state;

  //     if is_left {
  //       Either::Left(Spanned::new(span, tok))
  //     } else {
  //       Either::Right(Spanned::new(span, tok))
  //     }
  //   }))
  // }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Some(output)`, the token is consumed and `(output, token)` is returned.
  /// If `pred` returns `None`, the token remains in the cache and `Ok(None)` is returned.
  pub fn try_expect_map<O, F>(
    &mut self,
    mut pred: F,
  ) -> Result<
    Option<(O, Spanned<L::Token, L::Span>)>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> Option<O>,
  {
    trace_event!(self, "try_expect_map");
    // if cache is empty, directly try expect on input
    if self.cache.is_empty() {
      return self.try_expect_map_on_input(pred);
    }

    let mut output = None;
    Ok(
      self
        .cache
        .pop_front_if(|t| match pred(t.token().copied()) {
          Some(out) => {
            output = Some(out);
            true
          }
          None => false,
        })
        .map(|tok| {
          let (lexed, state) = tok.into_components();
          let (span, tok) = lexed.into_components();
          self.commit_token(&tok, &span);
          *self.state = state;
          (output.unwrap(), Spanned::new(span, tok))
        }),
    )
  }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Some(Ok(output))`, the token is consumed and `(output, token)` is returned.
  /// If `pred` returns `Some(Err(error))`, the token is consumed and `Err(error)` is returned.
  /// If `pred` returns `None`, the token remains in the cache and `Ok(None)` is returned.
  pub fn try_expect_and_then<O, F>(
    &mut self,
    mut pred: F,
  ) -> Result<
    Option<(O, Spanned<L::Token, L::Span>)>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(
      Spanned<&L::Token, &L::Span>,
    ) -> Option<Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>>,
  {
    trace_event!(self, "try_expect_and_then");
    // if cache is empty, directly try expect on input
    if self.cache.is_empty() {
      return self.try_expect_and_then_on_input(pred);
    }

    let mut output = None;
    if let Some(tok) = self.cache.pop_front_if(|t| match pred(t.token().copied()) {
      Some(res) => {
        output = Some(res);
        true
      }
      None => false,
    }) {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.commit_token(&tok, &span);
      *self.state = state;

      return match output {
        Some(res) => res.map(|o| Some((o, Spanned::new(span, tok)))),
        None => Ok(None),
      };
    }

    Ok(None)
  }

  #[inline]
  fn try_expect_and_then_on_input<O, F>(
    &mut self,
    mut pred: F,
  ) -> Result<
    Option<(O, Spanned<L::Token, L::Span>)>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(
      Spanned<&L::Token, &L::Span>,
    ) -> Option<Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>>,
  {
    // A sticky limit trip latches a poison boundary: at or past the durable
    // frontier, stop without rebuilding a lexer, mirroring the short-circuit in
    // `next()`; strictly before it, lexing proceeds (replaying a drained prefix). A
    // scan that finds no matching token yields `Ok(None)`, the poisoned outcome too.
    if self.reached_boundary(self.offset()) {
      return Ok(None);
    }

    let mut lex_at = self.offset().clone();
    let mut lexer = self.lexer();

    match self.scan_with(&mut lexer, &mut lex_at, &AtCursor)? {
      Scan::Token(tok) => match pred(tok.as_ref()) {
        Some(output) => {
          self.commit_token(tok.data(), tok.span_ref());
          *self.state = lexer.into_state();
          output.map(|o| Some((o, tok)))
        }
        None => {
          let (span, tok) = tok.into_components();
          // put back the token into cache as it was peeked
          let ct = CachedToken::new(Spanned::new(span, tok), lexer.state().clone());
          let _ = self.cache_push_back(ct);
          Ok(None)
        }
      },
      Scan::Tripped | Scan::Eof => Ok(None),
    }
  }

  #[inline]
  fn try_expect_on_input<F>(
    &mut self,
    mut pred: F,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
  {
    // A sticky limit trip latches a poison boundary: at or past the durable
    // frontier, stop without rebuilding a lexer, mirroring the short-circuit in
    // `next()`; strictly before it, lexing proceeds (replaying a drained prefix). A
    // scan that finds no matching token yields `Ok(None)`, the poisoned outcome too.
    if self.reached_boundary(self.offset()) {
      return Ok(None);
    }

    let mut lex_at = self.offset().clone();
    let mut lexer = self.lexer();

    match self.scan_with(&mut lexer, &mut lex_at, &AtCursor)? {
      Scan::Token(tok) => {
        // if the token matches, we return it
        if pred(tok.as_ref()) {
          self.commit_token(tok.data(), tok.span_ref());
          *self.state = lexer.into_state();
          Ok(Some(tok))
        } else {
          let (span, tok) = tok.into_components();
          // put back the token into cache as it was peeked
          let ct = CachedToken::new(Spanned::new(span, tok), lexer.state().clone());
          let _ = self.cache_push_back(ct);
          Ok(None)
        }
      }
      Scan::Tripped | Scan::Eof => Ok(None),
    }
  }

  #[inline]
  fn try_expect_map_on_input<O, F>(
    &mut self,
    mut pred: F,
  ) -> Result<
    Option<(O, Spanned<L::Token, L::Span>)>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> Option<O>,
  {
    // A sticky limit trip latches a poison boundary: at or past the durable
    // frontier, stop without rebuilding a lexer, mirroring the short-circuit in
    // `next()`; strictly before it, lexing proceeds (replaying a drained prefix). A
    // scan that finds no matching token yields `Ok(None)`, the poisoned outcome too.
    if self.reached_boundary(self.offset()) {
      return Ok(None);
    }

    let mut lex_at = self.offset().clone();
    let mut lexer = self.lexer();

    match self.scan_with(&mut lexer, &mut lex_at, &AtCursor)? {
      Scan::Token(tok) => {
        // if the token matches, we return it
        if let Some(out) = pred(tok.as_ref()) {
          self.commit_token(tok.data(), tok.span_ref());
          *self.state = lexer.into_state();
          Ok(Some((out, tok)))
        } else {
          let (span, tok) = tok.into_components();
          // put back the token into cache as it was peeked
          let ct = CachedToken::new(Spanned::new(span, tok), lexer.state().clone());
          let _ = self.cache_push_back(ct);
          Ok(None)
        }
      }
      Scan::Tripped | Scan::Eof => Ok(None),
    }
  }
}
