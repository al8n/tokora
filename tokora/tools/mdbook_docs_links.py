#!/usr/bin/env python3
"""mdbook preprocessor: turn the guide's rustdoc intra-doc links into links the book can follow.

The guide chapters are the single source for both rustdoc and this book, so their API links are
written as rustdoc intra-doc links (`[`InputRef`](crate::InputRef)`). rustdoc resolves and
validates those under `-D warnings`; mdbook would render them as dead links. This rewrites them:

  crate::guide::chNN_x, super::chNN_x  ->  chNN_x.md            (sibling chapter, in-book)
  everything else                      ->  a docs.rs URL

Every other target must appear in DOCS_RS_MAP below. An unmapped `crate::` target FAILS THE BUILD
rather than shipping a dead link -- so a new link in the guide forces a new map entry here.

The map is not guesswork: each value was resolved against real `cargo doc` output and accepted
only if it matched an href rustdoc itself emitted for that link, which is why re-exports point at
their canonical module (`crate::InputRef` -> `input/struct.InputRef.html`, *not*
`struct.InputRef.html`) and required trait methods use `#tymethod.` rather than `#method.`.

To refresh after an API move: RUSTDOCFLAGS= cargo doc -p tokora --all-features --no-deps, then
compare the hrefs under target/doc/tokora/guide/*/index.html with the values below.
"""

import json
import re
import sys

DOCS_RS = "https://docs.rs/tokora/latest/tokora/"

