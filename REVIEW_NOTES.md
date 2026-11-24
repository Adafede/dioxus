# Refactor Review Notes

Reviewer handoff for the structural refactoring of the `dioxus` Chemistry & Mass-Spectrometry WebAssembly workspace (`b596bfd5` through `Step 11`).

## 1. Architecture Summary

- **Layer 0 (Pure Domain, zero `dioxus` / `web-sys` / blocking IO)**:
  - Shared primitives: `stream-io`, `mgf`, `chem-depict`
  - App-backing cores: `json-count-core`, `cxsmiles-core`, `lipid-core`, `mgf-analysis-core`, `smellfish-core`
- **Layer 1 (Browser / UI / Governance Adapters)**:
  - `upload` (re-exports `stream-io`, adds `BlobSource`, `FileData` extraction, DOM download, `UploadError`)
  - `ui` (design system with `Theme`, `StyleBuilder`, `a11y`, `url`, `UploadZoneProps`/`UploadStatusTone`, and slimmed `DocumentHead`)
  - `workspace-gates` (isolated repository structural, documentation, workflow, and ratchet integration tests)
- **Layer 2 (Thin UI Shells & Excluded CLI)**:
  - `apps/{index,cxsmiles-yoga,json-count-rs,lipid-selecto-rs,mgf-precursor-erro-rs,smellfish-rs}`
  - Standalone excluded native binary: `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction`

See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) and [`docs/REFACTOR_PLAN.md`](./docs/REFACTOR_PLAN.md) for the complete crate map and completed migration sequence.

## 2. Key Architectural Decisions & Rejected Alternatives

1. **Eight focused Layer 0 crates vs. one shared `domain` crate**
   - *Decision*: Extracted three cross-app primitive crates (`stream-io`, `mgf`, `chem-depict`) plus five app-specific core crates (`*-core`).
   - *Rejected*: A single shared domain crate would couple unrelated dependency trees (`plotters`/`molecular-formulas`/`chematic`/`lotus-search`/`smarts-rs`) across every app and bloat feature matrices and WASM checks.

2. **Generic async/transport traits (`ChunkSource`, `lotus_search::Http`) vs. `#[cfg(target_arch = "wasm32")]` forks**
   - *Decision*: `stream-io` operates over `&mut S where S: ChunkSource` and `smellfish_core::qlever` operates over `&impl lotus_search::Http` (with `Scripted` native mock clients in unit tests).
   - *Rejected*: Keeping `web_sys::Blob` or concrete `reqwest` HTTP clients in domain crates would preserve test blindness on host targets and require duplicated host/WASM struct definitions.

3. **Typestate / enums over booleans and stringly dispatch**
   - `ui::theme::Theme` (`Light` / `Dark`) replaced raw `dark: bool` threading; `UploadStatusTone` replaced `status_text.contains("Error")`.
   - `smellfish_core` introduced `SourceClass`, `CheckStatus`, `RingFamily`, `MotifContext`, `VerdictCategory` (`csv_label` / `css_class`), `Verdict`, `Option<NpScore>`, and `InChIKey14`.
   - `mgf_analysis_core` replaced `"mDa"`/`"ppm"` strings with `ErrorUnit`, clamped `ShrinkageLambda`, and fixed the `within_*_da` naming/unit bug by introducing `ErrorThresholds`.
   - `lipid_core` replaced `HashMap<String, bool>` for fixed 86 LMSD rules with bitset-backed `ClassMatchSet` (`[u64; 2]`) and precompiled `CompiledSmartsRule`s (`OnceLock`).

4. **Standalone workspace preserved for `smarts-evoliposuction`**
   - *Decision*: Kept `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction` as an excluded `[workspace]` tested with `--locked`, modularized internally with `CliError`, `EvolveOptions`, `PipelineOptions`, `ResultRowFields`, `ParseCounts`, `SplitEmitter`, and `O(N)` precomputed row ordering.
   - *Rejected*: Merging into the root workspace fails `workspace-gates::gate_consistency::apps::the_excluded_tool_declares_itself_a_standalone_workspace` and breaks `--locked` resolution against upstream `smarts-evolution` vs `smarts-rs` `smiles-rs` versions.

5. **Real CSS/JS files via `include_str!` in `smellfish-rs`**
   - *Decision*: Moved inline stylesheet/bridge strings (`styles.rs`, `rdkit_bridge.rs`) into `apps/smellfish-rs/assets/{smellfish.css,rdkit_bridge.js}` with unused dark/legacy CSS trimmed after inspecting selectors.

## 3. Intentional Behavior Changes

All behavior changes below were deliberate bug fixes rather than silent refactors:

