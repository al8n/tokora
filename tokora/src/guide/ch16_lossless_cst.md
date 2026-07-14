# 16. Lossless CSTs with Rowan

Prerequisites: chapters 1, 2, and 11. The parser walkthroughs are helpful but not required.

This chapter uses Tokora's optional [`cst`](crate::cst) module with Rowan. A concrete syntax tree
is lossless only when every source token—including whitespace and comments—becomes a Rowan
builder event. Tokora supplies the builder wrapper; the language decides its syntax kinds and
when to start and finish nodes.

## Enable Rowan

Rowan is an optional dependency and Tokora does not re-export it. A downstream language therefore
needs both dependencies:

```toml
[dependencies]
tokora = { version = "0.1", features = ["logos", "rowan"] }
rowan = "0.16"
```

The `rowan` feature implies `std`, but it does not imply `logos`. This chapter's guide module is
therefore gated specifically on `rowan` while the outer guide retains its `std + logos_0_16` gate.

## Define syntax kinds

Define one syntax-kind enum for both nodes and tokens, then implement `rowan::Language`. The
[`cst::SyntaxTreeBuilder`](crate::cst::SyntaxTreeBuilder) converts those kinds to Rowan raw kinds
when it receives [`new`](crate::cst::SyntaxTreeBuilder::new),
[`start_node`](crate::cst::SyntaxTreeBuilder::start_node),
[`token`](crate::cst::SyntaxTreeBuilder::token), [`finish_node`](crate::cst::SyntaxTreeBuilder::finish_node),
and [`finish`](crate::cst::SyntaxTreeBuilder::finish) calls.

```rust
use rowan::{Language, SyntaxKind as RawKind, SyntaxNode};
use tokora::cst::SyntaxTreeBuilder;

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SyntaxKind { Root, Ident, Eq, Semi, Whitespace, Comment }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MiniLanguage {}

impl Language for MiniLanguage {
  type Kind = SyntaxKind;

  fn kind_from_raw(raw: RawKind) -> Self::Kind {
    match raw.0 {
      0 => SyntaxKind::Root,
      1 => SyntaxKind::Ident,
      2 => SyntaxKind::Eq,
      3 => SyntaxKind::Semi,
      4 => SyntaxKind::Whitespace,
      5 => SyntaxKind::Comment,
      _ => panic!("unknown syntax kind"),
    }
  }

  fn kind_to_raw(kind: Self::Kind) -> RawKind { RawKind(kind as u16) }
}

let builder = SyntaxTreeBuilder::<MiniLanguage>::new();
builder.start_node(SyntaxKind::Root);
builder.token(SyntaxKind::Ident, "answer");
builder.token(SyntaxKind::Whitespace, " ");
builder.token(SyntaxKind::Eq, "=");
builder.token(SyntaxKind::Whitespace, " ");
builder.token(SyntaxKind::Ident, "value");
builder.token(SyntaxKind::Semi, ";");
builder.finish_node();

let root: SyntaxNode<MiniLanguage> = SyntaxNode::new_root(builder.finish());
assert_eq!(root.to_string(), "answer = value;");
```

## Preserve trivia

Do not give the lossless lexer a Logos `skip` rule for whitespace or comments. Emit those tokens
and implement `Token::is_trivia` as classification only; it does not preserve anything by itself.
In particular, do not use `padded()` for the main walkthrough: it consumes trivia but emits no
Rowan builder events.

The helper below records every token before deciding whether it is significant. It uses
[`InputRef::slice`](crate::InputRef::slice) immediately after consumption, so Rowan receives
the exact source text rather than reconstructed formatting.

