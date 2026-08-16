# AI Agent Guide

## Workspace map

The workspace is a Cargo monorepo with two top-level groups plus the crates that
back them.

**Shared crates (`crates/`)** --- consumed by the apps; the source of truth, not
the apps:

- `crates/lotus` --- LOTUS domain models, SPARQL query builders, and
  platform-agnostic SPARQL-over-HTTP transport. The single shared data core for
  `lotus-api` and `lotus-explore-rs`.
- `crates/ui` --- unified, type-safe Dioxus design system: reusable components
  (`Button`, `Card`, `Footer`, `Header`, `NoticeBar`, `SegmentedControl`),
  `DocumentHead`/`DocumentLinks`, pure-Rust style builders (`styles::*`), and
  theme primitives (`theme::*`). All apps depend on `ui`; prefer it over
  inlining raw CSS. **Inline-style convention:** every inline `style:` must be a
  `StyleBuilder::new()...build()` value (or a `styles::*` helper), never a raw
  `style: "..."` string; reused sub-patterns should be extracted as local style
  fns rather than copy-pasted. Reference: `apps/cxsmiles-yoga/src/app.rs`.
- `crates/ui::signals` --- the `shared_signal!` / `shared_signals!` macros
  collapse the repeated `#[cfg(target_arch = "wasm32")] let x = use_signal(...)`
  / `#[cfg(not(...))] let mut x = use_signal(...)` pair into a single
  declaration that preserves per-platform `mut` semantics. New components should
  declare signals through these macros.
- `crates/upload` --- WASM streaming file I/O (`BlobCursor`, `BlobLines`),
  throttled progress, and unified download helpers. The shared upload/download
  crate for every upload-based WASM app.

**Applications (`apps/`)** --- thin shells over the shared crates:

- `apps/index` --- accessible landing page (WASM).
- `apps/json-count-rs` --- count non-null fields in uploaded JSON (WASM).
- `apps/lipid-selecto-rs` --- lipid classification & filtering via SMARTS
  (WASM).
- `apps/cxsmiles-yoga` --- CX-SMILES generation from related structures (WASM).
- `apps/smellfish-rs` --- NP-likeness scoring with RDKit.js + QLever (WASM).
- `apps/lotus-explore-rs` --- LOTUS Knowledge Explorer, LOTUS/Wikidata/QLever
  SPARQL explorer (WASM). Its `src/` is layered: `main.rs` exposes top-level
  *canonical* facades (`api`, `models`, `queries`, `sparql`, `state`,
  `repositories`, `services`) that are shared app-wide, while `src/features/`
  holds *feature-scoped* modules (`explore` engine, `curation` workflow) --- the
  `features/*/state|repositories|services` trees are **not** dead duplicates of
  the top-level ones; they are curation/explore specific. The `src/ui/` module
  holds LOTUS-specific style constants (`layout_styles`, `table_styles`,
  `search_controls`, `style_constants`, a11y contracts); the dead triplicate
  style-directory tree (`src/styles/`, `src/lotus_styles/`, `src/ui/styles/`)
  was consolidated in phase 6f --- only the LOTUS-specific helpers above remain
  (generics belong in `crates/ui`). `src/components/` and `src/pages/` are the
  UI layer.
- `apps/mgf-precursor-erro-rs` --- MGF precursor mass-error analysis (WASM +
  lib).
- `apps/lotus-api` --- native Axum API for LOTUS search and exports.

## Stable commands

```bash
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
```

```bash
dx serve --package lotus-explore-rs
cargo run --locked -p lotus-api
```

## Change protocol

1. Read the smallest relevant module set before editing.
2. Keep changes local and preserve existing public contracts.
3. Update tests, docs, and skill indexes when boundaries change.
4. Verify with format, check, test, and lint commands.

## Safety rules

- Prefer deterministic behavior over hidden state.
- Do not add new dependencies unless required by the architecture.
- Keep user-facing behavior stable unless a change is explicitly requested.
- Favor typed contracts, explicit errors, and narrow ownership.
- Prefer `crates/ui` components/styles and `crates/upload`/`crates/lotus` over
  reimplementing the same concern in an app.

## References

- Architecture: `apps/lotus-explore-rs/docs/ARCHITECTURE.md`
- Skills: `apps/lotus-explore-rs/SKILLS.md`
- AI contribution guide: `.github/CONTRIBUTING_AI.md`
- Agent efficiency guide: `.github/AI_EFFICIENCY_GUIDE.md`

## Phase status

- **Phase 1 (inventory + docs):** complete. Workspace map above revised;
  `apps/lotus-explore-rs` dead triplicate style tree removed; `crates/ui` =
  shared design system, `crates/upload` = shared upload crate, `crates/lotus` =
  shared SPARQL core.
- **Phase 2 (shared UI --- upload):** complete. `ui::UploadZone` extracted into
  `crates/ui`; `json-count-rs`, `mgf-precursor-erro-rs`, and `lipid-selecto-rs`
  migrated to it (the `StyleBuilder` inline drop-zone pattern is now deduped to
  one place). `smellfish-rs` keeps its CSS-class CSV drop zone (different
  aesthetic --- flagged for a class-based `UploadZone` variant if desired).