1. **`apps/smellfish-rs` dead motif-context check fixed**:
   - *Before*: `app/results.rs:137` checked `row.motif_context != "no motif signal"` against `"no structural motifs detected"`, which was always `true` and rendered `"no structural motifs detected"` on every card lacking motifs.
   - *After*: `row.motif_context.is_detected()` omits the motif-context line when `MotifContext::None`, preserving exact motif text whenever any motif signal is detected.

2. **`apps/smellfish-rs` concurrent drop-race guard**:
   - *Before*: Launching two file imports back-to-back allowed an older async drop task to overwrite UI `Signal`s after a newer drop completed.
   - *After*: `app::browser::ImportSignals::generation` increments monotonically per import and ignores stale async writes from superseded generations.

3. **`apps/smellfish-rs/build.rs` quote-aware CSV parsing**:
   - *Before*: `line.split(',')` miscounted columns when group names contained quoted commas.
   - *After*: Quote-aware field counting (`count_csv_fields`) parses RFC 4180 quoted commas safely, and `rerun-if-changed` watches only files the build script actually validates.

4. **`smarts-evoliposuction` single shuffle in `cap_and_shuffle` and I/O error propagation**:
   - *Before*: `cap_and_shuffle` called `items.shuffle(rng)` twice (once before `truncate(cap)` and again redundantly after truncation), and `read_smiles_lines` silently dropped line read errors via `.map_while(Result::ok)`.
   - *After*: Single `items.shuffle(rng); items.truncate(cap);` preserves uniform sampling while changing the exact output permutation for a fixed RNG seed; `read_smiles_lines` propagates `io::Error` via `io::Result<Vec<String>>` so truncated/non-UTF-8 files surface `EvolutionResult::ReadError`.

5. **Minor normalization fixes**:
   - `ui::url`: `extract_hostname` uses explicit authority/IPv6 bracket parsing instead of naive split shortcuts.
   - `smellfish_core::csv::export`: RFC 4180 formula-neutralizing quoting preserves embedded commas, quotes, and newlines consistently across all exported columns.

## 4. Conservative Choices / Open Questions for Reviewer

1. **`crates/lipid-core` `class_tags_from_block` vs. `class_tags_from_gallery` divergence**:
   - Preserved legacy behavior where `class_tags_from_gallery` formats `" | "`-joined multiple matched SMARTS class tags, while the block-level fallback beyond the 96-card gallery cap (`class_tags_from_block`) appends the single primary `classification.class_tag`. If uniform multi-tag joining is desired past 96 cards, align `class_tags_from_block` with `class_tags_from_gallery`.

2. **Shipped WASM bundle size delta (`+53,732` B / `+0.94%` across 6 apps)**:
   - `json-count-rs` shrank (`511,356 -> 509,994` B), while `mgf-precursor-erro-rs` (`+32,323` B) and `cxsmiles-yoga` (`+15,736` B) grew slightly due to typed error enums (`PlotError`, `MgfAnalysisError`, `CxError`), recalibration diagnostic downsampling, and virtualized/memoized helpers.
   - Notice `./mk web-size` sums every `*_bg-dxh*.wasm` under `target/dx/*/release/web/public/assets/`; when an older build artifact exists in `target/dx/`, clean old hashed `.wasm` files before running `./mk web-size` (per-app current builds verified by mtime: `cxsmiles-yoga` 1,326,003 B; `index` 415,865 B; `json-count-rs` 509,994 B; `lipid-selecto-rs` 1,265,644 B; `mgf-precursor-erro-rs` 1,302,996 B; `smellfish-rs` 921,584 B; total 5,742,086 B vs 5,688,354 B baseline).

## 5. Remaining Debt (Ranked) & Exception Justifications

### Priority 1 — Targeted `cargo-mutants` Survivors in Newly Extracted Layer 0 Crates
- Ran `cargo mutants -p chem-depict -p stream-io -j 2 -- --locked`: `149` mutants tested in `12m` -> `76` caught, `18` timeouts, `8` unviable, `47` missed.
- Remaining missed mutants to cover with unit tests:
  - `crates/stream-io`: `ChunkBuffer::compact_half` threshold boundary, `BlobCursor::current_byte` / `buffer`, `BlobLines::total_bytes` / `next_line` edge conditions, `ProgressThrottler::maybe_report` `<` vs `<=` boundary, and `slice_clock` ms calculation (`bytes.rs`).
  - `crates/chem-depict`: exact SVG fallback body assertions for `empty_svg` / `empty_compact_svg`, `validate_svg_or` non-`<svg` rejection, `panic_payload_message` formatting, and `render_smiles_svg_highlighted` option guard.
- Full branch-diff `mutants-diff` discovered `573` mutants across the whole refactor diff (too large to finish in a single interactive CI pass; `MUTANTS_PACKAGES` in `mk.local` still pins `(json-count-core cxsmiles-core)`).

