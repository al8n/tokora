#!/usr/bin/env python3
"""mdbook preprocessor: turn the guide's rustdoc intra-doc links into links the book can follow.

The guide chapters are the single source for both rustdoc and this book, so their API links are
written as rustdoc intra-doc links (`[`InputRef`](crate::InputRef)`). rustdoc resolves and
validates those under `-D warnings`; mdbook would render them as dead links. This rewrites them:

  crate::guide::chNN_x, super::chNN_x  ->  chNN_x.md            (sibling chapter, in-book)
  ... the same with `#heading-anchor`  ->  chNN_x.md#heading-anchor
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
  "crate::Accumulator::collect": "parse/trait.Accumulator.html#method.collect",
  "crate::Accumulator::collect_with": "parse/trait.Accumulator.html#method.collect_with",
  "crate::input::Balance": "input/enum.Balance.html",
  "crate::parse::Branch": "parse/struct.Branch.html",
  "crate::cache::Cache": "cache/trait.Cache.html",
  "crate::parse::Check": "parse/trait.Check.html",
  "crate::input::Complete": "input/struct.Complete.html",
  "crate::input::Completeness": "input/trait.Completeness.html",
  "crate::parse::Decision": "parse/trait.Decision.html",
  "crate::input::DelimClass": "input/trait.DelimClass.html",
  "crate::input::DropPolicy": "input/trait.DropPolicy.html",
  "crate::Emitter": "emitter/trait.Emitter.html",
  "crate::Emitter::bound_source": "emitter/trait.Emitter.html#method.bound_source",
  "crate::Emitter::checkpoint": "emitter/trait.Emitter.html#method.checkpoint",
  "crate::Emitter::commit_lexer_error": "emitter/trait.Emitter.html#method.commit_lexer_error",
  "crate::Emitter::commit_token": "emitter/trait.Emitter.html#method.commit_token",
  "crate::Emitter::emit_error": "emitter/trait.Emitter.html#tymethod.emit_error",
  "crate::Emitter::emit_lexer_error": "emitter/trait.Emitter.html#tymethod.emit_lexer_error",
  "crate::Emitter::emit_skipped_region": "emitter/trait.Emitter.html#method.emit_skipped_region",
  "crate::Emitter::emit_warning": "emitter/trait.Emitter.html#method.emit_warning",
  "crate::Emitter::exit_label": "emitter/trait.Emitter.html#method.exit_label",
  "crate::Emitter::release": "emitter/trait.Emitter.html#method.release",
  "crate::Emitter::rewind": "emitter/trait.Emitter.html#tymethod.rewind",
  # `EmitterView` is re-exported from the crate root AND from `tokora::emitter`; rustdoc's
  # canonical page for it is the latter, so the value follows the same rule the `InputRef`
  # entries do — the module the item is declared under, not the root re-export.
  "crate::EmitterView": "emitter/struct.EmitterView.html",
  "crate::EmitterView::emit_lexer_error": "emitter/struct.EmitterView.html#method.emit_lexer_error",
  "crate::ErrorOf": "parse/type.ErrorOf.html",
  "crate::FatalContext": "parse/type.FatalContext.html",
  "crate::input::Hole": "input/struct.Hole.html",
  "crate::input::Hole::skipped": "input/struct.Hole.html#method.skipped",
  "crate::input::Hole::span": "input/struct.Hole.html#method.span",
  "crate::InputRef": "input/struct.InputRef.html",
  "crate::InputRef::attempt": "input/struct.InputRef.html#method.attempt",
  "crate::InputRef::begin": "input/struct.InputRef.html#method.begin",
  "crate::InputRef::begin_point": "input/struct.InputRef.html#method.begin_point",
  "crate::InputRef::begin_stacked": "input/struct.InputRef.html#method.begin_stacked",
  "crate::InputRef::begin_with": "input/struct.InputRef.html#method.begin_with",
  "crate::InputRef::commit_point": "input/struct.InputRef.html#method.commit_point",
  "crate::InputRef::cursor": "input/struct.InputRef.html#method.cursor",
  "crate::InputRef::descend": "input/struct.InputRef.html#method.descend",
  "crate::InputRef::descending": "input/struct.InputRef.html#method.descending",
  "crate::InputRef::emit_error": "input/struct.InputRef.html#method.emit_error",
  "crate::InputRef::emit_lexer_error": "input/struct.InputRef.html#method.emit_lexer_error",
  "crate::InputRef::emitter_ref": "input/struct.InputRef.html#method.emitter_ref",
  "crate::InputRef::next": "input/struct.InputRef.html#method.next",
  "crate::InputRef::peek": "input/struct.InputRef.html#method.peek",
  "crate::InputRef::points": "input/struct.InputRef.html#method.points",
  "crate::InputRef::pratt": "input/struct.InputRef.html#method.pratt",
  "crate::InputRef::pratt_with_min_precedence": "input/struct.InputRef.html#method.pratt_with_min_precedence",
  "crate::InputRef::rollback_point": "input/struct.InputRef.html#method.rollback_point",
  "crate::InputRef::set_final": "input/struct.InputRef.html#method.set_final",
  "crate::InputRef::skip_while": "input/struct.InputRef.html#method.skip_while",
  "crate::InputRef::slice": "input/struct.InputRef.html#method.slice",
  "crate::InputRef::span": "input/struct.InputRef.html#method.span",
  "crate::InputRef::sync_balanced": "input/struct.InputRef.html#method.sync_balanced",
  "crate::InputRef::try_attempt": "input/struct.InputRef.html#method.try_attempt",
  "crate::InputRef::try_expect": "input/struct.InputRef.html#method.try_expect",
  "crate::InputRef::try_expect_take": "input/struct.InputRef.html#method.try_expect_take",
  "crate::InputRef::try_expect_take_or_stop": "input/struct.InputRef.html#method.try_expect_take_or_stop",
  "crate::Lexer": "lexer/trait.Lexer.html",
  "crate::Lexer#state-faithfulness-and-cheapness": "lexer/trait.Lexer.html#state-faithfulness-and-cheapness",
  "crate::Lexer#the-lexer-contract": "lexer/trait.Lexer.html#the-lexer-contract",
  "crate::Lexer::State": "lexer/trait.Lexer.html#associatedtype.State",
  "crate::Lexer::bump": "lexer/trait.Lexer.html#tymethod.bump",
  "crate::Lexer::check": "lexer/trait.Lexer.html#tymethod.check",
  "crate::Lexer::lex": "lexer/trait.Lexer.html#tymethod.lex",
  "crate::Lexer::new": "lexer/trait.Lexer.html#tymethod.new",
  "crate::Lexer::read_frontier": "lexer/trait.Lexer.html#tymethod.read_frontier",
  "crate::Lexer::slice": "lexer/trait.Lexer.html#tymethod.slice",
  "crate::Lexer::span": "lexer/trait.Lexer.html#tymethod.span",
  "crate::span::Located": "span/struct.Located.html",
  "crate::Parse": "parse/trait.Parse.html",
  "crate::Parse::parse": "parse/trait.Parse.html#method.parse",
  "crate::Parse::parse_bstr": "parse/trait.Parse.html#method.parse_bstr",
  "crate::Parse::parse_bytes": "parse/trait.Parse.html#method.parse_bytes",
  "crate::Parse::parse_hipstr": "parse/trait.Parse.html#method.parse_hipstr",
  "crate::Parse::parse_slice": "parse/trait.Parse.html#method.parse_slice",
  "crate::Parse::parse_str": "parse/trait.Parse.html#method.parse_str",
  "crate::Parse::parse_with_state": "parse/trait.Parse.html#tymethod.parse_with_state",
  "crate::ParseAttempt": "parse/enum.ParseAttempt.html",
  "crate::ParseAttempt::Decline": "parse/enum.ParseAttempt.html#variant.Decline",
  "crate::ParseAttempt::into_option": "parse/enum.ParseAttempt.html#method.into_option",
  "crate::ParseChoice": "parse/trait.ParseChoice.html",
  "crate::ParseChoice::dispatch_on_kind": "parse/trait.ParseChoice.html#method.dispatch_on_kind",
  "crate::ParseChoice::peek_then_choice": "parse/trait.ParseChoice.html#method.peek_then_choice",
  "crate::ParseChoice::peek_then_try_choice": "parse/trait.ParseChoice.html#method.peek_then_try_choice",
  "crate::ParseContext": "parse/trait.ParseContext.html",
  "crate::ComposableParseContext": "parse/trait.ComposableParseContext.html",
  "crate::ParseInput": "parse/trait.ParseInput.html",
  "crate::ParseInput::and_then": "parse/trait.ParseInput.html#method.and_then",
  "crate::ParseInput::by_ref": "parse/trait.ParseInput.html#method.by_ref",
  "crate::ParseInput::filter": "parse/trait.ParseInput.html#method.filter",
  "crate::ParseInput::filter_map": "parse/trait.ParseInput.html#method.filter_map",
  "crate::ParseInput::fold_while": "parse/trait.ParseInput.html#method.fold_while",
  "crate::ParseInput::ignore_then": "parse/trait.ParseInput.html#method.ignore_then",
  "crate::ParseInput::ignored": "parse/trait.ParseInput.html#method.ignored",
  "crate::ParseInput::inplace_recover": "parse/trait.ParseInput.html#method.inplace_recover",
  "crate::ParseInput::list_until": "parse/trait.ParseInput.html#method.list_until",
  "crate::ParseInput::located": "parse/trait.ParseInput.html#method.located",
  "crate::ParseInput::map": "parse/trait.ParseInput.html#method.map",
  "crate::ParseInput::padded": "parse/trait.ParseInput.html#method.padded",
  "crate::ParseInput::parse_input": "parse/trait.ParseInput.html#tymethod.parse_input",
  "crate::ParseInput::peek_then": "parse/trait.ParseInput.html#method.peek_then",
  "crate::ParseInput::peek_then_head": "parse/trait.ParseInput.html#method.peek_then_head",
  "crate::ParseInput::recover": "parse/trait.ParseInput.html#method.recover",
  "crate::ParseInput::repeated_while": "parse/trait.ParseInput.html#method.repeated_while",
  "crate::ParseInput::separated1_by": "parse/trait.ParseInput.html#method.separated1_by",
  "crate::ParseInput::separated_by_comma_while": "parse/trait.ParseInput.html#method.separated_by_comma_while",
  "crate::ParseInput::separated_while": "parse/trait.ParseInput.html#method.separated_while",
  "crate::ParseInput::skip_then_retry": "parse/trait.ParseInput.html#method.skip_then_retry",
  "crate::ParseInput::sliced": "parse/trait.ParseInput.html#method.sliced",
  "crate::ParseInput::spanned": "parse/trait.ParseInput.html#method.spanned",
  "crate::ParseInput::then": "parse/trait.ParseInput.html#method.then",
  "crate::ParseInput::then_ignore": "parse/trait.ParseInput.html#method.then_ignore",
  "crate::ParseInput::then_value": "parse/trait.ParseInput.html#method.then_value",
  "crate::ParseInput::validate": "parse/trait.ParseInput.html#method.validate",
  "crate::parse::ParseInputUnwrapExt::unwrap": "parse/trait.ParseInputUnwrapExt.html#method.unwrap",
  "crate::parse::ParseState": "parse/struct.ParseState.html",
  "crate::ParseTokenChoice": "parse/trait.ParseTokenChoice.html",
  "crate::ParseTokenChoice::fused_dispatch_on_kind": "parse/trait.ParseTokenChoice.html#method.fused_dispatch_on_kind",
  "crate::Parser": "parse/struct.Parser.html",
  "crate::Parser::apply": "parse/struct.Parser.html#method.apply",
  "crate::Parser::new": "parse/struct.Parser.html#method.new",
  "crate::Parser::with_context": "parse/struct.Parser.html#method.with_context",
  "crate::Parser::with_parser": "parse/struct.Parser.html#method.with_parser",
  "crate::Parser::with_parser_and_context": "parse/struct.Parser.html#method.with_parser_and_context",
  "crate::ParserContext::with_recursion_limiter": "parse/struct.ParserContext.html#method.with_recursion_limiter",
  "crate::input::Partial": "input/struct.Partial.html",
  "crate::parse::PolicyParseContext": "parse/trait.PolicyParseContext.html",
  "crate::ReadFrontier::SpanEnd": "lexer/enum.ReadFrontier.html#variant.SpanEnd",
  "crate::ReadFrontier::Unbounded": "lexer/enum.ReadFrontier.html#variant.Unbounded",
  "crate::ScanLookahead::Unbounded": "lexer/enum.ScanLookahead.html#variant.Unbounded",
  "crate::input::SessionPointId": "input/struct.SessionPointId.html",
  "crate::input::SurfaceIncomplete": "input/trait.SurfaceIncomplete.html",
  "crate::parse::Require": "parse/trait.Require.html",
  "crate::input::SavepointId": "input/struct.SavepointId.html",
  "crate::SimpleSpan": "span/struct.SimpleSpan.html",
  "crate::Slice": "slice/trait.Slice.html",
  "crate::lexer::SliceOf": "lexer/type.SliceOf.html",
  "crate::Source": "source/trait.Source.html",
  "crate::Span": "span/trait.Span.html",
  "crate::input::StackedTransaction": "input/struct.StackedTransaction.html",
  "crate::input::StackedTransaction::release": "input/struct.StackedTransaction.html#method.release",
  "crate::input::StackedTransaction::rollback_to": "input/struct.StackedTransaction.html#method.rollback_to",
  "crate::input::StackedTransaction::savepoint": "input/struct.StackedTransaction.html#method.savepoint",
  "crate::state::State": "state/trait.State.html",
  "crate::state::State::check": "state/trait.State.html#tymethod.check",
  "crate::Token": "token/trait.Token.html",
  "crate::Token::Kind": "token/trait.Token.html#associatedtype.Kind",
  "crate::Token::SCAN_LOOKAHEAD": "token/trait.Token.html#associatedconstant.SCAN_LOOKAHEAD",
  "crate::Token::SURFACES_TRIVIA": "token/trait.Token.html#associatedconstant.SURFACES_TRIVIA",
  "crate::Token::is_trivia": "token/trait.Token.html#tymethod.is_trivia",
  "crate::input::Transaction": "input/struct.Transaction.html",
  "crate::input::Transaction::commit": "input/struct.Transaction.html#method.commit",
  "crate::input::Transaction::rollback": "input/struct.Transaction.html#method.rollback",
  "crate::TryParseInput": "parse/trait.TryParseInput.html",
  "crate::TryParseInput::accepted": "parse/trait.TryParseInput.html#method.accepted",
  "crate::TryParseInput::fold": "parse/trait.TryParseInput.html#method.fold",
  "crate::TryParseInput::repeated": "parse/trait.TryParseInput.html#method.repeated",
  "crate::TryParseInput::rfold": "parse/trait.TryParseInput.html#method.rfold",
  "crate::TryParseInput::separated": "parse/trait.TryParseInput.html#method.separated",
  "crate::TryParseInput::separated_by_comma": "parse/trait.TryParseInput.html#method.separated_by_comma",
  "crate::TryParseInput::try_fold": "parse/trait.TryParseInput.html#method.try_fold",
  "crate::parse::Window": "parse/trait.Window.html",
  "crate::cache::Peeked": "cache/type.Peeked.html",
  "crate::conformance": "conformance/index.html",
  "crate::conformance::Harness": "conformance/struct.Harness.html",
  "crate::conformance::Harness::lossless": "conformance/struct.Harness.html#method.lossless",
  "crate::conformance::Harness::new": "conformance/struct.Harness.html#method.new",
  "crate::conformance::Harness::over": "conformance/struct.Harness.html#method.over",
  "crate::conformance::Harness::run": "conformance/struct.Harness.html#method.run",
  "crate::conformance::Harness::run_partial": "conformance/struct.Harness.html#method.run_partial",
  "crate::container::Container": "container/trait.Container.html",
  "crate::cst": "cst/index.html",
  "crate::cst::CastNode": "cst/trait.CastNode.html",
  "crate::cst::CastNode::cast_node": "cst/trait.CastNode.html#tymethod.cast_node",
  "crate::cst::Cst": "cst/struct.Cst.html",
  "crate::cst::Cst::finish": "cst/struct.Cst.html#method.finish",
  "crate::cst::Cst::finish_partial": "cst/struct.Cst.html#method.finish_partial",
  "crate::cst::Cst::resource_trips": "cst/struct.Cst.html#method.resource_trips",
  "crate::cst::Element": "cst/trait.Element.html",
  "crate::cst::FinishError": "cst/enum.FinishError.html",
  "crate::cst::FinishError::MismatchedFinish": "cst/enum.FinishError.html#variant.MismatchedFinish",
  "crate::cst::FinishError::StaleDemote": "cst/enum.FinishError.html#variant.StaleDemote",
  "crate::cst::FinishError::TooDeep": "cst/enum.FinishError.html#variant.TooDeep",
  "crate::cst::FinishError::UncoveredGap": "cst/enum.FinishError.html#variant.UncoveredGap",
  "crate::cst::FinishError::UnpairedSettle": "cst/enum.FinishError.html#variant.UnpairedSettle",
  "crate::cst::MAX_TREE_DEPTH": "cst/constant.MAX_TREE_DEPTH.html",
  "crate::cst::Node": "cst/trait.Node.html",
  "crate::cst::Node::try_cast_node": "cst/trait.Node.html#tymethod.try_cast_node",
  "crate::cst::NodeChildren": "cst/struct.NodeChildren.html",
  "crate::cst::Sink": "cst/struct.Sink.html",
  "crate::cst::Sink::inner_ref": "cst/struct.Sink.html#method.inner_ref",
  "crate::cst::parse_lossless": "cst/fn.parse_lossless.html",
  "crate::cst::parse_lossless_partial": "cst/fn.parse_lossless_partial.html",
  "crate::cst::SyntaxTreeBuilder": "cst/struct.SyntaxTreeBuilder.html",
  "crate::cst::SyntaxTreeBuilder::checkpoint": "cst/struct.SyntaxTreeBuilder.html#method.checkpoint",
  "crate::cst::SyntaxTreeBuilder::finish": "cst/struct.SyntaxTreeBuilder.html#method.finish",
  "crate::cst::SyntaxTreeBuilder::finish_node": "cst/struct.SyntaxTreeBuilder.html#method.finish_node",
  "crate::cst::SyntaxTreeBuilder::new": "cst/struct.SyntaxTreeBuilder.html#method.new",
  "crate::cst::SyntaxTreeBuilder::start_node": "cst/struct.SyntaxTreeBuilder.html#method.start_node",
  "crate::cst::SyntaxTreeBuilder::start_node_at": "cst/struct.SyntaxTreeBuilder.html#method.start_node_at",
  "crate::cst::SyntaxTreeBuilder::token": "cst/struct.SyntaxTreeBuilder.html#method.token",
  "crate::cst::Token": "cst/trait.Token.html",
  "crate::cst::TriviaPolicy": "cst/enum.TriviaPolicy.html",
  "crate::cst::cast": "cst/cast/index.html",
  "crate::cst::cast::child": "cst/cast/fn.child.html",
  "crate::cst::cast::children": "cst/cast/fn.children.html",
  "crate::cst::SyntaxError": "cst/enum.SyntaxError.html",
  "crate::cst::CompletedMarker": "cst/struct.CompletedMarker.html",
  "crate::cst::EventMark": "cst/struct.EventMark.html",
  "crate::cst::Marker": "cst/struct.Marker.html",
  "crate::cst::TOMBSTONE": "cst/constant.TOMBSTONE.html",
  "crate::delimiter::Delimiter": "delimiter/trait.Delimiter.html",
  "crate::delimiter::Delimiter::KIND": "delimiter/trait.Delimiter.html#associatedconstant.KIND",
  "crate::delimiter::Delimiter::name": "delimiter/trait.Delimiter.html#tymethod.name",
  "crate::delimiter::DelimiterKind": "delimiter/enum.DelimiterKind.html",
  "crate::delimiter::TypedDelimiter": "delimiter/trait.TypedDelimiter.html",
  "crate::emitter": "emitter/index.html",
  "crate::emitter::ComposableEmitter": "emitter/trait.ComposableEmitter.html",
  "crate::emitter::CstEmitter": "emitter/trait.CstEmitter.html",
  # `cst_demote` is the trait's one REQUIRED method, so rustdoc anchors it as `#tymethod.`, not
  # `#method.` like its four defaulted siblings. Re-defaulting it would move the anchor.
  "crate::emitter::CstEmitter::cst_demote": "emitter/trait.CstEmitter.html#tymethod.cst_demote",
  "crate::emitter::CstEmitter::cst_mark": "emitter/trait.CstEmitter.html#method.cst_mark",
  "crate::emitter::CstEmitter::cst_start_at": "emitter/trait.CstEmitter.html#method.cst_start_at",
  "crate::emitter::Diagnostic": "emitter/struct.Diagnostic.html",
  "crate::emitter::DiagnosticKind": "emitter/enum.DiagnosticKind.html",
  "crate::emitter::Diagnostics": "emitter/struct.Diagnostics.html",
  "crate::emitter::Fatal": "emitter/struct.Fatal.html",
  "crate::emitter::FromEmitterError": "emitter/trait.FromEmitterError.html",
  "crate::emitter::FromPrattError": "emitter/trait.FromPrattError.html",
  "crate::emitter::FromTokenErrors": "emitter/trait.FromTokenErrors.html",
  "crate::emitter::FromUnclosed": "emitter/trait.FromUnclosed.html",
  "crate::emitter::FullContainerEmitter": "emitter/trait.FullContainerEmitter.html",
  "crate::emitter::Ignored": "emitter/type.Ignored.html",
  "crate::emitter::MissingLeadingSeparatorEmitter": "emitter/trait.MissingLeadingSeparatorEmitter.html",
  "crate::emitter::MissingTrailingSeparatorEmitter": "emitter/trait.MissingTrailingSeparatorEmitter.html",
  "crate::emitter::PolicyComposableEmitter": "emitter/trait.PolicyComposableEmitter.html",
  "crate::emitter::PrattEmitter": "emitter/trait.PrattEmitter.html",
  "crate::emitter::SeparatedEmitter": "emitter/trait.SeparatedEmitter.html",
  "crate::emitter::Severity": "emitter/enum.Severity.html",
  "crate::emitter::Silent": "emitter/struct.Silent.html",
  "crate::emitter::TooFewEmitter": "emitter/trait.TooFewEmitter.html",
  "crate::emitter::TooManyEmitter": "emitter/trait.TooManyEmitter.html",
  "crate::emitter::UnexpectedLeadingSeparatorEmitter": "emitter/trait.UnexpectedLeadingSeparatorEmitter.html",
  "crate::emitter::UnexpectedTrailingSeparatorEmitter": "emitter/trait.UnexpectedTrailingSeparatorEmitter.html",
  "crate::emitter::UnclosedEmitter": "emitter/trait.UnclosedEmitter.html",
  "crate::emitter::Verbose": "emitter/struct.Verbose.html",
  "crate::emitter::Verbose::diagnostics": "emitter/struct.Verbose.html#method.diagnostics",
  "crate::emitter::Verbose::errors": "emitter/struct.Verbose.html#method.errors",
  "crate::emitter::Verbose::labels": "emitter/struct.Verbose.html#method.labels",
  "crate::emitter::Verbose::skipped_regions": "emitter/struct.Verbose.html#method.skipped_regions",
  "crate::emitter::Verbose::warnings": "emitter/struct.Verbose.html#method.warnings",
  "crate::error::ErrorNode": "error/trait.ErrorNode.html",
  "crate::error::Incomplete": "error/struct.Incomplete.html",
  "crate::error::IncompleteSyntax": "error/struct.IncompleteSyntax.html",
  "crate::error::IncompleteSyntax::new": "error/struct.IncompleteSyntax.html#method.new",
  "crate::error::IncompleteSyntax::push": "error/struct.IncompleteSyntax.html#method.push",
  "crate::error::Invalid": "error/struct.Invalid.html",
  "crate::error::Malformed": "error/struct.Malformed.html",
  "crate::error::MaybeIncomplete": "error/trait.MaybeIncomplete.html",
  "crate::error::MaybeIncomplete::is_incomplete": "error/trait.MaybeIncomplete.html#method.is_incomplete",
  "crate::error::MaybeTerminal": "error/trait.MaybeTerminal.html",
  "crate::error::MaybeTerminal::is_terminal": "error/trait.MaybeTerminal.html#method.is_terminal",
  "crate::error::NonAssociativeChain": "error/struct.NonAssociativeChain.html",
  "crate::error::RecursionLimitReached": "error/struct.RecursionLimitReached.html",
  "crate::error::RecursionLimitReached#the-stop-does-not-travel-in-the-payload-so-a-discarding-sink-cannot-drop-it": "error/struct.RecursionLimitReached.html#the-stop-does-not-travel-in-the-payload-so-a-discarding-sink-cannot-drop-it",
  "crate::error::Unclosed": "error/struct.Unclosed.html",
  "crate::error::Unclosed::kind": "error/struct.Unclosed.html#method.kind",
  "crate::error::Unclosed::name_ref": "error/struct.Unclosed.html#method.name_ref",
  "crate::error::Undelimited": "error/struct.Undelimited.html",
  "crate::error::UnexpectedEnd": "error/struct.UnexpectedEnd.html",
  "crate::error::UnexpectedEnd::eolhs_of": "error/struct.UnexpectedEnd.html#method.eolhs_of",
  "crate::error::UnexpectedEnd::eorhs_of": "error/struct.UnexpectedEnd.html#method.eorhs_of",
  "crate::error::UnexpectedEoLhs": "error/type.UnexpectedEoLhs.html",
  "crate::error::UnexpectedEoRhs": "error/type.UnexpectedEoRhs.html",
  "crate::error::UnexpectedEot": "error/type.UnexpectedEot.html",
  "crate::error::UnknownLexeme": "error/struct.UnknownLexeme.html",
  "crate::error::Unopened": "error/struct.Unopened.html",
  "crate::error::Unterminated": "error/struct.Unterminated.html",
  "crate::error::FullContainer": "error/struct.FullContainer.html",
  "crate::error::MissingSyntax": "error/struct.MissingSyntax.html",
  "crate::error::MissingSyntaxOf": "error/type.MissingSyntaxOf.html",
  "crate::error::TooFew": "error/struct.TooFew.html",
  "crate::error::TooMany": "error/struct.TooMany.html",
  "crate::error::MissingToken": "error/struct.MissingToken.html",
  "crate::error::MissingTokenOf": "error/type.MissingTokenOf.html",
  "crate::error::SeparatedError": "error/struct.SeparatedError.html",
  "crate::error::SeparatedErrorOf": "error/type.SeparatedErrorOf.html",
  "crate::error::UnexpectedToken": "error/struct.UnexpectedToken.html",
  "crate::error::UnexpectedTokenOf": "error/type.UnexpectedTokenOf.html",
  "crate::fuzz": "fuzz/index.html",
  "crate::fuzz::Case": "fuzz/struct.Case.html",
  "crate::fuzz::run_case": "fuzz/fn.run_case.html",
  "crate::fuzz::run_seeds": "fuzz/fn.run_seeds.html",
  "crate::input": "input/index.html",
  "crate::input#the-sans-io-resumption-loop": "input/index.html#the-sans-io-resumption-loop",
  "crate::input::Checkpoint": "input/struct.Checkpoint.html",
  "crate::input::Cursor": "input/struct.Cursor.html",
  "crate::input::Descent": "input/struct.Descent.html",
  "crate::input::InputContext::with_recursion_limiter": "input/struct.InputContext.html#method.with_recursion_limiter",
  "crate::input::PartialSession": "input/struct.PartialSession.html",
  "crate::input::SessionRefusal": "input/enum.SessionRefusal.html",
  "crate::keyword": "macro.keyword.html",
  "crate::combinator::labelled": "combinator/fn.labelled.html",
  "crate::lexer::LogosLexer": "lexer/struct.LogosLexer.html",
  # `pub use logos_0_16 as logos;` is an extern-crate re-export: rustdoc emits no page for it, so
  # point at the crate the prose actually means.
  "crate::logos": "https://docs.rs/logos/latest/logos/",
  "crate::parse_partial": "parse/fn.parse_partial.html",
  "crate::combinator": "combinator/index.html",
  "crate::parse::Action": "parse/enum.Action.html",
  "crate::combinator::Any": "combinator/struct.Any.html",
  "crate::combinator::Any::new": "combinator/struct.Any.html#method.new",
  "crate::combinator::Any::of": "combinator/struct.Any.html#method.of",
  "crate::parse::DelimiterHandler": "parse/trait.DelimiterHandler.html",
  "crate::combinator::DispatchOnKind": "combinator/struct.DispatchOnKind.html",
  "crate::combinator::DispatchOnKind#performance-keep-token-kind-discriminants-dense": "combinator/struct.DispatchOnKind.html#performance-keep-token-kind-discriminants-dense",
  "crate::combinator::Empty": "combinator/struct.Empty.html",
  "crate::combinator::FusedDispatchOnKind": "combinator/struct.FusedDispatchOnKind.html",
  "crate::combinator::InplaceRecover": "combinator/struct.InplaceRecover.html",
  "crate::pratt::NoCst": "pratt/struct.NoCst.html",
  "crate::pratt::ParsePrattLHS": "pratt/trait.ParsePrattLHS.html",
  "crate::pratt::ParsePrattRHS": "pratt/trait.ParsePrattRHS.html",
  "crate::parse::Parser::new": "parse/struct.Parser.html#method.new",
  "crate::pratt::Pratt": "pratt/struct.Pratt.html",
  "crate::pratt::Pratt::with_cst_kinds": "pratt/struct.Pratt.html#method.with_cst_kinds",
  "crate::pratt::PrattInfix": "pratt/enum.PrattInfix.html",
  "crate::pratt::PrattInfix::Left": "pratt/enum.PrattInfix.html#variant.Left",
  "crate::pratt::PrattLHS": "pratt/enum.PrattLHS.html",
  "crate::pratt::PrattLHS::Operand": "pratt/enum.PrattLHS.html#variant.Operand",
  "crate::pratt::PrattLHS::Prefix": "pratt/enum.PrattLHS.html#variant.Prefix",
  "crate::pratt::PrattPower": "pratt/trait.PrattPower.html",
  "crate::pratt::PrattPower::next": "pratt/trait.PrattPower.html#tymethod.next",
  "crate::pratt::PrattPower::prev": "pratt/trait.PrattPower.html#tymethod.prev",
  "crate::pratt::PrattRHS": "pratt/enum.PrattRHS.html",
  "crate::pratt::PrattRHS::End": "pratt/enum.PrattRHS.html#variant.End",
  "crate::pratt::Precedenced": "pratt/struct.Precedenced.html",
  "crate::combinator::Recover": "combinator/struct.Recover.html",
  "crate::combinator::Repeated": "combinator/struct.Repeated.html",
  "crate::combinator::Separated": "combinator/struct.Separated.html",
  "crate::combinator::Separated::allow_leading": "combinator/struct.Separated.html#method.allow_leading",
  "crate::combinator::Separated::allow_trailing": "combinator/struct.Separated.html#method.allow_trailing",
  "crate::combinator::Separated::at_least": "combinator/struct.Separated.html#method.at_least",
  "crate::combinator::Separated::at_most": "combinator/struct.Separated.html#method.at_most",
  "crate::combinator::Separated::bounded": "combinator/struct.Separated.html#method.bounded",
  "crate::combinator::Separated::delimited": "combinator/struct.Separated.html#method.delimited",
  "crate::combinator::Separated::require_leading": "combinator/struct.Separated.html#method.require_leading",
  "crate::combinator::Separated::require_trailing": "combinator/struct.Separated.html#method.require_trailing",
  "crate::combinator::SeparatedWhile": "combinator/struct.SeparatedWhile.html",
  "crate::parse::SeparatorHandler": "parse/trait.SeparatorHandler.html",
  "crate::combinator::Todo": "combinator/struct.Todo.html",
  "crate::pratt::WithCstKinds": "pratt/struct.WithCstKinds.html",
  "crate::combinator::angles": "combinator/fn.angles.html",
  "crate::combinator::braces": "combinator/fn.braces.html",
  "crate::combinator::brackets": "combinator/fn.brackets.html",
  "crate::combinator::delimited": "combinator/fn.delimited.html",
  "crate::combinator::dispatch_take": "combinator/fn.dispatch_take.html",
  "crate::combinator::expect": "combinator/fn.expect.html",
  "crate::combinator::expect_of": "combinator/fn.expect_of.html",
  "crate::combinator::fail": "combinator/fn.fail.html",
  "crate::combinator::list": "combinator/fn.list.html",
  "crate::combinator::list_of": "combinator/fn.list_of.html",
  "crate::combinator::node": "combinator/fn.node.html",
  "crate::combinator::node(": "combinator/fn.node.html",
  "crate::combinator::node_at": "combinator/fn.node_at.html",
  "crate::combinator::node_at(": "combinator/fn.node_at.html",
  "crate::combinator::node_opt": "combinator/fn.node_opt.html",
  "crate::combinator::node_opt(": "combinator/fn.node_opt.html",
  "crate::combinator::opt": "combinator/fn.opt.html",
  "crate::combinator::parens": "combinator/fn.parens.html",
  "crate::pratt::pratt": "pratt/fn.pratt.html",
  "crate::pratt::pratt_of": "pratt/fn.pratt_of.html",
  "crate::combinator::separated1": "combinator/fn.separated1.html",
  "crate::combinator::try_angles": "combinator/fn.try_angles.html",
  "crate::combinator::try_braces": "combinator/fn.try_braces.html",
  "crate::combinator::try_brackets": "combinator/fn.try_brackets.html",
  "crate::combinator::try_delimited": "combinator/fn.try_delimited.html",
  "crate::combinator::try_dispatch_take": "combinator/fn.try_dispatch_take.html",
  "crate::combinator::try_expect": "combinator/fn.try_expect.html",
  "crate::combinator::try_ident_list": "combinator/fn.try_ident_list.html",
  "crate::combinator::try_parens": "combinator/fn.try_parens.html",
  "crate::punct": "punct/index.html",
  "crate::punct::Angle": "punct/struct.Angle.html",
  "crate::punct::Brace": "punct/struct.Brace.html",
  "crate::punct::Bracket": "punct/struct.Bracket.html",
  "crate::punct::Colon": "punct/struct.Colon.html",
  "crate::punct::Comma": "punct/struct.Comma.html",
  "crate::punct::Paren": "punct/struct.Paren.html",
  "crate::punct::Punctuator": "punct/trait.Punctuator.html",
  "crate::punctuator": "macro.punctuator.html",
  "crate::select": "macro.select.html",
  "crate::slice::Sliced": "slice/struct.Sliced.html",
  "crate::span::AsSpan": "span/trait.AsSpan.html",
  "crate::span::IntoSpan": "span/trait.IntoSpan.html",
  "crate::span::Spanned": "span/struct.Spanned.html",
  "crate::state::RecursionLimiter": "state/struct.RecursionLimiter.html",
  "crate::state::RecursionLimiter#default-limit": "state/struct.RecursionLimiter.html#default-limit",
  "crate::state::RecursionLimiter::PARSE_DEFAULT_DEPTH": "state/struct.RecursionLimiter.html#associatedconstant.PARSE_DEFAULT_DEPTH",
  "crate::state::RecursionLimiter::SEGMENTED_PRATT_DEPTH": "state/struct.RecursionLimiter.html#associatedconstant.SEGMENTED_PRATT_DEPTH",
  "crate::state::RecursionLimiter::unlimited": "state/struct.RecursionLimiter.html#method.unlimited",
  "crate::syntax": "syntax/index.html",
  "crate::syntax::AstNode": "syntax/trait.AstNode.html",
  "crate::syntax::Language": "syntax/trait.Language.html",
  "crate::syntax::Syntax": "syntax/trait.Syntax.html",
  "crate::token::IdentifierToken": "token/trait.IdentifierToken.html",
  "crate::token::KeywordToken": "token/trait.KeywordToken.html",
  "crate::token::KeywordToken::keyword": "token/trait.KeywordToken.html#tymethod.keyword",
  "crate::token::LitToken": "token/trait.LitToken.html",
  "crate::token::PrattToken": "token/trait.PrattToken.html",
  "crate::token::PunctuatorToken": "token/trait.PunctuatorToken.html",
  "crate::combinator::traced": "combinator/fn.traced.html",
  "crate::try_select": "macro.try_select.html",
  "crate::types": "types/index.html",
  "crate::types::Ident": "types/struct.Ident.html",
  "crate::types::IdentList": "types/struct.IdentList.html",
  "crate::types::Keyword": "types/struct.Keyword.html",
  "crate::types::Lit": "types/struct.Lit.html",
  "crate::types::LitBinary": "types/struct.LitBinary.html",
  "crate::types::LitBool": "types/struct.LitBool.html",
  "crate::types::LitByte": "types/struct.LitByte.html",
  "crate::types::LitByteString": "types/struct.LitByteString.html",
  "crate::types::LitChar": "types/struct.LitChar.html",
  "crate::types::LitDecimal": "types/struct.LitDecimal.html",
  "crate::types::LitFalse": "types/struct.LitFalse.html",
  "crate::types::LitFloat": "types/struct.LitFloat.html",
  "crate::types::LitHex": "types/struct.LitHex.html",
  "crate::types::LitHexFloat": "types/struct.LitHexFloat.html",
  "crate::types::LitMultilineString": "types/struct.LitMultilineString.html",
  "crate::types::LitNull": "types/struct.LitNull.html",
  "crate::types::LitOctal": "types/struct.LitOctal.html",
  "crate::types::LitRawString": "types/struct.LitRawString.html",
  "crate::types::LitString": "types/struct.LitString.html",
  "crate::types::LitTrue": "types/struct.LitTrue.html",
  "crate::types::Recoverable": "types/enum.Recoverable.html",
  "crate::utils::CharLen": "utils/trait.CharLen.html",
  "crate::utils::CowStr": "utils/struct.CowStr.html",
  "crate::utils::Delimited": "utils/struct.Delimited.html",
  "crate::utils::EscapedLexeme": "utils/struct.EscapedLexeme.html",
  "crate::utils::Expected": "utils/enum.Expected.html",
  "crate::utils::Expected::one": "utils/enum.Expected.html#method.one",
  "crate::utils::Expected::one_of": "utils/enum.Expected.html#method.one_of",
  "crate::utils::ArrayDeque": "utils/struct.ArrayDeque.html",
  "crate::utils::IntoComponents": "utils/trait.IntoComponents.html",
  "crate::utils::IsAsciiChar": "utils/trait.IsAsciiChar.html",
  "crate::utils::Lexeme": "utils/enum.Lexeme.html",
  "crate::utils::MultiCharEscape": "utils/struct.MultiCharEscape.html",
  "crate::utils::OneOf": "utils/struct.OneOf.html",
  "crate::utils::PositionedChar": "utils/struct.PositionedChar.html",
  "crate::utils::SingleCharEscape": "utils/struct.SingleCharEscape.html",
  # `pub use typenum` is an extern-crate re-export: rustdoc emits no page for it, so point at the
  # crate the prose actually means.
  "crate::utils::typenum": "https://docs.rs/typenum/latest/typenum/",
  "crate::combinator::while_head": "combinator/fn.while_head.html",
  "crate::combinator::while_kind": "combinator/fn.while_kind.html",
  "crate::cst#the-event-vocabulary": "cst/index.html#the-event-vocabulary",
  "crate::utils::DisplayCompact": "utils/trait.DisplayCompact.html",
  "crate::utils::DisplayHuman": "utils/trait.DisplayHuman.html",
  "crate::utils::DisplayPretty": "utils/trait.DisplayPretty.html",
  "crate::utils::DisplaySDL": "utils/trait.DisplaySDL.html",
  "crate::utils::DisplaySyntaxTree": "utils/trait.DisplaySyntaxTree.html",
  "crate::utils::HumanDisplay": "utils/struct.HumanDisplay.html",
}

# `[text](crate::path)` / `[text](super::path)` -- the two intra-doc link spellings the guide uses.
LINK = re.compile(r"\[([^\]]*)\]\((crate::[^)\s]+|super::[^)\s]+)\)")
# A guide chapter, addressed either absolutely or from a sibling chapter's own module scope, and
# optionally a heading inside it (`super::ref_pratt#recursion-limits`). The anchor belongs to the
# chapter link, not to some separate target: without the trailing group the whole spelling falls
# through to DOCS_RS_MAP, and every value there sends the reader OUT to docs.rs -- the wrong
# destination for a link from one chapter of this book to a heading in another.
CHAPTER = re.compile(r"^(?:crate::guide::|super::)((?:ch\d{2}|arch|ref|recipe)_\w+)(#[\w-]+)?$")
# The two prefixes a chapter link is spelled with. No DOCS_RS_MAP key uses either, so a target
# that starts with one and that CHAPTER still rejects is a malformed chapter link -- a different
# failure from a `crate::` item missing its map entry, and one no map entry can fix.
CHAPTER_PREFIX = ("crate::guide::", "super::")

errors = set()


def rewrite(target: str) -> str:
  chapter = CHAPTER.match(target)
  if chapter:
    return f"{chapter.group(1)}.md{chapter.group(2) or ''}"
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
    # Two different failures, and the repair for one is wrong for the other: a chapter link added
    # to DOCS_RS_MAP would resolve, and would send the reader out of the book to docs.rs for a
    # link that should have landed on a heading two chapters over. Report them apart.
    chapters = sorted(t for t in errors if t.startswith(CHAPTER_PREFIX))
    items = sorted(t for t in errors if not t.startswith(CHAPTER_PREFIX))
    if items:
      print(
        "mdbook_docs_links: the guide links to items with no entry in DOCS_RS_MAP, which would "
        "render as dead links in the book. Add them to tokora/tools/mdbook_docs_links.py:",
        file=sys.stderr,
      )
      for target in items:
        print(f"  {target}", file=sys.stderr)
    if chapters:
      print(
        "mdbook_docs_links: the guide spells these as links to another guide chapter, and this "
        "tool could not parse them. Do NOT add them to DOCS_RS_MAP: every entry there points out "
        "to docs.rs, and a chapter link has to stay inside the book. The accepted spelling is "
        "`super::<chapter>` or `crate::guide::<chapter>`, where <chapter> is a chNN_/arch_/ref_/"
        "recipe_ file, optionally followed by `#heading-anchor`. Correct the target, or widen "
        "CHAPTER in tokora/tools/mdbook_docs_links.py:",
        file=sys.stderr,
      )
      for target in chapters:
        print(f"  {target}", file=sys.stderr)
    return 1

  json.dump(book, sys.stdout)
  return 0


if __name__ == "__main__":
  sys.exit(main())
