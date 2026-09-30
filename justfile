# Root task runner for the dioxus workspace.
# Run `just --list` to see available recipes.
#
# Web apps (use `just serve`/`just build` with one of these):
#	cxsmiles-yoga  index  json-count-rs  lipid-selecto-rs
#	mgf-precursor-erro-rs smellfish-rs
#
# Every check below is one CI job, in CI's order, and
# `crates/upload/tests/gate_consistency.rs` fails the build if that stops being
# true. That test is the only reason these two lists have not drifted: it holds
# the justfile, `prek.toml` and `.github/workflows/ci.yml` to a single mapping.

# ── Workspace gate (mirrors .github/workflows/ci.yml) ─────────────────────────

fmt:
	cargo fmt --all -- --check

# Every `.rs` file in the workspace carries both SPDX headers, on lines 1 and 2.
#
# Text-only, so it belongs on the fast path: without it a commit lands with a
# missing header and CI rejects it, which is a pointless round trip.
#
# `target/` and `graphify-out/` are build output and generated graph data
# respectively — neither is source. The copyright line is checked as a prefix
# rather than an exact string because each crate names its own project: the
# licence has to be identical everywhere, the copyright holder may not be.
license-headers:
	#!/usr/bin/env bash
	set -euo pipefail
	missing=0
	while IFS= read -r f; do
	  if ! head -1 "$f" | grep -qxF '// SPDX-License-Identifier: AGPL-3.0-only' \
	     || ! head -2 "$f" | tail -1 | grep -qE '^// SPDX-FileCopyrightText: Contributors to the .+$'; then
	    echo "missing or misplaced SPDX header: $f" >&2
	    missing=1
	  fi
	done < <(find . \( -name target -o -name graphify-out -o -name .git \) -prune -o -name '*.rs' -print)
	exit $missing

check:
	cargo check --workspace --all-targets --locked

clippy:
	cargo clippy --workspace --all-targets --locked -- -D warnings

test:
	cargo test --workspace --all-targets --locked --quiet

# `RUSTDOCFLAGS="-D warnings"` is the whole point of this recipe. Without it a
# rustdoc warning is printed and the recipe still passes, so it proves only that
# the docs build, not that they are warning-free — and a broken intra-doc link
# is a 404 in the generated docs, which no test here would notice.
doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

# ── Full CI gate (every check the pipeline runs, in order) ────────────────────
# `just ci`. Each step reuses a recipe above (single source of truth). Supply-chain
# tools that may be absent locally are skipped by their own recipes.

ci:
	just fmt
	just license-headers
	just check
	just clippy
	just test
	just doc
	just wasm
	just clippy-wasm
	just machete
	just deny
	just audit
	just readme

# The CI job `just ci` leaves out, and why: `mutants` is non-blocking in CI
# because survivors are a to-do list rather than a gate, and at 421 mutants it
# takes ~25 min rather than seconds. `just ci-slow` is where it runs.
ci-slow:
	just mutants

# WASM apps only — never `--workspace --target wasm32`: `crates/upload` is
# `cfg`-gated to wasm for the blob readers, so a workspace-wide wasm *test*
# target is meaningless, and its `download.rs` has a host-only path.
# One `cargo check -p <app>` per app keeps the wasm build green.
wasm:
	cargo check -p cxsmiles-yoga --target wasm32-unknown-unknown --locked
	cargo check -p index --target wasm32-unknown-unknown --locked
	cargo check -p json-count-rs --target wasm32-unknown-unknown --locked
	cargo check -p mgf-precursor-erro-rs --target wasm32-unknown-unknown --locked
	cargo check -p lipid-selecto-rs --target wasm32-unknown-unknown --locked
	cargo check -p smellfish-rs --target wasm32-unknown-unknown --locked

