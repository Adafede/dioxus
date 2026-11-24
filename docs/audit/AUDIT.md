# Workspace Architectural Audit (Phase 0 Recon)

Baseline snapshot of the workspace prior to structural refactoring.

## 1. Baseline Metrics

| Metric | Baseline Value | Notes |
| --- | ---: | --- |
| Workspace members | 8 | 2 shared crates (`ui`, `upload`), 6 apps (`index`, `cxsmiles-yoga`, `json-count-rs`, `lipid-selecto-rs`, `mgf-precursor-erro-rs`, `smellfish-rs`) |
| Excluded standalone crate | 1 | `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction` |
| Total Rust lines (`tokei`) | 36,299 | 27,803 code, 2,027 comments, 2,885 blanks, 3,584 doc markdown |
| Max `.rs` file LOC | 732 | `apps/json-count-rs/src/processing.rs` |
| Files > 300 LOC | 38 | 31 workspace files + 7 in `smarts-evoliposuction` |
| Max non-test function LOC | 456 | `ResultsPanel` in `apps/mgf-precursor-erro-rs/src/app/results.rs:33-488` |
| `#[allow(` occurrences | 85 | Across `apps/` and `crates/` |
| `.clone()` calls (`src/`) | 298 | `smellfish-rs` (89), `cxsmiles-yoga` (65), `mgf-precursor-erro-rs` (49), `lipid-selecto-rs` (46), `json-count-rs` (17), `ui` (24), `upload` (8) |
| Workspace unit/integration tests | 555 | Nextest run: 555 passed in 8.832s (cold build+test 6m52s) |
| Standalone CLI tests | 35 | `smarts-evoliposuction --locked`: 32 lib + 3 bin passed |
| Workspace doctests | 8 | All 8 in `crates/ui` |
| Rustdoc (`-D warnings`) | pass | 0 warnings (2m30s) |
| Shipped WASM bundle sizes (`./mk web-size`) | 5,688,354 B | `cxsmiles-yoga` 1,310,267 B; `index` 413,024 B; `json-count-rs` 511,356 B; `lipid-selecto-rs` 1,261,734 B; `mgf-precursor-erro-rs` 1,270,673 B; `smellfish-rs` 921,300 B |
| `cargo-echoes` findings | 0 | 8 doctests, 1,144 functions, 0 duplicate blocks |
| `cargo-deny` / `cargo-audit` / `cargo-machete` | pass | 1 `gloo-timers` unused-dep warning on host in `apps/lipid-selecto-rs/Cargo.toml:17` |

## 2. Findings Ranked by Impact × Effort

### High Impact / Low–Medium Effort

1. **Unconditional `dioxus` dependency in `crates/upload` (`crates/upload/Cargo.toml:15`)**
   - `upload` pulls `dioxus` (with `web` feature) solely for `dioxus::html::FileData` in `crates/upload/src/event.rs:14,42`.
   - All streaming primitives (`blob_cursor.rs`, `blob_lines.rs`, `bytes.rs`, `progress.rs`, `error.rs`) are Dioxus-independent.
   - Extracting a Dioxus-free `stream-io` crate decouples pure parsers (`json-count-core`, future streaming consumers) from the web framework.

2. **Misplaced workspace-wide gates inside `crates/upload/tests/gate_consistency/`**
   - 1,356 LOC of repo-wide doc/workflow/manifest/ratchet checks live inside `crates/upload/tests/` with `#![allow(unused_crate_dependencies)]` (`main.rs:40`), forcing `upload` to carry unrelated test churn.
   - Extracting `crates/workspace-gates` isolates governance gates cleanly.

3. **Triple-duplicated MGF parser with silent behavioral divergences**
   - `apps/mgf-precursor-erro-rs/src/parser/block.rs:48-192`
   - `apps/mgf-precursor-erro-rs/src/recalibration/parsing.rs:7-26` + `generator.rs:83-142`
   - `apps/lipid-selecto-rs/src/parser/parsing.rs:87-305`
   - Divergences include UTF-8 BOM handling, case sensitivity of header keys, inline `BEGIN IONS KEY=VAL` support, and unterminated EOF block handling.

4. **Duplicated SMILES SVG depiction with asymmetric panic safety**
   - `apps/lipid-selecto-rs/src/depict_simple.rs:8-33` wraps `chematic::smiles::parse` + `chematic::depict::depict_svg` in `catch_unwind` and checks `<svg` prefix.
   - `apps/cxsmiles-yoga/src/depict.rs:18-48` duplicates the wrapper without `catch_unwind`, risking tab aborts on panicking inputs.

