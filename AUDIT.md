# Repository Scientific Audit & Refactoring Ledger

## 1. Phase 0: Baseline & Specification

- **Baseline commit**: `e340e202` (`main`)
- **Toolchain**: Rust `1.100.0` (`rust-toolchain.toml`, edition `2024`, resolver `3`)
- **Baseline checks (all exited 0)**:
  - `./mk check`: 47 TOML files formatted and linted, typos clean, host clippy clean.
  - `./mk test`: 356 workspace nextest cases passed + 14 doctests passed (`0` failed, `0` ignored).
  - `./mk wasm` & `./mk local:lint-wasm`: `wasm32-unknown-unknown` check and clippy clean.
  - `./mk echoes`: 14 doctests (14 unique), 1048 functions (1046 unique), 0 duplicated groups, 3337 context blocks (902 unique), 0 approximate groups, 0 dependency reuse opportunities (29 packages).
  - `./mk refquiem`: no stale mentions.
  - `./mk local:readme-check`: 16 generated READMEs match `//!` docs.
  - `./mk local:license-headers`: all `.rs` files carry valid SPDX headers.
  - `cargo machete`: 0 unused dependencies.
  - Excluded standalone CLI (`apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction`, `--locked`): 29 lib + 6 bin unit tests passed.

### Architecture & Data Flow

- **Layer 0 (Pure domain crates, no `dioxus`/`web-sys`)**: `stream-io`, `mgf`, `chem-depict`, `cxsmiles-core`, `json-count-core`, `lipid-core`, `mgf-analysis-core`, `smellfish-core`.
- **Layer 1 (Web adapters, UI, gates)**: `upload`, `ui`, `workspace-gates`.
- **Layer 2 (WASM apps + standalone CLI)**: `apps/index`, `apps/json-count-rs`, `apps/mgf-precursor-erro-rs`, `apps/lipid-selecto-rs`, `apps/cxsmiles-yoga`, `apps/smellfish-rs`, and excluded `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction`.

### Canonical Domain Glossary

| Term | Canonical Meaning & Units |
| --- | --- |
| `m/z` | Dimensionless mass-to-charge ratio of an ionized precursor or fragment (Th / Da per unit elementary charge `|z|`). |
| Neutral monoisotopic exact mass | Sum of monoisotopic neutral atomic masses in Daltons (Da, unified atomic mass units `u`) from molecular formula or SMILES. |
| Adduct | Ionization species `[kM + A]^z` transforming neutral mass `M` to theoretical precursor `m/z = (k*M + shift -/+ z*m_e) / |z|`. |
| Mass error (Da / mDa / ppm) | Signed `observed_mz - expected_mz` in Da (`1 mDa = 1e-3 Da`); relative error `1e6 * (observed_mz - expected_mz) / expected_mz` in ppm. |
| Shrinkage `lambda` | Clamped recalibration weight in `[0.0, 1.0]` scaling precursor `MS2 - MS1` drift applied to MS2 fragments (`TOFDa` additive Da, `OrbitrapPPM` multiplicative/inverse ppm). |
| Ertl NP-likeness score | Size-normalized sum of Morgan radius-2 fragment log-odds weights divided by heavy-atom count `N`, log-compressed outside `[-4, 4]`; confidence is fraction of molecule bits present in model. |
| InChIKey / InChIKey14 | 27-character hashed identifier (`AAAAAAAAAAAAAA-BBBBBBBBBB-P`); first 14 characters encode molecular connectivity only (ignoring stereochemistry/isotopes/charge). |
| LIPID MAPS family & class | 8 structural categories (`FA`, `GL`, `GP`, `SP`, `ST`, `PR`, `SL`, `PK`) and 86 LMSD main classes indexed in `ClassMatchSet(u128)`. |
| CX-SMILES `m:` / `Sg:n:` | ChemAxon extended SMILES position-variation bond (`m:star:pos1.pos2...`) and repeating structural group (`Sg:n:atomIndices:count:ht`). |
| Round-trip coverage | Recall fraction `covered / total` of input structures recovered by enumerating the synthesized CX-SMILES. |
| Non-null JSON leaf count | Per-top-level-key count of non-null, non-empty JSON leaf values (excluding object keys). |

## 2. Coverage Ledger (407 / 407 `git ls-files`)

