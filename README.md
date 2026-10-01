# dioxus-apps

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-536-brightgreen)]() [![clippy: 0
warnings](https://img.shields.io/badge/clippy-0%20warnings-brightgreen)]()
[![WASM](https://img.shields.io/badge/WASM-6%20apps-brightgreen)]() [![cargo
deny](https://img.shields.io/badge/cargo%20deny-ok-brightgreen)]()
[![machete](https://img.shields.io/badge/machete-0%20unused-brightgreen)]()

A Cargo workspace for reproducible Dioxus web apps, pinned by
`rust-toolchain.toml`.

- **index** is the accessible landing page. **json-count-rs** counts non-null
  fields in uploaded JSON files.
- **mgf-precursor-erro-rs** analyzes uploaded MGF files and reports precursor
  mass errors in Da and ppm.
- **lipid-selecto-rs** classifies and filters lipid mass-spec data using LIPID
  MAPS-aligned SMARTS rules.
- **cxsmiles-yoga** generates CX-SMILES from lists of related structures by
  collapsing positional isomers (m: blocks) and variable-length repeats (Sg:n:
  blocks).
- **smellfish-rs** scores natural-product-like structures with literature-backed
  features and RDKit.js chemistry descriptors.

## Prerequisites

```bash
./mk setup
```

That installs the pinned toolchain from `rust-toolchain.toml` (Rust 1.98.1, with
`clippy`, `rustfmt`, `llvm-tools-preview` and the `wasm32-unknown-unknown`
target), the task runner, the test runner, the Dioxus CLI at the version the
lockfile resolves, and every tool the gate calls — each at a pinned version,
through `cargo binstall` where it is available.

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
│   ├── smellfish-rs/         ← NP-likeness scoring, RDKit.js, QLever (WASM + lib)
│   └── cxsmiles-yoga/        ← CX-SMILES generation from related structures (WASM + lib)
└── crates/
    ├── upload/               ← shared file-upload, progress, and blob utilities
    └── ui/                   ← shared accessibility-focused UI helpers (DocumentHead, etc.)
```

The task runner's own configuration:

```
├── Makefile.toml            ← task runner: shared config and the ci/ci-fast gates
├── make/                    ← the tasks, split by concern
├── .config/nextest.toml     ← nextest profiles
└── .cargo/                  ← cargo config and the cargo-mutants exclusions
```

Apps marked **(WASM + lib)** have a `lib.rs` alongside `main.rs`, which is what
makes their logic testable from the host. Apps without extensive unit tests use
`main.rs` only.

## Running apps locally

```bash
./mk web-dev APP=cxsmiles-yoga       # or index, json-count-rs,
                                            # lipid-selecto-rs, mgf-precursor-erro-rs,
                                            # smellfish-rs
```

Anything else the Dioxus CLI can do goes through the passthrough task:

```bash
./mk cli -- --help
```

## Building for production

```bash
./mk web-build APP=cxsmiles-yoga      # one app
./mk web-build-all                    # all six
./mk web-size                         # report each bundle, raw and wasm-opt'd
```

Output lands under `apps/<package>/target/dx/<package>/release/web/public/`.

## Quality gate and local checks

The task runner is **cargo-make**. Install the tools and the git hooks once:

```bash
./mk setup
cargo install prek --locked && prek install
```

The gate is one command, and it is the same command CI runs:

```bash
./mk ci
```

`./mk ci-fast` is the pre-push subset (the gate without the
supply-chain checks). For the full current list of tasks rather than a copy of
it, run `./mk --list-all-steps` -- that is the one that cannot go stale.
The individual checks it depends on are also tasks: `fmt-check`, `lint`,
`check`, `test`, `doc`, `check-wasm`, `lint-wasm`, `machete`, `deny`, `audit`,
`typos`, `tombi-check`, `tombi-lint`, `license-headers`, `feature-powerset`,
`readme-check`.

The hooks in `prek.toml` delegate to those tasks rather than spelling out cargo
flags, so the two cannot drift.

## Adding a new app

1. Copy an existing app directory (e.g. `apps/index`) as a starting point.
2. Edit `Cargo.toml` and `Dioxus.toml` to set `name` and `title`.
3. Add `"apps/my-new-app"` to `members` in the workspace `Cargo.toml`.
4. Add the app to `check-wasm`, `lint-wasm`, `web-build-all` and the `MAPPING`
   row in `crates/upload/tests/gate_consistency.rs`. That test fails the build
   otherwise, which is the point of it.
5. `./mk web-dev APP=my-new-app`

## Continuous integration

Every job runs a `./mk` task, and the per-push gate is the single task
`./mk ci`, so what CI runs is what the local gate runs:

- **Gate** (`./mk ci`): formatting, TOML validity and schema, spelling,
  SPDX headers, task-list sanity, `cargo check`, clippy on the host and on
  wasm32, the test suite (nextest, then the doctests), rustdoc with warnings
  denied, the feature powerset, the generated-README check, and the
  supply-chain checks.
- **MSRV** (`./mk check`, `./mk test`): the same builds on the
  oldest toolchain `rust-version` claims to support.
- **WASM bundle** (`./mk web-build-all`, `./mk web-size`): the only
  job that links a `.wasm`, and the one that reports how large it is.
- **Scheduled**, weekly: mutation testing, coverage, and the dependency-drift
  questions (`outdated`, `udeps`, `geiger`, `msrv`).

Artifacts: the nextest JUnit report from every gate run, the coverage report,
the mutants report, and the built wasm bundles.

There is **no release workflow**, and deliberately so: every crate is
`publish = false` and the public APIs are still moving. Release automation,
semver gates and packaging checks are deferred until a first release is
actually planned — see the "Deferred" note in
[`.github/CONTRIBUTING.md`](./.github/CONTRIBUTING.md).

## MCP

A project-level MCP config lives at `.mcp/mcp.json` --- the canonical
`mcpServers` convention (read by Claude Code / Cline). It registers a read-only
filesystem MCP server exposing the AI collaboration guides (`.github/ai/`) and
the lotus-explore architecture docs (`apps/lotus-explore-rs/docs/`) as MCP
resources, so agent tooling can read them through the Model Context Protocol.
In-page web agents instead use the page's JSON-LD / `llms.txt` / `.well-known/`
discovery.

## Governance

- Contributing: [`.github/CONTRIBUTING.md`](./.github/CONTRIBUTING.md)
- AI contributions:
  [`.github/ai/CONTRIBUTING_AI.md`](./.github/ai/CONTRIBUTING_AI.md)
- Security: [`.github/SECURITY.md`](./.github/SECURITY.md)
- Release process:
  [`.github/RELEASE_CHECKLIST.md`](./.github/RELEASE_CHECKLIST.md)
- Change history: [`CHANGELOG.md`](./CHANGELOG.md)
- License: `LICENSE` (GNU AGPL v3.0)
