#!/usr/bin/env bash
# Two checks about the task definitions themselves.
#
#   1. `./mk --list-all-steps` lists the gate's own tasks, from a clean
#      checkout, in the state a contributor is in when they ask what they can
#      run. If this file exists and says otherwise, either `extend` is no longer
#      the first key in `Makefile.toml` -- which cargo-make accepts silently, and
#      which makes every task "not found" -- or a task file does not parse.
#
#   2. Every tool the gate calls is a tool `./mk setup` installs. A gate
#      task that names a tool nothing installs fails on a clean machine with a
#      message about the change rather than about the missing tool, and on a
#      machine where the tool happens to exist it passes, so the two look like
#      different behaviours of the same check.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

echo "==> ./mk --list-all-steps"
list=$(./mk --list-all-steps 2>/dev/null)

# Every task in the `ci` dependency list must appear in the listing. That is the
# question this script is really asking: not "did the runner start" but "did it
# load the task files", which is the failure cargo-make reports as success.
gate=$(sed -n '/^\[tasks\."ci"\]/,/^\[/p' Makefile.toml | sed -n '/dependencies = \[/,/\]/p' \
  | sed '1s/^dependencies = \[//' | tr ',' '\n' | tr -d '[]" ' | grep -v '^$' || true)
if [ -z "$gate" ]; then
  echo "could not read the ci task's dependencies from Makefile.toml" >&2
  exit 1
fi
missing=""
for task in $gate; do
  case "$list" in
    *"$task"*) ;;
    *) missing="$missing $task" ;;
  esac
done
if [ -n "$missing" ]; then
  echo "these tasks are in the gate but the task runner does not list them:$missing" >&2
  echo "check that 'extend' is still the first key in Makefile.toml" >&2
  exit 1
fi
echo "    ${gate} listed"

# Task -> the crate or release `setup` installs for it. A gate task whose tool is
# not installed by `setup` is the drift this catches.
#
# The identifier is the crate name rather than the command, because that is what
# the `install <crate> <version>` lines in `setup.sh` name. `cargo fmt`,
# `cargo check`, `cargo clippy`, `cargo test` and `cargo doc` are the pinned
# toolchain itself -- rustup installs them from `rust-toolchain.toml`, so there is
# nothing for `setup` to install and nothing to check.
echo "==> gate tasks vs ./mk setup"
declare -a crate_of_task=(
  "fmt-check=rustup"
  "tombi-check=tombi"
  "tombi-lint=tombi"
  "typos=typos-cli"
  "check=rustup"
  "lint=rustup"
  "test=cargo-nextest"
  "test-doc=rustup"
  "doc=rustup"
  "check-wasm=rustup"
  "lint-wasm=rustup"
  "readme-check=cargo-readme"
  "feature-powerset=cargo-hack"
  "machete=cargo-machete"
  "deny=cargo-deny"
  "audit=cargo-audit"
)

undeclared=""
for entry in "${crate_of_task[@]}"; do
  task="${entry%%=*}"
  crate="${entry#*=}"
  if ! grep -qF "$crate" make/scripts/setup.sh; then
    undeclared="$undeclared\n  $task calls '$crate', which './mk setup' does not install"
  fi
done
if [ -n "$undeclared" ]; then
  printf "these gate tasks call a tool setup does not install:%b\n" "$undeclared" >&2
  exit 1
fi
echo "    every gate tool is installed by setup"

# `./mk test` must run the doctests too, or nextest alone reports a
# complete-looking run that never checked them. This is the assertion that keeps
# that true, and it is here rather than only in a test because a test can be
# deleted and this cannot be deleted silently.
echo "==> ./mk test depends on the doctests"
test_deps=$(sed -n '/^\[tasks\."test"\]/,/^\[/p' make/test.toml | sed -n '/dependencies = \[/,/\]/p' \
  | sed '1s/^dependencies = \[//' | tr ',' '\n' | tr -d '[]" ' | grep -v '^$')
case "$test_deps" in
  *test-doc*) echo "    test-doc is in the set" ;;
  *)
    echo "./mk test does not depend on test-doc, so the doctests are skipped" >&2
    exit 1
    ;;
esac

echo
echo "All task checks passed."