//! Scanner drain benchmarks for the `InputRef` hot paths.
//!
//! These exercise the real per-operation scanner protocol — build a fresh
//! lexer, lex within the poison boundary, dedup lexer errors, commit — over a
//! synthetic source large enough (~128 KiB) to dominate the fixed per-parse
//! setup cost, so a full drain resolves ~1-2% deltas.
//!
//! Benches:
//!   * `next_drain` — `while let Some = next()` to EOF. THE hot path.
//!   * `skip_trivia_next` — alternating `skip_while(trivia)` + `next()`.
//!   * `try_expect_hits` — `try_expect` with an always-matching predicate
//!     (commit path).
//!   * `try_expect_misses` — `try_expect` with a never-matching predicate
//!     (put-back path), interleaved with `next()` to advance.
//!   * `peek1_then_next` — `peek` + `next`. Control: `peek` is not part of the
//!     scanner-protocol unification, so this must not move; it detects
//!     accidental collateral.
//!
//! The source contains only well-formed idents/ints/puncts/whitespace, so the
//! emitter is never invoked — the measurement is pure lex/cache/commit.

use core::{fmt::Write as _, time::Duration};
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

use tokit::{
  Emitter, InputRef, Parse, ParseContext, Parser, Token,
  error::token::UnexpectedToken,
  lexer::LogosLexer,
  logos::{self, Logos},
};

// ── Fixture: a small ident/int/punct/whitespace-trivia token enum ─────────────

#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(crate = logos)]
enum BenchTok {
  /// Whitespace trivia — kept as a token (not `skip`ped) so `skip_while`
  /// and the trivia-skipping paths have something to consume.
  #[regex(r"[ \t\r\n]+")]
  Ws,
  #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
  Ident,
  #[regex(r"[0-9]+")]
  Int,
  #[token("+")]
  #[token("-")]
  #[token("*")]
  #[token("/")]
  #[token("=")]
  #[token(",")]
  #[token(";")]
  Punct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BenchKind {
  Ws,
  Ident,
  Int,
  Punct,
}

impl core::fmt::Display for BenchKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let s = match self {
      BenchKind::Ws => "whitespace",
      BenchKind::Ident => "identifier",
      BenchKind::Int => "integer",
      BenchKind::Punct => "punctuation",
    };
    f.write_str(s)
  }
}

impl core::fmt::Display for BenchTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    core::fmt::Display::fmt(&self.kind(), f)
  }
}

impl Token<'_> for BenchTok {
  type Kind = BenchKind;
  type Error = ();

  fn kind(&self) -> BenchKind {
    match self {
      BenchTok::Ws => BenchKind::Ws,
      BenchTok::Ident => BenchKind::Ident,
      BenchTok::Int => BenchKind::Int,
      BenchTok::Punct => BenchKind::Punct,
    }
  }

  fn is_trivia(&self) -> bool {
    matches!(self, BenchTok::Ws)
  }
}

type BenchLexer<'a> = LogosLexer<'a, BenchTok>;

// A trivial emitter error: the source is well-formed, so these `From`s are only
// needed to satisfy the `FromEmitterError` bound `Parser::new` requires — they
// are never constructed at runtime.
#[derive(Debug, Default, Clone)]
struct BenchError;

impl From<()> for BenchError {
  fn from(_: ()) -> Self {
    BenchError
  }
}

impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for BenchError {
  fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self {
    BenchError
  }
}

// ── Scan drivers (generic over the parse context, as external callers write) ──

fn next_drain<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, BenchLexer<'inp>, Ctx>,
) -> Result<usize, BenchError>
where
  Ctx: ParseContext<'inp, BenchLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, BenchLexer<'inp>, Error = BenchError>,
{
  let mut n = 0usize;
  while let Some(tok) = inp.next()? {
    black_box(&tok);
    n += 1;
  }
  Ok(n)
}

fn skip_trivia_next<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, BenchLexer<'inp>, Ctx>,
) -> Result<usize, BenchError>
where
  Ctx: ParseContext<'inp, BenchLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, BenchLexer<'inp>, Error = BenchError>,
{
  let mut n = 0usize;
  loop {
    inp.skip_while(|t| t.data.is_trivia())?;
    match inp.next()? {
      Some(tok) => {
        black_box(&tok);
        n += 1;
      }
      None => break,
    }
  }
  Ok(n)
}

