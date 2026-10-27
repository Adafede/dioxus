# AI Agent Guide

## Workspace map

The workspace is a Cargo monorepo with two top-level groups plus the crates that
back them.

**Shared crates (`crates/`)** --- consumed by the apps; the source of truth, not
the apps:

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
- `apps/mgf-precursor-erro-rs` --- MGF precursor mass-error analysis (WASM +
  lib).

## Stable commands

The task runner is cargo-make. `./mk ci` is the single list of what CI
runs, so verifying a change means running that rather than assembling a command
from memory. Run `./mk --list-all-steps` for the current task list.

```bash
./mk setup      # once: the pinned toolchain and every tool the gate calls
./mk check      # cargo check --workspace --all-targets
./mk test       # nextest, then cargo test --doc (nextest skips doctests)
./mk dejadoc    # duplicate doctests, functions and context blocks
./mk refquiem   # stale names the git history removed
./mk lint       # clippy, all targets, all features, warnings denied
./mk lint-wasm  # the cfg(wasm32) branches the host run never compiles
./mk doc        # rustdoc with -D warnings
./mk ci         # all of the above plus the hygiene and supply-chain checks
./mk ci-fast    # the pre-push subset, without the supply-chain checks
```

```bash
./mk web-dev APP=cxsmiles-yoga   # or the other five apps
./mk web-build-all
./mk cli -- --help               # anything else the Dioxus CLI can do
```

Two things that are easy to get wrong and are worth not having to remember:

- **nextest does not run doctests.** `./mk test` runs nextest and then
  `cargo test --doc`, and so does everything else that runs tests. A dropped
  doctest step leaves the examples in the module docs unchecked and nothing
  reports a failure.
- **The host lint never compiles `#[cfg(target_arch = "wasm32")]` branches.**
  `./mk check-wasm` and `./mk lint-wasm` are the tasks that do, one
  package at a time rather than `--workspace --target wasm32`, which cannot pass
  because `upload` has a host-only path.
- **`cargo dejadoc` compares for doctests only unless it is asked for more.**
  `./mk dejadoc` passes `--functions` and `--context-blocks`, which are
  opt-in since 0.6.0, so dropping either leaves the gate green while checking
  less. It also passes `--context-dependency-depth 1` to compare against the
  crates this workspace depends on, and `--context-check-latest false` because
  that option defaults to true and checks the latest published release — a
  network call this gate must not make. `dejadoc` is pinned to 0.6.0 in
  `ci.yml` and `make/scripts/setup.sh` for the same reason: the version decides
  whether the gate checks anything at all.

  A **doctest** group is shared code examples; a **function** group is one body
  at two sites (fix with `delete`, `helper-or-macro`, or `generic-or-macro`);
  and a **context** group is a fragment inside two functions, which has no copy
  to keep, so move the shared code into one helper. A deliberate copy needs
  `// dejadoc: allow` on the line right above it. There is no `.dejadoc.toml`,
  so nothing is filtered by a token threshold.

  Dependency findings are advisory, not gated: `smellfish-rs` has an
  `html_title` matching one in `lotus-search`, but the dependency's copy is
  private, so the report names it as `request public extraction` and stays at
  exit 0. It also warns that the git checkout's origin does not match the
  declared repository — cargo clones git dependencies through a local bare DB,
  so the recorded origin is a `file://` URL. That warning is expected here and
  does not fail the gate.

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
- Prefer `crates/ui` components/styles and `crates/upload` over
  reimplementing the same concern in an app.

## References

- Task list: `./mk --list-all-steps`
- The gate, and what each check is for: `.github/CONTRIBUTING.md`
- AI contribution guide: `.github/ai/CONTRIBUTING_AI.md`

## Phase status

> **Note:** The file paths below are historical. `crates/lotus` and
> `apps/lotus-explore-rs` were removed from the workspace in the Phase 1
> cleanup (see `Cargo.toml` -- only `crates/ui` and `crates/upload` remain
> under `crates/`, and the six apps under `apps/` are `cxsmiles-yoga`,
> `index`, `json-count-rs`, `lipid-selecto-rs`, `mgf-precursor-erro-rs`, and
> `smellfish-rs`). Do not chase those paths.

- **Phase 1 (inventory + docs):** complete. Workspace map above revised;
  the removed `apps/lotus-explore-rs` had a dead triplicate style tree;
  `crates/ui` = shared design system, `crates/upload` = shared upload crate.
- **Phase 2 (shared UI --- upload):** complete. `ui::UploadZone` extracted into
  `crates/ui`; `json-count-rs`, `mgf-precursor-erro-rs`, and `lipid-selecto-rs`
  migrated to it (the `StyleBuilder` inline drop-zone pattern is now deduped to
  one place). `smellfish-rs` keeps its CSS-class CSV drop zone (different
  aesthetic --- flagged for a class-based `UploadZone` variant if desired).
