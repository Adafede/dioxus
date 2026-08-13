# AI Agent Guide

## Workspace map

The workspace is a Cargo monorepo with two top-level groups plus the crates that
back them.

**Shared crates (`crates/`)** --- consumed by the apps; the source of truth, not
the apps:

- `crates/lotus` --- LOTUS domain models, SPARQL query builders, and
  platform-agnostic SPARQL-over-HTTP transport. The single shared data core for
  `lotus-api` and `lotus-explore-rs`.
- `crates/ui` --- unified, type-safe Dioxus design system: reusable components
  (`Button`, `Card`, `Footer`, `Header`, `NoticeBar`, `SegmentedControl`),
  `DocumentHead`/`DocumentLinks`, pure-Rust style builders (`styles::*`), and
  theme primitives (`theme::*`). All apps depend on `ui`; prefer it over
  inlining styles.
- `crates/ui::signals` --- the `shared_signal!` / `shared_signals!` macros
  collapse the repeated `#[cfg(target_arch = "wasm32")] let x = use_signal(...)`
  / `#[cfg(not(...))] let mut x = use_signal(...)` pair into a single
  declaration that preserves per-platform `mut` semantics. New components should
  declare signals through these macros.
- `crates/upload` --- WASM streaming file I/O (`BlobCursor`, `BlobLines`),
  throttled progress, and unified download helpers. The shared upload/download
  crate for every upload-based WASM app.

**Applications (`apps/`)** --- thin shells over the shared crates:

- `apps/index` --- accessible landing page (WASM).
- `apps/json-count-rs` --- count non-null fields in uploaded JSON (WASM).
- `apps/lipid-selecto-rs` --- lipid classification & filtering via SMARTS
  (WASM).
- `apps/cxsmiles-yoga` --- CX-SMILES generation from related structures (WASM).
- `apps/smellfish-rs` --- NP-likeness scoring with RDKit.js + QLever (WASM).
- `apps/lotus-explore-rs` --- LOTUS Knowledge Explorer, LOTUS/Wikidata/QLever
  SPARQL explorer (WASM). Its `src/` is layered: `main.rs` (facades: `api`,
  `models`, `queries`, `sparql`, `state`, `repositories`, `services`,
  `curation`, `ui`) wires up `src/features/` (`explore` engine, `curation`
  workflow) and `src/components/`/`src/pages/` (UI).
- `apps/mgf-precursor-erro-rs` --- MGF precursor mass-error analysis (WASM +
  lib).
- `apps/lotus-api` --- native Axum API for LOTUS search and exports.

## Stable commands

```bash
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
```

```bash
dx serve --package lotus-explore-rs
cargo run --locked -p lotus-api
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
- Prefer `crates/ui` components/styles and `crates/upload`/`crates/lotus` over
  reimplementing the same concern in an app.

## References

- Architecture: `apps/lotus-explore-rs/docs/ARCHITECTURE.md`
- Skills: `apps/lotus-explore-rs/SKILLS.md`
- AI contribution guide: `.github/CONTRIBUTING_AI.md`
