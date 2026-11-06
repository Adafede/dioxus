# Technical Debt & Ratchet Ledger

Every unavoidable lint suppression, duplicate-dependency skip, echoes/coverage ratchet, or mutation exclusion in this repository is recorded here with its reason, owner, and removal condition.

## Active Exceptions

| Location | Gate / Lint | Reason | Owner | Removal Condition |
|---|---|---|---|---|
| `Cargo.toml` (`[workspace.lints.clippy]`) | `multiple_crate_versions = "allow"` | Dioxus optional subsystems pull in two versions of `const-serialize` (`0.7.2` vs `0.8.0-alpha`); duplicate versions are audited and gated with exact reasons in `deny.toml`. | `@Adafede` | Upstream `dioxus` / `manganis` unification of `const-serialize`. |
| `Cargo.toml` (`[workspace.lints.clippy]`) | `suboptimal_flops = "allow"` | Fires on 6 plot-axis padding expressions (`min - span * 0.05`); FMA changes last-bit rounding for a cold once-per-plot calculation. | `@Adafede` | Permanent (intentional floating-point semantics). |
| `Cargo.toml` (`[workspace.lints.clippy]`) | `redundant_pub_crate = "allow"` | Contradicts `rustc`'s `unreachable_pub = "deny"` (one demands `pub(crate)`, the other `pub`). | `@Adafede` | Upstream `clippy` / `rustc` lint reconciliation. |
| `deny.toml` (`[bans] skip`) | 18 skipped duplicate crate names (`gloo-timers`, `const-serialize`, `const-serialize-macro`, `miniz_oxide`, `rand`, `rand_core`, `getrandom`, `syn`, `winnow`, `rustc-hash`, `windows-sys`, `jni`, `jni-sys`, `r-efi`, `thiserror`, `thiserror-impl`, `geometric-traits`, `wasm-streams`) | Transitive version splits across `dioxus` 0.7 optional desktop/mobile feature sets and EMI git dependencies (`mascot-rs`, `lotus-explore-rs`). | `@Adafede` | Remove individual entries as `dioxus` and upstream git dependencies upgrade; `cargo deny` warns on unnecessary skips. |
| `.cargo/mutants.toml` (`exclude_globs`) | 9 path globs (`**/build.rs`, `**/main.rs`, `**/app.rs`, `**/app/**`, `**/depict.rs`, `**/depict_simple.rs`, `**/examples.rs`, `**/plotting/**`, `**/processing/wasm_entry.rs`) | Build scripts, Dioxus RSX/SVG rendering components, and `wasm32`-only `Blob`/`Signal` glue (`wasm_entry.rs`) that cannot be linked or meaningfully asserted in host unit tests. Pure logic lives in `MUTANTS_PACKAGES=(json-count-rs cxsmiles-yoga)`. | `@Adafede` | Keep scoped to pure UI/WASM entry glue; run `./mk local:mutants-ui` or `./mk local:mutants-mgf` for opt-in UI/app sweeps. |

## Ratchet Floors (`mk.local`)

- `COVERAGE_FAIL_UNDER_LINES="67"`
- `ECHOES_MAX_APPROXIMATE=0`
- `ECHOES_MAX_REUSE=0`
- `Cargo.toml` lint `allow` count: `3`
- `MUTANTS_PACKAGES=(json-count-rs cxsmiles-yoga)`
