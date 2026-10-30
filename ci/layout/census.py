#!/usr/bin/env python3
"""The layout census: holds tokora's public export layout over an all-features rustdoc JSON.

Usage:
    census.py TOKORA.json [--root FILE] [--allow FILE] [--combinator-module NAME]
    census.py --selftest

`TOKORA.json` is `target/doc/tokora.json` from

    RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" \
      cargo doc -p tokora --no-deps --all-features --locked

with ALL features: the root list is feature-dependent, and a name behind a feature the dump was
built without is simply not in it, so a narrower dump would hold a narrower layout.

WHAT IS READ. Every public path is resolved the way the design measured the surface: from the
crate root, through each public item, each named `pub use` and each glob `pub use`, descending
into public modules. An enum variant is not an item (a glob over an enum makes its variants
reachable, and they are not counted); a module is not an item either, though the crate root's
modules are part of the frozen root list. A `use` whose target is outside the crate (`logos`,
`logos_0_16`, `utils::typenum`, `utils::arraydeque`, `utils::ArrayDeque`, …) is recorded as one
external item and never descended into: its contents are that crate's layout, not tokora's.

WHAT IS NOT READ. `#[doc(hidden)]` items are absent from rustdoc JSON unless rustdoc runs with
`--document-hidden-items`, and the CI dump does not, so this census never sees `__private` — the
macro-support module — or anything else the crate hides. A hidden item is outside the documented
surface by construction; holding it is not this census's job. Surface GROWTH inside a module
through a glob is not either: the `name-collision-probe` job reports every newly reachable name.

THE FOUR CHECKS:

  1. One canonical path. An item has at most two public paths, and when it has two, one of them
     is at the crate root. A path through a module marked `versioned` in the allow-list counts
     together with its unversioned alias one level up as ONE canonical path (rule 6:
     `lexer::logos_0_16::LogosLexer` and `lexer::LogosLexer` are the same integration item).
  2. The root is frozen. The crate root's public names, with their kinds, equal the committed
     list (`root.txt`), and the root has no glob `use` at all — enforced directly rather than
     inferred from the list, since a glob that happens to re-export nothing new today is still
     the mechanism the design removed.
  3. No trait is reachable under `combinator::` (rule 3). `--combinator-module NAME` points the
     check at another module name; it exists so the check can be shown to fire on a dump that
     predates the rename (`--combinator-module parser` on the 0.11-layout control).
  4. No path is deeper than `tokora::<module>::<Item>` outside the modules the allow-list names
     (rule 5). External re-exports are skipped as "target outside the crate", not allow-listed.

Every failure prints the item, its paths and the rule it breaks. Exit status: 0 when every check
holds, 1 when any check fails, 2 when the input cannot be read (including an unsupported
`format_version`).

FORMAT VERSIONS. The rustdoc JSON schema is versioned and unstable. This file reads `index`,
`root`, `paths`, items' `name`/`visibility`/`inner`, and in `inner` the `module.items`,
`enum.variants` and `use.{name, source, id, is_glob}` fields; it accepts only the versions it was
checked against and refuses any other by name, because a schema change it did not anticipate would
otherwise read as "no violations". To add a version, read the rustdoc-types changelog for every
field above and add it to `SUPPORTED_FORMAT_VERSIONS`.
"""

import argparse
import json
import os
import sys
from collections import defaultdict

SUPPORTED_FORMAT_VERSIONS = {61}

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_ROOT = os.path.join(HERE, "root.txt")
DEFAULT_ALLOW = os.path.join(HERE, "allow.txt")

# rustdoc's item kinds, spelled the way `root.txt` lists them.
KIND_WORDS = {
    "module": "mod",
    "struct": "struct",
    "enum": "enum",
    "union": "union",
    "trait": "trait",
    "trait_alias": "trait_alias",
    "type_alias": "type",
    "function": "fn",
    "constant": "const",
    "static": "static",
    "macro": "macro",
    "proc_macro": "proc_macro",
    "primitive": "primitive",
    "variant": "variant",
    "external": "extern",
    # `pub extern crate rowan_0_17 as rowan;` is an `extern_crate` item, not a `use`; it is the same
    # kind of root name as the `pub use logos_0_16 as logos;` re-export and is listed the same way.
    "extern_crate": "extern",
}