| File | Reviewed | Findings |
| --- | :---: | ---: |
| `.cargo/config.toml` | y | 1 |
| `.cargo/mutants.toml` | y | 3 |
| `.config/nextest.toml` | y | 2 |
| `.echoes.toml` | y | 0 |
| `.editorconfig` | y | 3 |
| `.gitattributes` | y | 1 |
| `.github/CODE_OF_CONDUCT.md` | y | 0 |
| `.github/CONTRIBUTING.md` | y | 4 |
| `.github/ISSUE_TEMPLATE/bug_report.yml` | y | 1 |
| `.github/ISSUE_TEMPLATE/config.yml` | y | 0 |
| `.github/ISSUE_TEMPLATE/feature_request.yml` | y | 1 |
| `.github/PULL_REQUEST_TEMPLATE.md` | y | 0 |
| `.github/RELEASE_CHECKLIST.md` | y | 3 |
| `.github/SECURITY.md` | y | 1 |
| `.github/SUPPORT.md` | y | 0 |
| `.github/actions/setup/action.yml` | y | 0 |
| `.github/ai/AI_AGENT_GUIDE.md` | y | 3 |
| `.github/ai/AI_EFFICIENCY_GUIDE.md` | y | 1 |
| `.github/ai/CONTRIBUTING_AI.md` | y | 0 |
| `.github/dependabot.yml` | y | 0 |
| `.github/workflows/ci.yml` | y | 1 |
| `.github/workflows/codeql.yml` | y | 1 |
| `.github/workflows/dependency-review.yml` | y | 0 |
| `.github/workflows/deploy.yml` | y | 1 |
| `.github/workflows/nightly.yml` | y | 0 |
| `.github/workflows/release.yml` | y | 0 |
| `.gitignore` | y | 3 |
| `.graphifyignore` | y | 0 |
| `.mcp.json` | y | 1 |
| `.mk/catalog.sh` | y | 1 |
| `.mk/core.sh` | y | 0 |
| `.mk/gates.sh` | y | 0 |
| `.mk/meta.py` | y | 0 |
| `.mk/mutants.sh` | y | 0 |
| `.mk/mutants_plan.sh` | y | 0 |
| `.mk/nightly.sh` | y | 0 |
| `.mk/tier.sh` | y | 0 |
| `.mk/tools.sh` | y | 0 |
| `.refquiem.toml` | y | 0 |
| `.tombi.toml` | y | 0 |
| `.typos.toml` | y | 1 |
| `.vscode/README.md` | y | 2 |
| `.vscode/extensions.json` | y | 0 |
| `.vscode/settings.json` | y | 1 |
| `AGENTS.md` | y | 0 |
| `CHANGELOG.md` | y | 0 |
| `Cargo.lock` | N/A (generated Cargo lockfile) | 0 |
| `Cargo.toml` | y | 4 |
| `LICENSE` | N/A (verbatim GNU AGPL v3.0 license text) | 0 |
| `Makefile.toml` | y | 3 |
| `README.md` | y | 4 |
| `apps/cxsmiles-yoga/Cargo.toml` | y | 1 |
| `apps/cxsmiles-yoga/Dioxus.toml` | y | 0 |
| `apps/cxsmiles-yoga/README.md` | y | 8 |
| `apps/cxsmiles-yoga/assets/.gitkeep` | y | 0 |
| `apps/cxsmiles-yoga/src/app.rs` | y | 3 |
| `apps/cxsmiles-yoga/src/app/clipboard.rs` | y | 3 |
| `apps/cxsmiles-yoga/src/app/controls.rs` | y | 1 |
| `apps/cxsmiles-yoga/src/app/results.rs` | y | 5 |
| `apps/cxsmiles-yoga/src/examples.rs` | y | 1 |
| `apps/cxsmiles-yoga/src/lib.rs` | y | 0 |
| `apps/cxsmiles-yoga/src/main.rs` | y | 1 |
| `apps/index/Cargo.toml` | y | 0 |
| `apps/index/Dioxus.toml` | y | 0 |
| `apps/index/README.md` | y | 0 |
| `apps/index/README.tpl` | y | 0 |
| `apps/index/src/app_card.rs` | y | 0 |
| `apps/index/src/main.rs` | y | 6 |
| `apps/json-count-rs/Cargo.toml` | y | 0 |
| `apps/json-count-rs/Dioxus.toml` | y | 0 |
| `apps/json-count-rs/README.md` | y | 0 |
| `apps/json-count-rs/README.tpl` | y | 0 |
| `apps/json-count-rs/src/app_state.rs` | y | 1 |
| `apps/json-count-rs/src/main.rs` | y | 2 |
| `apps/json-count-rs/src/wasm_entry.rs` | y | 4 |
| `apps/lipid-selecto-rs/Cargo.toml` | y | 0 |
| `apps/lipid-selecto-rs/Dioxus.toml` | y | 0 |
| `apps/lipid-selecto-rs/LIPID_ARCHITECTURE.md` | y | 3 |
| `apps/lipid-selecto-rs/README.md` | y | 0 |
| `apps/lipid-selecto-rs/README.tpl` | y | 0 |
| `apps/lipid-selecto-rs/RULES_GUIDE.md` | y | 3 |
| `apps/lipid-selecto-rs/assets/.gitkeep` | y | 1 |
| `apps/lipid-selecto-rs/assets/example_lipids.smi` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/README.md` | y | 3 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.lock` | N/A (generated Cargo lockfile) | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/README.md` | y | 3 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/download.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/evolve.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/ordering.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/report.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/split.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli_tests.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/download.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/download/sdf.rs` | y | 3 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/download/sdf/tests.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/error.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/evolve.rs` | y | 4 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/evolve/observer.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/evolve/tests.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/lib.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/main.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/manifest.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/results_io.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/splitting.rs` | y | 4 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/splitting/slug.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/splitting/tests.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/splitting/types.rs` | y | 1 |
| `apps/lipid-selecto-rs/src/app.rs` | y | 6 |
| `apps/lipid-selecto-rs/src/app/browser.rs` | y | 2 |
| `apps/lipid-selecto-rs/src/app/classes_card.rs` | y | 0 |
| `apps/lipid-selecto-rs/src/app/download.rs` | y | 0 |
| `apps/lipid-selecto-rs/src/app/filter_controls.rs` | y | 1 |
| `apps/lipid-selecto-rs/src/app/gallery_panel.rs` | y | 1 |
| `apps/lipid-selecto-rs/src/app/summary_panel.rs` | y | 3 |
| `apps/lipid-selecto-rs/src/app/types.rs` | y | 0 |
| `apps/lipid-selecto-rs/src/lib.rs` | y | 0 |
| `apps/lipid-selecto-rs/src/main.rs` | y | 2 |
| `apps/mgf-precursor-erro-rs/Cargo.toml` | y | 1 |
| `apps/mgf-precursor-erro-rs/Dioxus.toml` | y | 0 |
| `apps/mgf-precursor-erro-rs/README.md` | y | 0 |
| `apps/mgf-precursor-erro-rs/README.tpl` | y | 0 |
| `apps/mgf-precursor-erro-rs/examples/recalibration_demo.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app.rs` | y | 2 |
| `apps/mgf-precursor-erro-rs/src/app/browser.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app/diagnostics_panel.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app/plots.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app/recalibration_panel.rs` | y | 4 |
| `apps/mgf-precursor-erro-rs/src/app/results.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app/warnings_panel.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/errors.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/lib.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/main.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/plotting.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/plotting/color.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/plotting/cumulative.rs` | y | 6 |
| `apps/mgf-precursor-erro-rs/src/plotting/cumulative/tests.rs` | y | 2 |
| `apps/mgf-precursor-erro-rs/src/plotting/data.rs` | y | 6 |
| `apps/mgf-precursor-erro-rs/src/plotting/diagnostics.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/plotting/diagnostics/tests.rs` | y | 1 |
| `apps/mgf-precursor-erro-rs/src/plotting/histogram.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/plotting/scatter.rs` | y | 7 |
| `apps/mgf-precursor-erro-rs/src/plotting/scatter/tests.rs` | y | 2 |
| `apps/mgf-precursor-erro-rs/src/plotting/svg.rs` | y | 2 |
| `apps/smellfish-rs/Cargo.toml` | y | 0 |
| `apps/smellfish-rs/Dioxus.toml` | y | 1 |
| `apps/smellfish-rs/README.md` | y | 1 |
| `apps/smellfish-rs/README.tpl` | y | 0 |
| `apps/smellfish-rs/assets/rdkit_bridge.js` | y | 11 |
| `apps/smellfish-rs/assets/smellfish.css` | y | 1 |
| `apps/smellfish-rs/build.rs` | y | 1 |
| `apps/smellfish-rs/public/.gitkeep` | y | 1 |
| `apps/smellfish-rs/public/ertl_kingdom_enrichment.txt` | y | 2 |
| `apps/smellfish-rs/public/ertl_npsubstituents.txt` | y | 1 |
| `apps/smellfish-rs/public/ertl_source_vs_synthetic.txt` | y | 2 |
| `apps/smellfish-rs/public/group_names.txt` | y | 2 |
| `apps/smellfish-rs/public/lotus_1percent_scaffolds.txt` | y | 1 |
| `apps/smellfish-rs/public/motif-library.js` | y | 6 |
| `apps/smellfish-rs/public/np_model.bin` | y | 2 |
| `apps/smellfish-rs/public/np_model.bin.gz` | y | 1 |
| `apps/smellfish-rs/public/user_kingdom_enrichment.txt` | y | 1 |
| `apps/smellfish-rs/public/user_source_vs_synthetic.txt` | y | 1 |
| `apps/smellfish-rs/src/app.rs` | y | 1 |
| `apps/smellfish-rs/src/app/browser.rs` | y | 1 |
| `apps/smellfish-rs/src/app/footer.rs` | y | 1 |
| `apps/smellfish-rs/src/app/formatting.rs` | y | 1 |
| `apps/smellfish-rs/src/app/input_panel.rs` | y | 0 |
| `apps/smellfish-rs/src/app/molecule_card.rs` | y | 5 |
| `apps/smellfish-rs/src/app/motif_display.rs` | y | 1 |
| `apps/smellfish-rs/src/app/motif_display/tests.rs` | y | 1 |
| `apps/smellfish-rs/src/app/results.rs` | y | 1 |
| `apps/smellfish-rs/src/document_head.rs` | y | 4 |
| `apps/smellfish-rs/src/lib.rs` | y | 1 |
| `apps/smellfish-rs/src/literature.rs` | y | 1 |
| `apps/smellfish-rs/src/main.rs` | y | 1 |
| `apps/smellfish-rs/src/pipeline.rs` | y | 1 |
| `apps/smellfish-rs/src/rdkit.rs` | y | 0 |
| `apps/smellfish-rs/src/rdkit_bridge.rs` | y | 1 |
| `apps/smellfish-rs/src/styles.rs` | y | 0 |
| `bacon.toml` | y | 0 |
| `clippy.toml` | y | 1 |
| `crates/chem-depict/Cargo.toml` | y | 0 |
| `crates/chem-depict/README.md` | y | 1 |
| `crates/chem-depict/README.tpl` | y | 0 |
| `crates/chem-depict/src/error.rs` | y | 1 |
| `crates/chem-depict/src/lib.rs` | y | 2 |
| `crates/chem-depict/src/parse.rs` | y | 2 |
| `crates/chem-depict/src/svg.rs` | y | 7 |
| `crates/chem-depict/src/tests.rs` | y | 1 |
| `crates/cxsmiles-core/Cargo.toml` | y | 0 |
| `crates/cxsmiles-core/README.md` | y | 1 |
| `crates/cxsmiles-core/README.tpl` | y | 0 |
| `crates/cxsmiles-core/src/aromatic.rs` | y | 2 |
| `crates/cxsmiles-core/src/error.rs` | y | 3 |
| `crates/cxsmiles-core/src/fragment.rs` | y | 4 |
| `crates/cxsmiles-core/src/graph.rs` | y | 4 |
| `crates/cxsmiles-core/src/lib.rs` | y | 1 |
| `crates/cxsmiles-core/src/parse.rs` | y | 3 |
| `crates/cxsmiles-core/src/pipeline.rs` | y | 5 |
| `crates/cxsmiles-core/src/positional.rs` | y | 11 |
| `crates/cxsmiles-core/src/repeating.rs` | y | 8 |
| `crates/cxsmiles-core/src/roundtrip.rs` | y | 10 |
| `crates/cxsmiles-core/src/splice.rs` | y | 3 |
| `crates/cxsmiles-core/src/tests.rs` | y | 1 |
| `crates/cxsmiles-core/src/tests/e2e.rs` | y | 3 |
| `crates/cxsmiles-core/src/tests/graph.rs` | y | 2 |
| `crates/cxsmiles-core/src/tests/limits.rs` | y | 2 |
| `crates/cxsmiles-core/src/tests/parse_tests.rs` | y | 2 |
| `crates/cxsmiles-core/src/tests/positional_builders.rs` | y | 2 |
| `crates/cxsmiles-core/src/tests/positional_helpers.rs` | y | 2 |
| `crates/cxsmiles-core/src/tests/repeating_builders.rs` | y | 1 |
| `crates/cxsmiles-core/src/tests/repeating_helpers.rs` | y | 2 |
| `crates/cxsmiles-core/src/tests/roundtrip.rs` | y | 1 |
| `crates/cxsmiles-core/src/types.rs` | y | 2 |
| `crates/json-count-core/Cargo.toml` | y | 0 |
| `crates/json-count-core/README.md` | y | 0 |
| `crates/json-count-core/README.tpl` | y | 0 |
| `crates/json-count-core/src/container.rs` | y | 3 |
| `crates/json-count-core/src/error.rs` | y | 0 |
| `crates/json-count-core/src/lib.rs` | y | 1 |
| `crates/json-count-core/src/model.rs` | y | 2 |
| `crates/json-count-core/src/number.rs` | y | 0 |
| `crates/json-count-core/src/scanner.rs` | y | 2 |
| `crates/json-count-core/src/strings.rs` | y | 2 |
| `crates/json-count-core/src/tests.rs` | y | 0 |
| `crates/json-count-core/src/tests/boundaries.rs` | y | 1 |
| `crates/json-count-core/src/tests/escaping.rs` | y | 0 |
| `crates/json-count-core/src/tests/sorting.rs` | y | 0 |
| `crates/json-count-core/src/tests/streaming.rs` | y | 1 |
| `crates/json-count-core/src/unescape.rs` | y | 3 |
| `crates/lipid-core/Cargo.toml` | y | 0 |
| `crates/lipid-core/README.md` | y | 1 |
| `crates/lipid-core/README.tpl` | y | 0 |
| `crates/lipid-core/src/bitset.rs` | y | 2 |
| `crates/lipid-core/src/chemical_class.rs` | y | 1 |
| `crates/lipid-core/src/chemical_class/defaults.rs` | y | 2 |
| `crates/lipid-core/src/chemical_class/lmsd.rs` | y | 0 |
| `crates/lipid-core/src/chemical_class/lmsd_part1.rs` | y | 2 |
| `crates/lipid-core/src/chemical_class/lmsd_part2.rs` | y | 2 |
| `crates/lipid-core/src/examples.rs` | y | 0 |
| `crates/lipid-core/src/examples/fatty_acyls.rs` | y | 1 |
| `crates/lipid-core/src/examples/glycerolipids.rs` | y | 0 |
| `crates/lipid-core/src/examples/glycerophospholipids.rs` | y | 0 |
| `crates/lipid-core/src/examples/polyketides.rs` | y | 0 |
| `crates/lipid-core/src/examples/prenol_lipids.rs` | y | 0 |
| `crates/lipid-core/src/examples/saccharolipids.rs` | y | 0 |
| `crates/lipid-core/src/examples/sphingolipids.rs` | y | 0 |
| `crates/lipid-core/src/examples/sterol_lipids.rs` | y | 0 |
| `crates/lipid-core/src/export.rs` | y | 4 |
| `crates/lipid-core/src/family.rs` | y | 1 |
| `crates/lipid-core/src/filter.rs` | y | 2 |
| `crates/lipid-core/src/format.rs` | y | 1 |
| `crates/lipid-core/src/lib.rs` | y | 1 |
| `crates/lipid-core/src/lipids.rs` | y | 0 |
| `crates/lipid-core/src/lipids/classify.rs` | y | 4 |
| `crates/lipid-core/src/lipids/types.rs` | y | 2 |
| `crates/lipid-core/src/parser.rs` | y | 0 |
| `crates/lipid-core/src/parser/analysis.rs` | y | 3 |
| `crates/lipid-core/src/parser/block.rs` | y | 5 |
| `crates/lipid-core/src/parser/extractor.rs` | y | 3 |
| `crates/lipid-core/src/tests.rs` | y | 0 |
| `crates/lipid-core/src/tests/analysis.rs` | y | 1 |
| `crates/lipid-core/src/tests/chemical_class.rs` | y | 2 |
| `crates/lipid-core/src/tests/export_and_filter.rs` | y | 1 |
| `crates/lipid-core/src/tests/format.rs` | y | 0 |
| `crates/lipid-core/src/tests/lipids.rs` | y | 1 |
| `crates/mgf-analysis-core/Cargo.toml` | y | 0 |
| `crates/mgf-analysis-core/README.md` | y | 0 |
| `crates/mgf-analysis-core/README.tpl` | y | 0 |
| `crates/mgf-analysis-core/src/adduct.rs` | y | 1 |
| `crates/mgf-analysis-core/src/adduct/parse.rs` | y | 3 |
| `crates/mgf-analysis-core/src/diagnostics.rs` | y | 2 |
| `crates/mgf-analysis-core/src/diagnostics/model.rs` | y | 0 |
| `crates/mgf-analysis-core/src/diagnostics/stats.rs` | y | 2 |
| `crates/mgf-analysis-core/src/ingestor.rs` | y | 2 |
| `crates/mgf-analysis-core/src/lib.rs` | y | 0 |
| `crates/mgf-analysis-core/src/mass.rs` | y | 0 |
| `crates/mgf-analysis-core/src/metrics.rs` | y | 0 |
| `crates/mgf-analysis-core/src/metrics/record.rs` | y | 3 |
| `crates/mgf-analysis-core/src/metrics/stats.rs` | y | 1 |
| `crates/mgf-analysis-core/src/numeric.rs` | y | 1 |
| `crates/mgf-analysis-core/src/recalibration.rs` | y | 0 |
| `crates/mgf-analysis-core/src/recalibration/calibration.rs` | y | 2 |
| `crates/mgf-analysis-core/src/recalibration/generator.rs` | y | 2 |
| `crates/mgf-analysis-core/src/recalibration/parsing.rs` | y | 0 |
| `crates/mgf-analysis-core/src/recalibration/types.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/adduct.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/calibration.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/diagnostics.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/generator.rs` | y | 1 |
| `crates/mgf-analysis-core/src/tests/ingestor.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/mass.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/metrics.rs` | y | 2 |
| `crates/mgf/Cargo.toml` | y | 0 |
| `crates/mgf/README.md` | y | 0 |
| `crates/mgf/README.tpl` | y | 0 |
| `crates/mgf/src/lib.rs` | y | 0 |
| `crates/mgf/src/parser.rs` | y | 3 |
| `crates/mgf/src/tests.rs` | y | 1 |
| `crates/mgf/src/types.rs` | y | 0 |
| `crates/smellfish-core/Cargo.toml` | y | 0 |
| `crates/smellfish-core/README.md` | y | 0 |
| `crates/smellfish-core/README.tpl` | y | 0 |
| `crates/smellfish-core/src/csv.rs` | y | 3 |
| `crates/smellfish-core/src/error.rs` | y | 0 |
| `crates/smellfish-core/src/evidence.rs` | y | 0 |
| `crates/smellfish-core/src/evidence/assessment.rs` | y | 6 |
| `crates/smellfish-core/src/evidence/chemist.rs` | y | 4 |
| `crates/smellfish-core/src/evidence/ring_family.rs` | y | 2 |
| `crates/smellfish-core/src/evidence/verdict.rs` | y | 3 |
| `crates/smellfish-core/src/lib.rs` | y | 0 |
| `crates/smellfish-core/src/model.rs` | y | 2 |
| `crates/smellfish-core/src/model/classification.rs` | y | 0 |
| `crates/smellfish-core/src/model/types.rs` | y | 1 |
| `crates/smellfish-core/src/pipeline.rs` | y | 4 |
| `crates/smellfish-core/src/qlever.rs` | y | 2 |
| `crates/smellfish-core/src/query.rs` | y | 4 |
| `crates/smellfish-core/src/sparql.rs` | y | 2 |
| `crates/smellfish-core/src/svg.rs` | y | 1 |
| `crates/smellfish-core/src/tests.rs` | y | 0 |
| `crates/smellfish-core/src/tests/csv.rs` | y | 1 |
| `crates/smellfish-core/src/tests/evidence.rs` | y | 3 |
| `crates/smellfish-core/src/tests/pipeline.rs` | y | 1 |
| `crates/smellfish-core/src/tests/qlever.rs` | y | 0 |
| `crates/smellfish-core/src/tests/query.rs` | y | 0 |
| `crates/smellfish-core/src/tests/ring_family.rs` | y | 1 |
| `crates/smellfish-core/src/tests/sparql.rs` | y | 0 |
| `crates/smellfish-core/src/tests/svg.rs` | y | 0 |
| `crates/smellfish-core/src/tests/verdict.rs` | y | 1 |
| `crates/stream-io/Cargo.toml` | y | 0 |
| `crates/stream-io/README.md` | y | 0 |
| `crates/stream-io/README.tpl` | y | 0 |
| `crates/stream-io/src/blob_cursor.rs` | y | 2 |
| `crates/stream-io/src/blob_lines.rs` | y | 1 |
| `crates/stream-io/src/blob_lines/tests.rs` | y | 0 |
| `crates/stream-io/src/buffer.rs` | y | 1 |
| `crates/stream-io/src/bytes.rs` | y | 1 |
| `crates/stream-io/src/error.rs` | y | 0 |
| `crates/stream-io/src/lib.rs` | y | 1 |
| `crates/stream-io/src/progress.rs` | y | 0 |
| `crates/stream-io/src/progress/tests.rs` | y | 0 |
| `crates/ui/Cargo.toml` | y | 0 |
| `crates/ui/README.md` | y | 0 |
| `crates/ui/README.tpl` | y | 0 |
| `crates/ui/src/a11y.rs` | y | 1 |
| `crates/ui/src/a11y/tests.rs` | y | 0 |
| `crates/ui/src/components.rs` | y | 0 |
| `crates/ui/src/components/button.rs` | y | 0 |
| `crates/ui/src/components/card.rs` | y | 1 |
| `crates/ui/src/components/footer.rs` | y | 1 |
| `crates/ui/src/components/header.rs` | y | 0 |
| `crates/ui/src/components/notice.rs` | y | 0 |
| `crates/ui/src/components/segmented_control.rs` | y | 1 |
| `crates/ui/src/components/simple_analytics_badge.rs` | y | 1 |
| `crates/ui/src/components/upload_zone.rs` | y | 3 |
| `crates/ui/src/document.rs` | y | 4 |
| `crates/ui/src/lib.rs` | y | 0 |
| `crates/ui/src/signals.rs` | y | 0 |
| `crates/ui/src/theme.rs` | y | 4 |
| `crates/ui/src/theme/tests.rs` | y | 1 |
| `crates/ui/src/url.rs` | y | 0 |
| `crates/ui/src/webmcp.rs` | y | 2 |
| `crates/ui/src/webmcp/tests.rs` | y | 2 |
| `crates/upload/Cargo.toml` | y | 0 |
| `crates/upload/README.md` | y | 1 |
| `crates/upload/README.tpl` | y | 1 |
| `crates/upload/src/bytes.rs` | y | 1 |
| `crates/upload/src/download.rs` | y | 3 |
| `crates/upload/src/download/tests.rs` | y | 1 |
| `crates/upload/src/error.rs` | y | 0 |
| `crates/upload/src/event.rs` | y | 1 |
| `crates/upload/src/lib.rs` | y | 1 |
| `crates/workspace-gates/Cargo.toml` | y | 0 |
| `crates/workspace-gates/README.md` | y | 0 |
| `crates/workspace-gates/README.tpl` | y | 0 |
| `crates/workspace-gates/src/lib.rs` | y | 0 |
| `crates/workspace-gates/tests/gate_consistency/apps.rs` | y | 2 |
| `crates/workspace-gates/tests/gate_consistency/data_files.rs` | y | 1 |
| `crates/workspace-gates/tests/gate_consistency/main.rs` | y | 1 |
| `crates/workspace-gates/tests/gate_consistency/mk_runner.rs` | y | 2 |
| `crates/workspace-gates/tests/gate_consistency/mutants.rs` | y | 1 |
| `crates/workspace-gates/tests/gate_consistency/surfaces.rs` | y | 0 |
| `crates/workspace-gates/tests/gate_consistency/task_files.rs` | y | 1 |
| `crates/workspace-gates/tests/gate_consistency/workflows.rs` | y | 2 |
| `deny.toml` | y | 1 |
| `docs/ARCHITECTURE.md` | y | 1 |
| `docs/DEBT.md` | y | 1 |
| `make/all.toml` | y | 1 |
| `make/build.toml` | y | 3 |
| `make/format.toml` | y | 2 |
| `make/lint.toml` | y | 2 |
| `make/readme.toml` | y | 1 |
| `make/scripts/license-headers.sh` | y | 0 |
| `make/scripts/setup.sh` | y | 2 |
| `make/scripts/unsafe-audit.sh` | y | 4 |
| `make/supply-chain.toml` | y | 2 |
| `make/test.toml` | y | 2 |
| `make/web.toml` | y | 1 |
| `mk` | y | 0 |
| `mk.local` | y | 2 |
| `mk.versions` | y | 0 |
| `prek.toml` | y | 0 |
| `renovate.json` | y | 0 |
| `rust-toolchain.toml` | y | 0 |
| `rustfmt.toml` | y | 1 |

## 3. Findings Table

| ID | Severity | Category | Location | Description | Evidence | Fix | Commit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| MGF-01 | MAJOR | math/impl | `crates/mgf/src/parser.rs:10-70` | `parse_leading_float`, `parse_peak_line`, and `EXACTMASS`/`MOLECULEMASS` accept `NaN`/`Inf`; malformed `PRECURSOR_MZ=` overwrites valid `PEPMASS=` with `None` | Unit inspection; `NaN` poisons means/buckets | Require `.is_finite()`, only update `latest_precursor_mz` on `Some` | `fd03015c` |
| MGF-02 | MAJOR | math | `crates/mgf-analysis-core/src/numeric.rs:54-60` | Affine `reservoir_index` preserves parity for even `seen` (`seen=10000` -> `idx` always odd) | Algebraic parity check: `(k * odd + odd) % (2m)` parity locked to `k` | Replace with full SplitMix64 bit-mixing cascade | `9539e63f` |
| MGF-03 | MAJOR | math/impl | `crates/mgf-analysis-core/src/adduct/parse.rs:100-303` | `parse_ascii_digits_f64` concatenates non-contiguous digits (`"2+ 3+"` -> `23`), accepts `0` multiplicity, duplicate `M`, empty operands (`[M+]+`), and `normalize_adduct_label` accepts mismatched charge suffixes (`[M-H]--`) | Parser trace on `2+ 3+`, `[M+M+H]+`, `[M-H]--` | Parse first contiguous digit run > 0; reject duplicate `M`/empty operands; validate matching charge suffix; extract named element constants | `9539e63f` |
| MGF-04 | MAJOR | math | `crates/mgf-analysis-core/src/ingestor.rs:166-220` | Non-positive/non-finite `expected_mz` records `NaN`/signed-flipped ppm; missing SMILES/formula/exactmass increments `unparsed_smiles` | Trace on `EXACTMASS=0.0` or missing structure | Skip non-positive/non-finite `expected_mz`; count `unparsed_smiles` only when `smiles` is non-empty | `9539e63f` |
| MGF-05 | MAJOR | math | `crates/mgf-analysis-core/src/metrics/{record,stats}.rs` | Incremental RMS squares and takes `sqrt()` on every observation; `update_high_error_smiles` leaves stale `formula` on new max | Floating-point error accumulation across `N` steps | Track running `mean_sq` and Welford-style mean update; update `formula` with worst error; extract `HIGH_ERROR_THRESHOLD_DA` | `9539e63f` |
| MGF-06 | MAJOR | math | `crates/mgf-analysis-core/src/recalibration/{calibration,generator}.rs` | `OrbitrapPPM` uses first-order `mz*(1-lambda*delta/pm)` instead of exact `mz/(1+lambda*delta/pm)` (fails exact self-calibration at `lambda=1`); `recalibrate_fragments` allows negative `precursor_ms1`; `generate_recalibrated_mgf` emits synthetic `END IONS` on unterminated EOF block | At `lambda=1`, `ms2*(1-(ms2-ms1)/ms1) != ms1` (error `(delta/ms1)^2 * ms1`) | Use exact inverse scaling `mz / (1.0 + lambda * delta / pepmass)` with `denom > 0`; check `precursor_ms1 > 0.0`; preserve EOF without fake `END IONS` | `9539e63f` |
| MGF-07 | MAJOR | math/impl | `crates/mgf-analysis-core/src/diagnostics.rs:61-92`, `diagnostics/stats.rs` | `compute_recalibration_diagnostics` fabricates `delta=0` when `ms2_precursor_peak` is `None`, discards `adduct_family` (`error_by_adduct_*` stays empty), and `RunningStats` fallback mixes non-finite inputs | Code inspection of lines 61 and 90 | Skip points lacking `ms2_precursor_peak`; pass `Some(point.adduct_family.as_str())`; filter non-finite inputs | `9539e63f` |
| MGF-08 | CRITICAL | math | `apps/mgf-precursor-erro-rs/src/plotting/histogram.rs:108-118` | `before` bar width `0.9*w` vs `after` bar width `0.45*w` overlaps and inflates `before` area 2x | Rectangle bounds `[x, x+0.9w]` vs `[x+0.45w, x+0.9w]` | Use equal side-by-side widths `[x, x+0.45w]` and `[x+0.45w, x+0.90w]` plus 10% y-headroom | `0dcf2e8b` |
| MGF-09 | CRITICAL | math | `apps/mgf-precursor-erro-rs/src/plotting/cumulative.rs:12-30,108-147` | 3-curve cumulative plot is piecewise-linear on ties, drops 1-point curves, extends x-axis negative, ignores `_thresholds` | Code trace on `[1.0, 1.0, 2.0]` and `[1.0]` | Emit right-continuous step function starting at `(x_min, 0.0)`, clamp x >= 0, render threshold lines | `0dcf2e8b` |
| MGF-10 | CRITICAL | math | `apps/mgf-precursor-erro-rs/src/plotting/scatter.rs:53-114`, `data.rs:137-175` | Log-scale ECDF clamps out-of-range thresholds onto `x_min`; draws fake diagonal on empty data; `.take(50_000)` before sort; `f64::EPSILON` merges log-scale values; jumps backwards when `value > x_max` | Calculation when `observed_min = 15 ppm` | Skip out-of-range thresholds, return empty on no data, use exact tie equality, downsample post-sort, clip at `x_max` | `0dcf2e8b` |
| MGF-11 | MAJOR | math/impl | `apps/mgf-precursor-erro-rs/src/plotting/{scatter,diagnostics,svg,data}.rs`, `scatter/tests.rs` | Zero-error scatter produces `0..0` y-range; out-of-range ticks stack on `±y_limit`; BATLOW 4-vs-5 step mismatch; diagnostics legend missing swatches; `embed_svg_legend` overflows width without XML escaping | Test `scatter/tests.rs` accepts `Err(PlotError)`; `embed_svg_legend` omits `title_width + 20` | Add fallback `y_limit >= 1.0`, skip out-of-range ticks, unify 5-step BATLOW, add legend swatches, fix box width + XML escaping | `0dcf2e8b` |
| MGF-12 | MAJOR | semantic/impl | `apps/mgf-precursor-erro-rs/src/app/{results,recalibration_panel,diagnostics_panel,plots,browser}.rs`, `examples/recalibration_demo.rs`, `Cargo.toml` | `spectra==0` renders `0.0000`; `mean_imp` uses signed mean vs MAE; reservoir slots labeled `Scan Index`; `EcdfPlot` silently switches MS1->MS2; stale `original_mgf_content` leaked on error + concurrent upload race; demo units 20x inconsistent; unused `js-sys` | Code inspection + demo unit check (`10.5 ppm` vs `0.1050 Da`) | Show `n/a` when empty, show MAE alongside signed mean, label `Sample Index`, clarify ECDF subtitle, clear MGF buffer + generation guard, fix demo and `Cargo.toml` (`OQ-9` open) | `0dcf2e8b` |
| JSON-01 | CRITICAL | semantic/math | `crates/json-count-core/src/container.rs:47-101`, `tests/streaming.rs:19-32` | `scan_container` counts nested object keys as non-null leaf values (`{"x":{"k":null}}` -> 1) and tests codify it | Trace on nested objects/arrays | Track per-frame object/array state; never count object keys as values | `15674f5a` |
| JSON-02 | CRITICAL | impl | `crates/json-count-core/src/container.rs:31-101`, `tests/streaming.rs:26` | `scan_container` skips RFC 8259 grammar validation (accepts `[1,2,"b":3]`, trailing commas, missing colons) | Confirmed by `tests/streaming.rs:26` | Enforce RFC 8259 frame state transitions and return `JsonCountError` on malformed syntax | `15674f5a` |
| JSON-03 | CRITICAL | impl | `crates/json-count-core/src/unescape.rs:8-84`, `strings.rs:27-121` | `unescape.rs` accepts `\u+041`, silently drops invalid `\u00zz`/unknown escapes, produces Latin-1 mojibake on `\` + UTF-8; top-level scanners accept control chars `< 0x20` and invalid UTF-8; <!-- refquiem: allow(pass_bound) -->`pass_bound` breaks when `total_bytes == 0` | Rust `u16::from_str_radix("+041", 16) == Ok(0x41)` | Return `Result` from unescape, require 4 hex digits, reject invalid escapes/controls/UTF-8, stream until `fill() == false` | `15674f5a` |
| JSON-04 | MAJOR | semantic/impl | `crates/json-count-core/src/{model,scanner,lib}.rs`, `apps/json-count-rs/src/{wasm_entry,app_state,main}.rs` | `model.rs` `u64` overflow on `displayed*100` & `0/1 bytes`; concurrent upload race in `wasm_entry.rs`; missing `tr` `key`; `""`/`[]`/`{}` semantics (logged `OQ-8`) | Code inspection | Use `u128` for percentage, format `0/0`, add generation race guard, add `tr` key, use `#[expect]` (`OQ-8` open) | `15674f5a` |
| STRM-01 | MAJOR | impl | `crates/stream-io/src/{buffer,bytes,blob_lines,blob_cursor,lib}.rs` | `ChunkBuffer::pull_chunk` advances `source_read` by `end` on short reads; `SliceSource::slice_clock` panics on `wasm32` via `Instant::now()`; dead `.max(1)` and `#[allow]` | Code inspection of `buffer.rs:70` & `bytes.rs:81` | Advance by `chunk.len()`, handle empty EOF read, gate `slice_clock` on `wasm32`, use `#[expect]` | `5ef4f0ad` |
| UPLD-01 | MAJOR | impl/slop | `crates/upload/{README.tpl,README.md,src/download.rs,src/lib.rs}` | README falsely claims re-exporting `stream-io` types; `sanitize_filename` leaves trailing dot on `"a. ."` and passes `"CON .txt"`; 1s revoke timeout; `download_svg` falls back to `.txt` | String trace on `"a. ."` and `"CON .txt"` | Fix `README.tpl` + regenerate; strip dots/whitespace iteratively + trim stem before reserved check; 60s revoke timeout; `.svg` fallback | `5ef4f0ad` |
| CX-01 | CRITICAL | math/impl | `crates/cxsmiles-core/src/repeating.rs:131-210` | <!-- refquiem: allow(dfs_repeat) -->`dfs_repeat` expands only neighbors of `cur` (simple paths), failing on all 4+ atom branched repeat units (`-OCF2-`, PPG, GenX) | Graph trace on star $K_{1,3}$ | Replace with connected-subgraph frontier enumeration (canonical min-index deduplication) | `f54e987f` |
| CX-02 | CRITICAL | math/impl | `crates/cxsmiles-core/src/repeating.rs:45-48,231-244`, `splice.rs:10-135` | Duplicate `shortest` molecule defeats `coverage.covered < 2` guard; `is_internal` allows 2 bonds to same spiro anchor causing `splice_repeat` to corrupt atom 0 | Trace on `[CCCC, CCCC, CC(C)CC]` and spiro ring | Require distinct non-scaffold homologue; require 2 distinct external anchor atoms; avoid `.unwrap_or(0)` in `splice.rs` | `f54e987f` |
| CX-03 | CRITICAL | math/semantic | `crates/cxsmiles-core/src/aromatic.rs:47-82` | `expand_aromatic_equiv` equates `(comp, Z, deg)` without H-count or formal charge (equating `[nH]` vs `n`, `[c-]` vs `c`, ortho vs meta/para) while README claims graph automorphisms | Code trace on indole/biphenyl/phenylpyridine | Match formal charge & H availability (and fix README claim; `OQ-7` open) | `f54e987f` |
| CX-04 | CRITICAL | math/semantic | `crates/cxsmiles-core/src/roundtrip.rs:19-59,75-179`, `tests/limits.rs:49-55,164-174` | `enumerate` produces over-valent collisions; `MAX_MOLECULE_ATOMS` ignores scaffold/floating atom counts (513-atom scaffold passes); `.take(count_cap)` drops `n=512`; `InvalidPosition` unreachable while OOB skipped; `part_attach` desync | Tests in `limits.rs` snapshot 513-atom bypass & silent OOB skip | Track valence/implicit-H capacity in `build_one`; check true expanded atom count; validate positions in bounds; fix `limits.rs` | `f54e987f` |
| CX-05 | CRITICAL | math/semantic | `crates/cxsmiles-core/src/{graph,positional,pipeline,parse,error,types}.rs` | `bond_to_primitive` maps `Up`/`Down` to `BondPrimitive::Any`; `molecule_to_query` drops charge/isotope/aromaticity; `equiv_positions` 1st-match breaks on symmetric scaffolds (`e2e.rs` 3-Cl-biphenyl inflates 3->4); `split_floating_and_recovered` adds salts to scaffold; non-determinism on input order | Code trace + `e2e.rs:144-154` | Map `Up`/`Down` to `Single`, preserve atom charge/isotope/aromaticity, canonicalize `group` order, pick min-union embedding, exclude `boundary_edges==0` from `recovered`, try positional if repeat is non-clean | `f54e987f` |
| CX-06 | MAJOR | math/semantic | `crates/cxsmiles-core/src/repeating.rs:79-97` | `repeat_pattern` requires `shortest` to subgraph-embed into `longest`, failing on alpha-omega capped series (`-CF3` PFAS, diacids, diols) | Subgraph check on `OC(=O)C(F)(F)C(F)(F)F` vs `+CF2` | Documented limitation in crate/README; formula-diff redesign logged as `OQ-7` | `f54e987f` (`OQ-7`) |
| CX-07 | CRITICAL | impl/semantic | `crates/chem-depict/src/{parse,svg,lib,error,tests}.rs`, `README.md` | `catch_unwind` is a no-op on `wasm32-unknown-unknown` (`panic="abort"`); 0-atom `Molecule` returns blank rect instead of `empty_svg`; no size DoS guard; `empty_compact_svg` text clipped; missing `</svg>` check | `rustc --print cfg --target wasm32-unknown-unknown \| grep panic` | Add SMILES length & atom-count guards, handle 0-atom molecules, check `</svg>`, widen `empty_compact_svg`, document native-only `catch_unwind` | `7d35bfce` |
| CX-08 | MAJOR | impl/slop | `apps/cxsmiles-yoga/{src/app/*,Cargo.toml,README.md}` | Uncapped 10,000-SVG synchronous rendering freezes tab; `overflow-y:auto` missing `max-height`; `99.6%` rounds to `100%`; clipboard drops `Promise`; missing `label` `r#for`; README falsely claims PFOS (is C4H10O) + 6 other errors | Code & README inspection vs `examples.rs:60-68` | Paginate enumerated SVGs (cap 60 + toggle), fix CSS/rounding, await clipboard `Promise`, wire label `r#for`, fix README | `7d35bfce` |
| LIP-01 | CRITICAL | semantic/test | `crates/lipid-core/src/lipids/classify.rs:29-106`, `tests/lipids.rs:27-30` | `Q_TRIGLYCERIDE`/<!-- refquiem: allow(Q_DIGLYCERIDE) -->`Q_DIGLYCERIDE` are orthoester/gem-diester SMARTS; <!-- refquiem: allow(Q_MONOGLYCERIDE) -->`Q_MONOGLYCERIDE` matches any ester so methyl palmitate (`FA07`) classifies as `Glycerolipids`; acyclic `SP` classified as `GP` and fatty amides as `SP` | SMARTS inspection + `tests/lipids.rs:28` | Replace with glycerol-backbone SMARTS, require glycerol for GP and sphingoid for SP, support `[O-]` in COOH, fix `tests/lipids.rs` | `2987647b` |
| LIP-02 | CRITICAL | semantic | `crates/lipid-core/src/chemical_class/{defaults,lmsd_part1,lmsd_part2}.rs`, `parser/analysis.rs:85-89` | Broken `defaults.rs` SMARTS (phosphonates instead of phosphate esters, orthoesters, hydrindane); `FA`-first ordering labels TGs/PCs/ceramides/cholesterol esters as `Fatty esters [FA07]`; evolved `lmsd_part1/2` overfit artifacts (`OQ-5`) | SMARTS inspection across `defaults.rs` and `lmsd_part{1,2}.rs` | Fix `defaults.rs` SMARTS + add real positive/negative tests; select `primary_class_name` preferring complex lipid families over `FA` (`OQ-5` open) | `2987647b` |
| LIP-03 | CRITICAL | semantic/impl | `crates/lipid-core/src/parser/block.rs:144-185`, `lipids/classify.rs:124-168` | `FORMULA=` fallback fires when valid SMILES was rejected by all 86 SMARTS (classifying benzoic acid/coumarin as `FA00`); `classify_from_counts` accepts aromatic acids, `C15O6P`, non-GL oxygenated compounds | Code trace on `c1ccccc1C(=O)O` + `C7H6O2` | Gate formula fallback on `mol_opt.is_none()`; tighten `classify_from_counts` heteroatom/double-bond-equivalent/H bounds | `2987647b` |
| LIP-04 | MAJOR | impl/math | `crates/lipid-core/src/{export,filter,parser/block,parser/extractor,lipids/types,lib,bitset}.rs` | `export.rs` diverges between `<=96` (`gallery`) and `>96` (`block`); `insert_class_comment` corrupts single-line blocks; `NaN` bypasses `filter.rs`; `parse_smiles_line` swaps bad SMILES with space title; double-bond equivalents formula omits `P/2` | Code trace on `export.rs`, `filter.rs`, `extractor.rs` | Unify `ClassTags` via `tags_from_matched`, sanitize TSV titles & comments, filter non-finite floats, fix `parse_smiles_line`, add `P/2` to double-bond equivalents, fix slop docs (`OQ-6` open) | `2987647b` |
| LIP-05 | MAJOR | impl/slop | `apps/lipid-selecto-rs/{src/**,LIPID_ARCHITECTURE.md,RULES_GUIDE.md,lipidmaps/README.md,assets/.gitkeep}` | Summary count static; gallery caps 96 BEFORE filtering (hiding SP/ST/PR/SL/PK); `adduct_filter` not reset + upload race; bad oleic/arachidonic/PC/LPC SMILES & broken docs; redundant `.gitkeep` | Verified on 204-lipid dataset | Compute live filtered count, filter before cap 96, reset adduct filter + generation guard, fix docs & delete `.gitkeep` | `06f19d8b` |
| EVO-01 | CRITICAL | impl/math/semantic | `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/{download/sdf.rs,evolve/observer.rs,splitting/slug.rs}` | SDF parser panics without trailing newline; `NaN` MCC sorts above `1.0` (test codified); `category_of_main`/`category_rank` fail on real LMSD `[FA01]`/`[FA]` strings | Code trace + `evolve/tests.rs:107-117` | Fix EOF slice bounds + `$$$$`/`>` handling in SDF; sort `NaN` last; parse bracketed LMSD codes | `4b84c57b` |
| EVO-02 | MAJOR | impl/math/slop | `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/{src/**,README.md}` | Duplicate SMILES in class pools; `cli/split.rs` diverges from test `parse_csv` & `--smiles-cap` defeats `--max-negatives`; `--no-tui` buffers all results; `results_io` resume swallows errors; download lacks timeout/atomic write; README drift | Code inspection across `splitting.rs`, `cli/`, `results_io.rs` | Deduplicate class pools, unify parser, fix cap order, stream parallel CSV writes, atomic download, update README | `4b84c57b` |
| SML-01 | CRITICAL | math | `apps/smellfish-rs/assets/rdkit_bridge.js:53-175`, `public/np_model.bin` | Binary Morgan bit-vector used instead of count FP (`83%` error on benzene) + folded collision summing (+8.924 max, 22.4% false hit rate); `np_model.bin.gz` (824 KB) is an unused byte-identical duplicate | RDKit MinimalLib C++ check + binary inspection of `np_model.bin` | Delete unused `np_model.bin.gz`, validate `byteLength % 8 == 0`, fix false doc claims (`OQ-1` open) | `37e7c1a4` |
| SML-02 | CRITICAL | impl/semantic | `crates/smellfish-core/src/evidence/*`, `apps/smellfish-rs/public/motif-library.js`, `assets/rdkit_bridge.js` | 0/268 motif labels match English keywords (`chemist.rs`/`ring_family.rs`/`classifyKind` dead); naive `[R]->[*]` & raw SMILES in `get_qmol` match ethers/aldehydes/benzene; `get_stereo_tags()` returns object so `Array.isArray` yields `[]`; 123/268 motifs (`[R]F`, sulfonamide) marked `unknown`; `user_*.txt` has fake `100/0` gem-dimethyl; `ertl_kingdom_enrichment.txt:220` line-wrapped | Direct inspection of all `public/*.txt` + `minilib.cpp` | Rejoin broken line, clear fake `100/0` placeholder, fix `stereo_tags`/`num_atoms(true)`/`get_substruct_matches`, fall back to `kingdom.values` in `motif-library.js`, fix enrichment cutoffs & quoted `group_names` (`OQ-4` open) | `19901a23`, `37e7c1a4` |
| SML-03 | CRITICAL | semantic | `crates/smellfish-core/src/query.rs:18-70` | WDQS/QLever require exact 27-char `InChIKey` before prefix/isomer traversal; `?item wdt:P235 ?inchikey` sits outside `UNION` in QLever query, breaking flat 2D SMILES lookup | SPARQL query structure trace | Move exact `P235` match inside first `UNION` branch of `build_lotus_query_qlever` | `19901a23` |
| SML-04 | MAJOR | math/impl | `crates/smellfish-core/src/evidence/{verdict,assessment,chemist,ring_family}.rs`, `pipeline.rs`, `csv.rs`, `svg.rs`, `sparql.rs`, `qlever.rs`, `model/types.rs` | Non-monotonic `classify_with_pubchem` + inverted <!-- refquiem: allow(decoration_heavy) -->`decoration_heavy()`; contradictory thresholds across verdict/assessment/chemist; `None` -> `"weak NP signals"` & `0.000` CSV; `NaN` -> `"highly synthetic"`; `PubChem` 308 URL & 353-CID isotopologue flood; `locus` CSV typo & unescaped `np_label`; weak TSV/`;`/single-col CSV; SVG tag boundary gaps | Trace & live endpoint/HTTP checks | Unify thresholds, fix `None`/`NaN` handling, fix `PubChem` URL, fix `classify_with_pubchem`/`classify_motif_context`, fix CSV/SVG/InChIKey14 validation, remove `no_hallucinated_rules` slop test (`OQ-2`, `OQ-3` open) | `19901a23` |
| SML-05 | MAJOR | semantic/impl/slop | `apps/smellfish-rs/src/{literature,document_head,pipeline,app/*,lib,main}.rs`, `assets/smellfish.css`, `README.md` | Wrong Crossref paper title (`10.1021/acs.jnatprod.8b01022`); bad EDAM `operation_0250`; cancellation race still hits SPARQL; pasted SMILES labeled `Demo loaded`; 0-based progress; `lotus_taxa` hidden; `formatting.rs` trivial wrapper; `color-scheme: light dark` without dark CSS; `machine-learned` slop claim | Crossref API + EDAM.tsv + code inspection | Fix literature title, EDAM `operation_2480`, early race return, UI labels/taxa/chip colors/CSS, inline `formatting.rs`, fix docs | `37e7c1a4` |
| UI-01 | CRITICAL | impl | `crates/ui/src/theme.rs:86-107`, `a11y.rs:9-12`, `components/{footer,simple_analytics_badge,upload_zone,segmented_control}.rs`, `document.rs`, `webmcp.rs` | WCAG AA contrast failures in DARK red (2.00:1), DARK blue (2.86:1), LIGHT green (2.98:1), transparent skip link (2.66:1), un-underlined footer links (1.07:1); duplicate `<footer>` landmark; brittle `"error"` color match & missing focus-within on `UploadZone`; `{lang:?}` eval & incomplete `json_ld` escaping; wrong EDAM `operation_0366` | Exact WCAG 2.1 luminance calculations + EDAM.tsv | Fix DARK/LIGHT status hexes + add contrast test, opaque skip-link, underline footer links, `<div>` in badge, fix `UploadZone`, fix `document.rs` escaping & `operation_3432` | `3fbe999a` |
| IDX-01 | MAJOR | semantic/impl | `apps/index/src/main.rs:28-187` | Dark OS mode leaves `#apps-heading` dark-on-dark; `json-count-rs` card falsely claims cardinality/null distributions; `smellfish-rs` card omits NP-likeness; `id` mismatch & changelog comment | Code inspection | Set explicit light background/theme colors, fix card copy, align `id`, remove changelog comment | `3fbe999a` |
| CI-01 | MAJOR | impl | `make/{test,readme,supply-chain,lint,build,web}.toml`, `make/scripts/unsafe-audit.sh`, `.config/nextest.toml`, `.github/workflows/{ci,codeql,deploy}.yml`, `.vscode/{settings.json,README.md}`, `mk.local`, `crates/workspace-gates/tests/gate_consistency/*.rs` | `cargo llvm-cov nextest run` skips tests; `nextest.toml` targets `cxsmiles-yoga` instead of `cxsmiles-core`; `/tmp/readme_panache.md` race; `unsafe-audit.sh` word-splitting & BSD grep `\s`; `action-pins` misses `.github/actions/`; `lint-wasm`/`check-wasm` omit `ui`/`upload`; action SHA drift; invalid VS Code setting; stale gate comments | Help check + config inspection | Fix all CI scripts/configs/workflows and gate comments without weakening any ratchet | `5f74ad46`, `504250cd` |
| DOC-01 | MAJOR | semantic/slop | `docs/DEBT.md`, `README.md`, `docs/ARCHITECTURE.md`, `.github/{CONTRIBUTING,RELEASE_CHECKLIST,SECURITY}.md`, `.github/ISSUE_TEMPLATE/*`, `Cargo.toml`, `deny.toml`, configs, `.github/ai/*`, `.mcp.json` | DEBT.md says Plausible instead of Simple Analytics; README/CONTRIBUTING/ARCHITECTURE stale counts & false claims; GPL-3.0 mislabeled Permissive in `deny.toml`; `.mcp.json` inaccurate read-only claim | Line-by-line cross-check vs `Cargo.toml`/workflows | Fix human docs/configs (`90e48423`) and `.mcp.json` (`f006a60d`) in separate commits per `CONTRIBUTING_AI.md` | `90e48423`, `f006a60d` |

