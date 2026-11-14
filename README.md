# dioxus-apps

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)
[![Dependency
review](https://github.com/Adafede/dioxus/actions/workflows/dependency-review.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/dependency-review.yml)

Cargo workspace for six client-side Dioxus WebAssembly applications in chemistry
and mass spectrometry, backed by two shared crates (`ui` and `upload`) and
pinned to Rust `1.100.0` via `rust-toolchain.toml`.

- **index**: accessible landing page linking all applications in the suite.
- **json-count-rs**: streaming non-null field counter for uploaded JSON files.
- **mgf-precursor-erro-rs**: MGF precursor *m/z* recalibration, mass-error
  analysis in Da and ppm, and SVG histogram plotting.
- **lipid-selecto-rs**: browser-side MGF/SMILES lipid classification and
  filtering using 86 compiled-in LIPID MAPS SMARTS rules
  (`src/chemical_class/lmsd.rs`).
- **cxsmiles-yoga**: CX-SMILES generator and coordinate depiction tool
  collapsing positional isomers (`m:`) and variable-length repeats (`Sg:n:`).
- **smellfish-rs**: natural-product likeness scoring combining machine-learned
  features, RDKit.js descriptors, and QLever/LOTUS literature/SPARQL enrichment.

## Prerequisites

Install the pinned toolchain, `cargo-make`, `cargo-nextest`, `cargo-readme`,
`panache`, and all verification tools:

```bash
./mk setup
```

To install optional app development tools such as `dx` (`dioxus-cli`, pinned to
match `Cargo.lock`) for serving and bundling WASM apps locally:

```bash
./mk setup-extra
```

## Workspace structure

```text
dioxus/
├── Cargo.toml                workspace root
├── rust-toolchain.toml       pinned compiler, components, and wasm32 target
├── prek.toml                 git hooks calling ./mk gates
├── .github/                  CI workflows, governance, and templates
├── apps/
│   ├── index/                landing page (WASM + host tests)
│   ├── json-count-rs/        streaming JSON field counter (WASM + host tests)
│   ├── lipid-selecto-rs/     lipid classification & filtering (WASM + host tests;
│   │                         offline SMARTS generator under lipidmaps/smarts-evoliposuction)
│   ├── mgf-precursor-erro-rs/ MGF precursor mass-error analysis (WASM + rlib)
│   ├── smellfish-rs/         NP-likeness scoring and LOTUS/QLever lookup (WASM + host tests)
│   └── cxsmiles-yoga/        CX-SMILES generation from related structures (WASM + host tests)
└── crates/
    ├── upload/               chunked streaming readers (BlobCursor, BlobLines),
    │                         ChunkSource/SliceSource, and browser download helpers
    └── ui/                   shared Dioxus components (DocumentHead, UploadZone, etc.),
                              pure-Rust StyleBuilder, theme tokens, and signal macros
```

Task runner configuration lives in:

```text
├── Makefile.toml            shared cargo-make configuration
├── mk / mk.local / mk.versions standard gate runner and workspace overrides
├── make/                    cargo-make tasks split by concern
├── .config/nextest.toml     nextest profiles
└── .cargo/                  cargo and cargo-mutants configuration
```

## Running apps locally

Serve any of the six apps with the Dioxus CLI:

```bash
./mk web-dev APP=cxsmiles-yoga
```

Valid `APP` values are `cxsmiles-yoga`, `index`, `json-count-rs`,
`lipid-selecto-rs`, `mgf-precursor-erro-rs`, and `smellfish-rs`.

To pass arbitrary arguments directly to `dx`:

```bash
./mk cli ARGS="--help"
```

## Building for production

Production builds pass `--profile wasm-release` (`panic = "unwind"`,
`opt-level = "s"`) so third-party SMILES parser panics caught via `catch_unwind`
recover cleanly instead of aborting in the browser:

```bash
./mk web-build APP=cxsmiles-yoga      # one app (optional BASE_PATH=/subpath)
./mk web-build-all                    # all six apps
./mk web-size                         # report shipped *_bg-*.wasm byte sizes
```

Outputs are written to `target/dx/<package>/release/web/public/`.

## Quality gate and local checks

Install the verification tools and configure the git hooks once:

```bash
./mk setup
cargo install prek --locked && prek install
```

Run the fast inner check while editing, or the full CI gate before pushing:

```bash
./mk check      # format check, TOML check, typos check, and host clippy
./mk ci-fast    # pre-push subset (skips audit and mutants-diff)
./mk ci         # full CI gate
```

Run `./mk --list` to list the standard `./mk` gates and `local:*` tasks, or
`cargo make --no-workspace --list-all-steps` for every task in `make/*.toml`.

Core `crates/upload`, `crates/ui`, and app logic modules run unit, integration,
and doc tests on the host (`./mk test`). Because host builds skip
`#[cfg(target_arch = "wasm32")]` code, always run `./mk local:lint-wasm` and
`./mk wasm` (mapped to `check-wasm`) before pushing WASM changes.

The offline SMARTS generator CLI at
`apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction` is intentionally
excluded from the workspace so native-only bioinformatics dependencies never
enter WASM builds. Build and test it separately with `--locked`:

```bash
cargo clippy --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --locked
```

## Adding a new app

1. Start from an existing app directory such as `apps/index`.
2. Set the package `name` in `Cargo.toml` and `title` in `Dioxus.toml`.
3. Add `"apps/<new-app>"` to `[workspace].members` in the root `Cargo.toml`.
4. Wire the package into `make/build.toml` (`check-wasm`), `make/lint.toml`
   (`lint-wasm`), `make/web.toml` (`web-build-all`, `web-dev`, `web-build`), and
   `.github/workflows/deploy.yml`.
5. Ensure the app defines `DocumentHead`, `skip_link`, `id: "main-content"`,
   `crates/ui` components, and EDAM annotations, which are checked by
   `crates/upload/tests/gate_consistency/`.
6. Run `./mk web-dev APP=<new-app>` and `./mk ci`.

## Continuous integration

Workflows under `.github/workflows/` run through `./mk`:

- **CI (`ci.yml`)**: runs `./mk ci` (formatting, TOML check/lint, spelling, SPDX
  headers, shellcheck, commit format, ratchet check, host
  check/clippy/tests/doctests/doc, dependency audit/deny/machete, `msrv`,
  `echoes`, `refquiem`, `semver`, `wasm`, `mutants-diff`, `local:readme-check`,
  `local:lint-wasm`, `local:feature-powerset`) and links all six WASM bundles
  via `./mk web-build-all` + `./mk web-size`.
- **Nightly (`nightly.yml`)**: scheduled daily `./mk ci-nightly` (`mutants`,
  `coverage`, `powerset`, `miri`, `outdated`).
- **Deploy (`deploy.yml`)**: builds all six apps with `BASE_PATH` and deploys to
  GitHub Pages on pushes to `main`.
- **Release (`release.yml`)**: runs `./mk ci-release` on `v*` tags (skips
  crates.io publishing while every crate sets `publish = false`).

## Known limitations

- **WASM UI runtime tests**: host tests exercise pure domain logic (`cxsmiles`,
  `mgf`, `lipid-selecto-rs`, `json-count-rs`, `smellfish-rs` SPARQL/CSV/model,
  `upload` streaming, `ui` VirtualDom rendering), whereas browser DOM
  interactions (`web-sys` `Blob`, drag/drop events, live QLever/LIPID MAPS HTTP
  requests, and RDKit.js JS interop) are checked for `wasm32` compilation and
  linting but require browser testing at runtime.
- **Excluded CLI lockfile**: `smarts-evoliposuction` depends on
  `smarts-evolution` (`smiles-rs` 0.1.5) while `smarts-rs` `main` has moved to
  `smiles-rs` 0.2.1; always pass `--locked` rather than regenerating its
  `Cargo.lock`.

## Governance

- Contributing: [`.github/CONTRIBUTING.md`](./.github/CONTRIBUTING.md)
- Security: [`.github/SECURITY.md`](./.github/SECURITY.md)
- Release process:
  [`.github/RELEASE_CHECKLIST.md`](./.github/RELEASE_CHECKLIST.md)
- Architecture: [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md)
- Technical debt & ratchets: [`docs/DEBT.md`](./docs/DEBT.md)
- Change history: [`CHANGELOG.md`](./CHANGELOG.md)
- License: GNU AGPL v3.0 (`AGPL-3.0-only`)
