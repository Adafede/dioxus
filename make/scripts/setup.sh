#!/usr/bin/env bash
# Install every tool the other tasks call, at a pinned version.
#
# `cargo binstall` is used where it is available because compiling these from
# source is minutes each and they are prebuilt binaries. It is not a
# requirement: `cargo install --locked --version` produces the same binary, just
# slower, so a contributor without binstall gets the same set, later.
#
# Versions are pinned rather than floating. A tool that changes its output
# between releases changes what a task prints, and a task whose output is
# asserted on -- `readme-check` and the JUnit report both are -- becomes a task
# whose failure is a version bump rather than a defect.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

# The toolchain itself, from rust-toolchain.toml. rustup reads that file, so
# this is one line and it cannot disagree with the pin.
echo "==> toolchain"
rustup show active-toolchain >/dev/null || rustup toolchain install

have_binstall=false
if command -v cargo-binstall >/dev/null 2>&1; then
  have_binstall=true
fi

# Named `install_tool`, not `install`: a shell function called `install` shadows
# the coreutils `install` for the rest of the script, and the tombi step below
# calls `install -m 755` — which then runs *this* function with `-m` as the crate
# name and fails with "unexpected argument '-m' found". Cost one confused run.
install_tool() {
  local crate="$1" version="$2"
  local spec="$crate@$version"
  echo "==> $spec"
  if $have_binstall; then
    # `--no-confirm` because this is a script, and a prompt nobody is there to
    # answer is a hang rather than a question.
    if cargo binstall --no-confirm "$spec"; then
      return
    fi
    echo "    binstall could not provide $spec; falling back to cargo install" >&2
  fi
  cargo install --locked "$crate" --version "$version"
}

# Test runner and task runner. Without these every other task that mentions
# tests, or that is a task at all, is wrong.
install_tool cargo-nextest 0.9.146
install_tool cargo-make 0.37.24

# Lints and manifest hygiene.
install_tool cargo-hack 0.6.45    # feature combinations
install_tool cargo-edit 0.13.13   # `cargo upgrade`, `cargo add`, `cargo rm`
install_tool cargo-msrv 0.19.3    # the real MSRV, rather than the asserted one
install_tool typos-cli 1.50.3    # spelling

# Supply chain.
install_tool cargo-deny 0.20.2
install_tool cargo-audit 0.22.2
install_tool cargo-machete 0.9.2
install_tool cargo-outdated 0.19.0
# `cargo geiger` for the transitive unsafe audit and `cargo udeps` for unused
# dependencies the manifest scan cannot see. Both are on the weekly schedule
# rather than the gate, so a slow or advisory-heavy run cannot turn the gate red.
install_tool cargo-geiger 0.13.0
install_tool cargo-udeps 0.1.61

# Coverage and mutation testing. The nextest integration is what makes the
# coverage report describe the same run the suite performs.
install_tool cargo-llvm-cov 0.9.1
install_tool cargo-mutants 27.1.0

# Build size. `wasm-opt` supersedes `twiggy` for wasm and `cargo bloat` covers
# the native side; neither is in the gate.
install_tool cargo-bloat 0.12.1

# The crate READMEs are generated from each crate's `README.tpl` plus its module
# docs, and the diff is taken against the `panache`-formatted file because a
# pre-commit hook formats them. Both tools are needed, in that order.
#
# `cargo-readme` 3.4.0 specifically: 3.2.0 fails on a manifest that inherits
# `version` from `[workspace.package]` with "invalid type: map, expected a string
# for key `package.version`", and every crate here inherits it. The check then
# reports drift against an empty document rather than saying why.
install_tool cargo-readme 3.4.0
# `panache` is installed with `--bin panache` and without the binstall attempt,
# because its release archives are missing a binary its own manifest declares as
# required: cargo-binstall exits 76 with "When resolving panache bin
# distill_quarto_schema is not found. This binary is not optional so it must be
# included in the archive". Nothing here calls `distill_quarto_schema`, and
# 3.13.0's archive ships `panache` only, so bumping does not fix it. `--bin`
# skips building the missing one. Same reason in the CI workflow.
echo "==> panache 3.13.0"
cargo install --locked panache --version 3.13.0 --bin panache

# TOMBI is the TOML formatter and linter, and the tool the git hooks run through
# `tombi-pre-commit`. It cannot be installed the way everything above is: the
# `tombi-cli` crate on crates.io is a 0.0.1 placeholder with no binary, so
# `cargo binstall tombi-cli@1.6.1` fails with "no version matching requirement"
# and `cargo install tombi-cli` produces nothing. The real one is a GitHub
# release, and `install-tombi.sh` is the one place that knows how to fetch it.
#
# The CI workflow calls the same script, because when its install list said
# `tombi-cli@1.6.1` the Gate job died with exactly that error -- the knowledge was
# written down here and not acted on there.
#
# The version is read from `prek.toml` so the hook and the installer cannot be
# pinned to different releases.
TOMBI_VERSION=$(sed -n '/tombi-pre-commit/{n;s/rev = "v\([^"]*\)"/\1/p;}' prek.toml | head -1)
: "${TOMBI_VERSION:?could not read the tombi version from prek.toml}"
TOMBI_VERSION="$TOMBI_VERSION" bash make/scripts/install-tombi.sh

# The Dioxus CLI, for the `web-*` and `cli` tasks. The version is read out of the
# lockfile rather than the manifest: the manifest says `0.7`, and the resolved
# `dioxus` the apps actually compile against is the patch in `Cargo.lock`. A `dx`
# built against a different patch than the app is a build that succeeds and an
# app that does not run, which is exactly the failure this reading avoids.
#
# `dioxus-cli` is the crate; `dx` is the binary it installs, and `dx` is not on
# crates.io, so `dx@$DX_VERSION` does not resolve.
DX_VERSION=$(awk '/^name = "dioxus"$/ { found = 1; next } found && /^version = / { gsub(/"/, "", $3); print $3; exit }' Cargo.lock)
: "${DX_VERSION:?could not read the resolved dioxus version from Cargo.lock}"
echo "==> dioxus-cli@$DX_VERSION"
if ! command -v dx >/dev/null 2>&1 || [ "$(dx --version 2>/dev/null | awk '{print $2}')" != "$DX_VERSION" ]; then
  cargo install dioxus-cli --version "$DX_VERSION" --locked
else
  echo "    dx is already $DX_VERSION"
fi

echo
echo "Installed. The tasks are:"
echo
./mk --list-all-steps
