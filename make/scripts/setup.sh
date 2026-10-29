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

# Lints and manifest hygiene.
install_tool cargo-edit 0.13.13   # `cargo upgrade`, `cargo add`, `cargo rm`
install_tool cargo-msrv 0.19.3    # the real MSRV, rather than the asserted one

# Supply chain.
# `cargo udeps` for unused dependencies the manifest scan cannot see. It is on
# the weekly schedule rather than the gate, so a slow or advisory-heavy run
# cannot turn the gate red. The transitive unsafe audit beside it used to be
# `cargo geiger` and is now `make/scripts/unsafe-audit.sh`, which needs nothing
# installed — `cargo-geiger` 0.13.0 cannot parse this workspace's manifests.
install_tool cargo-udeps 0.1.61

# Coverage and mutation testing. The nextest integration is what makes the
# coverage report describe the same run the suite performs.

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
# panache is installed by `./mk setup` itself (the `install-panache` task, listed
# in mk.local), and tombi with the other gate tools from mk.versions.

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