class CensusError(Exception):
    """The input cannot be read as a supported rustdoc JSON dump."""


def kind_of(item):
    inner = item.get("inner")
    if isinstance(inner, dict) and len(inner) == 1:
        return next(iter(inner))
    return "?"


class Surface:
    """Every public path of one rustdoc JSON dump."""

    def __init__(self, doc):
        if not isinstance(doc, dict) or "format_version" not in doc:
            raise CensusError("not a rustdoc JSON dump: no `format_version`")
        version = doc["format_version"]
        if version not in SUPPORTED_FORMAT_VERSIONS:
            raise CensusError(
                "rustdoc JSON format_version %r is not one this census was checked against "
                "(supported: %s). Read the rustdoc-types changelog for the fields the module "
                "docstring lists, then add the version to SUPPORTED_FORMAT_VERSIONS."
                % (version, ", ".join(str(v) for v in sorted(SUPPORTED_FORMAT_VERSIONS)))
            )
        for key in ("index", "root"):
            if key not in doc:
                raise CensusError("rustdoc JSON has no `%s`" % key)
        self.version = version
        self.index = {str(k): v for k, v in doc["index"].items()}
        self.root = str(doc["root"])
        if self.root not in self.index:
            raise CensusError("the root id %s is not in the index" % self.root)
        self.paths = defaultdict(set)   # item key -> {path tuple}
        self.kind = {}                  # item key -> rustdoc kind ("external" for outside targets)
        self.root_names = set()         # {(kind word, name)} for every public root name
        self.root_globs = []            # sources of glob `use` items at the root
        self._seen = set()
        self._walk(self.root, ())

    def _get(self, item_id):
        if item_id is None:
            return None
        return self.index.get(str(item_id))

    def _record(self, key, kind, path):
        self.kind[key] = kind
        self.paths[key].add(path)
        if len(path) == 1:
            self.root_names.add((KIND_WORDS.get(kind, kind), path[0]))

    def _walk(self, module_id, prefix, depth=0):
        if (module_id, prefix) in self._seen or depth > 16:
            return
        self._seen.add((module_id, prefix))
        module = self._get(module_id)
        if module is None:
            return
        inner = module.get("inner", {})
        if "module" in inner:
            children = inner["module"].get("items", [])
        elif "enum" in inner:
            children = inner["enum"].get("variants", [])
        else:
            return
        for child_id in children:
            child = self._get(child_id)
            if child is None:
                continue
            k = kind_of(child)
            if k == "use":
                use = child["inner"]["use"]
                target_id = use.get("id")
                target = self._get(target_id)
                if use.get("is_glob"):
                    # Only a glob written IN the root module is a glob into the root; one met
                    # while following that glob into a private module is the same mechanism one
                    # level removed, and the glob that brought it in is the one reported.
                    if module_id == self.root:
                        self.root_globs.append(use.get("source", "?"))
                    if target is not None:
                        self._walk(str(target_id), prefix, depth + 1)
                    continue
                name = use.get("name")
                if not name:
                    continue
                if target is None:
                    self._record(("ext", use.get("source", name)), "external", prefix + (name,))
                    continue
                tk = kind_of(target)
                self._record(str(target_id), tk, prefix + (name,))
                if tk == "module":
                    self._walk(str(target_id), prefix + (name,), depth + 1)
                continue
            if k not in ("variant", "macro") and child.get("visibility") != "public":
                continue
            name = child.get("name")
            if k == "extern_crate":
                # The name the crate is visible under is the rename, when there is one.
                ec = child["inner"]["extern_crate"]
                name = ec.get("rename") or ec.get("name") or name
            if not name:
                continue
            self._record(str(child_id), k, prefix + (name,))
            if k == "module":
                self._walk(str(child_id), prefix + (name,), depth + 1)

    def items(self):
        """(key, kind, sorted paths) for every item — enum variants and modules excluded."""
        for key, ps in self.paths.items():
            k = self.kind.get(key, "?")
            if k in ("variant", "module"):
                continue
            yield key, k, sorted(ps, key=lambda p: (len(p), p))