# WASM lint gate. `cargo check` on wasm32 only type-checks; `#[cfg(wasm32)]`
# branches never get linted by the host `--workspace` clippy run, so lint
# regressions in web-only code — dead code behind a cfg gate, a redundant
# `pub(crate)` re-export, an `indexing_slicing` in a streaming scanner — slip
# through. Same per-app shape as `wasm` above.
#
# `crates/upload` is linted on its own and *with* `--all-targets`, which is the
# line that matters: `blob_cursor`, `blob_lines` and `progress` are compiled
# only for wasm, so without this their code and their tests are never built by
# any command in the gate. That is how a test calling a method that does not
# exist survived here — the host build never sees those modules at all.
#
# The same blindness runs the other way, and it is the more expensive one: pure
# code behind a `#[cfg(target_arch = "wasm32")]` that touches no browser API is
# invisible to every *host* check, so nothing can assert on it from a host test
# run. `json-count-rs`'s string-escaper and `lipid-selecto-rs`'s whole MGF
# parser were both that, and both are host-built and tested now. The pattern to
# look for is a `cfg(wasm32)` on a function whose body has no `web_sys`, no
# `Signal` and no `Blob` in it: the gate is then not saying anything.
clippy-wasm:
	cargo clippy -p cxsmiles-yoga --target wasm32-unknown-unknown --all-targets --locked -- -D warnings
	cargo clippy -p index --target wasm32-unknown-unknown --all-targets --locked -- -D warnings
	cargo clippy -p json-count-rs --target wasm32-unknown-unknown --all-targets --locked -- -D warnings
	cargo clippy -p mgf-precursor-erro-rs --target wasm32-unknown-unknown --all-targets --locked -- -D warnings
	cargo clippy -p lipid-selecto-rs --target wasm32-unknown-unknown --all-targets --locked -- -D warnings
	cargo clippy -p smellfish-rs --target wasm32-unknown-unknown --all-targets --locked -- -D warnings
	cargo clippy -p upload --target wasm32-unknown-unknown --all-targets --locked -- -D warnings

# ── Per-app dev servers / production builds ───────────────────────────────────

serve app:
	dx serve --package {{app}}

build app:
	dx build --release --package {{app}}

# ── Mutation testing ──────────────────────────────────────────────────────────
# `cargo mutants` rewrites one expression at a time and re-runs the tests: a
# mutant that survives is a behaviour the suite does not actually pin down,
# which a passing test run cannot tell you. Coverage counts executed lines; this
# checks that they are asserted on.
#
# Its two result words are worth learning, because they are the opposite of what
# they look like. `CAUGHT` is the test failing on the mutant — killed, good.
# `MISSED` is the test still passing — survived, and that is a gap in the suite.
#
# The scope is two crates. The third did not fit a CI job's budget and is
# `just mutants-mgf`: 717 mutants, about half an hour, opt-in.
#
# `cxsmiles-yoga` went from 7 tests to 107 on the strength of this. Before:
# 296 mutants, 164 killed, 45 survived. After:
#
#   cxsmiles-yoga    290 mutants   21 min   193 killed, 13 survived
#
# 13 survivors of 290 is a to-do list, not a gate, which is why the CI job is
# `continue-on-error`. The remaining ones are the end-to-end arithmetic
# (`build_repeating`'s count range, `repeat_pattern`'s unit-size filter) and one
# union-find index, none of which the current fixtures distinguish.
#
# `json-count-rs` earned its place here differently. Its string-escaping
# functions were `#[cfg(target_arch = "wasm32")]` even though they take `&[u8]`
# and return a `String` and touch no browser API at all — so the host test build
# never compiled them, and nothing could assert on them from the host. They are
# now `cfg(any(test, …))` with tests beside them. The first of those tests hung
# the test runner, which is how the trailing-backslash infinite loop below came
# to light: a JSON string body ending in a lone `\` made the scanner find the
# same backslash at offset zero forever and never advance. The gate is worth
# running for that alone.
#
# `mutants.toml` (at `.cargo/mutants.toml`, the only path `cargo-mutants` reads
# without a flag) records what is excluded from mutation and why.
mutants:
	@command -v cargo-mutants >/dev/null 2>&1 || { echo "cargo-mutants not installed; skipping"; exit 0; }
	cargo mutants --package json-count-rs --package cxsmiles-yoga --jobs 8 --timeout 300

