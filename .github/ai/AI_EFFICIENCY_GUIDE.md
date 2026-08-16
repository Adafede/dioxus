# AI Agent Efficiency Guide

How to drive the codemutation agent efficiently. It does its strongest work on
**small, ordered, natively-verifiable** steps — not room-temperature rewrites.

## Request shape that works well
- **One phase / one task per message.** Don't bundle
  "review X, do Y, plan Z, fix the build, verify". Batch a phase together, let
  the agent complete + gate it, *then* hand off the next.
- **State the target gate explicitly.** Native
  (`cargo clippy --workspace --all-targets --locked -- -D warnings`) vs wasm
  (`cargo check -p <pkg> --target wasm32-unknown-unknown --locked`). Mixing them
  stalls the native clippy/test gate, because `#[cfg(target_arch="wasm32")]`
  bodies can't compile natively and native bodies can't compile for wasm.
- **Give exact pointers** (file + line + the verified-before state you saw).
  Saves one exploration round-trip and avoids stale exact-match edits.
- **Let the agent finish + report one gate before stating the next.** Half a
  gate verified in two directions is worse than one whole gate.

## What the agent struggles with
- Large multi-phase rewrites in a single shot. On very long diffs the agent's
  context drifts → it re-reads stale strings and mis-edits. Fatigue shows up as
  "exact-match edit failed" loops.
- wasm-only behavioral reasoning. This environment has no wasm runtime and no
  `dx`/`wasm-pack`; only `cargo check --target wasm32-unknown-unknown`. Logic
  that only runs in the browser should be native-verifiable first, then wasm-checked.

## No tunable "hidden parameters"
There are no speed / temperature / parallelism knobs exposed. The single
highest-leverage input is **request granularity**: sequential, verified steps
beat one big ask.

## Suggested rhythm
1. One Phase (or one leftover task) at a time.
2. Agent runs the gate, reports green/red, asks what's next.
3. Repeat.