### Priority 2 — Justified Files Over 300 LOC (7 total across 262 Rust files, down from 38)
- **Non-test domain files (4 files at 301–310 LOC)**:
  - `crates/lipid-core/src/chemical_class/lmsd_part1.rs` (310 LOC) and `lmsd_part2.rs` (310 LOC): purely declarative static `const` tables encoding the 86 LIPID MAPS SMARTS rules; splitting further into 3 parts fragments a cohesive static registry without reducing branching logic.
  - `crates/cxsmiles-core/src/positional.rs` (310 LOC): 10 lines over 300 due to doc comments and aromatic positional attachment enumeration.
  - `crates/mgf-analysis-core/src/adduct/parse.rs` (301 LOC): 1 line over 300 holding recursive adduct grammar tokenization/normalization.
- **Test files (3 files)**: `crates/workspace-gates/tests/gate_consistency/surfaces.rs` (363 LOC), `crates/workspace-gates/tests/gate_consistency/apps.rs` (343 LOC), `crates/mgf-analysis-core/src/tests/adduct.rs` (303 LOC).

### Priority 3 — Justified `#[allow(` Attributes (14 across 11 files, down from 85) and UI RSX Size
- All remaining `#[allow(` occurrences are scoped, commented invariants where `#[expect]` is either target-conditional (`cfg(wasm32)` vs host) or already tied to a documented invariant:
  - `crates/stream-io/src/lib.rs`, `crates/json-count-core/src/lib.rs`, `apps/json-count-rs/src/wasm_entry.rs`, `crates/stream-io/src/bytes.rs`: `!Send` WASM futures (`clippy::future_not_send`, `async_fn_in_trait`).
  - `crates/upload/src/{bytes,event,download}.rs`, `apps/json-count-rs/src/wasm_entry.rs`: DOM `Blob::size`/`slice` `f64` casts and `#[cfg(not(target_arch = "wasm32"))]` `missing_const_for_fn` stubs.
  - `crates/cxsmiles-core/src/parse.rs` (`2` sites): bounds-invariant indexing in `cluster_ecfp4` / `uf_find`.
  - Test module declarations in `crates/{lipid-core,mgf-analysis-core,cxsmiles-core}/src/lib.rs` (`#[allow(clippy::indexing_slicing)]`).
- Longest non-test function across all 262 `.rs` files is `145` LOC (`apps/smellfish-rs/src/app/molecule_card.rs`, a declarative Dioxus `rsx!` card view; down from `456` LOC).

## 6. Verification Ledger (Final State)

- `./mk ci-core`: **PASS** (tools, actions, shell scripts, commits, ratchet, fmt, tombi, typos, lint, lints-inherit, test, doc, deny, audit, machete, echoes [`14` doctests, `1097` fns, `0` dupes], refquiem, wasm, `local:license-headers`, `local:check`, `local:readme-check`, `local:lint-wasm`, `local:feature-powerset`; `msrv` & `semver` skipped cleanly per repo config).
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`: **PASS** (0 warnings).
- `cargo nextest run --workspace --locked`: **372 passed, 0 skipped, 0 failed** (`13.728s`).
- Standalone `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction` (`fmt --check`, `clippy --all-targets --locked -- -D warnings`, `test --locked`): **37 passed (31 lib + 6 bin), 0 failed** (`Cargo.lock` untouched).
- `cargo mutants -p chem-depict -p stream-io -j 2 -- --locked`: `149` mutants tested (`76` caught, `47` missed, `18` timeouts, `8` unviable).

## 7. Reviewer Checklist — What to Scrutinize First

1. **`crates/smellfish-core/src/evidence/` & `pipeline.rs` + `apps/smellfish-rs/src/app/browser.rs`**: Verify typed `VerdictCategory` / `MotifContext::is_detected()` mapping, `Pass-1`/`Pass-2` single-pass aggregation, and the `ImportSignals::generation` drop-race guard.
2. **`crates/mgf` & `crates/mgf-analysis-core/src/{adduct,calibration,metrics}`**: Confirm BOM/case-insensitive MGF parsing matches both consumers cleanly, `ShrinkageLambda` clamping behavior, and `scatter` downsampling (`<= 600` points).
3. **`crates/lipid-core/src/{chemical_class,parser}`**: Inspect `ClassMatchSet` `[u64; 2]` bitmask indexing and the conservative `class_tags_from_block` vs `class_tags_from_gallery` choice noted in Section 4.
4. **`apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/{splitting,evolve,cli}`**: Confirm `splitting` `&str` slice borrowing preserves exact manifest output order/counts and `read_smiles_lines` error propagation on malformed files.
5. **`crates/ui/src/{theme,document,components/upload_zone}.rs`**: Check `Theme` (`Light`/`Dark`) color parity and `UploadZoneProps::id_prefix` default behavior (`"upload"`).