# The crate that did not fit in `mutants`: 717 mutants, about half an hour.
mutants-mgf:
	@command -v cargo-mutants >/dev/null 2>&1 || { echo "cargo-mutants not installed; skipping"; exit 0; }
	cargo mutants --package mgf-precursor-erro-rs --jobs 8 --timeout 300

# The Dioxus rendering crates, run on request. See `mutants.toml` for why they
# are out of the default scope.
mutants-ui:
	@command -v cargo-mutants >/dev/null 2>&1 || { echo "cargo-mutants not installed; skipping"; exit 0; }
	cargo mutants --package lipid-selecto-rs --package smellfish-rs --jobs 8 --timeout 300

# The default scope, listed without running it. Instant, and it is what to run
# after touching a mutated file: it shows what a change added before paying for
# the run.
mutants-list:
	@command -v cargo-mutants >/dev/null 2>&1 || { echo "cargo-mutants not installed; skipping"; exit 0; }
	cargo mutants --package json-count-rs --package cxsmiles-yoga --list

# ── Supply-chain hygiene (skip gracefully if a tool is not installed) ─────────

machete:
	@command -v cargo-machete >/dev/null 2>&1 && cargo machete || echo "cargo-machete not installed; skipping"

audit:
	@command -v cargo-audit >/dev/null 2>&1 && cargo audit || echo "cargo-audit not installed; skipping"

deny:
	@command -v cargo-deny >/dev/null 2>&1 && cargo deny check advisories bans licenses sources || echo "cargo-deny not installed; skipping"

outdated:
	@command -v cargo-outdated >/dev/null 2>&1 && cargo outdated --workspace --exit-code 1 || echo "cargo-outdated not installed; skipping"

# README sync: regenerate each crate README from README.tpl + source `//!`
# doc comments, lint, and diff against the checked-in README.md. If it reports
# "out of date", fix the source doc comments, then `just readme` to regenerate.
#
# Globs for `README.tpl` rather than naming crates: a crate with a template has
# its README generated from its `lib.rs` docs, and one without has no README to
# check. That is why the no-templates case is a pass rather than a failure, and
# why adding a crate cannot leave this recipe silently skipping it.
#
# The generated file is run through `panache format` before the diff, because a
# `panache-format` pre-commit hook reformats the committed README and raw
# `cargo readme` output does not. Diffing the raw output against the formatted
# file reports drift that does not exist, which is the same shape of failure as a
# check that cannot pass: it would be red from the day it was added.
readme:
	#!/usr/bin/env bash
	set -euo pipefail
	command -v cargo-readme >/dev/null 2>&1 || { echo "cargo-readme not installed; skipping"; exit 0; }
	command -v panache >/dev/null 2>&1 || { echo "panache not installed; skipping"; exit 0; }
	found=0
	for template in crates/*/README.tpl apps/*/README.tpl; do
	  [ -e "$template" ] || continue
	  dir=$(dirname "$template")
	  found=1
	  if ! ( cd "$dir" \
	         && cargo readme -t README.tpl -o /tmp/readme_panache.md 2>/dev/null \
	         && panache format /tmp/readme_panache.md > /dev/null \
	         && panache lint /tmp/readme_panache.md \
	         && diff -q /tmp/readme_panache.md README.md > /dev/null 2>&1 ); then
	    echo "README.md out of date for $dir" >&2
	    echo "  run: (cd $dir && cargo readme -t README.tpl -o README.md)" >&2
	    exit 1
	  fi
	done
	[ "$found" = 1 ] || echo "no crate READMEs to check"
