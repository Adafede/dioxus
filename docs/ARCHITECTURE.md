# Architecture

This repository is a Cargo monorepo of six client-side Dioxus WebAssembly chemistry and mass-spectrometry web applications backed by two shared foundation crates (`ui` and `upload`).

## Workspace Structure

```text
apps/
  index                  Landing page linking the suite of chemistry web apps
  cxsmiles-yoga          Canonical CX-SMILES generator and coordinate aligner
  json-count-rs          Streaming non-null field counter for uploaded JSON datasets
  lipid-selecto-rs       SMARTS-based lipid classification and filtering
  mgf-precursor-erro-rs  MGF MS/MS precursor mass-error analysis and plotting
  smellfish-rs           Natural-product likeness scoring with RDKit.js and QLever/LOTUS
crates/
  ui                     Type-safe Dioxus design system, style builders, and signal macros
  upload                 Browser Blob streaming (`BlobCursor`, `BlobLines`) and download helpers
```

## Shared Crates (`crates/`)

| Crate | Responsibility | Key Exports |
|---|---|---|
| `crates/ui` | Shared Dioxus component library, pure-Rust CSS builders, theme tokens, and cross-target signal declaration macros | `Button`, `Card`, `Header`, `Footer`, `NoticeBar`, `SegmentedControl`, `DocumentHead`, `styles::*`, `theme::*`, `shared_signal!`, `shared_signals!` |
| `crates/upload` | Zero-copy/chunked browser file reading over `web_sys::Blob`, progress throttling, and file download triggers | `BlobCursor`, `BlobLines`, `BlobSource` |

## Applications (`apps/`)

| App | Domain & Pure Logic | Target |
|---|---|---|
| `apps/index` | Accessible catalogue and navigation hub for the application suite | `wasm32` + host tests |
| `apps/cxsmiles-yoga` | Graph isomorphism, union-find fragment grouping, and CX-SMILES serialization | `wasm32` + host tests |
| `apps/json-count-rs` | Streaming JSON lexer/counter (`src/processing.rs`) decoupled from `Blob` entry points (`wasm_entry.rs`) | `wasm32` + host tests |
| `apps/lipid-selecto-rs` | LipidMaps hierarchy matching and SMARTS rule evaluation (offline CLI generator in `lipidmaps/smarts-evoliposuction`, excluded from workspace) | `wasm32` + host tests |
| `apps/mgf-precursor-erro-rs` | MGF spectrum parsing, precursor mass-error statistics, and SVG histogram plotting (`plotters`) | `wasm32` + `rlib` |
| `apps/smellfish-rs` | NP-likeness scoring and taxonomic/structure lookup against LOTUS (`lotus-search`) | `wasm32` + host tests |

## Design Invariants

- **Host-testable core logic**: Parsing, numerical analysis, and state transitions are kept free of `web_sys` where possible so `cargo nextest` and `cargo-mutants` execute natively without a headless browser.
- **Dual-target linting**: Because host `cargo clippy` skips `#[cfg(target_arch = "wasm32")]` blocks, `./mk local:lint-wasm` and `./mk wasm` (`check-wasm`) compile and lint each application for `wasm32-unknown-unknown`.
- **Unwind-safe WASM release profile**: `[profile.wasm-release]` keeps `panic = "unwind"` and `strip = "debuginfo"` so apps that guard third-party parsers with `catch_unwind` report recoverable errors instead of aborting the browser tab, while preserving `__wasm_bindgen_unstable` custom sections on LLVM 23+.
