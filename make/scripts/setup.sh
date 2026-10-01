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

install() {
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
install cargo-nextest 0.9.146
install cargo-make 0.37.24

# Lints and manifest hygiene.
install cargo-hack 0.6.45    # feature combinations
install cargo-edit 0.13.13   # `cargo upgrade`, `cargo add`, `cargo rm`
install cargo-msrv 0.19.3    # the real MSRV, rather than the asserted one
install typos-cli 1.50.3    # spelling

# Supply chain.
install cargo-deny 0.20.2
install cargo-audit 0.22.2
install cargo-machete 0.9.2
install cargo-outdated 0.19.0
# `cargo geiger` for the transitive unsafe audit and `cargo udeps` for unused
# dependencies the manifest scan cannot see. Both are on the weekly schedule
# rather than the gate, so a slow or advisory-heavy run cannot turn the gate red.
install cargo-geiger 0.13.0
install cargo-udeps 0.1.61

# Coverage and mutation testing. The nextest integration is what makes the
# coverage report describe the same run the suite performs.
install cargo-llvm-cov 0.9.1
install cargo-mutants 27.1.0

# Build size. `wasm-opt` supersedes `twiggy` for wasm and `cargo bloat` covers
# the native side; neither is in the gate.
install cargo-bloat 0.12.1

# The crate READMEs are generated from each crate's `README.tpl` plus its module
# docs, and the diff is taken against the `panache`-formatted file because a
# pre-commit hook formats them. Both tools are needed, in that order.
install cargo-readme 3.4.0
install panache 3.12.0

# TOML is handled by tombi, which is the tool the git hooks already run through
# `tombi-pre-commit`. It is NOT on crates.io: the `tombi-cli` crate there is a
# 0.0.1 placeholder with no binary, so `cargo install tombi-cli` produces
# nothing. The real binary is a GitHub release, and the version is read from
# `prek.toml` so the hook and this cannot be pinned to different releases -- the
# same reasoning as `dx` below, and for the same reason.
TOMBI_VERSION=$(sed -n '/tombi-pre-commit/{n;s/rev = "v\([^"]*\)"/\1/p;}' prek.toml | head -1)
: "${TOMBI_VERSION:?could not read the tombi version from prek.toml}"
echo "==> tombi ${TOMBI_VERSION}"
if ! command -v tombi >/dev/null 2>&1 || [ "$(tombi --version 2>/dev/null)" != "tombi ${TOMBI_VERSION} "* ]; then
  case "$(uname -s)/$(uname -m)" in
    Darwin/arm64) tombi_asset="tombi-cli-${TOMBI_VERSION}-aarch64-apple-darwin.tar.gz" ;;
    Darwin/x86_64) tombi_asset="tombi-cli-${TOMBI_VERSION}-x86_64-apple-darwin.tar.gz" ;;
    Linux/aarch64|Linux/arm64) tombi_asset="tombi-cli-${TOMBI_VERSION}-aarch64-unknown-linux-musl.tar.gz" ;;
    Linux/x86_64) tombi_asset="tombi-cli-${TOMBI_VERSION}-x86_64-unknown-linux-musl.tar.gz" ;;
    *) echo "no tombi release asset for $(uname -s)/$(uname -m)" >&2; exit 1 ;;
  esac
  tombi_dir=$(mktemp -d)
  curl -fsSL --retry 3 --retry-all-errors \
    "https://github.com/tombi-toml/tombi/releases/download/v${TOMBI_VERSION}/${tombi_asset}" \
    | tar xz -C "$tombi_dir"
  install -m 755 "$tombi_dir"/*/tombi "${CARGO_HOME:-$HOME/.cargo}/bin/tombi"
  rm -rf "$tombi_dir"
else
  echo "    tombi is already ${TOMBI_VERSION}"
fi

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
cargo make --list-all-steps