def show(path):
    return "tokora::" + "::".join(path)


def read_list(path):
    """Non-blank lines of a committed list, `#` comments stripped."""
    out = []
    with open(path) as f:
        for raw in f:
            line = raw.split("#", 1)[0].strip()
            if line:
                out.append(" ".join(line.split()))
    return out


def read_allow(path):
    """{module path: is_versioned} from allow.txt (`<module path> [versioned]`)."""
    allow = {}
    for line in read_list(path):
        parts = line.split()
        if len(parts) not in (1, 2) or (len(parts) == 2 and parts[1] != "versioned"):
            raise CensusError("allow.txt line %r: expected `<module path> [versioned]`" % line)
        allow[parts[0]] = len(parts) == 2
    return allow


def canonical_count(paths, versioned):
    """How many canonical paths `paths` amounts to, a versioned path and its unversioned alias one
    level up counting once."""
    ps = set(paths)
    for p in list(ps):
        if len(p) >= 3 and "::".join(p[:-1]) in versioned:
            alias = p[:-2] + (p[-1],)
            if alias in ps:
                ps.discard(alias)
    return ps


def census(surface, root_list, allow, combinator="combinator"):
    """Run the four checks. Returns a list of (check number, message)."""
    failures = []
    versioned = {m for m, v in allow.items() if v}

    # 1. at most two public paths, and a second one at the root
    for key, kind, ps in surface.items():
        counted = canonical_count(ps, versioned)
        if len(counted) <= 1:
            continue
        if len(counted) == 2 and any(len(p) == 1 for p in counted):
            continue
        failures.append((1, "%s %s has %d public paths (%s); rule 1 allows one canonical path "
                            "plus the crate root" % (KIND_WORDS.get(kind, kind), ps[0][-1],
                                                     len(counted), ", ".join(show(p) for p in ps))))

    # 2. the root equals the frozen list, and has no glob `use`
    expected = set(root_list)
    actual = {"%s %s" % (k, n) for k, n in surface.root_names}
    for line in sorted(actual - expected):
        kind, _, name = line.partition(" ")
        failures.append((2, "%s %s is at the crate root (tokora::%s) but not in root.txt; rule 2 "
                            "makes adding a root name a deliberate edit to the list"
                            % (kind, name, name)))
    for line in sorted(expected - actual):
        failures.append((2, "%s is in root.txt but not at the crate root; rule 2 freezes the root "
                            "both ways" % line))
    for source in surface.root_globs:
        failures.append((2, "glob `pub use %s::*` at the crate root; rule 2 forbids a glob "
                            "re-export into the root" % source))

    # 3. no trait under combinator::
    for key, kind, ps in surface.items():
        if kind != "trait":
            continue
        under = [p for p in ps if p[0] == combinator]
        if under:
            failures.append((3, "trait %s is reachable under %s:: (%s); rule 3 keeps every trait "
                                "out of that module" % (ps[0][-1], combinator,
                                                        ", ".join(show(p) for p in ps))))

    # 4. no path deeper than two outside the allow-list (external targets skipped)
    for key, kind, ps in surface.items():
        if kind == "external":
            continue
        deep = [p for p in ps if len(p) > 2 and "::".join(p[:-1]) not in allow]
        if deep:
            failures.append((4, "%s %s has a path deeper than tokora::<module>::<Item> outside "
                                "allow.txt (%s); rule 5" % (KIND_WORDS.get(kind, kind), ps[0][-1],
                                                            ", ".join(show(p) for p in deep))))
    return failures