## 4. Behavior Changes

| Component | Metric / Output | Before | After | Justification |
| --- | --- | --- | --- | --- |
| `mgf` | `NaN`/`Inf` floats & invalid `PRECURSOR_MZ=` after valid `PEPMASS=` | Accepted non-finite floats; overwrote `latest_precursor_mz` with `None` | Non-finite values rejected/ignored; valid precursor m/z preserved | Prevents `NaN` propagation into means/RMS and prevents malformed optional tags from erasing valid `PEPMASS` (`fd03015c`) |
| `mgf-analysis-core` | `OrbitrapPPM` fragment shift (`lambda = 1.0`) & `reservoir_index` parity | `mz * (1 - lambda*delta/pm)` (error `(delta/ms1)^2 * ms1` at `lambda=1`); affine `(seed*k + b) % (2m)` parity lock | Exact inverse `mz / (1 + lambda*delta/pm)`; full SplitMix64 mixing | Restores exact self-calibration (`ms2 -> ms1` at `lambda=1`) and removes parity bias in reservoir sampling (`9539e63f`) |
| `mgf-precursor-erro-rs` | Histogram bar widths & cumulative curves | `before` width `0.90*w` vs `after` `0.45*w` (2x area distortion); linear ties & dropped singletons | Side-by-side equal `0.45*w` bars; right-continuous step function | Eliminates 2x visual area inflation and renders mathematically valid ECDF steps (`0dcf2e8b`) |
| `json-count-core` | Nested object keys & malformed RFC 8259 inputs | `{"x":{"k":null}}` returned `1`; malformed syntax accepted | Keys not counted (`0`); malformed RFC 8259 returns `JsonCountError` | Distinguishes JSON object member keys from non-null values and enforces RFC 8259 grammar (`15674f5a`) |
| `cxsmiles-core` | Branched repeat units & `MAX_MOLECULE_ATOMS` | Missed `>= 4`-atom branched repeats; allowed 513-atom scaffolds; over-valent collisions | Enumerates connected branched subgraphs; validates total atoms and valence/H limits | Captures branched homologues (`-OCF2-`, PPG) and closes enumeration overflow/valence bugs (`f54e987f`) |
| `lipid-core` / `lipid-selecto-rs` | Fallback classification, complex lipid classing & 96-tile cap | Invalid/aromatic structures matching 0 SMARTS fell back to `FORMULA=` (`FA00`); `FA` hid TG/PC/SP; slice capped before filter | `FORMULA=` gated on `mol_opt.is_none()`; complex families prioritized over `FA`; filter applied before 96-card slice | Prevents non-lipid false positives and exposes `SP`/`ST`/`PR`/`SL`/`PK` results in large datasets (`2987647b`, `06f19d8b`) |
| `smarts-evoliposuction` | `NaN` MCC ranking & LMSD bracket parsing | `NaN` sorted above `+1.0`; failed on `[FA01]`/`[FA]` headers; `--smiles-cap` throttled negatives | `NaN` sorts last; bracketed LMSD codes parsed; `--max-negatives` applied independently | Prevents invalid SMARTS rules from outranking valid models and aligns CLI with LMSD files (`4b84c57b`) |
| `smellfish-core` / `smellfish-rs` | NP evidence labels, missing/NaN NP scores, QLever `P235` lookup, CSV schema | `None` labeled `"weak NP signals"` & `0.000` in CSV; `NaN` labeled `"highly synthetic"`; `P235` outside `UNION` broke 2D lookup; CSV header `locus` | Unified thresholds (`>=2.0`, `>=1.0`, `>=0.0`, `>-2.0`, `<=-2.0`), `"unavailable"` / empty CSV cell for missing/NaN; `P235` inside `UNION`; `lotus_qids` + `lotus_taxa` | Removes false NP certainty on missing/NaN scores, fixes flat 2D SMILES lookup in QLever, and fixes CSV schema (`19901a23`, `37e7c1a4`) |
| `ui` | `ColorScheme` status palette & skip link | `LIGHT.green` `2.98:1`, `DARK.red` `2.00:1`, `DARK.blue` `2.86:1` below WCAG AA | Updated hex tokens verified `>= 4.5:1`; opaque skip link & underlined footer links | Meets WCAG 2.1 AA normal-text contrast (`>= 4.5:1`) across light and dark schemes (`3fbe999a`) |