```rust
use rowan::{GreenNode, Language, SyntaxKind as RawKind, SyntaxNode};
use tokora::{
  Emitter, InputRef, Parse, ParseContext, Parser, Token as TokenT,
  cst::SyntaxTreeBuilder,
  logos::{self, Logos},
};

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SyntaxKind { Root, Ident, Eq, Semi, Whitespace, Comment }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MiniLanguage {}
impl Language for MiniLanguage {
  type Kind = SyntaxKind;
  fn kind_from_raw(raw: RawKind) -> Self::Kind {
    match raw.0 {
      0 => SyntaxKind::Root, 1 => SyntaxKind::Ident, 2 => SyntaxKind::Eq,
      3 => SyntaxKind::Semi, 4 => SyntaxKind::Whitespace, 5 => SyntaxKind::Comment,
      _ => panic!("unknown syntax kind"),
    }
  }
  fn kind_to_raw(kind: Self::Kind) -> RawKind { RawKind(kind as u16) }
}
impl core::fmt::Display for SyntaxKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(match self {
      Self::Root => "root", Self::Ident => "identifier", Self::Eq => "=", Self::Semi => ";",
      Self::Whitespace => "whitespace", Self::Comment => "comment",
    })
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct LexError;
impl From<()> for LexError { fn from(_: ()) -> Self { Self } }
#[derive(Clone, Debug, Logos)]
#[logos(crate = logos, error = LexError)]
enum Token {
  #[regex(r"[ \t\r\n]+")] Whitespace,
  #[regex(r"//[^\r\n]*", allow_greedy = true)] Comment,
  #[regex(r"[A-Za-z_][A-Za-z0-9_]*")] Ident,
  #[token("=")] Eq,
  #[token(";")] Semi,
}
impl core::fmt::Display for Token {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { core::fmt::Display::fmt(&self.kind(), f) }
}
impl TokenT<'_> for Token {
  type Kind = SyntaxKind;
  type Error = LexError;
  fn kind(&self) -> SyntaxKind {
    match self {
      Self::Whitespace => SyntaxKind::Whitespace, Self::Comment => SyntaxKind::Comment,
      Self::Ident => SyntaxKind::Ident, Self::Eq => SyntaxKind::Eq, Self::Semi => SyntaxKind::Semi,
    }
  }
  fn is_trivia(&self) -> bool { matches!(self, Self::Whitespace | Self::Comment) }
}
type MiniLexer<'a> = tokora::lexer::LogosLexer<'a, Token>;
#[derive(Debug)]
enum ParseError { Lex, Unexpected }
impl From<LexError> for ParseError { fn from(_: LexError) -> Self { Self::Lex } }
impl<'inp> From<tokora::error::token::UnexpectedTokenOf<'inp, MiniLexer<'inp>>> for ParseError {
  fn from(_: tokora::error::token::UnexpectedTokenOf<'inp, MiniLexer<'inp>>) -> Self { Self::Unexpected }
}

fn record_next<'inp, Ctx>(
  input: &mut InputRef<'inp, '_, MiniLexer<'inp>, Ctx>,
  builder: &SyntaxTreeBuilder<MiniLanguage>,
) -> Result<Option<Token>, ParseError>
where
  Ctx: ParseContext<'inp, MiniLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, MiniLexer<'inp>, Error = ParseError>,
{
  let Some(token) = input.next()? else { return Ok(None) };
  builder.token(token.data().kind(), input.slice());
  Ok(Some(token.into_data()))
}

fn next_significant<'inp, Ctx>(
  input: &mut InputRef<'inp, '_, MiniLexer<'inp>, Ctx>,
  builder: &SyntaxTreeBuilder<MiniLanguage>,
) -> Result<Option<Token>, ParseError>
where
  Ctx: ParseContext<'inp, MiniLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, MiniLexer<'inp>, Error = ParseError>,
{
  while let Some(token) = record_next(input, builder)? {
    if !token.is_trivia() {
      return Ok(Some(token));
    }
  }
  Ok(None)
}

fn assignment_tokens<'inp, Ctx>(
  input: &mut InputRef<'inp, '_, MiniLexer<'inp>, Ctx>,
  builder: &SyntaxTreeBuilder<MiniLanguage>,
) -> Result<(), ParseError>
where
  Ctx: ParseContext<'inp, MiniLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, MiniLexer<'inp>, Error = ParseError>,
{
  builder.start_node(SyntaxKind::Root);
  for expected in [SyntaxKind::Ident, SyntaxKind::Eq, SyntaxKind::Ident, SyntaxKind::Semi] {
    match next_significant(input, builder)? {
      Some(token) if token.kind() == expected => {}
      _ => return Err(ParseError::Unexpected),
    }
  }
  while record_next(input, builder)?.is_some() {}
  builder.finish_node();
  Ok(())
}

fn assignment<'inp, Ctx>(
  input: &mut InputRef<'inp, '_, MiniLexer<'inp>, Ctx>,
) -> Result<GreenNode, ParseError>
where
  Ctx: ParseContext<'inp, MiniLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, MiniLexer<'inp>, Error = ParseError>,
{
  let builder = SyntaxTreeBuilder::<MiniLanguage>::new();
  assignment_tokens(input, &builder)?;
  Ok(builder.finish())
}

let source = "answer = value; // retained\n";
let green = Parser::new().apply(assignment).parse_str(source).unwrap();
let root: SyntaxNode<MiniLanguage> = SyntaxNode::new_root(green);
assert_eq!(root.to_string(), source);
```

## Build nodes during parsing

Start a root node before consuming grammar tokens, record all tokens through the helper, and
finish the node once the parse completes. The builder uses interior mutability so an immutable
reference can pass through parser functions. That convenience does not make external state part
of Tokora's input transaction.

## Prove round-trip losslessness

The test above is the essential property:
`SyntaxNode::new_root(green).to_string() == source`. It detects a skipped whitespace token, a
comment accidentally discarded by a lexer rule, or a reconstructed token spelling that differs
from the source.

For typed traversal after building, orient around [`CstElement`](crate::cst::CstElement),
[`CstNode`](crate::cst::CstNode), [`CstToken`](crate::cst::CstToken), and
[`cst::cast`](crate::cst::cast). They wrap and cast Rowan elements; they do not change the
losslessness rule.

## Checkpoints and transactional limits

[`checkpoint`](crate::cst::SyntaxTreeBuilder::checkpoint) and
[`start_node_at`](crate::cst::SyntaxTreeBuilder::start_node_at) let a later grammar decision
retroactively wrap already-recorded children. Use them after a deterministic decision, for
example to wrap a parsed left operand in a binary-expression node.

Do not emit irreversible builder events inside a speculative or backtracking path. Tokora input
rollback restores cursor, lexer, cache, and diagnostics; it cannot roll back external Rowan
builder state. Either decide before emitting events, buffer your own reversible events, or make
the builder transaction explicit in the surrounding application.

For more detail, read the [`cst`](crate::cst) API reference and use the maintained parser examples
as the grammar-level counterparts to this lossless tree-building pattern.