5. **Host `unused dependency gloo-timers` warning (`apps/lipid-selecto-rs/Cargo.toml:17`, `apps/cxsmiles-yoga/Cargo.toml:20`)**
   - `gloo-timers` is listed under unconditional `[dependencies]` instead of `[target.'cfg(target_arch = "wasm32")'.dependencies]`.

6. **Domain logic trapped inside `apps/` behind `#[cfg(any(test, target_arch = "wasm32"))]` gates**
   - ~45 `#[cfg]` / `allow(dead_code)` annotations across `json-count-rs/src/processing.rs`, `lipid-selecto-rs/src/{chemical_class,lipids,parser}/`, `mgf-precursor-erro-rs/src/parser/block.rs`, and `smellfish-rs/src/{model,evidence,csv,query,sparql}.rs`.
   - Several `#[cfg(test)]`-only production-claiming items are dead in release builds (e.g., `mgf-precursor-erro-rs/src/metrics/merge.rs` 245 LOC; `lipid-selecto-rs/src/chemical_class/defaults.rs` 174 LOC).

### High Impact / Medium–High Effort

7. **Inverted layer dependencies and mixed UI/domain boundaries**
   - `apps/mgf-precursor-erro-rs/src/diagnostics.rs:10` and `metrics/plot.rs:11` import `crate::plotting::data::usize_to_f64`.
   - `apps/mgf-precursor-erro-rs/src/plotting/diagnostics.rs:363-419` generates inline-styled HTML `<table>` markup inside `plotting`.
   - `apps/mgf-precursor-erro-rs/src/app/diagnostics.rs:10-76` computes 4-stage recalibration statistics inside UI `Signal` code.
   - `apps/json-count-rs/src/processing.rs:22` imports `crate::ColumnResult` from UI `src/main.rs:35-38`.
   - `apps/cxsmiles-yoga/src/examples.rs:26-28,88-90` couples static SMILES fixtures to `ui::prelude::SegmentedControlItem`.
   - `apps/lipid-selecto-rs/src/app/{family,analysis,class_tags,gallery,download}.rs` place pure domain logic inside `src/app/`.
   - `apps/smellfish-rs/src/qlever.rs:15,25,80,104,147,195,223` hardcodes `ReqwestClient` instead of `&impl lotus_search::Http`, keeping QLever enrichment untestable natively.

8. **God files and oversized functions**
   - `apps/json-count-rs/src/processing.rs` (732 LOC), `main.rs` (499 LOC; `app()` 233 LOC), `scan_container` (102 LOC).
   - `apps/lipid-selecto-rs/src/chemical_class/lmsd.rs` (703 LOC), `app/components.rs` (521 LOC; gallery-filter component 97 LOC, 7 params).
   - `apps/mgf-precursor-erro-rs/src/app/results.rs` (488 LOC; `ResultsPanel` 456 LOC, 8 params), `diagnostics.rs` (471 LOC), `parser/adduct.rs` (424 LOC), `plotting/diagnostics.rs` (423 LOC), `parser/block.rs` (416 LOC).
   - `apps/cxsmiles-yoga/src/cxsmiles/positional.rs` (509 LOC; `build_positional_with_options` 122 LOC), `app.rs` (451 LOC), `repeating.rs` (439 LOC), `roundtrip.rs` (349 LOC) with bidirectional `positional`/`repeating` ↔ `roundtrip` module cycles.
   - `apps/smellfish-rs/src/styles.rs` (548 LOC), `rdkit_bridge.rs` (519 LOC), `app/results.rs` (398 LOC; `MoleculeCard` 147 LOC), `pipeline.rs` (378 LOC; CSV import pipeline 183 LOC), `evidence/verdict.rs` (`classify_np_evidence` 158 LOC).
   - `smarts-evoliposuction/src/splitting.rs` (728 LOC; `split_dataset` 169 LOC), `main.rs` (727 LOC; `split_step` 135 LOC, `evolve_step` 120 LOC, 11 params), `evolve.rs` (495 LOC), `lipidsdl/sdf.rs` (367 LOC).

