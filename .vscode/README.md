# `.vscode/`

Editor configuration for this workspace, tracked deliberately: what is here is
toolchain config rather than personal preference, so it is committed rather than
left to each contributor's defaults. Two of the defaults that matter most here
are the wrong ones for a Dioxus project, and a mismatch is a diff on every save.

`settings.json` and `extensions.json` are **strict JSON**, not JSONC, so that
`check-json` can parse them and fail on a real syntax error. The reasoning
behind individual settings lives here instead of in comments inside the files.

The rules for what is checked are `rustfmt.toml`, `clippy.toml` and
`.editorconfig`; `settings.json` only makes an editor use them.

## `settings.json`

### rust-analyzer

`cargo.allTargets: true` and `cargo.features: "all"` mirror `./mk lint` so library,
binary, test, and example targets and all features are checked on save.

One target directory for the whole workspace (`cargo.targetDir`), which is also
what the Dioxus CLI uses; a per-crate target directory rebuilds dependencies
twice.

Nextest is the test runner (`cargo.testRunner: "nextest"` with
`NEXTEST_PROFILE: "default"`), so editor test runs use `.config/nextest.toml`
instead of compiling per-crate `cargo test` harnesses. Doctests still run via
`cargo test --doc` (`./mk test`).

`check.command: "clippy"` (with `checkOnSave: true`) matches `./mk lint`, so a
squiggle in the editor and a CI failure report the same finding.

`procMacro.enable: true` and `cargo.buildScripts.enable: true` expand Dioxus
`#[component]`/`rsx!`, `serde`, and `thiserror` macros;
`diagnostics.experimental.enable: true` surfaces experimental `rust-analyzer`
diagnostics in the editor.

### Formatting

rustfmt only, with the checked-in config. No formatter for TOML or markdown on
save: `tombi format` and `panache format` are what CI compares against, and an
editor running a different one is a diff per save.

The generated READMEs are diffed by `./mk local:readme-check`; editing one by
hand is undone on the next regeneration. Markdown lint is still fixed on save,
because that is the same rule CI applies.

### Exclusions

The wasm apps' output and the Dioxus CLI's build directory are generated, large,
and in `target/`. Excluded from search and from the file watcher so the editor
does not index a release build.

## `extensions.json`

Deliberately short. Each one earns its place:

- **rust-analyzer** is the language server for the whole project.
- **tamasfe.even-better-toml** is a TOML language server, a separate extension
  from the `tombi` formatter the hooks run; they answer different questions and
  neither replaces the other.
- **DavidAnson.vscode-markdownlint** is the only formatter the prose in
  `.github/` and the READMEs has.

`typos` is **not** here on purpose: it is a CLI check, installed by `./mk setup`
and run by `./mk typos` and the git hook. An editor extension would underline a
word the CLI does not flag, or the reverse, and the gate is what decides.
