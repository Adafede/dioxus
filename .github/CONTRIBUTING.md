# Contributing

## Project governance

- Branch protection baseline: `.github/BRANCH_PROTECTION.md`
- Code ownership rules: `.github/CODEOWNERS`
- Conduct expectations: `CODE_OF_CONDUCT.md`
- Support policy: `SUPPORT.md`

## License agreement

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0-only)**.
By submitting a pull request you agree that your contribution is made available
under the same license, and you certify that you have the right to do so
(see [Developer Certificate of Origin](https://developercertificate.org/)).

Add a `Signed-off-by` trailer to each commit:

```bash
git commit --signoff -m "feat: your change"
```

## Development workflow

1. Create a branch from `main`.
2. Make focused commits with tests.
3. Run local quality gates before opening a PR.
4. Open a PR with rationale, risk notes, and validation output.

## Required local checks

```bash
make qa
```

Optional (recommended when touching dependencies):

```bash
make supply-chain
```

## Commit standards

- Keep commits atomic and reversible.
- Update docs when behavior, config, or interfaces change.
- Prefer explicit error handling and avoid panic-style control flow.

## Pull request checklist

- [ ] Tests added/updated for behavior changes
- [ ] `make qa` passes locally
- [ ] New env vars and operational changes documented
- [ ] API behavior changes reflected in OpenAPI docs
