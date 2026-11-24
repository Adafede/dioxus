# Structural Refactor Plan

Ordered migration plan transforming the workspace into the three-layer
architecture documented in [`ARCHITECTURE.md`](./ARCHITECTURE.md) while keeping
every gate green on each commit.

## Migration Steps

- [x] **Step 0 — Baseline measurements & Phase 0/1 docs**: Record test, build,
  and WASM size baselines; commit [`docs/audit/AUDIT.md`](./audit/AUDIT.md),
  [`docs/ARCHITECTURE.md`](./ARCHITECTURE.md), and `docs/REFACTOR_PLAN.md`.
  - *Verify*: `./mk check && ./mk test && ./mk refquiem && ./mk commits`
  - *Rollback*: `git revert HEAD`

- [x] **Step 1 — Extract `crates/workspace-gates` & gate `gloo-timers` to `wasm32`**:
  Move repo-wide gate tests out of `crates/upload/tests/` into a dedicated
  test-only crate and move `gloo-timers` into target-specific `wasm32`
  dependencies in `cxsmiles-yoga` and `lipid-selecto-rs`.
  - *Verify*: `./mk ci-fast` + `cargo machete`
  - *Rollback*: `git revert HEAD`

- [x] **Step 2 — Extract Dioxus-free `crates/stream-io` & unify `UploadError`**:
  Move pure byte/line streaming and progress throttling into `crates/stream-io`,
  deduplicate chunk buffer refills, switch `BlobSource::clock` to `performance.now()`,
  and unify `crates/upload` downloads and event extraction onto `UploadError`.
  - *Verify*: `./mk ci-fast` + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
  - *Rollback*: `git revert HEAD`

- [x] **Step 3 — Introduce shared `crates/mgf` and `crates/chem-depict`**:
  Consolidate MGF streaming parsing and panic-guarded `chematic` SMILES/SVG
  depiction into two reusable Layer 0 crates with characterization and property tests.
  - *Verify*: `./mk ci-fast` + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
  - *Rollback*: `git revert HEAD`

- [x] **Step 4 — Extract `crates/json-count-core` & slim `apps/json-count-rs`**:
  Move streaming JSON counting, unescaping, number validation, and sorting into
  `json-count-core` with typed `JsonCountError`, explicit scanner state machine,
  reusable row/key buffers, and memoized UI rendering.
  - *Verify*: `./mk ci-fast` + `./mk mutants -p json-count-core`
  - *Rollback*: `git revert HEAD`

- [x] **Step 5 — Extract `crates/cxsmiles-core` & slim `apps/cxsmiles-yoga`**:
  Break internal module cycles, move CX-SMILES generation into `cxsmiles-core`
  with `CxError`, sum-typed constructs, O(V+E) aromatic expansion, backtracking
  DFS, and memoized UI depictions using `ui::theme` tokens.
  - *Verify*: `./mk ci-fast` + `./mk mutants -p cxsmiles-core`
  - *Rollback*: `git revert HEAD`

- [x] **Step 6 — Extract `crates/lipid-core` & slim `apps/lipid-selecto-rs`**:
  Unify LIPID MAPS family metadata into `LipidFamily`, wire MGF/SMILES parsing
  onto `mgf` + `chem-depict`, replace `HashMap<String, bool>` with `ClassMatchSet`,
  precompile SMARTS, and split `app/components.rs`.
  - *Verify*: `./mk ci-fast`
  - *Rollback*: `git revert HEAD`

- [x] **Step 7 — Extract `crates/mgf-analysis-core` & modularize `apps/mgf-precursor-erro-rs`**:
  Move adduct grammar, mass derivation, precursor stats, recalibration, and
  diagnostic computation into `mgf-analysis-core`; decouple HTML tables from
  `plotting`; downsample scatter SVGs; split and memoize `ResultsPanel`.
  - *Verify*: `./mk ci-fast`
  - *Rollback*: `git revert HEAD`

- [x] **Step 8 — Extract `crates/smellfish-core` & slim `apps/smellfish-rs`**:
  Move evidence, typed verdicts, CSV I/O, pipeline aggregation, and generic-HTTP
  SPARQL/QLever enrichment into `smellfish-core`; eliminate concurrent drop races;
  split `styles.rs` and `results.rs`; fix `build.rs` CSV parsing.
  - *Verify*: `./mk ci-fast`
  - *Rollback*: `git revert HEAD`

- [x] **Step 9 — Refactor `crates/ui` & `apps/index`**:
  Replace `common` with focused accessibility and URL modules, introduce `Theme`
  and `UploadStatus`, pre-allocate `StyleBuilder`, decompose `DocumentHead` and
  `UploadZone`, and reuse the shared `Card` component in `apps/index`.
  - *Verify*: `./mk ci-fast` + `./mk local:readme-check`
  - *Rollback*: `git revert HEAD`

- [ ] **Step 10 — Refactor standalone CLI `smarts-evoliposuction`**:
  Fix `O(N log N)` disk reads in `order_rows_for_evolution`, eliminate ~48k-row
  cloning in `splitting.rs`, split `main.rs`/`splitting.rs`/`evolve.rs`/`sdf.rs`,
  and introduce typed errors/structs under `--locked`.
  - *Verify*: standalone `clippy` + `test --locked`, plus `./mk local:license-headers && ./mk test`
  - *Rollback*: `git revert HEAD`

- [ ] **Step 11 — Final verification, metrics comparison & `REVIEW_NOTES.md`**:
  Run full CI gates, WASM size/timing measurements, mutation checks, update
  [`DEBT.md`](./DEBT.md) if ratchets tightened, and commit `REVIEW_NOTES.md`.
  - *Verify*: `./mk ci` + `./mk commits` + standalone `--locked` checks
  - *Rollback*: `git revert HEAD`
