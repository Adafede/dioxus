# Contributing

## Project governance

- Conduct expectations: [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md)
- Support policy: [`SUPPORT.md`](./SUPPORT.md)
- Security reporting: [`SECURITY.md`](./SECURITY.md)

There is no `CODEOWNERS` and no branch-protection baseline document because
there is one active maintainer. Required status checks are listed under
[Required status checks](#required-status-checks).

## License agreement

This project is licensed under the **GNU Affero General Public License v3.0
(AGPL-3.0-only)**. By submitting a pull request you agree that your contribution
is made available under the same license, and you certify that you have the
right to do so (see [Developer Certificate of
Origin](https://developercertificate.org/)).

Add a `Signed-off-by` trailer to each commit:

```bash
git commit --signoff -m "feat: your change"
```

Every `.rs` file carries both SPDX headers on lines 1 and 2:

```rust
// SPDX-License-Identifier: AGPL-3.0-only
```

## Development workflow

1. `./mk setup` --- install the pinned toolchain and verification tools, then
   `cargo install prek --locked && prek install` to enable git hooks.
2. `./mk setup-extra` --- install `dx` (`dioxus-cli`) when running or bundling
   WASM apps locally.
3. Create a branch from `main`.
4. Make focused commits with tests. Keep every `.rs` file at or below 800 lines
   and ensure zero `ignore` doctest fences exist in workspace crates.
5. `./mk ci` --- run the full CI gate from the workspace root before pushing.
6. Open a PR with rationale, risk notes, and validation output.

## The gate

`./mk` is the single runner shared between local hooks (`prek.toml`) and GitHub
Actions (`.github/workflows/ci.yml`). It invokes `cargo make --no-workspace`
where needed so repository tasks run once from the workspace root rather than
repeating per member crate.

Run `./mk --list` for the available standard gates and `local:*` tasks, or
`cargo make --no-workspace --list-all-steps` for every task in `make/*.toml`.

`./mk ci-fast` is the pre-push subset (run by `prek.toml` on push) that omits
`audit` and `mutants-diff`.

`./mk ci` runs the standard gate sequence plus `LOCAL_CI_GATES` from `mk.local`:

  | Check                                                                                  | What it verifies                                                                                                                                                                |
  | ---                                                                                    | ---                                                                                                                                                                             |
  | `./mk tools`, `./mk actions`, `./mk shell`                                             | pinned tool versions match `mk.versions`, workflow security (`zizmor`), shell scripts (`shellcheck`)                                                                            |
  | `commits`, `ratchet`                                                                   | Conventional Commits subjects and non-decreasing coverage/debt ratchets (`mk.local`, `Cargo.toml`)                                                                              |
  | `fmt`, `tombi`, `typos`                                                                | `rustfmt`, TOML formatting & schema (`tombi`), and spelling (`.typos.toml`)                                                                                                     |
  | `lint`, `lints-inherit`, `wasm`, `local:lint-wasm`                                     | host `--all-targets --all-features` clippy (`-D warnings`), `[lints] workspace = true`, and per-app `wasm32-unknown-unknown` check & clippy                                     |
  | `test`                                                                                 | `cargo nextest run --workspace --locked` (host unit & integration tests, including `crates/upload/tests/gate_consistency/`) followed by `cargo test --workspace --doc --locked` |
  | `doc`                                                                                  | rustdoc across all workspace crates with `-D warnings`                                                                                                                          |
  | `deny`, `audit`, `machete`, `msrv`, `echoes`, `refquiem`, `semver`                     | licenses/bans (`deny.toml`), advisories, unused deps, declared MSRV check, duplicate code/doctest detection, removed-name check, and semver check                               |
  | `mutants-diff`                                                                         | mutation testing (`cargo-mutants`) on `.rs` diffs against the base branch                                                                                                       |
  | `local:license-headers`, `local:check`, `local:readme-check`, `local:feature-powerset` | SPDX headers, JSON validation + unsafe audit, generated README sync (`./mk local:readme-write`), and feature powerset check (`cargo-hack`)                                      |

`./mk ci-nightly` runs the slower scheduled suite (`mutants`, `coverage`,
`powerset`, `miri`, `outdated`). `.cargo/mutants.toml` scopes heavy UI/SVG/WASM
glue exclusions and `mk.local` sets `MUTANTS_PACKAGES` (`json-count-rs`,
`cxsmiles-yoga`).

`crates/upload/tests/gate_consistency/` checks structural invariants across the
repository (`apps`, `.gitignore` data exclusions, `mk`, `mutants` scope,
`make/*.toml`, workflows, documentation surface separation, resolved relative
links, and `docs/DEBT.md` ratchets).

## Required status checks

`.github/workflows/ci.yml` defines two blocking CI jobs:

  | Job                | Command                                | What it protects                                                                          |
  | ---                | ---                                    | ---                                                                                       |
  | `./mk ci`          | `./mk ci`                              | Full lint, test, doctest, WASM check/lint, hygiene, MSRV, and supply-chain gate           |
  | `WASM bundle size` | `./mk web-build-all` + `./mk web-size` | Links all six production WASM bundles (`--profile wasm-release`) and reports bundle sizes |

## Standalone CLI crate

`apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction` is intentionally
excluded from `[workspace]` in `Cargo.toml` because it pulls native
bioinformatics and HTTP/archive crates incompatible with `wasm32`. Run checks
there with `--locked`:

```bash
cargo clippy --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --locked
```

## Local-only tasks

Run manually when needed (not part of every PR gate unless listed above):

- `./mk local:readme-write`: regenerates the seven `README.md` files from
  `README.tpl` + `//!` docs and formats them with `panache`.
- `./mk local:mutants-ui` / `./mk local:mutants-mgf`: opt-in mutation sweeps
  outside `MUTANTS_PACKAGES`.
- `./mk outdated` / `./mk udeps`: advisory dependency freshness / nightly
  unused-dependency checks.
- `./mk unsafe` / `make/scripts/unsafe-audit.sh`: advisory transitive `unsafe`
  audit (`unsafe_code = "forbid"` applies to all workspace code).

## Dependencies

Upgrade compatible versions with `./mk upgrade` or inspect majors one at a time
with `./mk upgrade-major`. Keep commits atomic so every upgrade can be built and
tested independently.

`cargo deny check` verifies license compatibility (`AGPL-3.0-only` workspace)
and duplicate bans (`deny.toml`). Any temporary `[bans] skip` entry must be
recorded in [`../docs/DEBT.md`](../docs/DEBT.md).

## Commit standards

- Follow Conventional Commits (checked by `./mk commits`): `feat`, `fix`,
  `refactor`, `test`, `docs`, `ci`, `chore`, `build`, `style`, `revert`, with
  imperative subjects under 72 characters.
- Keep commits atomic and keep tests and CI checks passing on every commit.

## Deferred until a first release is planned

`.github/workflows/release.yml` runs `./mk ci-release` on `v*` tags, which skips
`cargo publish --workspace --dry-run` while every crate sets `publish = false`.
None of the following is configured yet:

- crates.io publishing (`publish = true`) and packaging metadata checks
- `release-plz`, `cargo-release`, `git-cliff`, or automated changelog tooling
- `cargo-dist`, release binary uploads, SBOMs, provenance attestations, or
  container images

## Pull request checklist

- [ ] Tests added or updated for behavior changes
- [ ] `./mk ci` and `./mk local:lint-wasm` pass locally
- [ ] Standalone `smarts-evoliposuction` `--locked` tests/clippy run if touched
- [ ] Any `//!` / `README.tpl` changes regenerated with
  `./mk local:readme-write`
- [ ] Any new dependency passes `cargo deny check` and is documented in
  `docs/DEBT.md` if skipped