def stats(surface):
    items = list(surface.items())
    root_items = sum(1 for _, _, ps in items if any(len(p) == 1 for p in ps))
    multi = sum(1 for _, _, ps in items if len(ps) > 1)
    three = sum(1 for _, _, ps in items if len(ps[0]) >= 3)
    return {"items": len(items), "root_items": root_items, "multi_path": multi,
            "shortest_three_level": three}


def run(json_path, root_path, allow_path, combinator):
    try:
        with open(json_path) as f:
            doc = json.load(f)
        surface = Surface(doc)
        allow = read_allow(allow_path)
        root_list = read_list(root_path)
    except (OSError, ValueError, CensusError) as e:
        print("layout census: cannot read the input — %s" % e, file=sys.stderr)
        return 2
    s = stats(surface)
    print("layout census: format_version %d; %d items, %d at the root, %d with more than one "
          "path, %d whose shortest path is three levels deep"
          % (surface.version, s["items"], s["root_items"], s["multi_path"],
             s["shortest_three_level"]))
    failures = census(surface, root_list, allow, combinator)
    by_check = defaultdict(int)
    for check, message in failures:
        by_check[check] += 1
        print("  check %d: %s" % (check, message))
    for check in (1, 2, 3, 4):
        print("layout census: check %d — %s" % (check, "%d failure(s)" % by_check[check]
                                                if by_check[check] else "holds"))
    if failures:
        print("layout census: FAILED (%d failure(s))" % len(failures))
        return 1
    print("layout census: OK")
    return 0


# ── self-test ─────────────────────────────────────────────────────────────────────────────────
#
# Small synthetic dumps in the shape rustdoc writes: one that holds, and one per check that
# breaks only that check. A census that cannot be watched failing is not a census.

class Fixture:
    """A builder for minimal rustdoc-JSON-shaped dumps."""

    def __init__(self, version=61):
        self.version = version
        self.index = {}
        self.next_id = 1
        self.root = self.module("fixture", [])

    def _add(self, name, inner, visibility="public"):
        item_id = self.next_id
        self.next_id += 1
        self.index[str(item_id)] = {"id": item_id, "name": name, "visibility": visibility,
                                    "inner": inner}
        return item_id

    def module(self, name, items, visibility="public"):
        return self._add(name, {"module": {"items": list(items), "is_crate": False}}, visibility)

    def item(self, kind, name, visibility="public"):
        return self._add(name, {kind: {}}, visibility)

    def use(self, name, target, glob=False, source=None):
        return self._add(None if glob else name, {"use": {
            "source": source or name, "name": name, "id": target, "is_glob": glob}})

    def add(self, module_id, *children):
        self.index[str(module_id)]["inner"]["module"]["items"].extend(children)

    def doc(self):
        return {"format_version": self.version, "root": self.root, "index": self.index,
                "paths": {}, "crate_version": "0.0.0", "includes_private": False,
                "external_crates": {}, "target": {}}


def passing_fixture():
    """A layout that holds every check: a trait in `parse` re-exported at the root, a struct in
    `combinator`, a versioned integration pair, an allow-listed three-level module, and an external
    crate re-exported at the root and at depth three."""
    f = Fixture()
    trait_t = f.item("trait", "ParseInput")
    private_input = f.module("input", [trait_t], visibility="crate")
    parse = f.module("parse", [private_input, f.use("ParseInput", trait_t)])
    struct_s = f.item("struct", "Any")
    combinator = f.module("combinator", [struct_s])
    lexer_struct = f.item("struct", "LogosLexer")
    versioned = f.module("logos_0_16", [lexer_struct])
    lexer = f.module("lexer", [versioned, f.use("LogosLexer", lexer_struct)])
    deep = f.item("struct", "IntLiteral")
    knowledge = f.module("knowledge", [deep])
    utils = f.module("utils", [knowledge, f.use("typenum", 999_999)])
    macro = f.item("macro", "select")
    f.add(f.root, parse, combinator, lexer, utils, macro, f.use("ParseInput", trait_t),
          f.use("logos", 999_998))
    root_list = ["mod parse", "mod combinator", "mod lexer", "mod utils", "macro select",
                 "trait ParseInput", "extern logos"]
    allow = {"lexer::logos_0_16": True, "utils::knowledge": False}
    return f, root_list, allow