# rustdoc-verified: crate:: path -> page (+anchor) under https://docs.rs/tokora/latest/tokora/.
# An absolute http(s) value is used verbatim.
DOCS_RS_MAP = {
  "crate::Accumulator::collect": "trait.Accumulator.html#method.collect",
  "crate::Balance": "input/enum.Balance.html",
  "crate::Branch": "struct.Branch.html",
  "crate::Complete": "input/struct.Complete.html",
  "crate::Completeness": "input/trait.Completeness.html",
  "crate::DelimClass": "input/trait.DelimClass.html",
  "crate::DropPolicy": "input/trait.DropPolicy.html",
  "crate::Emitter": "emitter/trait.Emitter.html",
  "crate::Emitter::emit_error": "emitter/trait.Emitter.html#tymethod.emit_error",
  "crate::Emitter::emit_skipped_region": "emitter/trait.Emitter.html#method.emit_skipped_region",
  "crate::Hole": "input/struct.Hole.html",
  "crate::Hole::skipped": "input/struct.Hole.html#method.skipped",
  "crate::Hole::span": "input/struct.Hole.html#method.span",
  "crate::InputRef": "input/struct.InputRef.html",
  "crate::InputRef::attempt": "input/struct.InputRef.html#method.attempt",
  "crate::InputRef::begin": "input/struct.InputRef.html#method.begin",
  "crate::InputRef::begin_point": "input/struct.InputRef.html#method.begin_point",
  "crate::InputRef::begin_stacked": "input/struct.InputRef.html#method.begin_stacked",
  "crate::InputRef::begin_with": "input/struct.InputRef.html#method.begin_with",
  "crate::InputRef::commit_point": "input/struct.InputRef.html#method.commit_point",
  "crate::InputRef::next": "input/struct.InputRef.html#method.next",
  "crate::InputRef::points": "input/struct.InputRef.html#method.points",
  "crate::InputRef::pratt": "input/struct.InputRef.html#method.pratt",
  "crate::InputRef::pratt_with_min_precedence": "input/struct.InputRef.html#method.pratt_with_min_precedence",
  "crate::InputRef::rollback_point": "input/struct.InputRef.html#method.rollback_point",
  "crate::InputRef::set_final": "input/struct.InputRef.html#method.set_final",
  "crate::InputRef::slice": "input/struct.InputRef.html#method.slice",
  "crate::InputRef::sync_balanced": "input/struct.InputRef.html#method.sync_balanced",
  "crate::InputRef::try_attempt": "input/struct.InputRef.html#method.try_attempt",
  "crate::InputRef::try_expect": "input/struct.InputRef.html#method.try_expect",
  "crate::Lexer": "lexer/trait.Lexer.html",
  "crate::Lexer#the-lexer-contract": "lexer/trait.Lexer.html#the-lexer-contract",
  "crate::Lexer::bump": "lexer/trait.Lexer.html#tymethod.bump",
  "crate::Lexer::lex": "lexer/trait.Lexer.html#tymethod.lex",
  "crate::Parse": "parser/trait.Parse.html",
  "crate::Parse::parse": "parser/trait.Parse.html#method.parse",
  "crate::Parse::parse_bstr": "parser/trait.Parse.html#method.parse_bstr",
  "crate::Parse::parse_bytes": "parser/trait.Parse.html#method.parse_bytes",
  "crate::Parse::parse_hipstr": "parser/trait.Parse.html#method.parse_hipstr",
  "crate::Parse::parse_slice": "parser/trait.Parse.html#method.parse_slice",
  "crate::Parse::parse_str": "parser/trait.Parse.html#method.parse_str",
  "crate::Parse::parse_with_state": "parser/trait.Parse.html#tymethod.parse_with_state",
  "crate::ParseChoice::dispatch_on_kind": "trait.ParseChoice.html#method.dispatch_on_kind",
  "crate::ParseChoice::peek_then_choice": "trait.ParseChoice.html#method.peek_then_choice",
  "crate::ParseChoice::peek_then_try_choice": "trait.ParseChoice.html#method.peek_then_try_choice",
  "crate::ParseInput": "trait.ParseInput.html",
  "crate::ParseInput::ignored": "trait.ParseInput.html#method.ignored",
  "crate::ParseInput::ignore_then": "trait.ParseInput.html#method.ignore_then",
  "crate::ParseInput::located": "trait.ParseInput.html#method.located",
  "crate::ParseInput::map": "trait.ParseInput.html#method.map",
  "crate::ParseInput::padded": "trait.ParseInput.html#method.padded",
  "crate::ParseInput::repeated_while": "trait.ParseInput.html#method.repeated_while",
  "crate::ParseInput::separated_by_comma_while": "trait.ParseInput.html#method.separated_by_comma_while",
  "crate::ParseInput::separated_while": "trait.ParseInput.html#method.separated_while",
  "crate::ParseInput::skip_then_retry": "trait.ParseInput.html#method.skip_then_retry",
  "crate::ParseInput::sliced": "trait.ParseInput.html#method.sliced",
  "crate::ParseInput::spanned": "trait.ParseInput.html#method.spanned",
  "crate::ParseInput::then": "trait.ParseInput.html#method.then",
  "crate::ParseInput::then_ignore": "trait.ParseInput.html#method.then_ignore",
  "crate::ParseTokenChoice": "trait.ParseTokenChoice.html",
  "crate::ParseTokenChoice::fused_dispatch_on_kind": "trait.ParseTokenChoice.html#method.fused_dispatch_on_kind",
  "crate::Parser::apply": "parser/struct.Parser.html#method.apply",
  "crate::Parser::new": "parser/struct.Parser.html#method.new",
  "crate::Partial": "input/struct.Partial.html",
  "crate::SavepointId": "input/struct.SavepointId.html",
  "crate::StackedTransaction": "input/struct.StackedTransaction.html",
  "crate::StackedTransaction::release": "input/struct.StackedTransaction.html#method.release",
  "crate::StackedTransaction::rollback_to": "input/struct.StackedTransaction.html#method.rollback_to",
  "crate::StackedTransaction::savepoint": "input/struct.StackedTransaction.html#method.savepoint",
  "crate::State": "state/trait.State.html",
  "crate::Token": "token/trait.Token.html",
  "crate::Token::Kind": "token/trait.Token.html#associatedtype.Kind",
  "crate::Token::is_trivia": "token/trait.Token.html#tymethod.is_trivia",
  "crate::Transaction": "input/struct.Transaction.html",
  "crate::Transaction::commit": "input/struct.Transaction.html#method.commit",
  "crate::Transaction::rollback": "input/struct.Transaction.html#method.rollback",
  "crate::TryParseInput": "try_parse_input/trait.TryParseInput.html",
  "crate::TryParseInput::repeated": "try_parse_input/trait.TryParseInput.html#method.repeated",
  "crate::TryParseInput::separated": "try_parse_input/trait.TryParseInput.html#method.separated",
  "crate::TryParseInput::separated_by_comma": "try_parse_input/trait.TryParseInput.html#method.separated_by_comma",
  "crate::cache::Peeked": "cache/type.Peeked.html",
  "crate::conformance": "conformance/index.html",
  "crate::conformance::Harness": "conformance/struct.Harness.html",
  "crate::conformance::Harness::lossless": "conformance/struct.Harness.html#method.lossless",
  "crate::conformance::Harness::new": "conformance/struct.Harness.html#method.new",
  "crate::conformance::Harness::over": "conformance/struct.Harness.html#method.over",
  "crate::conformance::Harness::run": "conformance/struct.Harness.html#method.run",
  "crate::conformance::Harness::run_partial": "conformance/struct.Harness.html#method.run_partial",
  "crate::cst": "cst/index.html",
  "crate::cst::Element": "cst/trait.Element.html",
  "crate::cst::Node": "cst/trait.Node.html",
  "crate::cst::Token": "cst/trait.Token.html",
  "crate::cst::SyntaxTreeBuilder": "cst/struct.SyntaxTreeBuilder.html",
  "crate::cst::SyntaxTreeBuilder::checkpoint": "cst/struct.SyntaxTreeBuilder.html#method.checkpoint",
  "crate::cst::SyntaxTreeBuilder::finish": "cst/struct.SyntaxTreeBuilder.html#method.finish",
  "crate::cst::SyntaxTreeBuilder::finish_node": "cst/struct.SyntaxTreeBuilder.html#method.finish_node",
  "crate::cst::SyntaxTreeBuilder::new": "cst/struct.SyntaxTreeBuilder.html#method.new",
  "crate::cst::SyntaxTreeBuilder::start_node": "cst/struct.SyntaxTreeBuilder.html#method.start_node",
  "crate::cst::SyntaxTreeBuilder::start_node_at": "cst/struct.SyntaxTreeBuilder.html#method.start_node_at",
  "crate::cst::SyntaxTreeBuilder::token": "cst/struct.SyntaxTreeBuilder.html#method.token",
  "crate::cst::cast": "cst/cast/index.html",
  "crate::container::Container": "container/trait.Container.html",
  "crate::emitter": "emitter/index.html",
  "crate::emitter::Diagnostic": "emitter/struct.Diagnostic.html",
  "crate::emitter::DiagnosticKind": "emitter/enum.DiagnosticKind.html",
  "crate::emitter::Fatal": "emitter/struct.Fatal.html",
  "crate::emitter::FromEmitterError": "emitter/trait.FromEmitterError.html",
  "crate::emitter::Ignored": "emitter/type.Ignored.html",
  "crate::emitter::Severity": "emitter/enum.Severity.html",
  "crate::emitter::Silent": "emitter/struct.Silent.html",
  "crate::emitter::Verbose": "emitter/struct.Verbose.html",
  "crate::emitter::Verbose::diagnostics": "emitter/struct.Verbose.html#method.diagnostics",
  "crate::emitter::Verbose::errors": "emitter/struct.Verbose.html#method.errors",
  "crate::emitter::Verbose::labels": "emitter/struct.Verbose.html#method.labels",
  "crate::emitter::Verbose::skipped_regions": "emitter/struct.Verbose.html#method.skipped_regions",
  "crate::emitter::Verbose::warnings": "emitter/struct.Verbose.html#method.warnings",
  "crate::error::Incomplete": "error/struct.Incomplete.html",
  "crate::error::MaybeIncomplete::is_incomplete": "error/trait.MaybeIncomplete.html#method.is_incomplete",
  "crate::error::UnexpectedEnd": "error/struct.UnexpectedEnd.html",
  "crate::error::UnexpectedEot": "error/type.UnexpectedEot.html",
  "crate::error::syntax::TooFew": "error/syntax/struct.TooFew.html",
  "crate::error::token::UnexpectedToken": "error/token/struct.UnexpectedToken.html",
  "crate::error::token::UnexpectedTokenOf": "error/token/type.UnexpectedTokenOf.html",
  "crate::fuzz": "fuzz/index.html",
  "crate::fuzz::Case": "fuzz/struct.Case.html",
  "crate::fuzz::run_case": "fuzz/fn.run_case.html",
  "crate::fuzz::run_seeds": "fuzz/fn.run_seeds.html",
  "crate::input": "input/index.html",
  "crate::input#the-sans-io-resumption-loop": "input/index.html#the-sans-io-resumption-loop",
  "crate::labelled": "parser/fn.labelled.html",
  "crate::lexer::LogosLexer": "lexer/struct.LogosLexer.html",
  # `pub use logos_0_16 as logos;` is an extern-crate re-export: rustdoc emits no page for it, so
  # point at the crate the prose actually means.
  "crate::logos": "https://docs.rs/logos/latest/logos/",
  "crate::parse_partial": "input/fn.parse_partial.html",
  "crate::parser": "parser/index.html",
  "crate::parser::Action": "parser/enum.Action.html",
  "crate::parser::DispatchOnKind": "parser/struct.DispatchOnKind.html",
  "crate::parser::DispatchOnKind#performance-keep-token-kind-discriminants-dense": "parser/struct.DispatchOnKind.html#performance-keep-token-kind-discriminants-dense",
  "crate::parser::FusedDispatchOnKind": "parser/struct.FusedDispatchOnKind.html",
  "crate::parser::Parser::new": "parser/struct.Parser.html#method.new",
  "crate::parser::PrattInfix": "parser/enum.PrattInfix.html",
  "crate::parser::PrattLHS": "parser/enum.PrattLHS.html",
  "crate::parser::PrattPower": "parser/trait.PrattPower.html",
  "crate::parser::PrattPower::next": "parser/trait.PrattPower.html#tymethod.next",
  "crate::parser::PrattPower::prev": "parser/trait.PrattPower.html#tymethod.prev",
  "crate::parser::PrattRHS": "parser/enum.PrattRHS.html",
  "crate::parser::Precedenced": "parser/struct.Precedenced.html",
  "crate::parser::Recover": "parser/struct.Recover.html",
  "crate::parser::Separated": "parser/struct.Separated.html",
  "crate::parser::expect": "parser/fn.expect.html",
  "crate::parser::pratt": "parser/fn.pratt.html",
  "crate::parser::pratt_of": "parser/fn.pratt_of.html",
  "crate::punct": "punct/index.html",
  "crate::punct::Brace": "punct/struct.Brace.html",
  "crate::punct::Bracket": "punct/struct.Bracket.html",
  "crate::punct::Colon": "punct/struct.Colon.html",
  "crate::punct::Comma": "punct/struct.Comma.html",
  "crate::token::PrattToken": "token/trait.PrattToken.html",
  "crate::token::PunctuatorToken": "token/trait.PunctuatorToken.html",
  "crate::traced": "fn.traced.html",
  "crate::try_parse_input": "try_parse_input/index.html",
  "crate::try_parse_input::ParseAttempt": "try_parse_input/enum.ParseAttempt.html",
  "crate::utils::Expected": "utils/enum.Expected.html",
}

