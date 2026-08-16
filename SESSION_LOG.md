# SESSION_LOG

Record of the overnight structural-refactor session on the `Adafede/dioxus`
workspace (Rust 1.97.0, ed. 2024, Dioxus 0.7.10, macOS arm64).

## §0 --- Morning summary

**Status at handoff (2026-08-14T06:07:49+02:00):** smokefish `app.rs` split was
IN-FLIGHT --- the module split was written (`app/mod.rs`, `app/browser.rs`,
`app/results.rs`) but did not pass clippy. PRIMARY goal: finish the smokefish #7
split.

**What was done first:** fixed the 4 clippy errors blocking the smokefish split: -
`app/mod.rs`: dropped unused `mut` on the `status` signal (its `set()` only runs
inside `browser.rs` on captured copies, so the binding is never mutated in scope ---
confirmed via `dioxus-signals` 0.7.10 `WritableExt::set(&mut self)`; mgf's green
macro-declared `status` is the same pattern). - `app/browser.rs`: added
`clippy::too_many_arguments` to both `#[allow(...)]` (9 args is intrinsic to the
8-signal dispatch signature, same as mgf's 7-arg threshold approach). -
`app/browser.rs`: `files.into_iter().next()` → `files.first()`
(`into_iter_on_ref`). - `app/results.rs`: fixed a **wasm-only** `format!` arg
bug in `escape_csv` (`format!("\"{s}\"", s.replace(...))` → unused arg). Native
was green because `escape_csv` is `#[cfg(target_arch = "wasm32")]`.

**Verification:** - `cargo fmt --all --check` --- clean. -
`cargo clippy --workspace --all-targets --locked -- -D warnings` --- clean
(native). - `cargo test --workspace --all-targets --locked` --- all pass, 0
failures. -
`cargo check -p smellfish-rs --target wasm32-unknown-unknown --locked` ---
compiles. - `cargo clippy -p smellfish-rs --target wasm32` --- **blocked by
pre-existing `upload` crate breakage** (21 `ignored_unit_patterns` errors in
upload's wasm-only code). This is a workspace-wide pre-existing condition
(reproduced on mgf too) and is explicitly **out of scope** per the constraint:
"DO NOT open crates/upload/src/download.rs wasm clippy nitpicks (not in any
gate)."

**Action taken:** committed the smokefish #7 split (5 files: `app/` x3 new,
`app.rs` deletion, `model.rs` PartialEq derives). `app.rs.bak` backup deleted.

