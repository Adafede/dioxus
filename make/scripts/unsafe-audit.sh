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
# What is here instead asks the same question with the tool that already works:
# `cargo metadata`, which reads those manifests correctly because it is Cargo.
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

cargo metadata --format-version 1 --locked \
  | python3 "$(dirname "$0")/unsafe-audit.py"