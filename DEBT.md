# Technical Debt & Refactoring Ledger (`DEBT.md`)

Baseline commit: `2bab1111` (safety-net commit: `1d65b8d1`).
Hardware & toolchain: Apple M1 Pro (8 cores, 16 GiB RAM, macOS arm64), Rust `1.100.0` (edition 2024, resolver 3), `dx 0.7.10`.

## 1. Coverage Ledger (406 / 406 `git ls-files`)

| File | Reviewed | Debt Items |
| --- | --- | ---: |
| `.cargo/config.toml` | y | 0 |
| `.cargo/mutants.toml` | y | 1 |
| `.config/nextest.toml` | y | 0 |
| `.echoes.toml` | y | 0 |
| `.editorconfig` | y | 0 |
| `.gitattributes` | y | 0 |
| `.github/CODE_OF_CONDUCT.md` | y | 0 |
| `.github/CONTRIBUTING.md` | y | 1 |
| `.github/ISSUE_TEMPLATE/bug_report.yml` | y | 0 |
| `.github/ISSUE_TEMPLATE/config.yml` | y | 0 |
| `.github/ISSUE_TEMPLATE/feature_request.yml` | y | 0 |
| `.github/PULL_REQUEST_TEMPLATE.md` | y | 0 |
| `.github/RELEASE_CHECKLIST.md` | y | 1 |
| `.github/SECURITY.md` | y | 0 |
| `.github/SUPPORT.md` | y | 0 |
| `.github/actions/setup/action.yml` | y | 1 |
| `.github/ai/AI_AGENT_GUIDE.md` | y | 0 |
| `.github/ai/AI_EFFICIENCY_GUIDE.md` | y | 0 |
| `.github/ai/CONTRIBUTING_AI.md` | y | 0 |
| `.github/dependabot.yml` | y | 0 |
| `.github/workflows/ci.yml` | y | 3 |
| `.github/workflows/codeql.yml` | y | 1 |
| `.github/workflows/dependency-review.yml` | y | 0 |
| `.github/workflows/deploy.yml` | y | 2 |
| `.github/workflows/nightly.yml` | y | 1 |
| `.github/workflows/release.yml` | y | 0 |
| `.gitignore` | y | 0 |
| `.graphifyignore` | y | 0 |
| `.mcp.json` | y | 0 |
| `.mk/catalog.sh` | y (upstream rusti-graben runner) | 0 |
| `.mk/core.sh` | y (upstream rusti-graben runner) | 0 |
| `.mk/gates.sh` | y (upstream rusti-graben runner) | 0 |
| `.mk/meta.py` | y (upstream rusti-graben runner) | 0 |
| `.mk/mutants.sh` | y (upstream rusti-graben runner) | 0 |
| `.mk/mutants_plan.sh` | y (upstream rusti-graben runner) | 0 |
| `.mk/nightly.sh` | y (upstream rusti-graben runner) | 0 |
| `.mk/tier.sh` | y (upstream rusti-graben runner) | 0 |
| `.mk/tools.sh` | y (upstream rusti-graben runner) | 0 |
| `.refquiem.toml` | y | 0 |
| `.tombi.toml` | y | 0 |
| `.typos.toml` | y | 0 |
| `.vscode/README.md` | y | 1 |
| `.vscode/extensions.json` | y | 0 |
| `.vscode/settings.json` | y | 0 |
| `AGENTS.md` | y | 0 |
| `AUDIT.md` | y | 0 |
| `CHANGELOG.md` | y | 0 |
| `Cargo.lock` | N/A (generated Cargo lockfile) | 0 |
| `Cargo.toml` | y | 0 |
| `LICENSE` | N/A (verbatim GNU AGPL v3.0 text) | 0 |
| `Makefile.toml` | y | 1 |
| `README.md` | y | 1 |
| `apps/cxsmiles-yoga/Cargo.toml` | y | 0 |
| `apps/cxsmiles-yoga/Dioxus.toml` | y | 0 |
| `apps/cxsmiles-yoga/README.md` | y (generated from //! + README.tpl) | 0 |
| `apps/cxsmiles-yoga/assets/.gitkeep` | N/A (empty asset dir placeholder) | 0 |
| `apps/cxsmiles-yoga/src/app.rs` | y | 1 |
| `apps/cxsmiles-yoga/src/app/clipboard.rs` | y | 1 |
| `apps/cxsmiles-yoga/src/app/controls.rs` | y | 0 |
| `apps/cxsmiles-yoga/src/app/results.rs` | y | 2 |
| `apps/cxsmiles-yoga/src/examples.rs` | y | 0 |
| `apps/cxsmiles-yoga/src/lib.rs` | y | 0 |
| `apps/cxsmiles-yoga/src/main.rs` | y | 0 |
| `apps/index/Cargo.toml` | y | 0 |
| `apps/index/Dioxus.toml` | y | 0 |
| `apps/index/README.md` | y (generated from //! + README.tpl) | 0 |
| `apps/index/README.tpl` | y | 0 |
| `apps/index/src/app_card.rs` | y | 0 |
| `apps/index/src/main.rs` | y | 1 |
| `apps/json-count-rs/Cargo.toml` | y | 0 |
| `apps/json-count-rs/Dioxus.toml` | y | 0 |
| `apps/json-count-rs/README.md` | y (generated from //! + README.tpl) | 0 |
| `apps/json-count-rs/README.tpl` | y | 0 |
| `apps/json-count-rs/src/app_state.rs` | y | 2 |
| `apps/json-count-rs/src/main.rs` | y | 2 |
| `apps/json-count-rs/src/wasm_entry.rs` | y | 3 |
| `apps/lipid-selecto-rs/Cargo.toml` | y | 1 |
| `apps/lipid-selecto-rs/Dioxus.toml` | y | 0 |
| `apps/lipid-selecto-rs/LIPID_ARCHITECTURE.md` | y | 0 |
| `apps/lipid-selecto-rs/README.md` | y (generated from //! + README.tpl) | 0 |
| `apps/lipid-selecto-rs/README.tpl` | y | 0 |
| `apps/lipid-selecto-rs/RULES_GUIDE.md` | y | 0 |
| `apps/lipid-selecto-rs/assets/example_lipids.smi` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/README.md` | y (generated from //! + README.tpl) | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.lock` | N/A (pinned standalone Cargo lockfile) | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/README.md` | y (generated from //! + README.tpl) | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/examples/bench_smoke.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli.rs` | y | 3 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/download.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/evolve.rs` | y | 3 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/ordering.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/report.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli/split.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/cli_tests.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/download.rs` | y | 3 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/download/sdf.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/download/sdf/tests.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/error.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/evolve.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/evolve/observer.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/evolve/tests.rs` | y | 0 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/lib.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/main.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/manifest.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/results_io.rs` | y | 3 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/splitting.rs` | y | 4 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/splitting/slug.rs` | y | 1 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/splitting/tests.rs` | y | 2 |
| `apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/src/splitting/types.rs` | y | 2 |
| `apps/lipid-selecto-rs/src/app.rs` | y | 3 |
| `apps/lipid-selecto-rs/src/app/browser.rs` | y | 4 |
| `apps/lipid-selecto-rs/src/app/classes_card.rs` | y | 2 |
| `apps/lipid-selecto-rs/src/app/download.rs` | y | 0 |
| `apps/lipid-selecto-rs/src/app/filter_controls.rs` | y | 1 |
| `apps/lipid-selecto-rs/src/app/gallery_panel.rs` | y | 3 |
| `apps/lipid-selecto-rs/src/app/summary_panel.rs` | y | 3 |
| `apps/lipid-selecto-rs/src/app/types.rs` | y | 2 |
| `apps/lipid-selecto-rs/src/lib.rs` | y | 0 |
| `apps/lipid-selecto-rs/src/main.rs` | y | 1 |
| `apps/mgf-precursor-erro-rs/Cargo.toml` | y | 1 |
| `apps/mgf-precursor-erro-rs/Dioxus.toml` | y | 0 |
| `apps/mgf-precursor-erro-rs/README.md` | y (generated from //! + README.tpl) | 0 |
| `apps/mgf-precursor-erro-rs/README.tpl` | y | 0 |
| `apps/mgf-precursor-erro-rs/examples/recalibration_demo.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/app.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app/browser.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app/diagnostics_panel.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app/plots.rs` | y | 1 |
| `apps/mgf-precursor-erro-rs/src/app/recalibration_panel.rs` | y | 4 |
| `apps/mgf-precursor-erro-rs/src/app/results.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/app/warnings_panel.rs` | y | 1 |
| `apps/mgf-precursor-erro-rs/src/errors.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/lib.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/main.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/plotting.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/plotting/color.rs` | y | 1 |
| `apps/mgf-precursor-erro-rs/src/plotting/cumulative.rs` | y | 5 |
| `apps/mgf-precursor-erro-rs/src/plotting/cumulative/tests.rs` | y | 1 |
| `apps/mgf-precursor-erro-rs/src/plotting/data.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/plotting/diagnostics.rs` | y | 3 |
| `apps/mgf-precursor-erro-rs/src/plotting/diagnostics/tests.rs` | y | 1 |
| `apps/mgf-precursor-erro-rs/src/plotting/histogram.rs` | y | 2 |
| `apps/mgf-precursor-erro-rs/src/plotting/scatter.rs` | y | 4 |
| `apps/mgf-precursor-erro-rs/src/plotting/scatter/tests.rs` | y | 0 |
| `apps/mgf-precursor-erro-rs/src/plotting/svg.rs` | y | 1 |
| `apps/smellfish-rs/Cargo.toml` | y | 0 |
| `apps/smellfish-rs/Dioxus.toml` | y | 0 |
| `apps/smellfish-rs/README.md` | y (generated from //! + README.tpl) | 0 |
| `apps/smellfish-rs/README.tpl` | y | 0 |
| `apps/smellfish-rs/assets/rdkit_bridge.js` | y | 4 |
| `apps/smellfish-rs/assets/smellfish.css` | y | 2 |
| `apps/smellfish-rs/build.rs` | y | 1 |
| `apps/smellfish-rs/public/ertl_kingdom_enrichment.txt` | N/A (scientific reference data table) | 0 |
| `apps/smellfish-rs/public/ertl_npsubstituents.txt` | N/A (scientific reference SMARTS table) | 0 |
| `apps/smellfish-rs/public/ertl_source_vs_synthetic.txt` | N/A (scientific reference data table) | 0 |
| `apps/smellfish-rs/public/group_names.txt` | y | 1 |
| `apps/smellfish-rs/public/lotus_1percent_scaffolds.txt` | N/A (scientific reference SMARTS table) | 0 |
| `apps/smellfish-rs/public/motif-library.js` | y | 1 |
| `apps/smellfish-rs/public/np_model.bin` | N/A (binary Ertl NP-likeness model weights) | 0 |
| `apps/smellfish-rs/public/user_kingdom_enrichment.txt` | y | 1 |
| `apps/smellfish-rs/public/user_source_vs_synthetic.txt` | y | 1 |
| `apps/smellfish-rs/src/app.rs` | y | 1 |
| `apps/smellfish-rs/src/app/browser.rs` | y | 2 |
| `apps/smellfish-rs/src/app/footer.rs` | y | 0 |
| `apps/smellfish-rs/src/app/input_panel.rs` | y | 2 |
| `apps/smellfish-rs/src/app/molecule_card.rs` | y | 3 |
| `apps/smellfish-rs/src/app/motif_display.rs` | y | 0 |
| `apps/smellfish-rs/src/app/motif_display/tests.rs` | y | 0 |
| `apps/smellfish-rs/src/app/results.rs` | y | 2 |
| `apps/smellfish-rs/src/document_head.rs` | y | 1 |
| `apps/smellfish-rs/src/lib.rs` | y | 0 |
| `apps/smellfish-rs/src/literature.rs` | y | 1 |
| `apps/smellfish-rs/src/main.rs` | y | 0 |
| `apps/smellfish-rs/src/pipeline.rs` | y | 2 |
| `apps/smellfish-rs/src/rdkit.rs` | y | 0 |
| `apps/smellfish-rs/src/rdkit_bridge.rs` | y | 0 |
| `apps/smellfish-rs/src/styles.rs` | y | 0 |
| `bacon.toml` | y | 0 |
| `clippy.toml` | y | 0 |
| `crates/chem-depict/Cargo.toml` | y | 1 |
| `crates/chem-depict/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/chem-depict/README.tpl` | y | 0 |
| `crates/chem-depict/src/error.rs` | y | 0 |
| `crates/chem-depict/src/lib.rs` | y | 1 |
| `crates/chem-depict/src/parse.rs` | y | 1 |
| `crates/chem-depict/src/svg.rs` | y | 1 |
| `crates/chem-depict/src/tests.rs` | y | 1 |
| `crates/cxsmiles-core/Cargo.toml` | y | 1 |
| `crates/cxsmiles-core/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/cxsmiles-core/README.tpl` | y | 0 |
| `crates/cxsmiles-core/src/aromatic.rs` | y | 2 |
| `crates/cxsmiles-core/src/error.rs` | y | 0 |
| `crates/cxsmiles-core/src/fragment.rs` | y | 3 |
| `crates/cxsmiles-core/src/graph.rs` | y | 2 |
| `crates/cxsmiles-core/src/lib.rs` | y | 0 |
| `crates/cxsmiles-core/src/parse.rs` | y | 1 |
| `crates/cxsmiles-core/src/pipeline.rs` | y | 2 |
| `crates/cxsmiles-core/src/positional.rs` | y | 5 |
| `crates/cxsmiles-core/src/repeating.rs` | y | 4 |
| `crates/cxsmiles-core/src/roundtrip.rs` | y | 2 |
| `crates/cxsmiles-core/src/splice.rs` | y | 2 |
| `crates/cxsmiles-core/src/tests.rs` | y | 0 |
| `crates/cxsmiles-core/src/tests/e2e.rs` | y | 1 |
| `crates/cxsmiles-core/src/tests/graph.rs` | y | 0 |
| `crates/cxsmiles-core/src/tests/limits.rs` | y | 0 |
| `crates/cxsmiles-core/src/tests/parse_tests.rs` | y | 0 |
| `crates/cxsmiles-core/src/tests/positional_builders.rs` | y | 0 |
| `crates/cxsmiles-core/src/tests/positional_helpers.rs` | y | 1 |
| `crates/cxsmiles-core/src/tests/repeating_builders.rs` | y | 0 |
| `crates/cxsmiles-core/src/tests/repeating_helpers.rs` | y | 0 |
| `crates/cxsmiles-core/src/tests/roundtrip.rs` | y | 0 |
| `crates/cxsmiles-core/src/types.rs` | y | 0 |
| `crates/json-count-core/Cargo.toml` | y | 0 |
| `crates/json-count-core/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/json-count-core/README.tpl` | y | 0 |
| `crates/json-count-core/src/container.rs` | y | 4 |
| `crates/json-count-core/src/error.rs` | y | 1 |
| `crates/json-count-core/src/lib.rs` | y | 0 |
| `crates/json-count-core/src/model.rs` | y | 1 |
| `crates/json-count-core/src/number.rs` | y | 1 |
| `crates/json-count-core/src/scanner.rs` | y | 3 |
| `crates/json-count-core/src/strings.rs` | y | 2 |
| `crates/json-count-core/src/tests.rs` | y | 0 |
| `crates/json-count-core/src/tests/boundaries.rs` | y | 2 |
| `crates/json-count-core/src/tests/escaping.rs` | y | 0 |
| `crates/json-count-core/src/tests/sorting.rs` | y | 0 |
| `crates/json-count-core/src/tests/streaming.rs` | y | 0 |
| `crates/json-count-core/src/unescape.rs` | y | 2 |
| `crates/lipid-core/Cargo.toml` | y | 1 |
| `crates/lipid-core/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/lipid-core/README.tpl` | y | 0 |
| `crates/lipid-core/src/bitset.rs` | y | 0 |
| `crates/lipid-core/src/chemical_class.rs` | y | 1 |
| `crates/lipid-core/src/chemical_class/defaults.rs` | y | 1 |
| `crates/lipid-core/src/chemical_class/lmsd.rs` | y | 0 |
| `crates/lipid-core/src/chemical_class/lmsd_part1.rs` | y | 0 |
| `crates/lipid-core/src/chemical_class/lmsd_part2.rs` | y | 0 |
| `crates/lipid-core/src/examples.rs` | y | 0 |
| `crates/lipid-core/src/examples/fatty_acyls.rs` | y | 0 |
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
| `crates/lipid-core/src/format.rs` | y | 2 |
| `crates/lipid-core/src/lib.rs` | y | 0 |
| `crates/lipid-core/src/lipids.rs` | y | 0 |
| `crates/lipid-core/src/lipids/classify.rs` | y | 2 |
| `crates/lipid-core/src/lipids/types.rs` | y | 2 |
| `crates/lipid-core/src/parser.rs` | y | 0 |
| `crates/lipid-core/src/parser/analysis.rs` | y | 2 |
| `crates/lipid-core/src/parser/block.rs` | y | 2 |
| `crates/lipid-core/src/parser/extractor.rs` | y | 2 |
| `crates/lipid-core/src/tests.rs` | y | 0 |
| `crates/lipid-core/src/tests/analysis.rs` | y | 2 |
| `crates/lipid-core/src/tests/chemical_class.rs` | y | 0 |
| `crates/lipid-core/src/tests/export_and_filter.rs` | y | 0 |
| `crates/lipid-core/src/tests/format.rs` | y | 0 |
| `crates/lipid-core/src/tests/lipids.rs` | y | 0 |
| `crates/mgf-analysis-core/Cargo.toml` | y | 1 |
| `crates/mgf-analysis-core/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/mgf-analysis-core/README.tpl` | y | 0 |
| `crates/mgf-analysis-core/src/adduct.rs` | y | 3 |
| `crates/mgf-analysis-core/src/adduct/parse.rs` | y | 4 |
| `crates/mgf-analysis-core/src/diagnostics.rs` | y | 2 |
| `crates/mgf-analysis-core/src/diagnostics/model.rs` | y | 1 |
| `crates/mgf-analysis-core/src/diagnostics/stats.rs` | y | 4 |
| `crates/mgf-analysis-core/src/ingestor.rs` | y | 3 |
| `crates/mgf-analysis-core/src/lib.rs` | y | 1 |
| `crates/mgf-analysis-core/src/mass.rs` | y | 1 |
| `crates/mgf-analysis-core/src/metrics.rs` | y | 1 |
| `crates/mgf-analysis-core/src/metrics/record.rs` | y | 3 |
| `crates/mgf-analysis-core/src/metrics/stats.rs` | y | 2 |
| `crates/mgf-analysis-core/src/numeric.rs` | y | 1 |
| `crates/mgf-analysis-core/src/recalibration.rs` | y | 0 |
| `crates/mgf-analysis-core/src/recalibration/calibration.rs` | y | 1 |
| `crates/mgf-analysis-core/src/recalibration/generator.rs` | y | 1 |
| `crates/mgf-analysis-core/src/recalibration/parsing.rs` | y | 1 |
| `crates/mgf-analysis-core/src/recalibration/types.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/adduct.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/calibration.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/diagnostics.rs` | y | 1 |
| `crates/mgf-analysis-core/src/tests/generator.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/ingestor.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/mass.rs` | y | 0 |
| `crates/mgf-analysis-core/src/tests/metrics.rs` | y | 0 |
| `crates/mgf/Cargo.toml` | y | 0 |
| `crates/mgf/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/mgf/README.tpl` | y | 0 |
| `crates/mgf/src/lib.rs` | y | 0 |
| `crates/mgf/src/parser.rs` | y | 4 |
| `crates/mgf/src/tests.rs` | y | 0 |
| `crates/mgf/src/types.rs` | y | 1 |
| `crates/smellfish-core/Cargo.toml` | y | 0 |
| `crates/smellfish-core/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/smellfish-core/README.tpl` | y | 1 |
| `crates/smellfish-core/src/csv.rs` | y | 5 |
| `crates/smellfish-core/src/error.rs` | y | 0 |
| `crates/smellfish-core/src/evidence.rs` | y | 1 |
| `crates/smellfish-core/src/evidence/assessment.rs` | y | 1 |
| `crates/smellfish-core/src/evidence/chemist.rs` | y | 2 |
| `crates/smellfish-core/src/evidence/ring_family.rs` | y | 1 |
| `crates/smellfish-core/src/evidence/verdict.rs` | y | 2 |
| `crates/smellfish-core/src/lib.rs` | y | 0 |
| `crates/smellfish-core/src/model.rs` | y | 1 |
| `crates/smellfish-core/src/model/classification.rs` | y | 2 |
| `crates/smellfish-core/src/model/types.rs` | y | 1 |
| `crates/smellfish-core/src/pipeline.rs` | y | 3 |
| `crates/smellfish-core/src/qlever.rs` | y | 1 |
| `crates/smellfish-core/src/query.rs` | y | 2 |
| `crates/smellfish-core/src/sparql.rs` | y | 0 |
| `crates/smellfish-core/src/svg.rs` | y | 1 |
| `crates/smellfish-core/src/tests.rs` | y | 0 |
| `crates/smellfish-core/src/tests/csv.rs` | y | 0 |
| `crates/smellfish-core/src/tests/evidence.rs` | y | 0 |
| `crates/smellfish-core/src/tests/pipeline.rs` | y | 0 |
| `crates/smellfish-core/src/tests/qlever.rs` | y | 0 |
| `crates/smellfish-core/src/tests/query.rs` | y | 0 |
| `crates/smellfish-core/src/tests/ring_family.rs` | y | 0 |
| `crates/smellfish-core/src/tests/sparql.rs` | y | 0 |
| `crates/smellfish-core/src/tests/svg.rs` | y | 0 |
| `crates/smellfish-core/src/tests/verdict.rs` | y | 0 |
| `crates/stream-io/Cargo.toml` | y | 0 |
| `crates/stream-io/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/stream-io/README.tpl` | y | 1 |
| `crates/stream-io/src/blob_cursor.rs` | y | 4 |
| `crates/stream-io/src/blob_lines.rs` | y | 4 |
| `crates/stream-io/src/blob_lines/tests.rs` | y | 1 |
| `crates/stream-io/src/buffer.rs` | y | 1 |
| `crates/stream-io/src/bytes.rs` | y | 1 |
| `crates/stream-io/src/error.rs` | y | 0 |
| `crates/stream-io/src/lib.rs` | y | 0 |
| `crates/stream-io/src/progress.rs` | y | 0 |
| `crates/stream-io/src/progress/tests.rs` | y | 1 |
| `crates/ui/Cargo.toml` | y | 0 |
| `crates/ui/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/ui/README.tpl` | y | 1 |
| `crates/ui/src/a11y.rs` | y | 0 |
| `crates/ui/src/a11y/tests.rs` | y | 0 |
| `crates/ui/src/components.rs` | y | 0 |
| `crates/ui/src/components/button.rs` | y | 1 |
| `crates/ui/src/components/card.rs` | y | 2 |
| `crates/ui/src/components/footer.rs` | y | 2 |
| `crates/ui/src/components/header.rs` | y | 2 |
| `crates/ui/src/components/notice.rs` | y | 2 |
| `crates/ui/src/components/segmented_control.rs` | y | 3 |
| `crates/ui/src/components/simple_analytics_badge.rs` | y | 1 |
| `crates/ui/src/components/upload_zone.rs` | y | 2 |
| `crates/ui/src/document.rs` | y | 2 |
| `crates/ui/src/lib.rs` | y | 0 |
| `crates/ui/src/signals.rs` | y | 1 |
| `crates/ui/src/theme.rs` | y | 2 |
| `crates/ui/src/theme/tests.rs` | y | 0 |
| `crates/ui/src/url.rs` | y | 0 |
| `crates/ui/src/webmcp.rs` | y | 3 |
| `crates/ui/src/webmcp/tests.rs` | y | 0 |
| `crates/upload/Cargo.toml` | y | 0 |
| `crates/upload/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/upload/README.tpl` | y | 1 |
| `crates/upload/src/bytes.rs` | y | 2 |
| `crates/upload/src/download.rs` | y | 3 |
| `crates/upload/src/download/tests.rs` | y | 0 |
| `crates/upload/src/error.rs` | y | 0 |
| `crates/upload/src/event.rs` | y | 0 |
| `crates/upload/src/lib.rs` | y | 1 |
| `crates/workspace-gates/Cargo.toml` | y | 0 |
| `crates/workspace-gates/README.md` | y (generated from //! + README.tpl) | 0 |
| `crates/workspace-gates/README.tpl` | y | 1 |
| `crates/workspace-gates/examples/bench_smoke.rs` | y | 0 |
| `crates/workspace-gates/src/lib.rs` | y | 0 |
| `crates/workspace-gates/tests/gate_consistency/apps.rs` | y | 4 |
| `crates/workspace-gates/tests/gate_consistency/data_files.rs` | y | 1 |
| `crates/workspace-gates/tests/gate_consistency/main.rs` | y | 2 |
| `crates/workspace-gates/tests/gate_consistency/mk_runner.rs` | y | 0 |
| `crates/workspace-gates/tests/gate_consistency/mutants.rs` | y | 1 |
| `crates/workspace-gates/tests/gate_consistency/surfaces.rs` | y | 2 |
| `crates/workspace-gates/tests/gate_consistency/task_files.rs` | y | 1 |
| `crates/workspace-gates/tests/gate_consistency/workflows.rs` | y | 2 |
| `deny.toml` | y | 0 |
| `docs/ARCHITECTURE.md` | y | 0 |
| `docs/DEBT.md` | y | 0 |
| `make/all.toml` | y | 1 |
| `make/build.toml` | y | 3 |
| `make/format.toml` | y | 1 |
| `make/lint.toml` | y | 3 |
| `make/readme.toml` | y | 2 |
| `make/scripts/license-headers.sh` | y | 0 |
| `make/scripts/setup.sh` | y | 2 |
| `make/scripts/unsafe-audit.sh` | y | 1 |
| `make/supply-chain.toml` | y | 2 |
| `make/test.toml` | y | 3 |
| `make/web.toml` | y | 1 |
| `mk` | y (upstream rusti-graben runner) | 0 |
| `mk.local` | y | 3 |
| `mk.versions` | y (upstream rusti-graben runner) | 0 |
| `prek.toml` | y | 0 |
| `renovate.json` | y | 1 |
| `rust-toolchain.toml` | y | 0 |
| `rustfmt.toml` | y | 0 |

## 2. Debt Ledger

| ID | Cat | Location | Evidence | Fix | Risk | Status | Commit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| TD-CD-01 | deps | `crates/chem-depict/Cargo.toml:15` | Unused `chem` feature on `chematic` | Remove `"chem"` feature | low | open | — |
| TD-CD-02 | hygiene | `crates/chem-depict/src/lib.rs:8-9` | Doc claims wasm32 uses `panic="abort"` (`wasm-release` uses `unwind`) | Clarify doc and regenerate README | low | open | — |
| TD-CD-03 | SOLID | `crates/chem-depict/src/parse.rs:16` | Helper only used inside `parse.rs` marked `pub(crate)` | Narrow to private `fn` | low | open | — |
| TD-CD-04 | DRY | `crates/chem-depict/src/svg.rs:44-93` | Duplicated atom guard, `catch_unwind`, `depict_svg`, SVG check | Extract `render_molecule_guarded` | low | open | — |
| TD-CD-05 | DRY | `crates/chem-depict/src/tests.rs:93,100` | Hardcoded `16_385` and `513` instead of constants | Reference `MAX_SMILES_LEN + 1` / `MAX_DEPICT_ATOMS + 1` | low | open | — |
| TD-CX-01 | deps | `crates/cxsmiles-core/Cargo.toml:15` | Unused `chem` feature on `chematic` | Remove `"chem"` feature | low | open | — |
| TD-CX-02 | perf | `crates/cxsmiles-core/src/aromatic.rs:50-81` | Linear `targets.contains` inside loops (`O(V*k)`) | Use `FxHashSet` and skip non-aromatic atoms | low | open | — |
| TD-CX-03 | hygiene | `crates/cxsmiles-core/src/aromatic.rs:88-94` | Test-only helper inside production module | Move helper into test module | low | open | — |
| TD-CX-04 | perf | `crates/cxsmiles-core/src/fragment.rs:37-104` | Unconditional atom/bond clones even when `attachment == 0` | Borrow directly when `attachment == 0`, mutate in place | low | open | — |
| TD-CX-05 | perf | `crates/cxsmiles-core/src/fragment.rs:127-133` | Linear `subset.contains` beside existing `FxHashMap` | Use `map.contains_key` | low | open | — |
| TD-CX-06 | perf | `crates/cxsmiles-core/src/fragment.rs:58-74` | Allocates `Vec<String>` before `.join(".")` | Build directly into `String` | low | open | — |
| TD-CX-07 | DRY | `crates/cxsmiles-core/src/graph.rs:18-24` | `try_from().unwrap_or(MAX)` repeated 14+ times | Extract `u32_to_usize` / `usize_to_u32` | low | open | — |
| TD-CX-08 | perf | `crates/cxsmiles-core/src/graph.rs:111-133` | Un-sized `FxHashSet` allocations in hot match loop | Pre-size `FxHashSet::with_capacity_and_hasher` | low | open | — |
| TD-CX-09 | perf | `crates/cxsmiles-core/src/parse.rs:34-69`, `pipeline.rs:45-55` | Clones all `Molecule`s per cluster and on fallback | Index-based clustering + `Cow` borrow | low | open | — |
| TD-CX-10 | perf | `crates/cxsmiles-core/src/pipeline.rs:40-66`, `repeating.rs:45-52` | Recomputes canonical SMILES up to 5x per molecule | Precompute canonical SMILES once and pass down | low | open | — |
| TD-CX-11 | perf | `crates/cxsmiles-core/src/positional.rs:43-168` | Clones `FloatingDef`/positions/targets and formats `m:` via `Vec` | Move owned collections, precompute frag SMILES, `fmt::Write` | low | open | — |
| TD-CX-12 | SOLID | `crates/cxsmiles-core/src/positional.rs:170-285` | Oversized 55/59-line fns, redundant `matched_mask`, 1-atom subgraph extract | Direct 1-atom extraction, derive mask from `rev`, split helpers | low | open | — |
| TD-CX-13 | perf | `crates/cxsmiles-core/src/positional.rs:287-302` | Clones floating component `Vec<u32>`s from owned input | Pass `Vec<Vec<u32>>` by value | low | open | — |
| TD-CX-14 | perf | `crates/cxsmiles-core/src/repeating.rs:19-120` | Clones `Vec<Molecule>`, `Vec<u32>`, `FxHashMap<u8,_>`, `Vec<String>` | Sort `&Molecule`, `swap_remove`, `[usize;128]`, `fmt::Write` | low | open | — |
| TD-CX-15 | perf | `crates/cxsmiles-core/src/repeating.rs:139-273` | Allocates `next_ext` `Vec` per DFS step + `state.best` clones | Reusable `ext_stack` slice indices + `clone_from`, split helper | med | open | — |
| TD-CX-16 | perf | `crates/cxsmiles-core/src/roundtrip.rs:43-171` | Recomputes `scaffold.implicit_hydrogen_count` per arrangement | Precompute `base_rem_h` once before loop + scratch `Vec` | low | open | — |
| TD-CX-17 | DRY | `crates/cxsmiles-core/src/roundtrip.rs:188-202` | Duplicate `ExpansionOverflow` construction | Chain `checked_mul`/`checked_add` | low | open | — |
| TD-CX-18 | DRY | `crates/cxsmiles-core/src/splice.rs:107-192` | `u32::MAX` sentinels and 3x copied scaffold atoms | Extract `copy_scaffold_atoms` and `skip_bonds: &[(u32, u32)]` | low | open | — |
| TD-CX-19 | perf | `crates/cxsmiles-core/src/splice.rs:10-44,195-201` | `O(E * |unit|)` linear scans in `unit` | Precompute `unit_pos: Vec<Option<usize>>` | low | open | — |
| TD-CX-20 | DRY | `crates/cxsmiles-core/src/tests/{e2e,positional_helpers}.rs` | Repeated biphenyl/PFAS literals and split import | Extract test constants and merge import | low | open | — |
| TD-LP-01 | deps | `crates/lipid-core/Cargo.toml:16` | Unused `smiles` feature on `chematic` in prod | Remove `"smiles"`, use `chem_depict` in tests | low | open | — |
| TD-LP-02 | hygiene | `crates/lipid-core/src/chemical_class/defaults.rs`, `family.rs` | ~200 lines `#[cfg(test)]`-only legacy defaults module | Delete legacy defaults module and narrow palette constants | low | open | — |
| TD-LP-03 | hygiene | `crates/lipid-core/src/format.rs:42-78` | Unused `#[cfg(test)]` detector and unused `extension()` | Delete dead detector; call `extension()` in app download | low | open | — |
| TD-LP-04 | hygiene | `crates/lipid-core/src/lipids/{types,classify}.rs` | `#[cfg(test)]`-only helpers + divergent spectrum fallback | Delete dead helpers; test fallback via `SpectrumBlock` | low | open | — |
| TD-LP-05 | SOLID | `crates/lipid-core/src/lipids/types.rs:65-71` | 2-byte `LipidClassification` lacks `Copy` | Derive `Copy` and remove `.clone()` | low | open | — |
| TD-LP-06 | DRY | `crates/lipid-core/src/lipids/classify.rs:69-74` | Duplicate SMARTS ring-check helper | Delegate to `!has_query(...)` | low | open | — |
| TD-LP-07 | DRY | `crates/lipid-core/src/filter.rs:43-76` | Duplicate finite range predicate and O(N) adduct `String` clones | Extract `in_finite_range`; dedup `&str` in `BTreeSet` | low | open | — |
| TD-LP-08 | perf | `crates/lipid-core/src/export.rs:71-241` | 3x comment loop, intermediate `Vec`s, 2x TSV `String` allocs | `impl Iterator`, `write_class_comments`, `Cow` `sanitize_tsv`, zero-vec tags | low | open | — |
| TD-LP-09 | perf | `crates/lipid-core/src/parser/analysis.rs:77-311` | Heap `String` for static color hex, dup lookup, inline test | `&'static str` color, `find_gallery_item`, consolidate tests | low | open | — |
| TD-LP-10 | perf | `crates/lipid-core/src/parser/block.rs:132-209` | Double formula parse + stringly `contains("[FA00]")` | Parse formula once, add `code` field on `ChemicalClass`, split helpers | low | open | — |
| TD-LP-11 | perf | `crates/lipid-core/src/parser/extractor.rs:108-158` | Unnecessary `try_parse_smiles(sec)` + test helper in prod | Guard `sec` parse on condition; move test helper | low | open | — |
| TD-LP-12 | hygiene | `crates/lipid-core/src/tests/{analysis,export_and_filter}.rs` | 5x `FilterCriteria` literals + crate->app `include_str!` | Extract `wide_criteria`; move asset parity check to app | low | open | — |
| TD-ST-01 | perf | `crates/stream-io/src/{blob_lines,buffer}.rs` | Redundant half-compaction `memmove` + `O(k^2)` multi-chunk scan | Remove half-compaction and track `scan_offset` | low | open | — |
| TD-ST-02 | SOLID | `crates/stream-io/src/blob_lines.rs:22-149` | Duplicate `processed` field + test-only `expect` suppressions | Read `inner.source_read`, gate `#[cfg(test)]`, `Result` tests | low | open | — |
| TD-ST-03 | DRY | `crates/stream-io/src/blob_cursor.rs:19-193` | Redundant `total_bytes` param, `eof` flag, byte-by-byte `skip_ws` | Read `source.total_bytes()`, add `remaining()`, slice-scan `skip_ws` | low | open | — |
| TD-ST-04 | SOLID | `crates/stream-io/src/bytes.rs:84-96` | `thread_local!` clock epoch drifts across threads | Replace with `OnceLock<Instant>` | low | open | — |
| TD-ST-05 | hygiene | `crates/stream-io/README.tpl:10` | Inconsistent LICENSE URL vs sibling core crates | Standardize LICENSE link across templates + `readme-write` | low | open | — |
| TD-MG-01 | perf | `crates/mgf/src/parser.rs:44-158` | 4x `.trim()` and redundant `'='`/ws scans per MS2 peak line | Add trimmed internal path; infer 2nd token from `split_whitespace` | low | open | — |
| TD-MG-02 | SOLID | `crates/mgf/src/parser.rs:62-131` | 12-branch header chain, `usize` BOM counter, incomplete doc | Extract `parse_finite_f64`, use `seen_first_line: bool`, fix doc | low | open | — |
| TD-MG-03 | hygiene | `crates/mgf/src/types.rs:20-25` | `pepmass`/`precursor_mz` written in parser, never read | Prune or document DTO fields (check test constructors) | low | open | — |
| TD-MA-01 | SOLID | `crates/mgf-analysis-core/src/diagnostics/{model,stats}.rs` | 20 unused `pub` fields + 6 unused `RunningStats` in `RecalibrationStats` | Remove unused fields and accumulators | low | open | — |
| TD-MA-02 | DRY | `crates/mgf-analysis-core/src/diagnostics/stats.rs:23-176` | Duplicate reservoir logic, 20k `family.to_string()`, 4-pass fallback | Shared `ReservoirAction`, `get_mut(family)`, 1-pass fold | low | open | — |
| TD-MA-03 | SOLID | `crates/mgf-analysis-core/src/adduct.rs:12-225` | Unbounded `thread_local!` adduct cache + 10-arm branch chain | Move cache to `LineIngestor`; table-driven `CANONICAL_ADDUCTS` | low | open | — |
| TD-MA-04 | perf | `crates/mgf-analysis-core/src/ingestor.rs:69-225` | Resolves adduct spec twice per block; 3x skip increment | Pass resolved spec once; `metrics.record_skipped()` | low | open | — |
| TD-MA-05 | SOLID | `crates/mgf-analysis-core/src/adduct/parse.rs:55-286` | 6-param helper, 3x `String` allocs, duplicate digit split | `AdductAccumulator`, byte slicing, `split_leading_count`, early check | low | open | — |
| TD-MA-06 | DRY | `crates/mgf-analysis-core/src/mass.rs:14-56` | Identical SMILES/formula cache lookup functions | Extract `cached_mass_lookup` | low | open | — |
| TD-MA-07 | hygiene | `crates/mgf-analysis-core/src/metrics/stats.rs:11-92` | 5 unused `pub` fields; doc inaccurately claims bounded memory | Remove 5 dead fields; document `O(N)` median heaps | low | open | — |
| TD-MA-08 | DRY | `crates/mgf-analysis-core/src/metrics.rs`, `record.rs` | Duplicate thresholds, double `String` alloc on max error, 4x ppm math | Shared threshold constants, branch fix, `da_to_ppm` | low | open | — |
| TD-MA-09 | SOLID | `crates/mgf-analysis-core/src/recalibration/{generator,calibration}.rs` | 7-param writer + redundant `!mz.is_finite()` | `RecalibrationParams` struct (7->4 params), drop redundant check | low | open | — |
| TD-MA-10 | DRY | `crates/mgf-analysis-core/src/tests/diagnostics.rs:78-130` | 3x repeated 12-field `ErrorMeasurement` literals | Extract `sample_measurement` helper | low | open | — |
| TD-JC-01 | hygiene | `crates/json-count-core/src/error.rs:81-83` | Dead error variant never constructed | Remove unused variant | low | open | — |
| TD-JC-02 | perf | `crates/json-count-core/src/container.rs:187-248` | Copies `StringValidator` by value per byte; byte-by-byte ASCII | Match `&mut` in-place; slice-scan plain ASCII runs | low | open | — |
| TD-JC-03 | perf | `crates/json-count-core/src/scanner.rs:47-96` | `HashSet<usize>` cleared & SipHashed per row | Generation-counter `row_gen` + `Vec<u64>` | low | open | — |
| TD-JC-04 | perf | `crates/json-count-core/src/strings.rs:36-58`, `unescape.rs:167-174` | Double-validates every unescaped JSON key | Track `has_backslash` to skip redundant scans | low | open | — |
| TD-JC-05 | DRY | `crates/json-count-core/src/{container,scanner,strings,unescape,model}.rs` | 4x loop glue, dup comma/close, dup escape list, non-`const` sort | Use `remaining()`, `expect_comma_or_close`, reuse `single_char_escape`, `const fn` | low | open | — |
| TD-JC-06 | DRY | `crates/json-count-core/src/tests/{boundaries,streaming}.rs` | Duplicate 1 MiB `MAX_KEY_BYTES` test fixtures | Consolidate boundary test into `boundaries.rs` | low | open | — |
| TD-SF-01 | DRY | `crates/smellfish-core/src/evidence/{chemist,ring_family}.rs` | 18 motif keywords duplicated across both modules | Share `RING_FAMILY_MOTIFS` table | low | open | — |
| TD-SF-02 | DRY | `crates/smellfish-core/src/{query,qlever}.rs` | WDQS/QLever queries duplicated; unused `rdfs:`; string dispatch | Unify via `LotusDialect` enum; drop unused `PREFIX` | low | open | — |
| TD-SF-03 | DRY | `crates/smellfish-core/src/csv.rs:28-359` | Duplicate header lists, `Cow::Owned(format!)`, test `expect`, 3x join | Hoist header constants, direct `'`, `#[cfg(test)]`, `push_csv_row` | low | open | — |
| TD-SF-04 | SOLID | `crates/smellfish-core/src/{model,classification,pipeline}.rs` | Dead marker method, excess `pub` fields, duplicate `motifs` `Vec<String>` | Remove dead method, narrow to `pub(crate)`, read `motif_hits` | low | open | — |
| TD-SF-05 | perf | `crates/smellfish-core/src/{pipeline,qlever,svg}.rs` | 4-tuple accumulator, double `kingdoms` clone, 2x SVG copy | `struct MotifAcc`, `InChIKey14::into_string`, `Cow` `is_safe_svg` | low | open | — |
| TD-SF-06 | DRY | `crates/smellfish-core/src/evidence/{verdict,assessment,chemist}.rs` | Bare Ertl cutoffs (`2.0`/`1.0`/`0.0`/`-2.0`) + dead `else` branch | Shared `ERTL_*` constants; simplify `classify_unbacked` | low | open | — |
| TD-UI-01 | DRY | `crates/ui/src/{webmcp,document}.rs` | Duplicate 6-field EDAM serialization, 3-pass escape, double `eval` | Single EDAM writer, single-pass `escape_script_json`, drop dup `use_effect` | low | open | — |
| TD-UI-02 | DRY | `crates/ui/src/{theme,components}.rs`, `components/*.rs` | 12+ `format!` allocs for `StyleBuilder`, 3x body/flex style chains | Add `StyleBuilder::property_fmt`, `body_text_style`, `flex_row_wrap` | low | open | — |
| TD-UI-03 | hygiene | `crates/ui/src/{webmcp,signals,theme}.rs`, `components/*.rs` | Dead `EdamTerm` helpers, `"true"` sentinel, `"{var}"` allocs | Prune dead helpers, `Option<&'static str>`, pass `String` directly | low | open | — |
| TD-UP-01 | hygiene | `crates/upload/src/{lib,bytes,download}.rs` | 5x `#[cfg]` in `bytes.rs`, zero-fill buffer, MIME sniffing, 2x alloc | Gate `mod bytes;`, `array.to_vec()`, explicit fallback, in-place truncate | low | open | — |
| TD-WG-01 | DRY | `crates/workspace-gates/tests/gate_consistency/*.rs` | 3x dir walk, 2x `app_dirs`, split single-owner constants, `Vec<String>` | Unify `collect_dir`/`make_toml_files`, co-locate items, `Vec<&str>` | low | open | — |
| TD-AP-01 | DRY | `apps/{json-count-rs,lipid-selecto-rs,mgf-precursor-erro-rs,smellfish-rs}` | 4x upload generation counter & file-extraction error/host glue | Add `UploadGen` + `dispatch_file_upload` in `crates/upload` | low | open | — |
| TD-AP-02 | hygiene | `apps/index/src/main.rs:85-106`, `json-count-rs`, `smellfish-rs` | Inline `WebMcpConfig`, hardcoded hex chrome, unused `shared_signal!` | Hoist `const`, use `ColorScheme` tokens, plain `use_signal` | low | open | — |
| TD-AP-03 | perf | `apps/cxsmiles-yoga/src/app/*.rs` | Eager all-SVG depiction, keystroke root re-render, timer race | Lazy page depiction, `use_memo(empty)`, copy gen counter, split `ResultsPanel` | low | open | — |
| TD-AP-04 | DRY | `apps/lipid-selecto-rs/src/**`, `Cargo.toml` | Dup `96` cap, 3x filter read, redundant `web-sys`, no-op yield, missing `key` | Centralize cap/`with_criteria`/styles, `upload::Blob`, memoize, `key:` | low | open | — |
| TD-AP-05 | DRY | `apps/mgf-precursor-erro-rs/src/app/*.rs` | 2x compliance grid, 3x chart box, 7-param panels, full stats clone | Shared grid/card helpers, `UploadSignals`, `SummaryMetrics` `Copy` struct, `&str` SVG | low | open | — |
| TD-AP-06 | DRY | `apps/mgf-precursor-erro-rs/src/plotting/*.rs` | Stringly `unit: &str`, dup ECDF/downsample/mesh/palette/`</svg>`, 2x `unwrap` | Use `ErrorUnit`, shared helpers/palette, `Result` tests, `prismatica` no-default | low | open | — |
| TD-AP-07 | SOLID | `apps/smellfish-rs/{src/**,assets/*,public/*,build.rs}` | Oversized cards, dead JS/CSS/empty `.txt` fetches, `MoleculeRow::clone`, `build.rs` `curl` | Split cards, prune dead JS/CSS fields, lazy yield, offline `build.rs` | low | open | — |
| TD-SE-01 | hygiene | `apps/.../smarts-evoliposuction/{Cargo.toml,src/**}` | Missing restriction lints, unfulfilled `#[expect]`, mixed SPDX, 86 `pub` | Align lints, standardize SPDX, narrow `pub` surface | low | open | — |
| TD-SE-02 | DRY | `apps/.../smarts-evoliposuction/src/{cli*,results_io,splitting/slug}.rs` | Dup CLI args, seq/par loops, positional CSV arrays, 2x category map | `CommonEvolveArgs`, `LevelFilter`, `evolve_one_row`, `serde::Serialize`, 1 table | low | open | — |
| TD-SE-03 | perf | `apps/.../smarts-evoliposuction/src/{download*,splitting*,cli/ordering}.rs` | 19x `$$$$` scan, 3x cell `.replace()`, full zip RAM, 90-line test-only CSV parser | Scan `$$$$` once, fast-path TSV write, stream zip, remove dead test parser | low | open | — |
| TD-CI-01 | DRY | `make/{build,lint,web,format,test,supply-chain,readme}.toml`, `mk.local`, `.github/*` | 16x wasm check/lint, 3 redundant builds, uncached `panache`, CLI untested in CI | Unified wasm clippy, `wasm=lint-wasm`, cache `panache`, CI job, `setup-dx`, prune docs | low | open | — |

## 3. Metrics Before → After

| Metric | Before (Baseline) | After | Delta |
| --- | --- | --- | --- |
| Tracked files (`git ls-files`) | 404 (406 w/ bench_smoke) | TBD | TBD |
| Rust LOC (`tokei` total / code) | 31,781 / 28,386 | TBD | TBD |
| Exact 6-line normalized clone ratio | 2.61% (601 / 23,040 lines) | TBD | TBD |
| Function length (max / mean / >40 / >60) | 151 / 18.4 / 119 / 50 | TBD | TBD |
| Approx branch complexity (max / mean / >15) | 36 / 3.0 / 15 | TBD | TBD |
| Non-`pub(crate)` `pub` items (`.rs`) | 674 | TBD | TBD |
| `#[expect]` / `#[allow]` sites (`.rs`) | 15 / 0 | TBD | TBD |
| `TODO` / `FIXME` / `unsafe` / prod `unwrap` | 0 / 0 / 0 / 0 | TBD | TBD |
| External crates (workspace instances / unique / dups) | 415 / 383 / 30 | TBD | TBD |
| External crates (`smarts-evoliposuction`) | 407 / 376 / 27 | TBD | TBD |
| `./mk check` / `./mk test` | 2.52 s / 18.20 s (366+14) | TBD | TBD |
| `smarts-evoliposuction` clippy + test (`--locked`) | 26.45 s / 34.49 s (32+6) | TBD | TBD |
| `./mk local:lint-wasm` vs unified wasm clippy | 14.68 s / 22.82 s (cold) | TBD | TBD |
| Workspace line coverage (`llvm-cov`) | 81.06% (7,656 / 9,445 lines) | TBD | TBD |
| WASM bundle `cxsmiles-yoga` | 1,341,229 bytes | TBD | TBD |
| WASM bundle `index` | 416,439 bytes | TBD | TBD |
| WASM bundle `json-count-rs` | 516,401 bytes | TBD | TBD |
| WASM bundle `lipid-selecto-rs` | 1,274,570 bytes | TBD | TBD |
| WASM bundle `mgf-precursor-erro-rs` | 1,314,643 bytes | TBD | TBD |
| WASM bundle `smellfish-rs` | 927,777 bytes | TBD | TBD |
| WASM bundle total (all 6 apps) | 5,791,059 bytes | TBD | TBD |
| Bench `stream_io_500k_lines` | 23.087 ms | TBD | TBD |
| Bench `mgf_parse_10k_spectra` | 19.372 ms | TBD | TBD |
| Bench `json_count_50k_rows` | 69.049 ms | TBD | TBD |
| Bench `cxsmiles_pos_and_repeat` | 494.866 ms | TBD | TBD |
| Bench `lipid_core_204_classify_export` | 11,995.530 ms | TBD | TBD |
| Bench `mgf_analysis_5k_ingest_recalibrated` | 201.218 ms | TBD | TBD |
| Bench `smellfish_200_pipeline_csv_svg` | 0.537 ms | TBD | TBD |
| Bench `mgf_plotting_diag_and_hist` | 16.267 ms | TBD | TBD |
| Bench `evoliposuction_sdf_zip_2k_to_tsv` | 9.898 ms | TBD | TBD |
| Bench `evoliposuction_split_2k_tsv` | 1.240 ms | TBD | TBD |

## 4. Structural Changes

*(Populated during Steps 2–6.)*

## 5. API Changes and Migrations

*(Populated during Steps 2–6; all workspace crates set `publish = false`.)*

## 6. Findings Handed Off (Suspected Scientific / Correctness Issues — Not Fixed Here)

1. **`FIND-01` (`crates/mgf-analysis-core/src/adduct.rs:71-72`)**: Substring `"CA"` check classifies `"[M+FormicAcid-H]-"` and `"[M+AceticAcid-H]-"` as `AdductFamily::MetalComplex` instead of `Other`.
2. **`FIND-02` (`crates/mgf-analysis-core/src/adduct.rs:190-194`)**: `normalize_adduct_label` slices `1..idx` before `']'`, dropping the first character on malformed inputs lacking leading `'['`.
3. **`FIND-03` (`crates/mgf-analysis-core/src/metrics/record.rs:97-103`)**: Incremental square-and-`sqrt` RMS accumulation introduces floating-point rounding drift relative to single final `sqrt` of mean-square.
4. **`FIND-04` (`crates/mgf-analysis-core/src/recalibration/{parsing,calibration,generator}.rs`)**: `parse_fragment_prefix` omits finiteness/positivity guards enforced by `mgf`, `recalibrate_fragments` applies deadzones absent from diagnostics/MGF export, and `generate_recalibrated_mgf` buffers peaks until `END IONS`.
5. **`FIND-05` (`crates/mgf-analysis-core/src/adduct/parse.rs:71`)**: `compact.contains("+H")` matches unbracketed `M+H2O` / `M+HCOONa` as positive polarity.
6. **`FIND-06` (`crates/lipid-core/src/lipids/classify.rs:153-160`)**: Unchecked `u32` addition/multiplication on untrusted `FORMULA=` counts can overflow on extreme inputs.
7. **`FIND-07` (`crates/lipid-core/src/parser/block.rs:169-172` vs `lipids/classify.rs:201`)**: `classify_formula` trims while `formula_exact_mass` passes untrimmed strings.
8. **`FIND-08` (`crates/cxsmiles-core/src/repeating.rs:107-120`)**: Integer division per element in `unit_multiset` can yield a multiset shorter than `unit_size` when pattern length is not an exact multiple.
9. **`FIND-09` (`crates/cxsmiles-core/src/positional.rs:124-126`)**: `build_targets` implicitly assumes component definitions have length 1 or 2.
10. **`FIND-10` (`crates/smellfish-core/src/evidence/{chemist,ring_family}.rs` vs `apps/smellfish-rs/public/*`)**: Shipped Ertl tables emit raw FG notation while Rust/JS keyword matching checks English words (`steroid`, `flavone`, `ring`), yielding 0 runtime scaffold/core hits on shipped tables.
11. **`FIND-11` (`crates/ui/src/components/upload_zone.rs:164-168` & `theme.rs`)**: `LIGHT.surface == LIGHT.surface2` and `DARK.surface == DARK.surface2`, making drag-hover background color change a visual no-op.
12. **`FIND-12` (`crates/ui/src/document.rs:68-81`)**: `DocumentHead` `use_effect` captures a non-reactive `String` clone of `lang`.
13. **`FIND-13` (`apps/lipid-selecto-rs/src/app/browser.rs:58-133`)**: Failed or empty subsequent file upload leaves stale `analysis` results visible and downloadable.
14. **`FIND-14` (`apps/smellfish-rs/src/app/input_panel.rs:39-41`)**: Hand-rolled `on_drop` omits `prevent_default()` and fails to reset `drag_active` on empty/error extraction.
15. **`FIND-15` (`apps/smellfish-rs/src/document_head.rs:25`)**: Unpinned `unpkg.com/@rdkit/rdkit/dist/RDKit_minimal.js` URL lacks version pin and SRI hash.
16. **`FIND-16` (`apps/smellfish-rs/public/group_names.txt` & `literature.rs:36`)**: Empty `group_names.txt` leaves Ertl FG labels untranslated while `literature.rs` cites it as populated.
17. **`FIND-17` (`apps/mgf-precursor-erro-rs/src/plotting/histogram.rs`)**: Histogram SVG lacks an embedded SVG legend (HTML-only legend in `DiagnosticsPanel`).
18. **`FIND-18` (`apps/.../smarts-evoliposuction/src/cli/split.rs` vs `splitting.rs`)**: `smiles_cap` (2000) truncates positives after `target_negative_count` already sampled up to 5000 negatives, creating 1:2.5 imbalance and double-shuffling negatives.
19. **`FIND-19` (`apps/.../smarts-evoliposuction/src/splitting.rs:240,264`)**: `main_class` and `subclass` splits leave `category` empty despite `category_of_main` existing.
20. **`FIND-20` (`apps/.../smarts-evoliposuction/src/download.rs:4,18-19`)**: Doc claims SDF-to-TSV deduplicates by `SMILES+LM_ID`, whereas it only filters non-empty records.

## 7. Deliberately Not Changed

- **`.cargo/config.toml` `smiles-parser` git patch**: Required by `./mk refquiem` for historical commit graph resolution.
- **`smarts-evoliposuction` workspace exclusion and `--locked` lockfile**: Prevents native CLI dependencies (`reqwest`/`tokio`/`zip`/`rayon`) from entering browser WASM graphs and keeps `smarts-evolution`/`smarts-rs` `smiles-rs` versions aligned.
- **`crates/lipid-core/src/chemical_class/lmsd_part{1,2}.rs` SMARTS & `examples/*.rs`**: Evolved scientific classification rules and reference examples; preserved verbatim.
- **`crates/smellfish-core/src/sparql.rs` vs `crates/ui/src/url.rs` percent encoding**: 8-line independent helpers in decoupled crates; unifying would introduce coupling for zero gain.
- **`.mk/*`, `mk`, `mk.versions`**: Synced upstream `rusti-graben` runner scripts; kept intact.
- **`SimpleAnalyticsBadge` per-app usage & `recalibration_demo.rs` explicit setup**: Trivial 3-line component calls and pedagogical standalone example.
- **`crates/ui/src/a11y.rs` & per-module `#![expect(clippy::empty_enums)]`**: Compile-time skip-link constants and scoped Dioxus `#[component]` macro suppressions.

## 8. Residual Debt and Risk

- Browser DOM/`web-sys`/`RDKit.js` runtime behavior is verified via `wasm32-unknown-unknown` compilation, clippy, and `dx build --profile wasm-release`, not headless browser execution.
- Scientific/correctness items (`FIND-01`..`FIND-20`) remain deliberately unfixed pending domain sign-off.
