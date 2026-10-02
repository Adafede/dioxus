#!/usr/bin/env bash
# Install `tombi`, the TOML formatter and linter, from its GitHub release.
#
# It cannot come from install-action or `cargo install`, and the reason is worth
# writing down because both were tried:
#
# * `cargo install tombi-cli` produces nothing. The `tombi-cli` crate on
#   crates.io is a 0.0.1 placeholder with no binary; the real one is a GitHub
#   release. `cargo binstall tombi-cli@1.6.1` fails with "no version matching
#   requirement '=1.6.1'" for the same reason.
#
# The version is passed in and is not defaulted, because two places pin it --
# `prek.toml` (the hook) and the CI workflow's install list -- and a default here
# would be a third. Read the version from `prek.toml` and pass it.
#
# Set TOMBI_VERSION, or leave it unset and have the caller set it.
set -euo pipefail

: "${TOMBI_VERSION:?set TOMBI_VERSION to the tombi release to install}"

want="tombi ${TOMBI_VERSION} "
if command -v tombi > /dev/null 2>&1 && [[ "$(tombi --version 2>/dev/null)" == "${want}"* ]]; then
  echo "==> tombi is already ${TOMBI_VERSION}"
  exit 0
fi

case "$(uname -s)/$(uname -m)" in
  Darwin/arm64) asset="tombi-cli-${TOMBI_VERSION}-aarch64-apple-darwin.tar.gz" ;;
  Darwin/x86_64) asset="tombi-cli-${TOMBI_VERSION}-x86_64-apple-darwin.tar.gz" ;;
  Linux/aarch64 | Linux/arm64) asset="tombi-cli-${TOMBI_VERSION}-aarch64-unknown-linux-musl.tar.gz" ;;
  Linux/x86_64) asset="tombi-cli-${TOMBI_VERSION}-x86_64-unknown-linux-musl.tar.gz" ;;
  *)
    echo "no tombi release asset for $(uname -s)/$(uname -m)" >&2
    exit 1
    ;;
esac

echo "==> tombi ${TOMBI_VERSION} (${asset})"
dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT
curl -fsSL --retry 3 --retry-all-errors \
  "https://github.com/tombi-toml/tombi/releases/download/v${TOMBI_VERSION}/${asset}" \
  | tar xz -C "$dir"
install -m 755 "$dir"/*/tombi "${CARGO_HOME:-$HOME/.cargo}/bin/tombi"
echo "==> tombi $(tombi --version)"