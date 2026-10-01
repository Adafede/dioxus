# Release Checklist

> **Not in use yet.** Every crate is `publish = false` and the public APIs are
> still moving, so there is no release to cut. This file records what the
> process will be when one is actually planned, and the release tooling it
> assumes — a semver gate, a changelog generator, binaries, SBOMs, provenance,
> signed tags, image push — is deliberately **not** configured, referenced or
> stubbed anywhere in this repository until then. See "Deferred" in
> [`CONTRIBUTING.md`](./CONTRIBUTING.md).
>
> When that day comes, the first step is to choose the tooling and write it
> down here; the steps below are the shape of the process, not the tools.

Use this checklist every time you publish a new release to the organization.

--------------------------------------------------------------------------------

## 1. Pre-release quality gate

```bash
# The full CI gate — the same command .github/workflows/ci.yml runs
./mk ci
```

Must succeed **on a clean working tree** before continuing. It includes the
supply-chain checks (advisories, licences, bans, sources), so there is no second
command to remember.

--------------------------------------------------------------------------------

## 2. CHANGELOG

- Move entries from `[Unreleased]` to a new versioned section, e.g.
  `## [0.2.0] --- YYYY-MM-DD`.
- Update the comparison URLs at the bottom of `CHANGELOG.md`.

--------------------------------------------------------------------------------

## 3. Version bump

Bump `version` in `[workspace.package]` inside the root `Cargo.toml`. All crates
inherit this version automatically via `workspace = true`.

```bash
# Verify consistency
grep -r '^version' apps/*/Cargo.toml crates/*/Cargo.toml
```

--------------------------------------------------------------------------------

## 4. Commit, tag, push

```bash
git add -A
git commit -m "chore: release vX.Y.Z"
git tag -s vX.Y.Z -m "Release vX.Y.Z"   # signed tag (preferred)
git push origin main --follow-tags
```

*Annotated, signed tags (`-s`) are required for org-policy compliance.*

--------------------------------------------------------------------------------

## 5. CI green light

Wait for every check in [`CONTRIBUTING.md`](./CONTRIBUTING.md) to pass on the
tagged commit: `Gate`, `MSRV (1.97)` and `WASM bundle size`.

**Do not let `deploy.yml` publish anything until the gate is green.** It is a
separate workflow and does not depend on `ci`, so that a deploy is not blocked by
a slow MSRV bisect — which also means the ordering is on purpose and must stay
understood.

--------------------------------------------------------------------------------

## 6. Container images

There is no container image for this repository and no registry behind one: the
deployable artefacts are static wasm bundles, published by `deploy.yml` to GitHub
Pages. Nothing to pull.

--------------------------------------------------------------------------------

## 7. GitHub / Codeberg release

- Create a new release on the forge, pointing at the signed tag.
- Title: `vX.Y.Z`
- Body: paste the relevant `CHANGELOG.md` section.
- Attach any binary artifacts if applicable.

--------------------------------------------------------------------------------

## 8. Post-release

- Update `[Unreleased]` in `CHANGELOG.md` to start accumulating the next cycle's
  entries.
- Announce internally / in the project channel.
- If the release addressed security issues, make sure `SECURITY.md` advisory
  notes are published.

--------------------------------------------------------------------------------

## Dependency update cycle

After each release, run:

```bash
cargo update
./mk ci
```

Review and commit `Cargo.lock` with any intentional updates. Treat each
lock-file commit as a mini release validation.
