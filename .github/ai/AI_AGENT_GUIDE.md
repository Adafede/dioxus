# AI Agent Guide

## Workspace Map

### Shared Crates (`crates/`)

- `crates/ui` --- Pure-Rust Dioxus design system re-exported via `ui::prelude`:
  components (`Button`, `ButtonVariant`, `Card`, `Footer`, `Header`,
  `NoticeBar`, `NoticeTone`, `SegmentedControl`, `SegmentedControlItem`,
  `SimpleAnalyticsBadge`, `UploadZone`), `DocumentHead`, `skip_link`,
  `SKIP_LINK_STYLE`, `WebMcpConfig`, `EdamAnnotations`, `EdamTerm`, and theme
  tokens (`StyleBuilder`,
  `theme::{ColorScheme, Interaction, Radius, Shadow, Spacing, Typography}`).
  Apps must use `StyleBuilder` or local style helpers instead of raw
  `style: "..."` strings. `ui::shared_signal!` / `ui::shared_signals!` collapse
  host vs. `wasm32` `use_signal` mutability pairs.
- `crates/upload` --- Streaming file I/O and downloads for WASM apps:
  `BlobCursor`, `BlobLines`, `BlobSource` (`wasm32`), `ChunkSource`,
  `SliceSource`, `UploadError`, `extract_blob_from_file_data`, `Blob`,
  `ExtractedFile`, and `download_text` / `download_svg` / `download_with_mime`.
  `crates/upload/tests/gate_consistency/` holds the repository-wide structural,
  documentation, and ratchet integration tests.

### Applications (`apps/`)

- `apps/index` --- Portal landing page (`wasm32`).
- `apps/json-count-rs` --- Streaming JSON key/non-null counter (`wasm32` + host
  streaming tests).
- `apps/lipid-selecto-rs` --- MGF SMILES lipid classifier, SVG grid, and MGF
  filter (`wasm32` + host library tests).
- `apps/cxsmiles-yoga` --- CX-SMILES generator for congeneric and positional
  series (`wasm32` + host graph/roundtrip tests).
- `apps/smellfish-rs` --- NP-likeness scorer with RDKit.js and serialized QLever
  SPARQL enrichment (`wasm32` + host CSV/verdict/query tests).
- `apps/mgf-precursor-erro-rs` --- MGF precursor mass-error analysis, plotting,
  and mass recalibration (`wasm32` + host parser/calibration/plotting tests).

### Excluded Standalone Binary

- `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction` --- Listed under
  `exclude` in root `Cargo.toml`, with its own `[workspace]` table and committed
  `Cargo.lock`.

## Verification and Build Commands

- Inspect all custom `cargo-make` tasks with `./mk --list` (`Makefile.toml` and
  `make/*.toml`); invoke non-default `cargo-make` tasks via `./mk local:<task>`.
- WASM bundle builds:

```bash
./mk setup-extra
./mk web-dev APP=cxsmiles-yoga
./mk web-build-all
./mk web-size
dx build --profile wasm-release --platform web --package cxsmiles-yoga
```

- Excluded standalone binary checks:

```bash
cargo clippy --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --locked
```

## Executable Invariants (Never Break)

1. **SPDX headers**: Every `.rs` file starts with the 2-line header checked by
   `./mk local:license-headers`:
   - `// SPDX-License-Identifier: AGPL-3.0-only`
   - `// SPDX-FileCopyrightText: Contributors to the dioxus-apps project`
2. **800-line file limit**: No `.rs` file in the repository (including
   `smarts-evoliposuction`) may exceed 800 lines unless explicitly justified in
   `gate_consistency::apps::OVERSIZED`.
3. **Accessibility landmarks**: Every app pairs `skip_link` with
   `id: "main-content"` (`gate_consistency::apps`).
4. **Workspace lint discipline**: Root `Cargo.toml` forbids `unsafe_code` and
   denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`,
   `unimplemented`, `missing_assert_message`, and `clippy::pedantic` across
   libs, bins, and test targets.
5. **No ignored doctests**: `///` and `//!` fences under `apps/` and `crates/`
   must not use `ignore` (`gate_consistency::surfaces`).
6. **Documentation surface separation**: Human docs (`README.md`,
   `CHANGELOG.md`, `docs/*.md`, `.github/*.md`, `.github/ISSUE_TEMPLATE/*`,
   `.vscode/README.md`, and `.md`/`.tpl` files under `apps/` and `crates/`) must
   never cite `AGENTS.md`, `CLAUDE.md`, `AGENT_AUDIT.md`, `.github/ai`, `.mcp`,
   or `graphify`, must not contain marketing fluff, and all relative markdown
   links must resolve on disk (`gate_consistency::surfaces`).
7. **Generated crate READMEs**: Crate/app `README.md` files paired with
   `README.tpl` are generated from `//!` crate docs via
   `./mk local:readme-write` and checked by `./mk local:readme-check`.
8. **Ratchets**: `COVERAGE_FAIL_UNDER_LINES`, `ECHOES_MAX_APPROXIMATE`, and
   `ECHOES_MAX_REUSE` in `mk.local`, plus `[workspace.lints.*]` `= "allow"`
   count in `Cargo.toml`, may only tighten relative to base (`./mk ratchet`) and
   must match `docs/DEBT.md` (`gate_consistency::surfaces`).

## Audit Pitfalls to Avoid

- **Host builds skip `wasm32` branches**: Always run `./mk wasm` and
  `./mk local:lint-wasm` after editing `ui`, `upload`, or any app component.
- **`nextest` skips doctests**: Run `./mk test` (nextest + `cargo test --doc`),
  never only `nextest`.
- **Duplicate detection**: `./mk echoes` rejects duplicate doctests, identical
  functions, and duplicate context blocks; extract shared parameterized helpers
  instead of copy-pasting blocks.
- **Branch symbol drift (`./mk refquiem`)**: `refquiem` fails on removed item
  names across history AND denies untouched files mentioning items whose value
  or signature changed on the branch (`[changed] level = "deny"`). Update
  doc/comment references in the same commit as signature changes, and never cite
  deleted symbol names in prose or code blocks.
- **Standalone `--locked` builds**: Always pass `--locked` to Cargo for
  `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction` so its pinned
  `Cargo.lock` keeps compatible transitive `smiles-rs` versions across
  `smarts-rs` and `smarts-evolution`.
- **QLever / WDQS burst limits (`apps/smellfish-rs/src/sparql.rs`)**: Serialize
  chunked `VALUES` queries, cap transient retries to one attempt, respect HTTP
  429 responses, and never issue concurrent POST bursts against public SPARQL
  endpoints.
- **Pinned tool drift**: If `./mk tools` or `./mk tombi` reports a version
  mismatch against `mk.versions`, run `./mk setup`.
