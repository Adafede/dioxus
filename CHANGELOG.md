# Changelog

## Unreleased

### Fixed

- **`upload`**: fixed `SliceSource` slice-read panic on out-of-range offsets,
  validated progress intervals before clamping in `ProgressThrottler`, sanitized
  download filenames, and enabled native compilation and host unit tests for
  `BlobCursor`, `BlobLines`, and `ChunkSource`.
- **`json-count-rs`**: decoupled pure streaming JSON lexer/counter from `wasm32`
  `Blob` entry points, guarded malformed escape sequences and deep nesting, and
  added host tests across edge cases and WASM entry boundaries.
- **`lipid-selecto-rs`**: replaced placeholder tests and removed bogus external
  YAML rule files; documented the 86 compiled-in LMSD main classes
  (`src/chemical_class/lmsd.rs`) and 204 curated examples (`src/examples/`).
- **`mgf-precursor-erro-rs`**: fixed adduct parsing, histogram bin-width edge
  cases (`bin_count == 0`), typed `MgfError` propagation across SVG renderers,
  and MGF streaming capacity guards.
- **`cxsmiles-yoga`**: fixed round-trip coverage denominators, multi-position
  `m:` attachment indexing, aromatic equivalence bounds, and native test
  coverage.
- **`smellfish-rs`**: properly escaped QLever SPARQL literals and IRIs, quoted
  multi-line/delimiter CSV export cells cleanly, fixed NP-likeness edge cases,
  and moved SPARQL query/CSV/model tests onto host CI.
- **`smarts-evoliposuction`**: synced `Cargo.lock` with `--locked` builds, fixed
  `zip` 9.0.5 entry name extraction and `NaN`-safe score sorting
  (`cmp_priority`), fixed UTF-8 boundary slicing in `truncate_str`, and replaced
  empty test placeholders with unit tests.
- **`ui`**: fixed `DocumentHead` and `PageTitle` doctests, enabled all eight
  runnable doctests (`./mk test`), added `VirtualDom::rebuild_to_vec` render
  tests across UI components, and expanded `StyleBuilder` theme tests.

### Changed

- Consolidated CI, local hooks (`prek.toml`), and release/nightly automation
  around `./mk` and `crates/upload/tests/gate_consistency/` (surface separation,
  SPDX headers, 800-line Rust file ceiling, zero ignored doctests, and
  ratchets).
- Separated human documentation (`README.md`, `docs/`, `.github/*.md`, app
  `README.md`s) from workspace agent instructions and verified every CLI
  example, file path, and CI description against the workspace.
