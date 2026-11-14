# AI Agent Efficiency Guide

## Iteration Order (Cheapest Check First)

Run checks from narrowest/fastest to broadest workspace gate:

1. `./mk check` or `cargo check -p <pkg> --all-targets --locked` while
   iterating.
2. `cargo nextest run -p <pkg> --locked` (or
   `cargo test -p <pkg> --doc --locked` when touching `//!` docs).
3. `./mk local:lint-wasm` (or
   `cargo check -p <pkg> --target wasm32-unknown-unknown --locked`) for
   `#[cfg(target_arch = "wasm32")]` changes.
4. `./mk echoes` and `./mk refquiem` after refactoring symbols or deleting code.
5. `./mk ci-fast` before pushing; `./mk ci` before finishing.

## Design for Fast Verification

- Keep parsing, chemistry, scoring, and formatting logic free of browser APIs so
  it compiles and runs under native host `nextest` without a browser runtime.
- Keep `#[cfg(target_arch = "wasm32")]` glue thin around pure Rust helpers (for
  example, feeding `crates/upload`'s `BlobLines` via `ChunkSource` /
  `SliceSource` in host tests).
- Batch related code edits before triggering full-workspace checks (`./mk ci`),
  and regenerate `//!`-backed `README.md` files once per documentation batch
  with `./mk local:readme-write`.