- **Phase 2/6e (Footer/Card):** `ui::Footer` + `ui::Card` rich surface extracted
  into `crates/ui`; migrating remaining apps off local
  reimplementations.
- **Phase 3 (god-file splitting):** `mgf app.rs` (684→137) split complete ---
  `app/results.rs` (`ResultsPanel`), `app/browser.rs` (shared
  `attempt_analysis_from_files` +
  `begin_analysis_from_blob`/`load_example_mgf`), `app::example_load_button`;
  `app()` body < 100 lines, `#[allow(clippy::too_many_lines)]` removed,
  `# Errors` doc added. `cxsmiles/app.rs`,
  `mgf plotting.rs`→`plotting/{mod,data,scatter,diagnostics,cumulative,color}`,
  `mgf parser.rs`→`parser/{mod,adduct,mass,block}`,   `mgf metrics.rs`
  (850→`metrics/{mod,merge}`) splits also complete, as well as
  `recalibration.rs` (833→`calibration/{types,parsing,calibration,generator}` +
  re-export `mod.rs`); the `plotting/diagnostics.rs` histogram
  `bin_count==0` div-by-zero is guarded. Remaining larger god-files:
  smellfish `app.rs`/`verdict.rs`
  (tests only).
- **Phase 4 (shared signals):** complete (verified) --- `mgf-precursor-erro-rs` +
  `json-count-rs` declare signals via `ui::shared_signal!` / `shared_signals!`;
  `cxsmiles-yoga`, `lipid-selecto-rs`, and other apps use plain
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
   `./mk lint`/`./mk test`/`./mk check` + 3-app wasm all green.
- **Phase 6f (styles/services consolidation):** complete
   (triplicate style dir removed; only app-specific helpers remain).
> The Phase 6f entry above was originally labeled "lotus-sparql styles/services
> consolidation" and referenced `crates/lotus`, which was removed from the
> workspace. It is kept for historical record; the code described here no
> longer exists.

## Incident: lotus-explore Qlever 429 storm (FIXED)

**Symptom:** a single `Gentiana lutea` search (and curation) produced a
permanent IP 429 from `https://qlever.dev/api/wikidata`. Captured logs
(15:00:06--07) showed 5 SPARQL POSTs within \~150 ms (3×200 in 30--135 ms +
2×429 in 24--25 ms).

**Root cause:** an anonymous-quota _burst_, not heavy queries --- the 200s
returned in 30--135 ms. Two sites fired Qlever POSTs concurrently: 1. **search** ---
`try_join!(count, display)` ran the COUNT(DISTINCT ...) over the full base
("dumb pagination" request) _concurrently_ with the display query, and the 429
retry re-ran the whole pipeline with only a short backoff, amplifying it. 2.
**curation** --- `buffer_unordered(CURATION_CONCURRENCY = 8)` fired up to 8
rows' Qlever POSTs at the same time (compound fetch + ASK per row). (The
lotus-API fast-path was ALSO on by default --- `ExecutionStrategy` defaulted to
`ApiFirst` --- so every search/429-retry wasted a request on the absent API
"builder error" path. That is why "it was working 48 h ago".)

**Fix (all gated green):** - **API off by default.** `ExecutionStrategy::Direct`
is the default for interactive searches; `ApiFirst` only runs when a non-empty
base URL is explicitly configured
(`crate::api::api_base_url().is_some_and(|b| !b.is_empty())`). No dead "builder
error" waste. - **Respect the endpoint on 429.** `rate_limit_backoff_ms` (1 s
base, doubling, capped at 10 s) + `plan_retry` caps 429 retries at **1**
(`effective_max = 1` for `ErrorClass::RateLimit`) instead of the generic 3. -
  **COUNT restored, but serialized + best-effort** (no `try_join!`): the display
  query runs first; the COUNT (`query_counts_from_base`) fires strictly _after_,
  sequentially, and a 429 on it is swallowed → `None` (falls back to
  `rows.len()`). Rationale was inline in
  `apps/lotus-explore-rs/src/features/explore/service/fetch_results/wasm.rs`
  (now removed from the workspace; the principle remains). -
**Curation serialized.** `CURATION_CONCURRENCY` `8 → 1` in
`features/curation/services/pipeline.rs`: rows are curated one at a time so
Qlever POSTs never overlap. (This also unblocks the mass/RDKit step, which was
being aborted mid-row when the row's Qlever POST 429'd.)

No API key is available or required --- the throttle is beaten by staying under
the anonymous quota (one POST at a time), never by retrying harder.

> Do not lower the 429 backoff or raise curation concurrency to "speed things
> up" --- that is exactly what re-triggers the permanent throttle. If Qlever
> throughput must improve, add a client-side results cache, not more concurrent
> POSTs.