# `[text](crate::path)` / `[text](super::path)` -- the two intra-doc link spellings the guide uses.
LINK = re.compile(r"\[([^\]]*)\]\((crate::[^)\s]+|super::[^)\s]+)\)")
# A guide chapter, addressed either absolutely or from a sibling chapter's own module scope.
CHAPTER = re.compile(r"^(?:crate::guide::|super::)(ch\d{2}_\w+)$")

errors = set()


def rewrite(target: str) -> str:
  chapter = CHAPTER.match(target)
  if chapter:
    return f"{chapter.group(1)}.md"
  page = DOCS_RS_MAP.get(target)
  if page is None:
    errors.add(target)
    return target
  return page if page.startswith("http") else DOCS_RS + page


def convert(content: str) -> str:
  out = LINK.sub(lambda m: f"[{m.group(1)}]({rewrite(m.group(2))})", content)
  # Belt and braces: nothing of the intra-doc form may survive into the rendered book.
  errors.update(re.findall(r"\]\(((?:crate|super)::[^)\s]+)\)", out))
  return out


def walk(items):
  for item in items:
    chapter = item.get("Chapter") if isinstance(item, dict) else None
    if chapter is None:
      continue  # Separator / PartTitle
    chapter["content"] = convert(chapter["content"])
    walk(chapter.get("sub_items") or chapter.get("items") or [])


def main() -> int:
  if len(sys.argv) > 2 and sys.argv[1] == "supports":
    return 0  # every renderer

  _context, book = json.load(sys.stdin)
  # mdbook >= 0.5 calls the top-level list `items`; 0.4 called it `sections`.
  items = book.get("items")
  if items is None:
    items = book.get("sections")
  if items is None:
    print(f"mdbook_docs_links: unrecognised book shape: {sorted(book)}", file=sys.stderr)
    return 1
  walk(items)

  if errors:
    print(
      "mdbook_docs_links: the guide links to items with no entry in DOCS_RS_MAP, which would "
      "render as dead links in the book. Add them to tokora/tools/mdbook_docs_links.py:",
      file=sys.stderr,
    )
    for target in sorted(set(errors)):
      print(f"  {target}", file=sys.stderr)
    return 1

  json.dump(book, sys.stdout)
  return 0


if __name__ == "__main__":
  sys.exit(main())
