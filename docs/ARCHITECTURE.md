# Architecture

This repository is a Cargo monorepo of six client-side Dioxus WebAssembly
chemistry and mass-spectrometry web applications backed by shared domain and
web-adapter crates under `crates/`, plus one excluded standalone CLI binary.

## Layering Rule

Dependencies strictly flow downward in one direction:

```text
Layer 2: Browser Applications (apps/*) & Standalone CLI
            │  depends on Layer 1 + Layer 0
            ▼
Layer 1: Web Adapters, UI Design System & Workspace Gates
            │  (crates/ui, crates/upload, crates/workspace-gates)
            │  depends on Layer 0 + browser/Dioxus APIs
            ▼
Layer 0: Pure Domain & Streaming Core Crates
            │  (stream-io, mgf, chem-depict, cxsmiles-core,
            │   json-count-core, lipid-core, mgf-analysis-core, smellfish-core)
            │  ZERO dioxus, ZERO web-sys, host + wasm32 portable
```

Layer 0 crates contain pure data types, parsers, algorithms, and formatting.
They compile and run natively under `cargo nextest`, `cargo test --doc`, and
`cargo-mutants` without browser APIs or `#[cfg(target_arch = "wasm32")]` stubs,
while remaining `wasm32-unknown-unknown` compatible.

## Crate Responsibilities

### Layer 0 — Pure Domain Crates (`crates/`)

  | Crate               | One-Line Responsibility                                                                      |
  | ---                 | ---                                                                                          |
  | `stream-io`         | Async chunked byte and line streaming over `ChunkSource` with throttled progress tracking    |
  | `mgf`               | Streaming MGF spectrum block parser and header/peak model                                    |
  | `chem-depict`       | Panic-guarded `chematic` SMILES parsing and 2D SVG depiction                                 |
  | `cxsmiles-core`     | CX-SMILES positional (`m:`) and repeating (`Sg:n:`) generation and round-trip verification   |
  | `json-count-core`   | Streaming RFC 8259 JSON non-null field counter, string unescaper, and result sorting         |
  | `lipid-core`        | LIPID MAPS taxonomy, 86 LMSD SMARTS rules, filtering, tagging, and MGF/SMILES export         |
  | `mgf-analysis-core` | Adduct mass grammar, precursor mass-error statistics, MS2 recalibration, and diagnostics     |
  | `smellfish-core`    | Natural-product evidence assessment, typed verdicts, CSV I/O, and generic-HTTP SPARQL client |

### Layer 1 — Web Adapters, UI & Governance (`crates/`)

  | Crate             | One-Line Responsibility                                                                           |
  | ---               | ---                                                                                               |
  | `upload`          | Browser `BlobSource`, Dioxus `FileData` extraction, and DOM downloads                             |
  | `ui`              | Pure-Rust Dioxus design system, `Theme`, `StyleBuilder`, accessible components, and WebMCP head   |
  | `workspace-gates` | Repository-wide structural, documentation, workflow, and ratchet integration tests                |

### Layer 2 — Applications (`apps/`) & Standalone CLI

  | Crate                                                   | One-Line Responsibility                                                           | Target                |
  | ---                                                     | ---                                                                               | ---                   |
  | `apps/index`                                            | Accessible catalogue and navigation hub for the application suite                 | `wasm32` + host tests |
  | `apps/cxsmiles-yoga`                                    | Dioxus UI, memoized SVG depiction, and clipboard adapter for `cxsmiles-core`      | `wasm32` + host tests |
  | `apps/json-count-rs`                                    | Dioxus UI and `BlobSource` adapter for `json-count-core`                          | `wasm32` + host tests |
  | `apps/lipid-selecto-rs`                                 | Dioxus UI, filter controls, and streaming adapter for `lipid-core`                | `wasm32` + host tests |
  | `apps/mgf-precursor-erro-rs`                            | Dioxus UI, `plotters` SVG adapter, and streaming adapter for `mgf-analysis-core`  | `wasm32` + host tests |
  | `apps/smellfish-rs`                                     | Dioxus UI, RDKit.js interop, and `ReqwestClient` adapter for `smellfish-core`     | `wasm32` + host tests |
  | `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction` | Excluded standalone CLI for offline LIPID MAPS SMARTS evolution (`--locked`)      | host native CLI       |

## Cross-Cutting Standards

- **Minimal public surface**: Every crate exports only its intentional API and
  keeps internal modules and helpers `pub(crate)` or private.
- **Type-driven invariants**: Domain crates model states with explicit enums,
  validated newtypes, and `FromStr`/`TryFrom` conversions rather than boolean
  flags, sentinel floats, or stringly-typed dispatch.
- **Error strategy**: Fallible library crates define typed `#[non_exhaustive]`
  `thiserror` error enums carrying structured context; binary/UI edges format
  errors cleanly for users.
- **Concurrency & browser responsiveness**: Pure async logic stays generic over
  `ChunkSource` or `lotus_search::Http` with no blocking I/O; browser adapters
  yield across chunk boundaries, prevent concurrent import races, and memoize
  expensive SVG/sort derivations.
- **Dual-target verification**: `./mk wasm` and `./mk local:lint-wasm` verify
  `wasm32-unknown-unknown` builds across all Layer 0 crates, `upload`, and all
  six apps, while `./mk test` runs native unit, integration, and doc tests.
- **Unwind-safe WASM release profile**: `[profile.wasm-release]` keeps
  `panic = "unwind"` and `strip = "debuginfo"` so `chem-depict` recovers from
  panics in third-party SMILES parsing without aborting the browser tab, while
  preserving `__wasm_bindgen_unstable` custom sections on LLVM 23+.

## Glossary

Canonical domain and system terms shared across crate APIs, CLI help, and UI copy:

  | Term              | Meaning                                                                                  |
  | ---               | ---                                                                                      |
  | `ChunkSource`     | Async byte-range reader abstraction implemented by native `SliceSource` and browser `BlobSource` |
  | `SpectrumBlock`   | One parsed `BEGIN IONS ... END IONS` MGF spectrum or SMILES row in `lipid-core`          |
  | `MgfBlock`        | Parsed MGF header and fragment peak record emitted by `mgf::MgfLineParser`               |
  | `ChemicalClass`   | Indexed LMSD main class entry with precompiled SMARTS query and parent `LipidFamily`     |
  | `LipidFamily`     | Top-level LIPID MAPS structural category (`FA`, `GL`, `GP`, `SP`, `ST`, `PR`, `SL`, `PK`) |
  | `PrecursorStats`  | Bounded streaming summary of precursor mass errors and sampled scatter points             |
  | `CalibrationModel` | MS2 fragment mass-error correction mode (`None`, `OrbitrapPPM`, `TOFDa`) parameterized by `ShrinkageLambda` |
  | `EvidenceInputs`  | Orthogonal descriptor, motif, and SPARQL signals combined into a `smellfish-core` `VerdictCategory` |
