# Contributing

## Project governance

- Conduct expectations: [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md)
- Support policy: [`SUPPORT.md`](./SUPPORT.md)
- Security reporting: [`SECURITY.md`](./SECURITY.md)
- AI-assisted contributions: [`ai/CONTRIBUTING_AI.md`](./ai/CONTRIBUTING_AI.md)

There is no `CODEOWNERS` and no branch-protection baseline document, because
there is one maintainer: a `CODEOWNERS` file with a single name in it is a
review requirement that nobody can satisfy, and a policy document nobody
maintains is worse than none. Required status checks are listed under
[The gate](#required-status-checks) above, which is the part that would need
reviewing if there were ever a second maintainer.

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

Every `.rs` file carries both SPDX headers, on lines 1 and 2. `cargo make
license-headers` checks them, and it is the only check in the gate that does not
compile anything, so it is the one to run while writing rather than after.

## Development workflow

1. `cargo make setup` — install the pinned toolchain and every tool the gate
   calls, then `cargo install prek --locked && prek install` for the hooks.
2. Create a branch from `main`.
3. Make focused commits with tests.
4. `cargo make ci` — the full gate, and the same command CI runs.
5. Open a PR with rationale, risk notes, and validation output.

## The gate

The task runner is **cargo-make**, and `cargo make ci` is the single list of
what CI runs. Run `cargo make --list-all-steps` for the current task list rather
than reading one here: that is the only copy that cannot go stale.

`cargo make ci-fast` is the pre-push subset — the gate without the supply-chain
checks, which are the slowest part and only change when a dependency does. The
git hooks in `prek.toml` delegate to those same tasks, so the hook and the
pipeline cannot disagree about a cargo flag.

One cargo-make behaviour worth knowing: `CARGO_MAKE_EXTEND_WORKSPACE_MAKEFILE`
in `Makefile.toml` is load-bearing — remove it and `cargo make ci` reports
`Task "ci" not found` — and it also means each task is *executed* once per
workspace member. Eight members, so eight runs, seven of them no-ops because
Cargo has cached the build. A warm `cargo make ci` is about 16 s of work behind
about 2 min of wall clock, which reads as the gate being slow and is not.
`cargo make --no-workspace ci` is the same gate once, in 17 s.

Note that a task is still not reachable *from inside* a member directory
(`cd crates/upload && cargo make lint` reports "not found") — the env var merges
the makefile per member for the root invocation, it does not make the root
makefile visible downward. Run the tasks from the repository root.

The gate, in order:

| Task | What it checks |
| --- | --- |
| `fmt-check` | rustfmt, no writes |
| `tombi-check`, `tombi-lint` | every TOML file is formatted, and valid against its schema — for a `Cargo.toml`, Cargo's own |
| `typos` | spelling |
| `license-headers` | both AGPL SPDX headers on every `.rs` file |
| `verify-tasks` | the task list loads, every gate task names a tool `setup` installs, and `test` still runs the doctests |
| `check` | `cargo check --workspace --all-targets` |
| `lint` | clippy, all targets, all features, warnings denied |
| `test` | nextest, then `cargo test --doc` — nextest does not run doctests, so the second step is not optional |
| `doc` | rustdoc with `-D warnings`, so a broken intra-doc link fails rather than prints |
| `check-wasm`, `lint-wasm` | each wasm crate for `wasm32-unknown-unknown`, one package at a time. Not `--workspace --target wasm32`: `upload` has a host-only path |
| `readme-check` | the generated crate READMEs match their templates and module docs |
| `feature-powerset` | every feature combination |
| `machete`, `deny`, `audit` | unused dependencies, advisories, licences, sources |

`cargo make ci-slow` is what the weekly workflow runs: mutation testing and
coverage. Both are minutes rather than seconds, and mutation testing's survivors
are a to-do list rather than a verdict — 13 of `cxsmiles-yoga`'s 290 survive
today, so it is deliberately not a gate.

`crates/upload/tests/gate_consistency.rs` holds the local gate, the hooks and
the workflow to one mapping and fails the build if they disagree. Adding a check
to one of the three and forgetting the others is now a test failure.

## Required status checks

These are the checks a pull request has to pass. `ci` is the gate above;
`msrv` builds the same things on the oldest supported toolchain; `wasm` is the
only job that links a `.wasm` and reports the bundle sizes.

| Check | What it protects |
| --- | --- |
| `Gate` | everything in the table above, in one job |
| `MSRV (1.97)` | the `rust-version` claim in the root manifest |
| `WASM bundle size` | the apps' shipped size, which nothing else in the gate produces |

`crates/upload/tests/gate_consistency.rs` names every workflow job and every
task, so this list cannot silently fall out of date with the workflow.

## Local-only tasks

Run on purpose, not in the gate. Each has a stated reason in `make/*.toml`.

| Task | Why it is not a gate |
| --- | --- |
| `outdated` | a version report; it fails on any dependency at all, so as a gate it is red from the day it is added |
| `udeps` | needs nightly and is slow; it catches a dependency used only behind a `#[cfg]`, which `machete` cannot |
| `geiger` | the transitive unsafe audit — advisory, and `unsafe_code = "forbid"` already covers this repository's own code |
| `msrv` | bisects a build per toolchain; minutes |
| `minimal-versions` | a second resolution that frequently fails for reasons unrelated to this code |
| `mutants`, `mutants-mgf`, `mutants-ui`, `mutants-safe*` | minutes, and the survivors are a to-do list |
| `cov` | rebuilds the whole crate graph under instrumentation |
| `readme-write` | a gate that rewrites files is a gate that changes files |

## Dependencies

`cargo make upgrade` takes the newest compatible versions; `cargo make
upgrade-major` takes the majors, which need a changelog read and usually a code
change. Do majors **one at a time**, each in its own commit, with the build
green at each — a commit that upgrades five crates and fixes the fallout is not
reviewable and not revertable in a piece.

`cargo deny check advisories licenses` is the gate. If a new dependency is
unmaintained or carries a licence incompatible with AGPL-3.0, deny says so
before it is merged rather than after.

## Commit standards

- Keep commits atomic and reversible, one concern each: toolchain, test runner,
  task runner, dependencies, lints, CI, docs.
- Update docs when behavior, config, or interfaces change.
- Prefer explicit error handling and avoid panic-style control flow.

## Deferred until a first release is planned

None of the following is configured, referenced or stubbed, on purpose:

- `cargo semver-checks`, `cargo public-api`, or any semver gate in CI
- `release-plz`, `cargo-release`, `git-cliff`, or a generated `CHANGELOG.md`
- `cargo-dist`, release workflows producing binaries, SBOMs, provenance or
  attestation, signed tags, or image push on tags
- crates.io packaging checks (`cargo package --list`, `include`/`exclude`,
  `[package.metadata.binstall]`)

Every crate is `publish = false` and the public APIs and crate boundaries are
still moving, so none of that would be checking anything real yet. They are
picked up when a release is actually being made.

## Pull request checklist

- [ ] Tests added/updated for behavior changes
- [ ] `cargo make ci` passes locally
- [ ] New env vars and operational changes documented
- [ ] Any new dependency passes `cargo deny check licenses` and is used, not just
      declared