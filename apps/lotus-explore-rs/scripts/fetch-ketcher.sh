#!/usr/bin/env bash
# Fetch the latest Ketcher standalone build from GitHub releases.
#
# Ketcher is a ~115 MB React-based chemical structure editor that is loaded
# via an <iframe> in the lotus-explore-rs draw page.  Rather than vendoring
# the bundle in git (which bloats the repo by 115 MB), we download it on
# demand during development setup and before production builds.
#
# RDKit and citation-js are already loaded from CDN — see index.html.
#
# Usage:  ./scripts/fetch-ketcher.sh
#
# Environment:
#   KETCHER_VERSION  — pinned release (default: 3.17.0)
set -euo pipefail

KETCHER_DIR="public/assets/ketcher"
KETCHER_VERSION="${KETCHER_VERSION:-3.17.0}"
KETCHER_URL="https://github.com/petrucci42/ketcher/releases/download/v${KETCHER_VERSION}/ketcher-${KETCHER_VERSION}.zip"

if [ -f "$KETCHER_DIR/index.html" ]; then
    echo "✓ Ketcher v${KETCHER_VERSION} already present in $KETCHER_DIR"
    echo "  (set KETCHER_VERSION to upgrade, then re-run)"
    exit 0
fi

echo "Downloading Ketcher v${KETCHER_VERSION}..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -L --fail -o "$TMPDIR/ketcher.zip" "$KETCHER_URL"
mkdir -p "$KETCHER_DIR"
unzip -q "$TMPDIR/ketcher.zip" -d "$KETCHER_DIR"

echo "✓ Ketcher v${KETCHER_VERSION} extracted to $KETCHER_DIR"
echo "  Run 'dx serve --package lotus-explore-rs' to use it."
