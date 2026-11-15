# Changelog

## Unreleased

### Fixed

- **`upload`**: fixed `SliceSource` slice-read panic on out-of-range offsets,
  validated and clamped `ProgressThrottler` on clock rollbacks, saturated
  `BlobCursor::advance` bounds, prefixed Windows reserved stems and capped UTF-8
  lengths in `sanitize_filename`, and enabled native compilation and host unit
  tests for `BlobCursor`, `BlobLines`, and `ChunkSource`.
- **`json-count-rs`**: deduplicated repeated object keys within a single array
  row when `per_row` mode is enabled, decoupled pure streaming JSON scanning
  from `wasm32` `Blob` entry points, guarded malformed escape sequences and deep
  nesting, and expanded host streaming tests.
- **`index`**: aligned crate metadata and added `VirtualDom` render and catalog
  unit tests.
- **`lipid-selecto-rs`**: activated `FORMULA=` neutral exact-mass and `[XX00]`
  family classification fallbacks, added space-delimited SMILES/ID support and
  ASCII case-insensitive `BEGIN IONS` detection, wired per-family color
  palettes into `defaults.rs`, and documented the 86 LMSD main classes
  (`src/chemical_class/lmsd.rs`) and 204 curated examples (`src/examples/`).
- **`mgf-precursor-erro-rs`**: fixed neutral `AMMONIUM_MASS` + single-electron
  `[M+NH4]+` mass calculation, multi-charge/unsigned `CHARGE`/`ADDUCT` parsing,
  ASCII case-insensitive MGF tags, finite validation on calibration parameters,
  stream-wide `RunningStats::max_abs` and MAE improvement tracking, dynamic SVG
  sizing/legends, typed `MgfError` propagation, and streaming capacity guards.
- **`cxsmiles-yoga`**: enforced single-external-bond and `>= 2` coverage in
  `build_repeating`, omitted empty `||` extension blocks, preserved `BondOrder`
  across split floating attachments on re-expansion, moved `skip_link` before
  `Header`, and expanded host property and mutation tests.
- **`smellfish-rs`**: fixed `motif-library.js` initialization ordering and
  `response.ok` fetch checks, spelled `ERTL_SUBSTITUENTS` consistently, chunked
  WDQS `VALUES` queries and reported QLever fallback reachability accurately,
  corrected saturated monocycle vs. polycyclic ring heuristics and distinct
  verdict labels, added three-pass CSV column detection and CWE-1236 formula
  escaping, and validated inline SVGs before rendering.
- **`smarts-evoliposuction`**: synced `Cargo.lock` with `--locked` builds, set
  default `--input` (`LMSD.sdf.tsv`) and `SUB_CLASS`/`SUBCLASS` aliases,
  supported multi-space/tab SDF tags and CRLF/tab TSV sanitization, checked
  HTTP error status on downloads, extracted `results_io` with `--resume` filtering
  for `status == "ok"`, and replaced placeholder tests with CLI and library
  unit tests.
- **`ui`**: escaped HTML/script breakout characters and JS line separators in
  WebMCP scripts and caller JSON-LD, replaced `dangerous_inner_html` in
  `SimpleAnalyticsBadge` with typed `rsx!` and hostname validation, fixed
  `DocumentHead` and `PageTitle` doctests, and added `VirtualDom` render tests.

### Changed

- Consolidated CI, local hooks (`prek.toml`), and release/nightly automation
  around `./mk` and `crates/upload/tests/gate_consistency/` (surface separation,
  SPDX headers, 800-line Rust file ceiling, zero ignored doctests, `Dioxus.toml`
  path and ASCII title checks, `.vscode/*.json` separation, and ratchets).
- Separated human documentation (`README.md`, `docs/`, `.github/*.md`, app
  `README.md`s) from workspace agent instructions and verified every CLI
  example, file path, and CI description against the workspace.