## 5. Renames

| Old Identifier / Label | New Identifier / Label | Scope |
| --- | --- | --- |
| `locus` CSV header | `lotus_qids` | `crates/smellfish-core/src/csv.rs`, tests (`19901a23`) |
| `Scan Index` plot label | `Sample Index` | `apps/mgf-precursor-erro-rs/src/app/results.rs` (`0dcf2e8b`) |
| `mgf-precursor-error-rs` app card ID | `mgf-precursor-erro-rs` | `apps/index/src/main.rs` (`3fbe999a`) |
| `apps/smellfish-rs/src/app/formatting.rs` | Inlined directly at single call site | `apps/smellfish-rs` (`37e7c1a4`) |

## 6. Open Questions

1. **OQ-1 (`smellfish-rs` Ertl NP-likeness FP vs model)**: `RDKit_minimal.js` exposes only binary folded bit vectors (`get_morgan_fp_as_uint8array`) while `np_model.bin` sums colliding buckets (max `+8.924`, `22.4%` occupancy). Should Ertl scoring move to a pure-Rust unfolded Morgan radius-2 count fingerprint backed by the original `publicnp.model.gz` weights, or keep binary RDKit.js with recalibrated thresholds?
2. **OQ-2 (`smellfish-rs` LOTUS-backed synthetic contaminants & stereo mismatch)**: When a compound has a LOTUS organism record and `Ertl <= -2.0`, should the verdict be `Fishy` (current at `<= -2.0`), `Caution` (verify possible contaminant), or `Likely` (current at `-1.99`)? When stereospecific SMILES match only via 14-char connectivity to another stereoisomer in LOTUS, should the verdict be annotated/downgraded?
3. **OQ-3 (`smellfish-rs` PubChem `CHEMINF_000462`)**: Should `build_pubchem_sparql` keep `CHEMINF_000462` (same connectivity, returning up to 353+ isotopologues) or restrict to exact CID + `CHEMINF_000461` (stereoisomers)?
4. **OQ-4 (`smellfish-rs` motif taxonomy & `user_*.txt`)**: Should `user_source_vs_synthetic.txt` / `user_kingdom_enrichment.txt` / `group_names.txt` be populated with a curated functional-group + scaffold SMARTS schema shared between JS and Rust (replacing the dead English-keyword heuristics in `chemist.rs`/`ring_family.rs`), or removed?
5. **OQ-5 (`lipid-core` 86 LMSD SMARTS rules)**: Evolved SMARTS in `lmsd_part1.rs`/`lmsd_part2.rs` contain single-fold overfitting (`PK11` `[#6]/[x4]`, `FA09` `@OH23` under `use_chirality: false`, `GL04` explicit `[#1]`, `FA00` `[As]`, `GP00`, `GP02`, `GP18`/`GP19` lacking P, `SP03`, `SL03`). Offline re-evolution with held-out cross-validation against `chematic::smarts` requires LMSD download and domain sign-off.
6. **OQ-6 (`lipid-core` SMILES export format)**: Should `build_filtered_smiles_content` preserve input `.smi` lines verbatim (as `RULES_GUIDE.md` claims) or keep the 5-column annotated TSV (`SMILES\tTitle\tCATEGORY\tMAIN_CLASS\t-`)?
7. **OQ-7 (`cxsmiles-core` representation & scope)**: (a) Should `floating_defs` emit standard intact fragments (`[*]OC(=O)C`) instead of splitting degree-1 pendant chains into double-`m:` (`[*]O.[*]C(=O)C`)? (b) Should `Sg:n:` encode observed repeat bounds (`:1-3:ht`) and support alpha-omega capped series (`-CF3` PFAS, diacids) via formula-difference alignment? (c) Should aromatic equivalence use strict graph automorphisms (matching `README.md`) or ring-level CH expansion? (d) Should all-unrelated inputs (`Tanimoto < 0.3`) error instead of falling back to all `mols`?
8. **OQ-8 (`json-count-core` leaf & duplicate-key semantics)**: Should `""`, `[]`, and `{}` count as `1` (strict RFC 8259 non-null) or `0` (non-empty, current behavior)? Should top-level duplicate keys (`{"a":1,"a":2}`) sum (`2`), take last-wins, or deduplicate (`1`)?
9. **OQ-9 (`mgf-precursor-erro-rs` plot pairing & outliers)**: When recalibration is active, should `EcdfPlot` switch to recalibrated MS2 preview while `AbsoluteMassBiasPlot` remains on MS1, or stay on MS1? Should scatter outliers beyond `±y_limit` be clamped with distinct markers or omitted with an out-of-range counter?

