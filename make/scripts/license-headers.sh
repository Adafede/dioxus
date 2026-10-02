#!/usr/bin/env bash
# Every `.rs` file in the workspace carries both AGPL SPDX headers, on lines 1
# and 2.
#
# `target/` and `graphify-out/` are build output and generated graph data
# respectively -- neither is source. The copyright line is checked as a prefix
# rather than an exact string because each crate names its own project: the
# licence has to be identical everywhere, the copyright holder may not be.
set -euo pipefail

missing=0
while IFS= read -r f; do
  if ! head -1 "$f" | grep -qxF '// SPDX-License-Identifier: AGPL-3.0-only' \
     || ! head -2 "$f" | tail -1 | grep -qE '^// SPDX-FileCopyrightText: Contributors to the .+$'; then
    echo "missing or misplaced SPDX header: $f" >&2
    missing=1
  fi
done < <(find . \( -name target -o -name graphify-out -o -name .git \) -prune -o -name '*.rs' -print)
exit "$missing"