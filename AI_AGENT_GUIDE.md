# AI Agent Guide

## Workspace map

- `apps/lotus-explorer` — Dioxus WASM explorer for LOTUS linked-open-data workflows.
- `apps/lotus-api` — native Axum API for search execution, export URLs, and exports.
- `crates/shared` — shared SPARQL and LOTUS data helpers.

## Stable commands

```bash
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
```

```bash
dx serve --package lotus-explorer
cargo run -p lotus-api
```

## Change protocol

1. Read the smallest relevant module set before editing.
2. Keep changes local and preserve existing public contracts.
3. Update tests, docs, and skill indexes when boundaries change.
4. Verify with format, check, test, and lint commands.

## Safety rules

- Prefer deterministic behavior over hidden state.
- Do not add new dependencies unless required by the architecture.
- Keep user-facing behavior stable unless a change is explicitly requested.
- Favor typed contracts, explicit errors, and narrow ownership.

## References

- Architecture: `apps/lotus-explorer/docs/ARCHITECTURE.md`
- Skills: `apps/lotus-explorer/SKILLS.md`
- AI contribution guide: `.github/CONTRIBUTING_AI.md`