9. **Severe algorithmic, memory, and UI re-render hotspots**
   - `smarts-evoliposuction/src/main.rs:605-611`: `order_rows_for_evolution` opens and reads `.smiles` files from disk inside the `sort_by` comparator (`O(N log N)` disk reads).
   - `smarts-evoliposuction/src/splitting.rs:299-305,452`: clones ~48k SMILES strings twice per class.
   - `apps/cxsmiles-yoga/src/app.rs:237-283`: `results()` re-parses SMILES and re-renders 2D SVGs for all candidates synchronously on every keystroke; `cxsmiles/mod.rs:120-125` runs `build_repeating` on equal-size inputs and recomputes `roundtrip_coverage` twice; `repeating.rs:222-227` clones `Vec<u32>` up to 200k times in DFS; `positional.rs:364-424` performs `O(|pos| * N * (V+E))` aromatic DFS and linear `FxHashMap` scans.
   - `apps/json-count-rs/src/processing.rs:588-613`: allocates a new `HashSet<usize>` per JSON object row and a new `Vec<u8>` + `String` per key; `main.rs:387` clones and re-sorts all rows on every render.
   - `apps/lipid-selecto-rs`: LMSD table access (`lmsd.rs:701-703`) and `group_classes_by_family` (`app/analysis.rs:26-39`) deep-clone 86 compiled SMARTS `QueryMolecule`s on every render; `lipids/classify.rs:73-78` re-parses 10 SMARTS strings per molecule; `SpectrumBlock::raw` retains entire file text unboundedly; `parse_smiles_line` + `classify_and_match` + `render_svg` parse SMILES up to 4× per molecule.
   - `apps/mgf-precursor-erro-rs`: 3 `MedianTracker` dual-heaps retain every spectrum in memory; `app/results.rs:345-448` re-renders 4 SVGs unmemoized per slider drag event; `plotting/diagnostics.rs` emits up to 10,000 `<circle>` DOM elements; `app/results.rs:281-291` scans error arrays 10× per render.
   - `apps/smellfish-rs`: `app/results.rs:96,119` clones every `MoleculeRow` + SVG and runs `to_ascii_lowercase()` SVG sanitization per render; `pipeline.rs:190,211` calls `count_evidence` twice per molecule; `qlever.rs` ignores QLever probe outcome and runs `ASK {}` unconditionally.

10. **Stringly-typed state, primitive obsession, and ad-hoc `String` errors**
    - `smellfish-rs/src/evidence/verdict.rs:219` vs `app/formatting.rs:29` vs `app/csv_export.rs:42`: `"PubChem + strong NP evidence"` maps to `"likely"` (green) in CSV export but falls through to `"verdict-neutral"` (blue) in UI formatting due to prose re-parsing.
    - `smellfish-rs/src/app/results.rs:137`: dead comparison `row.motif_context != "no motif signal"` vs `"no structural motifs detected"` (`assessment.rs:265`).
    - `mgf-precursor-erro-rs/src/metrics/stats.rs:118-122,245-246`: fields named `within_*_da` actually bucket mDa values (`abs_error_da * 1000.0`); `plotting/data.rs` passes `"mDa"`/`"mda"`/`"ppm"` strings; `calibration.rs:9` ignores `CalibrationModel.lambda` and fails to clamp `lambda`.
    - `cxsmiles-yoga/src/cxsmiles/types.rs:42-83`: `CxError(String)`, redundant `Confidence.clean`, flat `CxResult` + `FloatingPart.split` bool.
    - `lipid-selecto-rs`: `HashMap<String, bool>` for fixed 86 LMSD classes, `exact_mass: f64` `0.0` sentinel, 4 copies of LIPID MAPS family tables, 7–10 param filter/download functions.
    - `crates/upload`: download helpers and `extract_blob_from_file_data` return `Result<_, String>` despite `UploadError::BrowserOnly` existing; two `UploadError` variants were unused dead code.

### Medium Impact / Low–Medium Effort

11. **Shared `crates/ui` and `apps/index` cleanup**
    - `crates/ui/src/common.rs` mixes skip-link accessibility with `ANALYTICS_PIXEL_CSS`; `UploadZone` takes 10 loose args, hardcodes `upload-input`/`upload-status` IDs, and detects errors via `status_text.contains("Error")`; `DocumentHead` is 156 LOC; `ColorScheme::LIGHT`/`DARK` branching is repeated across 6 components; `StyleBuilder::new()` starts at 0 capacity.
    - `apps/index/src/components/app_card.rs` duplicates `Card` styling from `crates/ui`.

12. **`smellfish-rs` CSS, build script, and concurrency hygiene**
    - `apps/smellfish-rs/src/styles.rs` (548 LOC) declares `color-scheme: light dark` with zero dark-mode rules, 5 unused variables, duplicate `.chip.good`/`.chip-np` rules, and dead selectors.
    - `apps/smellfish-rs/build.rs` reads `line.split(',')` without quote awareness and emits `rerun-if-changed` for unused files.
    - `apps/smellfish-rs/src/app/mod.rs:105` allows concurrent drop tasks to race on 10 loose `Signal`s.