- **Phase 2/6e (Footer/Card):** `ui::Footer` + `ui::Card` rich surface extracted
  into `crates/ui`; migrating lotus-explore and remaining apps off local
  reimplementations.
- **Phase 3 (god-file splitting):** `mgf app.rs` (684→137) split complete ---
  `app/results.rs` (`ResultsPanel`), `app/browser.rs` (shared
  `attempt_analysis_from_files` +
  `begin_analysis_from_blob`/`load_example_mgf`), `app::example_load_button`;
  `app()` body < 100 lines, `#[allow(clippy::too_many_lines)]` removed,
  `# Errors` doc added. `cxsmiles/app.rs`,
  `mgf plotting.rs`→`plotting/{mod,data,scatter,diagnostics,cumulative,color}`,
  `mgf parser.rs`→`parser/{mod,adduct,mass,block}`, `mgf metrics.rs`
  (850→`metrics/{mod,merge}`) splits also complete. Completed this session:
  `recalibration.rs` (833→`calibration/{types,parsing,calibration,generator}` +
  re-export `mod.rs`) and `apps/lotus-explore-rs` `sections.rs`
  (521→`sections/{mod,styles}`); the `plotting/diagnostics.rs` histogram
  `bin_count==0` div-by-zero is guarded. Remaining larger god-files:
  `crates/lotus/src/sparql.rs` (840), `crates/ui/styles/lotus/responsive.rs`
  (910), smellfish `app.rs`/`verdict.rs`, lotus-api `tests.rs` (tests only).
- **Phase 4 (shared signals):** complete (verified) --- `mgf-precursor-erro-rs` +
  `json-count-rs` declare signals via `ui::shared_signal!` / `shared_signals!`;
  `cxsmiles-yoga`, `lipid-selecto-rs`, and `lotus-explore-rs` use plain
  `let`/`let mut use_signal` (a workspace-wide grep confirms no
  `#[cfg(target_arch = "wasm32")]` signal-declaration duplication remains in any
  app).
- **Phase 5 (typed errors):** complete (verified) --- `MgfError` +
  `MgfErrorKind::Drawing` introduced in
  `apps/mgf-precursor-erro-rs/src/errors.rs` and wired into the module tree; the
  10 `render_*` SVG helpers in `plotting/{scatter,diagnostics,cumulative}.rs`
  now return `Result<String, MgfError>` (54 plotters `.unwrap()` → `?`); all
  early/final returns wrapped in `Ok(...)`; all call sites in `app.rs`,
  `app/plots.rs`, and `recalibration_demo.rs` degrade with
  `.unwrap_or_default()`; the 9 stale `# Panics` doc blocks converted to
  `# Errors` (summary_text got a new `# Errors` block); redundant `#[must_use]`
  dropped from the `Result`-returning renderers; workspace
  `cargo clippy`/`cargo test`/`cargo check` + 3-app wasm all green.
- **Phase 6f (lotus-sparql styles/services consolidation):** complete
  (triplicate style dir removed; only LOTUS-specific helpers remain).

## Incident: lotus-explore Qlever 429 storm (FIXED)

**Symptom:** a single Explore search hammered `https://qlever.dev/api/wikidata`
with ~2 SPARQL POSTs × 4 retries (plus a dead lotus-API "builder error" attempt
each time) = ~16 requests in ~1.4s, producing a permanent IP 429. The search
"copy query" button also vanished because the results panel never rendered.

**Root cause:** (1) the lotus-API fast-path was ON by default
(`ExecutionStrategy` defaulted to `ApiFirst`), so every search — and every 429
retry — first tried the absent/misconfigured API before falling back to direct
SPARQL; (2) the 429 retry re-ran the *entire* pipeline (`do_search`: api attempt +
taxon + display + COUNT) with a 200/400/800 ms backoff, amplifying the burst;
(3) the COUNT query (the "dumb pagination" request) fired concurrently with the
display query, so a 429 on either failed and retried the whole search.

**Fix:**
- **API off by default.** `ExecutionStrategy::Direct` is now the default for
  interactive searches; `ApiFirst` only runs when a non-empty base URL is
  explicitly configured (`crate::api::api_base_url().is_some_and(|b| !b.is_empty())`).
  The empty dev-auto-detect base and `HybridRepository::api_search` both treat an
  empty base as `NotConfigured`, so no "builder error" waste.
- **Respect the endpoint on 429.** RateLimit uses `rate_limit_backoff_ms`
  (1 s base, doubling, capped at 10 s) instead of the 100 ms base, and `plan_retry`
  caps 429 retries at **1** (`effective_max = 1` for `ErrorClass::RateLimit`)
  instead of the generic 3. A transient 429 gets one long-backoff retry; a
  persistent one surfaces immediately so Qlever's window can reset.
- **COUNT query is best-effort.** `fetch_results/wasm.rs` no longer fails the
  search when the COUNT POST 429s: the display query is authoritative and the
  COUNT is best-effort (`None` → `total_matches` falls back to `rows.len()` in
  `finalize`). Only a display 429 retries, and only per the capped policy above.

> Do not lower the 429 backoff or remove the retry cap to "be more resilient" —
> that is exactly what re-triggers the permanent throttle. If Qlever throughput
> must improve, add a client-side results cache, not more concurrent POSTs.