## 7. Residual Risk

- Browser DOM / `web-sys` runtime interactions (`Blob`, drag/drop, clipboard, live WDQS/QLever HTTP, RDKit.js WASM interop) are compiled and linted for `wasm32-unknown-unknown` but not executed in a headless browser in this host audit.
- `.mk/*`, `mk`, and `mk.versions` carry an upstream `rusti-graben` template sync notice (`docs/GATES.md` hint in `.mk/catalog.sh:89`); kept untouched locally.

## 8. Slop Removal

| Kind | Target | Proof of Non-Use / Justification | Commit |
| --- | --- | --- | --- |
| Duplicate asset | `apps/smellfish-rs/public/np_model.bin.gz` | Unused 824 KB byte-identical gzip duplicate of `np_model.bin` (unreferenced across Rust/JS/HTML) | `37e7c1a4` |
| Placeholder files | `apps/lipid-selecto-rs/assets/.gitkeep`, `apps/smellfish-rs/public/.gitkeep` | Empty placeholders in non-empty asset directories | `06f19d8b`, `37e7c1a4` |
| Unused dependency | `js-sys` in `apps/mgf-precursor-erro-rs/Cargo.toml` | Zero imports (`cargo machete` / `rg` confirmed unused) | `0dcf2e8b` |
| Trivial wrapper module | `apps/smellfish-rs/src/app/formatting.rs` | Single trivial helper; inlined at single call site | `37e7c1a4` |
| Slop test / dead helper | `no_hallucinated_rules`, <!-- refquiem: allow(decoration_heavy) -->`decoration_heavy()` | Asserted hardcoded list / inverted logic replaced by real property tests | `19901a23` |
| Fake placeholder data | `user_*.txt` (`100/0` gem-dimethyl) & broken `ertl_kingdom_enrichment.txt:220` | Synthetic placeholder weights removed; wrapped record rejoined | `37e7c1a4` |
| Unicode emoji & filler prose | Fish emoji in UI, `Thank you...` template copy, changelog comment in `index` | Stripped decorative/conversational prose from UI, templates, and docs | `19901a23`, `37e7c1a4`, `3fbe999a`, `90e48423` |
| False claims & drift | `PFOS` (`C4H10O`), `machine-learned features`, `Plausible Analytics`, `stream-io` re-export, wrong `Ertl` DOI title, read-only `.mcp.json` | Corrected against source code, Crossref metadata, and `Dioxus.toml`/`deny.toml` manifests | `5ef4f0ad`, `7d35bfce`, `06f19d8b`, `37e7c1a4`, `90e48423`, `f006a60d` |
