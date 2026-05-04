# Release Checklist

Use this checklist every time you publish a new release to the organization.

---

## 1. Pre-release quality gate

```bash
# Full CI-equivalent gate — must pass with zero warnings
make qa

# Supply-chain gate — advisories, licenses, bans, sources
make supply-chain
```

Both must succeed **on a clean working tree** before continuing.

---

## 2. CHANGELOG

- Move entries from `[Unreleased]` to a new versioned section, e.g. `## [0.2.0] — YYYY-MM-DD`.
- Update the comparison URLs at the bottom of `CHANGELOG.md`.

---

## 3. Version bump

Bump `version` in `[workspace.package]` inside the root `Cargo.toml`.
All crates inherit this version automatically via `workspace = true`.

```bash
# Verify consistency
grep -r '^version' apps/*/Cargo.toml crates/*/Cargo.toml
```

---

## 4. Commit, tag, push

```bash
git add -A
git commit -m "chore: release vX.Y.Z"
git tag -s vX.Y.Z -m "Release vX.Y.Z"   # signed tag (preferred)
git push origin main --follow-tags
```

*Annotated, signed tags (`-s`) are required for org-policy compliance.*

---

## 5. CI green light

Wait for all CI jobs to pass on the tagged commit:
- `native` (check + test + doc)
- `wasm` (wasm32 type-check)
- `clippy`
- `fmt`
- `supply-chain`

**Do not publish Docker images or Pages artefacts until all jobs are green.**

---

## 6. Docker image (lotus-api)

The deploy pipeline builds and pushes automatically on `main`. Verify the
image tag exists before advertising it:

```bash
docker pull codeberg.org/YOUR_ORG/lotus-api:vX.Y.Z
docker pull ghcr.io/YOUR_ORG/lotus-api:vX.Y.Z
```

---

## 7. GitHub / Codeberg release

- Create a new release on the forge, pointing at the signed tag.
- Title: `vX.Y.Z`
- Body: paste the relevant `CHANGELOG.md` section.
- Attach any binary artefacts if applicable.

---

## 8. Post-release

- Update `[Unreleased]` in `CHANGELOG.md` to start accumulating the next cycle's entries.
- Announce internally / in the project channel.
- If the release addressed security issues, make sure `SECURITY.md` advisory notes are published.

---

## Dependency update cycle

After each release, run:

```bash
cargo update
make qa
make supply-chain
```

Review and commit `Cargo.lock` with any intentional updates.
Treat each lock-file commit as a mini release validation.