def _check_set(doc, root_list, allow, combinator="combinator"):
    return {check for check, _ in census(Surface(doc), root_list, allow, combinator)}


def selftest():
    failures = []

    def expect(label, got, want):
        ok = got == want
        print("  %-4s %s (checks failing: %s)" % ("ok" if ok else "FAIL", label,
                                                    sorted(got) if got else "none"))
        if not ok:
            failures.append(label)

    f, root_list, allow = passing_fixture()
    expect("the passing fixture holds every check", _check_set(f.doc(), root_list, allow), set())

    # check 1 — a second non-root path
    f, root_list, allow = passing_fixture()
    struct_s = next(k for k, v in f.index.items() if v["name"] == "Any")
    parse = next(k for k, v in f.index.items() if v["name"] == "parse")
    f.add(int(parse), f.use("Any", int(struct_s)))
    expect("an item at combinator::Any and parse::Any fails check 1",
           _check_set(f.doc(), root_list, allow), {1})

    # check 1 — the versioned pair is one canonical path, but an unversioned allow-listed module
    # buys no such pairing
    f, root_list, allow = passing_fixture()
    deep = next(k for k, v in f.index.items() if v["name"] == "IntLiteral")
    utils = next(k for k, v in f.index.items() if v["name"] == "utils")
    f.add(int(utils), f.use("IntLiteral", int(deep)))
    expect("utils::knowledge::X beside utils::X fails check 1 (knowledge is not versioned)",
           _check_set(f.doc(), root_list, allow), {1})

    # check 2 — a root name the list does not have
    f, root_list, allow = passing_fixture()
    struct_s = next(k for k, v in f.index.items() if v["name"] == "Any")
    f.add(f.root, f.use("Any", int(struct_s)))
    expect("a root name missing from root.txt fails check 2",
           _check_set(f.doc(), root_list, allow), {2})

    # check 2 — a listed name the root does not have
    f, root_list, allow = passing_fixture()
    expect("a root.txt line the root lacks fails check 2",
           _check_set(f.doc(), root_list + ["fn parse"], allow), {2})

    # check 2 — a glob at the root, even one that adds no name
    f, root_list, allow = passing_fixture()
    empty = f.module("located", [], visibility="crate")
    f.add(f.root, empty, f.use("located", empty, glob=True))
    expect("a glob `use` at the root fails check 2 even when it re-exports nothing",
           _check_set(f.doc(), root_list, allow), {2})

    # check 3 — a trait under combinator, and the module name is a parameter
    f, root_list, allow = passing_fixture()
    trait_t = next(k for k, v in f.index.items() if v["name"] == "ParseInput")
    combinator = next(k for k, v in f.index.items() if v["name"] == "combinator")
    handler = f.item("trait", "SeparatorHandler")
    f.add(int(combinator), handler)
    expect("a trait reachable under combinator:: fails check 3",
           _check_set(f.doc(), root_list, allow), {3})
    expect("the same dump holds check 3 when told the module is called `parser`",
           _check_set(f.doc(), root_list, allow, combinator="parser"), set())

    # check 4 — a three-level path outside the allow-list
    f, root_list, allow = passing_fixture()
    deep = next(k for k, v in f.index.items() if v["name"] == "IntLiteral")
    combinator = next(k for k, v in f.index.items() if v["name"] == "combinator")
    many = f.module("many", [f.use("IntLiteral", int(deep))])
    f.add(int(combinator), many)
    expect("combinator::many::X fails check 4 (and check 1: it is a second non-root path)",
           _check_set(f.doc(), root_list, allow), {1, 4})
    f, root_list, allow = passing_fixture()
    combinator = next(k for k, v in f.index.items() if v["name"] == "combinator")
    lone = f.item("struct", "Repeated")
    f.add(int(combinator), f.module("many", [lone]))
    expect("an item whose only path is combinator::many::X fails check 4 alone",
           _check_set(f.doc(), root_list, allow), {4})

    # external re-exports are skipped by check 4, and enum variants are not items
    f, root_list, allow = passing_fixture()
    utils = next(k for k, v in f.index.items() if v["name"] == "utils")
    ext_mod = f.module("ext", [f.use("Foreign", 999_997)])
    f.add(int(utils), ext_mod)
    enum_e = f._add("ParseAttempt", {"enum": {"variants": []}})
    variant = f._add("Accept", {"variant": {}})
    f.index[str(enum_e)]["inner"]["enum"]["variants"].append(variant)
    parse = next(k for k, v in f.index.items() if v["name"] == "parse")
    f.add(int(parse), enum_e, f.use("ParseAttempt", enum_e, glob=True))
    expect("an external item at utils::ext::Foreign and a glob over an enum hold every check",
           _check_set(f.doc(), root_list, allow), set())

    # a `pub extern crate rowan_0_17 as rowan;` at the root is an `extern` name under its rename,
    # beside the `pub use rowan_0_17;` that lists the versioned name
    f, root_list, allow = passing_fixture()
    ext = f._add("rowan", {"extern_crate": {"name": "rowan_0_17", "rename": "rowan"}})
    f.add(f.root, ext, f.use("rowan_0_17", 999_996))
    expect("a pub extern crate and its versioned re-export hold every check when listed",
           _check_set(f.doc(), root_list + ["extern rowan", "extern rowan_0_17"], allow), set())
    expect("a pub extern crate not listed in root.txt fails check 2 alone",
           _check_set(f.doc(), root_list + ["extern rowan_0_17"], allow), {2})

    # a failure names the item, every path it has, and the rule it breaks
    f, root_list, allow = passing_fixture()
    struct_s = next(k for k, v in f.index.items() if v["name"] == "Any")
    parse = next(k for k, v in f.index.items() if v["name"] == "parse")
    f.add(int(parse), f.use("Any", int(struct_s)))
    messages = [m for _, m in census(Surface(f.doc()), root_list, allow)]
    named = any("struct Any" in m and "tokora::combinator::Any" in m and "tokora::parse::Any" in m
                and "rule 1" in m for m in messages)
    expect("the check-1 message names the item, both paths and the rule",
           set() if named else {1}, set())

    # an unsupported format_version is refused by name, never read
    f, root_list, allow = passing_fixture()
    f.version = 9999
    try:
        Surface(f.doc())
        refused = False
    except CensusError as e:
        refused = "9999" in str(e)
    expect("format_version 9999 is refused, naming the version", set() if refused else {0},
           set())

    if failures:
        print("layout census selftest: %d case(s) FAILED: %s" % (len(failures), failures))
        return 1
    print("layout census selftest: all cases passed")
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("json", nargs="?", help="all-features rustdoc JSON of tokora")
    parser.add_argument("--root", default=DEFAULT_ROOT, help="the frozen root list")
    parser.add_argument("--allow", default=DEFAULT_ALLOW, help="the depth allow-list")
    parser.add_argument("--combinator-module", default="combinator",
                        help="the module check 3 keeps traits out of (diagnostic)")
    parser.add_argument("--selftest", action="store_true", help="run the synthetic fixtures")
    args = parser.parse_args(argv)
    if args.selftest:
        return selftest()
    if not args.json:
        parser.error("a rustdoc JSON path is required (or --selftest)")
    return run(args.json, args.root, args.allow, args.combinator_module)


if __name__ == "__main__":
    sys.exit(main())
