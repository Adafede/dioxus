# `.vscode/`

Editor configuration for this workspace, tracked deliberately: what is here is
toolchain config rather than personal preference, so it is committed rather than
left to each contributor's defaults. Two of the defaults that matter most here
are the wrong ones for a Dioxus project, and a mismatch is a diff on every save.

`settings.json` and `extensions.json` are **strict JSON**, not JSONC, so that
`check-json` can parse them and fail on a real syntax error. The reasoning behind
individual settings lives here instead of in comments inside the files.

The rules for what is checked are `rustfmt.toml`, `clippy.toml` and
`.editorconfig`; `settings.json` only makes an editor use them.

## `settings.json`

### rust-analyzer

The workspace has no root package, so the server has to be told to run the cargo
check for a file's own crate rather than for the workspace, or every keystroke
tries to build the whole dependency graph. `cargo.features: "all"` is the same
reasoning for the feature resolver.

One target directory for the whole workspace (`cargo.targetDir`), which is also
what the Dioxus CLI uses; a per-crate target dir means building a dependency
twice.

Nextest is the test runner (`cargo.testRunner`). Without this the editor builds a
test binary per crate with `cargo test`, which is the slow path the gate moved
off. `testRunnerExtraEnv` exists because nextest does not run doctests, so the
editor's "run doctest" needs the one runner that does.

`check.command: "clippy"` matches what the gate runs rather than plain `cargo
check`, so a squiggle and a CI failure are the same finding.

`procMacro.enable` is on because the crates are browser targets and half the
workspace does not type-check for the host without its cfg gates; enabling the
server outright turns every `crates/upload` error into a squiggle.

`diagnostics.experimental.enable` is on because the workspace denies
`unused_qualifications`, and an editor that shows it inline inside an `rsx!`
block sends you to delete an attribute that is load-bearing.

### Formatting

rustfmt only, with the checked-in config. No formatter for TOML or markdown on
save: `tombi format` and `panache format` are what CI compares against, and an
editor running a different one is a diff per save.

The generated READMEs are diffed by `cargo make readme-check`; editing one by hand
is undone on the next regeneration. Markdown lint is still fixed on save, because
that is the same rule CI applies.

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

`typos` is **not** here on purpose: it is a CLI check, installed by `cargo make
setup` and run by `cargo make typos` and the git hook. An editor extension would
underline a word the CLI does not flag, or the reverse, and the gate is what
decides.