fn try_expect_hits<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, BenchLexer<'inp>, Ctx>,
) -> Result<usize, BenchError>
where
  Ctx: ParseContext<'inp, BenchLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, BenchLexer<'inp>, Error = BenchError>,
{
  let mut n = 0usize;
  // Always-matching predicate: every token commits via the on-input hit path.
  while let Some(tok) = inp.try_expect(|_t| true)? {
    black_box(&tok);
    n += 1;
  }
  Ok(n)
}

fn try_expect_misses<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, BenchLexer<'inp>, Ctx>,
) -> Result<usize, BenchError>
where
  Ctx: ParseContext<'inp, BenchLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, BenchLexer<'inp>, Error = BenchError>,
{
  let mut n = 0usize;
  loop {
    // Never-matching predicate: the on-input scan lexes one token, peeks it,
    // and puts it back into the cache (the miss path). `next()` then consumes
    // that cached token (no re-lex) so the scan advances one token per cycle.
    let put_back = inp.try_expect(|_t| false)?;
    debug_assert!(put_back.is_none());
    black_box(&put_back);
    match inp.next()? {
      Some(tok) => {
        black_box(&tok);
        n += 1;
      }
      None => break,
    }
  }
  Ok(n)
}

fn peek1_then_next<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, BenchLexer<'inp>, Ctx>,
) -> Result<usize, BenchError>
where
  Ctx: ParseContext<'inp, BenchLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, BenchLexer<'inp>, Error = BenchError>,
{
  let mut n = 0usize;
  loop {
    // Scope the peek borrow so it is released before `next()`.
    let got = {
      let peeked = inp.peek_one()?;
      peeked.is_some()
    };
    black_box(got);
    match inp.next()? {
      Some(tok) => {
        black_box(&tok);
        n += 1;
      }
      None => break,
    }
  }
  Ok(n)
}

// ── Synthetic source ──────────────────────────────────────────────────────────

/// ~128 KiB of well-formed `ident = int + ident ;` lines. Every byte belongs to
/// a token (ident/int/punct/whitespace), so the lexer never errors.
fn synthetic_source() -> String {
  const TARGET: usize = 128 * 1024;
  let mut s = String::with_capacity(TARGET + 64);
  let mut i = 0u32;
  while s.len() < TARGET {
    let a = i;
    let m = i.wrapping_mul(2654435761) % 100_000;
    let b = i % 4093;
    let _ = writeln!(s, "var{a} = {m} + val{b} ;");
    i = i.wrapping_add(1);
  }
  s
}

fn bench(c: &mut Criterion) {
  let src = synthetic_source();

  let mut group = c.benchmark_group("input/scan");
  group.throughput(Throughput::Bytes(src.len() as u64));
  group.measurement_time(Duration::from_secs(3));
  group.warm_up_time(Duration::from_secs(1));

  group.bench_function("next_drain", |b| {
    b.iter(|| {
      let n = Parser::new()
        .apply(next_drain)
        .parse_str(black_box(src.as_str()))
        .unwrap();
      black_box(n)
    })
  });

  group.bench_function("skip_trivia_next", |b| {
    b.iter(|| {
      let n = Parser::new()
        .apply(skip_trivia_next)
        .parse_str(black_box(src.as_str()))
        .unwrap();
      black_box(n)
    })
  });

  group.bench_function("try_expect_hits", |b| {
    b.iter(|| {
      let n = Parser::new()
        .apply(try_expect_hits)
        .parse_str(black_box(src.as_str()))
        .unwrap();
      black_box(n)
    })
  });

  group.bench_function("try_expect_misses", |b| {
    b.iter(|| {
      let n = Parser::new()
        .apply(try_expect_misses)
        .parse_str(black_box(src.as_str()))
        .unwrap();
      black_box(n)
    })
  });

  group.bench_function("peek1_then_next", |b| {
    b.iter(|| {
      let n = Parser::new()
        .apply(peek1_then_next)
        .parse_str(black_box(src.as_str()))
        .unwrap();
      black_box(n)
    })
  });

  group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
