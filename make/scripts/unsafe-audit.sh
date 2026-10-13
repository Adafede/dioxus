#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# SPDX-FileCopyrightText: Contributors to the dioxus-apps project
#
# The transitive unsafe audit: how much `unsafe` the dependency tree brings in.
#
# This was `cargo geiger`, and it is not any more. cargo-geiger 0.13.0 is the
# last release, it is from 2021 in everything but its version number, and it
# carries its own copy of `cargo` — which means its own copy of the manifest
# parser. That parser is TOML 0.5, and TOML 0.5 has no multi-line inline table,
# so it cannot read this workspace at all:
#
#   error: invalid inline table
#   expected `}`
#     --> lotus-explore-rs/Cargo.toml:71:12
#      |
#    71 | web-sys = {
#
# One git dependency writing `web-sys = { version = "0.3", features = [ ... ] }`
# across lines — which is valid TOML 1.0, which is what Cargo reads — is enough.
# There is no newer cargo-geiger to move to and no flag to move around it, so
# the tool could not answer the question it was installed to answer, and the
# weekly job was reporting the parse error as a red X rather than as a finding.
#
# What is here instead asks the same question with tools that are already
# required — `cargo`, `grep`, `sed`, `awk` and `find` — and there is no language
# runtime among them. `setup.sh` and the CI workflow both install nothing for
# this task, which is the point: the audit needs nothing installed.
#
# The crate list comes out of `cargo metadata`, but only through the
# `"manifest_path":"..."` field of each package, which is a single JSON string
# Cargo emits in one shape for every package it reports. Reading it with grep
# rather than parsing the document is not a shortcut: the document is two
# megabytes of nested arrays, and a hand-written parser for it is a thing that
# breaks when Cargo changes an unrelated field. The field is a path to a
# `Cargo.toml`, and every other fact is then read off the workspace it points at.
#
# `cargo tree --depth 1` supplies what a manifest cannot: which of those crates
# are a direct dependency rather than something four levels down. Cargo
# resolves that, and re-deriving it from `[dependencies]` keys would be wrong for
# any dependency renamed with `package = "..."`.
#
# The counts are lexical — how many times the token `unsafe` appears in a
# crate's `.rs` files — so they are the same number `cargo geiger` reports in
# its "unsafe usage" column and not a finer measure of it. That is the right
# precision for the question, which is how much unsafe the tree pulls in and
# not whether any of it is sound.
#
# Advisory, as it always was: the report is the output and the exit status is
# success. `unsafe_code = "forbid"` in the workspace lints is what actually
# gates this repository's own code, and nothing here can fail on a dependency
# someone else's release notes have to explain.
set -euo pipefail
cd "$(dirname "$0")/../.."

root=$(pwd)

# ── the crate list, and which of them are ours ──────────────────────────────

