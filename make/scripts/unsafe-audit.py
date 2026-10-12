# SPDX-License-Identifier: AGPL-3.0-only
# SPDX-FileCopyrightText: Contributors to the dioxus-apps project

"""The transitive unsafe audit, over `cargo metadata` on stdin.

Reads the JSON `cargo metadata` emits and reports, for every package in the
resolved graph, how much `unsafe` its source contains and whether it has opted
out of it with `#![forbid(unsafe_code)]`.

The counts are lexical. That is the same measurement `cargo geiger` makes in
its usage column, and it is the one that answers "how much unsafe am I pulling
in", which is the question this report exists for.
"""

from __future__ import annotations

import json
import os
import re
import sys
from collections import defaultdict

# `unsafe` as a word, not as the start of `unsafety` or `unsafe_code` spelled
# inside a longer identifier, and not inside a string or a comment. The last of
# those is not attempted: a lexical count that tried would need to be a parser,
# and this is a count.
UNSAFE = re.compile(r"\bunsafe\b")

# `#![forbid(unsafe_code)]`, `#![forbid(unsafe_code)]` with a reason string, and
# the attribute written with spaces.
FORBID = re.compile(r"#!\s*\[\s*forbid\s*\(\s*unsafe_code\s*\)")


def count_in(crate_root: str) -> tuple[int, int, bool]:
    """`(unsafe uses, files read, forbids unsafe)` for one crate.

    `crate_root` is the directory holding its `Cargo.toml`. Sources are read
    from there rather than from `src/` alone, because a crate can put its
    modules anywhere it likes — this repository's own apps use `src/<mod>/`
    with a `src/<mod>.rs` alongside, and a `src/*.rs` glob would miss the
    sibling `.rs` files a test-only module reads.
    """
    uses = 0
    files = 0
    forbid = False
    for dirpath, dirnames, filenames in os.walk(crate_root):
        dirnames[:] = [d for d in dirnames if d not in {".git", "target"}]
        for name in filenames:
            if not name.endswith(".rs"):
                continue
            path = os.path.join(dirpath, name)
            try:
                with open(path, encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
            except OSError:
                continue
            files += 1
            uses += len(UNSAFE.findall(text))
            if FORBID.search(text):
                forbid = True
    return uses, files, forbid


def main() -> int:
    metadata = json.load(sys.stdin)
    workspace_members = set(metadata["workspace_members"])

    # Manifest path -> package, so a path dependency that appears more than
    # once is only read once.
    by_root: dict[str, str] = {}
    for package in metadata["packages"]:
        root = os.path.dirname(package["manifest_path"])
        by_root.setdefault(root, package["name"])

    # Which packages the workspace's own crates depend on, so the report can
    # separate "unsafe in something we asked for" from "unsafe four crates
    # down". Both are interesting and they are not the same number.
    direct: set[str] = set()
    for package in metadata["packages"]:
        if package["id"] not in workspace_members:
            continue
        for dependency in package["dependencies"]:
            direct.add(dependency["name"])

    rows = []
    for package in metadata["packages"]:
        root = os.path.dirname(package["manifest_path"])
        uses, files, forbid = count_in(root)
        rows.append(
            (
                uses,
                package["name"],
                package["version"],
                "workspace" if package["id"] in workspace_members else "direct"
                if package["name"] in direct
                else "transitive",
                files,
                forbid,
            )
        )

    rows.sort(reverse=True)

    header = f"{'unsafe':>7}  {'crates':>6}  {'kind':<10}  crate"
    print(header)
    print("-" * len(header))
    for uses, name, version, kind, files, forbid in rows:
        note = "  #![forbid(unsafe_code)]" if forbid else ""
        print(f"{uses:>7}  {files:>6}  {kind:<10}  {name} {version}{note}")

    total_uses = sum(row[0] for row in rows)
    with_unsafe = sum(1 for row in rows if row[0])
    forbidden = sum(1 for row in rows if row[5])
    print()
    print(
        f"{len(rows)} crates, {with_unsafe} of them using unsafe, "
        f"{forbidden} declaring #![forbid(unsafe_code)], "
        f"{total_uses} uses in total"
    )
    # Advisory, as this has always been: the counts are the report and a
    # dependency's unsafe code is not this repository's decision to fail on.
    return 0


if __name__ == "__main__":
    sys.exit(main())