**Next:** secondary god-file splits remaining in *shared* crates (higher blast
radius) --- `crates/lotus/src/sparql.rs` (840),
`crates/ui/styles/lotus/responsive.rs` (910); plus Phase 2 (consolidate apps
onto `crates/ui`, the user's #2). App-local god-files in mgf/lotus-explore are
now complete.

## §3 --- Recommendation log

State of each recommendation at session start, with starting state and outcome.
One line per recommendation; append completion entries as work proceeds.

  | #   | Rec                                                                                                          | Starting state                       | Outcome                                                                                                             |
  | --- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
  | 1   | cxsmiles-yoga + index into WASM CI matrix                                                                    | done by prior session                | verified green (see §1)                                                                                             |
  | 2   | index app: depend on `ui`                                                                                    | done by prior session                | verified                                                                                                            |
  | 3   | lotus-api doc (Warp→Axum) + lotus Dioxus.toml watch_path                                                     | done by prior session                | verified                                                                                                            |
  | 4   | mgf plotting.rs split (scatter/diags/cumulative)                                                             | done by prior session                | verified                                                                                                            |
  | 5   | lotus-api clap CLI (HOST/PORT derive)                                                                        | done by prior session                | verified                                                                                                            |
  | 6   | json-count-rs main.rs split                                                                                  | done by prior session                | verified                                                                                                            |
  | 7   | oversized app.rs splits: lipid-selecto (done), mgf (done), **smellfish (in-flight → DONE this session)**     | smellfish in-flight, 4 clippy errors | **completed**: smokefish `app/` split committed green                                                               |
  | 8   | lotus-api `.expect()` → typed errors                                                                         | done by prior session                | verified                                                                                                            |
  | 9   | root justfile                                                                                                | done by prior session                | verified                                                                                                            |
  | 10  | `cfg(wasm32)` signal pairs → `shared_signal!` (cxsmiles/lipid)                                               | done by prior session                | verified                                                                                                            |
  | 4b  | mgf plotting panic-hardening (`plotting/diagnostics.rs` `bin_count==0` div-by-zero)                          | in-flight, was riskiest              | **completed**: early-return guard committed green; scatter.rs/cumulative.rs verified already panic-hardened (no-op) |
  | 7b  | split `recalibration.rs` (833) into `calibration/{types,parsing,calibration,generator}` + re-export `mod.rs` | pending secondary god-file           | **completed**: committed green (mgf fmt+clippy+test+wasm)                                                           |
  | 7c  | split lotus-explore `sections.rs` (521) into `sections/{mod,styles}`                                         | pending secondary god-file           | **completed**: committed green (327 tests; wasm check ok)                                                           |

## §1 --- Notes

- Type fact confirmed during this session: `dioxus::html::FileData` is the
  upload-file type (not `Files`); `HasFileData` provides
  `FormData::files() -> Vec<FileData>`; `&evt.data().files()` coerces to
  `&[FileData]`.
- `Signal<T>: PartialEq` is pointer-based (does not require `T: PartialEq`);
  `#[component]` props get `#[derive(PartialEq)]`, so by-value prop types (e.g.
  `MoleculeRow`) must be `PartialEq` (added to model.rs).
- smokefish crate-level `#![allow(...)]` at `lib.rs` covers
  `too_many_lines`/`missing_errors_doc` etc., so `app()` may exceed 100 lines
  and `app()` is a plain `pub fn app() -> Element` (no `#[component]`, to avoid
  E0255 with `pub mod app;` + `pub use app::app`).
- Native `Signal` storage type used (`Signal<T, UnsyncStorage>`).
- Phase 2 enabler: `crates/ui::Button` gained an optional `onclick` prop
  (`Option<EventHandler<Event<MouseData>>>`, default None) and `ButtonVariant`
  was re-exported from `ui::components`/`ui::prelude`. `smellfish-rs` primary
  "Analyze pasted SMILES" button migrated off its local `<button class="btn
  btn-primary">` onto `ui::Button`. Theme stays consistent (smellfish already
  injects `ui::styles::bundled_lotus_styles`). Full gate green: fmt + clippy
  --workspace (11 crates) + test + 7 per-app wasm32 checks. (commit e87b395)
- Phase 2, continued: migrated cxsmiles-yoga "Generate" button (app.rs) onto
  `ui::Button` (Primary; its `background_color(colors.accent).color(colors.bg)`
  equals ui::Button Primary) and dropped the now-unused `colors` local.
  Combined with the smellish primary-button migration (commit e87b395) and the
  `ui::Button` `onclick` enabler, 2 clean, color-consistent button dedups are
  done. Remaining local buttons are not 1:1 ui::Button targets: cxsmiles
  CopyCell (clipboard + "Copied!" state-swap widget), lipid & mgf "Load
  example" buttons (neutral/non-accent colors #f8fafc, #2563eb).
  lotus-explore still has ~11 local onclick buttons (sidebar/notice/loading/
  form_inputs/sections/download_actions) to migrate; pending user scope call.

- Phase 2, lotus-explore: migrated 4 accent "Primary" buttons onto ui::Button —
  SearchButton (form_inputs.rs) and add-row / load-examples / second-pass
  (sections/mod.rs). All used lotus --btn-primary-bg = #0b5cab = ui::Button
  Primary accent (color-consistent). Removed now-unused primary_buttons
  re-exports (button_primary_style, button_primary_block_style) from
  ui/style_constants.rs. Remaining lotus-explore locals correctly NOT migrated:
  sidebar toggle (ARIA aria_controls/expanded/pressed + dynamic label), the
  "generate" button (dynamic processing label), parse-tsv / remove-row /
  load buttons (neutral/non-accent: button_sm/xs_style, #f8fafc/#cbd5e1),
  and download_actions + lotus copy_button (title/aria_label/title +
  CopyButton clipboard state-swap widget). (commit next)

## §2 --- Incident: lotus-explore Qlever 429 storm (FIXED)

User reported a single `Gentiana lutea` search producing a permanent IP 429 from
`https://qlever.dev/api/wikidata`. Logs showed ~2 SPARQL POSTs × 4 retries **plus
a dead lotus-API "builder error" attempt per retry** = ~16 requests in ~1.4s; the
query/copy button vanished because results never rendered.

**Fix (3 parts), all gated green:**
1. **API off by default** — `ExecutionStrategy::Direct` is now the default for
   interactive searches; `ApiFirst` only when a non-empty base URL is explicitly
   configured. Removes the dead "builder error" api attempt on every search/retry.
   (`strategy.rs`, `executor.rs`, `repositories/hybrid.rs`)
2. **Respect the 429** — `rate_limit_backoff_ms` (1 s base) + `plan_retry` caps
   RateLimit retries at 1 (vs the generic 3). A transient 429 gets one long-backoff
   retry; a persistent one surfaces immediately so Qlever's window can reset.
   (`error_recovery_coordinator.rs`, `retryable_orchestrator.rs`)
3. **COUNT query best-effort** — `fetch_results/wasm.rs` no longer fails/retries
   the whole search on a COUNT 429 (display query is authoritative; COUNT → `None`
   → `total_matches` falls back to `rows.len()` via `finalize`). The expensive
   "dumb pagination" COUNT POST no longer amplifies the storm.

**Verification:** `cargo fmt --all -- --check` + `cargo clippy --workspace
--all-targets --locked -- -D warnings` + `cargo test --workspace --all-targets
--locked` (all crates pass; lotus-explore 330) + 7 per-app
`cargo check --target wasm32-unknown-unknown --locked` all green.