# `name|version` for one manifest, from its `[package]`.
#
# `|` and not a tab, because a tab is whitespace to `read` and `read` collapses
# runs of a whitespace IFS: "name\t\t1" would come back as two fields with the
# second being `1`, where the empty slot was. An empty version is how an
# inherited one is reported, so the field has to survive.
#
# The eight workspace members leave it empty — they say
# `version.workspace = true` and take the number from `[workspace.package]` —
# and they are also the only crates whose manifest is inside this repository,
# so the empty field and the location cannot disagree without the workspace
# being broken.
read_crate() {
    awk '
        function unq(s) { sub(/^"/, "", s); sub(/".*$/, "", s); return s }
        BEGIN { in_pkg = 0; name = ""; ver = "" }
        /^\[/ { in_pkg = ($0 ~ /^\[package\]/); next }
        in_pkg && /^[ \t]*name[ \t]*=/ {
            v = $0; sub(/^[^=]*=[ \t]*/, "", v)
            if (v ~ /^"/) { name = unq(v) }
        }
        in_pkg && /^[ \t]*version[ \t]*=/ {
            v = $0; sub(/^[^=]*=[ \t]*/, "", v)
            if (v ~ /workspace/) { ver = "" }
            else if (v ~ /^"/) { ver = unq(v) }
        }
        in_pkg && /^[ \t]*version\.workspace[ \t]*=/ { ver = "" }
        END { print name "|" ver }
    ' "$1"
}

# `head -1` because `exit` inside awk still runs the `END` block, and the `END`
# that would run here is the one that prints the crate fields for a manifest
# that has none. Two values out, and one of them is a tab.
workspace_version=$(awk '
    /^\[workspace.package\]/ { in_ws = 1; next }
    /^\[/ { in_ws = 0 }
    in_ws && /^[ \t]*version[ \t]*=/ {
        v = $0; sub(/^[^=]*=[ \t]*/, "", v); gsub(/"/, "", v)
        print v; exit
    }
' Cargo.toml | head -1)
: "${workspace_version:?could not read the workspace version from Cargo.toml}"

# The manifests in the resolved graph, each read exactly once. Two different
# `PackageId`s can point at one checkout — a git dependency declared with and
# without a `branch = ...` — and scanning it twice would double-count it.
crate_roots=$(
    cargo metadata --format-version 1 --locked 2>/dev/null \
        | grep -o '"manifest_path":"[^"]*"' \
        | sed 's/^"manifest_path":"//; s/"$//' \
        | grep '/Cargo\.toml$' \
        | sort -u
)
if [ -z "$crate_roots" ]; then
    echo "cargo metadata reported no packages; is Cargo.lock up to date?" >&2
    exit 1
fi

# Direct dependencies by name, from Cargo's own resolution at depth one. The
# tree lists the workspace members too, and they are separated below by location
# rather than by the shape of the line.
direct_names=$(
    cargo tree --edges normal --prefix none --depth 1 --all-features 2>/dev/null \
        | awk '{ print $1 }' | sort -u
)

# ── the scan ────────────────────────────────────────────────────────────────

rows=$(mktemp)
trap 'rm -f "$rows"' EXIT

printf '%7s  %7s  %-10s  %s\n' 'unsafe' 'files' 'kind' 'crate'

total_uses=0
crates_with_unsafe=0
crates_forbidding=0

for root_of_crate in $crate_roots; do
    IFS='|' read -r name version < <(read_crate "$root_of_crate")
    [ -n "$name" ] || continue
    if [ -z "$version" ]; then version=$workspace_version; fi

    case $root_of_crate in
        "$root"/*) kind=workspace ;;
        *)
            if printf '%s\n' "$direct_names" | grep -qx -- "$name"; then
                kind=direct
            else
                kind=transitive
            fi
            ;;
    esac

    # `dirname`, because the metadata field is a path to a manifest and everything
    # below reads the crate the manifest describes. Scanning the manifest itself
    # would count the `[dependencies]` table and nothing else.
    crate_dir=$(dirname "$root_of_crate")

    # `--exclude-dir` because a workspace member sits inside this repository,
    # where `target/` is a sibling of the crate rather than a child; without it a
    # build tree there would be scanned as though it were source.
    scan=(--include='*.rs' --exclude-dir=target --exclude-dir=.git -r)

    # `|| true` because `grep` exits 1 on no match under `set -e`, and a crate
    # with no unsafe in it is the common case, not an error.
    uses=$(grep -o "${scan[@]}" -E '\bunsafe\b' -- "$crate_dir" 2>/dev/null | wc -l) || true
    files=$(find "$crate_dir" -type f -name '*.rs' \
        -not -path '*/target/*' -not -path '*/.git/*' -print 2>/dev/null | wc -l) || true

    forbid=""
    if grep -q "${scan[@]}" -E '#![[:space:]]*\[[[:space:]]*forbid[[:space:]]*\([[:space:]]*unsafe_code\s*\)' \
        -- "$crate_dir" 2>/dev/null; then
        forbid="  #![forbid(unsafe_code)]"
        crates_forbidding=$((crates_forbidding + 1))
    fi

    printf '%d\t%d\t%s\t%s %s%s\n' \
        "$uses" "$files" "$kind" "$name" "$version" "$forbid" >>"$rows"

    total_uses=$((total_uses + uses))
    if [ "$uses" -gt 0 ]; then crates_with_unsafe=$((crates_with_unsafe + 1)); fi
done

# Most unsafe first, then alphabetically, so the interesting rows are the ones on
# screen and a big number is not hidden behind a big name. One `awk` does the
# formatting: the rows are tab-separated and the columns in them are not, so
# splitting a row on anything else would cut a crate name in half.
sort -t$'\t' -k1,1nr -k4,4 "$rows" | awk -F'\t' '
    { printf "%7s  %7s  %-10s  %s\n", $1, $2, $3, $4 }
'

crates_scanned=$(wc -l <"$rows" | tr -d ' ')
printf '\n%s crates scanned, %d using unsafe, %d declaring #[forbid(unsafe_code)], %d uses in total\n' \
    "$crates_scanned" "$crates_with_unsafe" "$crates_forbidding" "$total_uses"
