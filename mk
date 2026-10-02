#!/usr/bin/env bash
# The task runner, with the one flag this repository needs.
#
# Run `./mk <task>`. Nothing else in this repository invokes `cargo make`
# directly.
#
# Why this exists rather than a `cargo make` invocation in the docs: cargo-make
# 0.37 enables "workspace support", which re-runs the requested task once per
# workspace member with the working directory set to that member. This
# workspace has eight members, so
#
#   cargo make tombi-check       7.3s   (8 executions of a 0.6s task)
#   ./mk tombi-check             0.6s
#
# and the whole gate:
#
#   cargo make ci                ~2 min
#   ./mk ci                      ~17s
#
# Cargo tasks hid most of it, because the second `cargo check` of an unchanged
# workspace is a cache hit — so the 420-test suite ran to 420 passing, twice,
# then again six more times. Everything that is not cargo paid the full factor:
# tombi, typos, the license-header scan, `cargo deny`, `cargo audit` and `setup`,
# each of which fetches a database or walks the tree.
#
# There is no configuration key and no environment variable for this in 0.37.
# `CARGO_MAKE_CRATE_IS_WORKSPACE` is read from `cargo metadata` before the
# makefile's own `[env]` is applied, so it cannot be set from there; both it and
# `CARGO_MAKE_WORKSPACE_EMULATION` and `CARGO_MAKE_WORKSPACE_SKIP_MEMBERS` were
# tried and all three left the fan-out in place. The flag is the supported
# mechanism, and this is the one place it is written.
#
# Arguments are forwarded verbatim, so `./mk ci` and `./mk ci -- --profile ci`
# both work and nothing is hidden behind the wrapper.
set -euo pipefail
exec cargo make --no-workspace "$@"
