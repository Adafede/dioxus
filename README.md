# dioxus-apps

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-536-brightgreen)]() [![clippy: 0
warnings](https://img.shields.io/badge/clippy-0%20warnings-brightgreen)]()
[![WASM](https://img.shields.io/badge/WASM-5%20apps-brightgreen)]() [![cargo
deny](https://img.shields.io/badge/cargo%20deny-ok-brightgreen)]()
[![machete](https://img.shields.io/badge/machete-0%20unused-brightgreen)]()

A Cargo workspace for reproducible Dioxus web apps, pinned by
`rust-toolchain.toml`.

- **index** is the accessible landing page. **json-count-rs** counts non-null
  fields in uploaded JSON files.
- **lotus-explore-rs** explores the LOTUS compounds knowledge graph from
  Wikidata via SPARQL.
- **lotus-api** provides a native HTTP API for advanced search and export.
- **mgf-precursor-erro-rs** analyzes uploaded MGF files and reports precursor
  mass errors in Da and ppm.
- **lipid-selecto-rs** classifies and filters lipid mass-spec data using LIPID
  MAPS-aligned SMARTS rules.
- **smellfish-rs** scores natural-product-like structures with literature-backed
  features and RDKit.js chemistry descriptors.

## Prerequisites

```bash
rustup toolchain install 1.97 --profile minimal
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
```

The repo pins Rust 1.97, `clippy`, `rustfmt`, and `wasm32-unknown-unknown` in
`rust-toolchain.toml`.

## Structure

```
dioxus-apps/
├── Cargo.toml                ← workspace root
├── rust-toolchain.toml       ← pinned compiler, components, target
├── prek.toml                 ← repo hooks and quality gate
├── .github/                  ← CI, deploy, governance
├── apps/
│   ├── index/                ← accessible landing page (WASM)
│   ├── json-count-rs/        ← upload a JSON file and count non-null values (WASM)
│   ├── lipid-selecto-rs/     ← lipid classification and filtering via SMARTS (WASM)
│   ├── mgf-precursor-erro-rs/← MGF precursor mass-error analysis (WASM + lib)
│   ├── lotus-api/            ← OpenAPI service for LOTUS search and exports (native)
│   ├── lotus-explore-rs/     ← LOTUS Wikidata natural-product explorer (WASM)
│   └── smellfish-rs/         ← NP-likeness scoring, RDKit.js integration (WASM + lib)
└── crates/
    ├── lotus/                ← SPARQL client, LOTUS models, transport, export
    ├── upload/               ← shared file-upload, progress, and blob utilities
    └── ui/                   ← shared accessibility-focused UI helpers (DocumentHead, etc.)
```

Apps marked **(WASM + lib)** have a `lib.rs` alongside `main.rs` to enable
`cargo test --lib`. Apps without extensive unit tests use `main.rs` only.

## Running apps locally

```bash
dx serve --package lotus-explore-rs
cargo run --locked -p lotus-api
dx serve --package json-count-rs
dx serve --package index
dx serve --package lipid-selecto-rs
dx serve --package mgf-precursor-erro-rs
dx serve --package smellfish-rs
```

## Building for production

```bash
dx build --release --package lotus-explore-rs
dx build --release --package json-count-rs
dx build --release --package index
dx build --release --package lipid-selecto-rs
dx build --release --package mgf-precursor-erro-rs
dx build --release --package smellfish-rs
```

Output lands under `target/dx/<package>/release/web/public/`.

## Quality gate and local checks

Install the repo hooks once:

```bash
cargo install prek --locked
prek install
```

Run the repo quality gate manually:

```bash
prek run cargo-qa
```

The `cargo-qa` hook chain runs the following checks (matching CI):

```bash
prek run cargo-fmt-check         # rustfmt --all -- --check
prek run cargo-check             # cargo check --workspace --all-targets --locked
prek run cargo-clippy             # cargo clippy --workspace --all-targets --locked -- -D warnings
prek run cargo-test               # cargo test --workspace --all-targets --locked --quiet
prek run cargo-check-wasm-all     # cargo check for all 5 WASM apps
prek run cargo-doc               # cargo doc --workspace --no-deps --locked
prek run cargo-machete           # cargo machete check --workspace
prek run cargo-audit             # cargo audit
prek run cargo-deny              # cargo deny check advisories bans licenses sources
prek run cargo-readme-panache    # cargo-readme sync + panache lint
```

## Adding a new app

1. Copy an existing app directory (e.g. `apps/index`) as a starting point.
2. Edit `Cargo.toml` and `Dioxus.toml` to set `name` and `title`.
3. Add `"apps/my-new-app"` to `members` in the workspace `Cargo.toml`.
4. Add a `cargo check` line to the **wasm** job and a `dx build` line to the
   **wasm-build** job in `.github/workflows/ci.yml`.
5. `dx serve --package my-new-app`

## Continuous integration

On every push to `main`:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --all-targets --locked`
- `cargo check` for all 5 WASM apps (`json-count-rs`, `mgf-precursor-erro-rs`,
  `lipid-selecto-rs`, `lotus-explore-rs`, `smellfish-rs`)
- `cargo doc --workspace --no-deps --locked`
- `cargo machete check --workspace`
- `cargo audit`
- `cargo deny check advisories bans licenses sources`
- WASM build and deploy artifact for all 5 WASM apps (with Ketcher fetch for
  `lotus-explore-rs`)

## AI and agent docs

- [`AI_AGENT_GUIDE.md`](./AI_AGENT_GUIDE.md)
- [`PROJECT_METADATA.json`](./PROJECT_METADATA.json)
- [`apps/lotus-explore-rs/SKILLS.md`](./apps/lotus-explore-rs/SKILLS.md)

## Governance

- Contributing: [`.github/CONTRIBUTING.md`](./.github/CONTRIBUTING.md)
- AI contributions: [`.github/CONTRIBUTING_AI.md`](./.github/CONTRIBUTING_AI.md)
- Security: [`.github/SECURITY.md`](./.github/SECURITY.md)
- Release process:
  [`.github/RELEASE_CHECKLIST.md`](./.github/RELEASE_CHECKLIST.md)
- Change history: [`CHANGELOG.md`](./CHANGELOG.md)
- License: `LICENSE` (GNU AGPL v3.0)
