## Summary

<!-- One-paragraph description of what this PR does and why. -->

Closes #<!-- issue number, if applicable -->

--------------------------------------------------------------------------------

## Type of change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fixes or features that would break existing behavior)
- [ ] Refactor / internal improvement (no user-visible change)
- [ ] Documentation update
- [ ] Dependency update

--------------------------------------------------------------------------------

## Changes

<!-- Bullet-point list of what changed and why each change was made. -->

- Change 1

--------------------------------------------------------------------------------

## Testing

<!-- Describe how you verified this change. -->

- [ ] Existing tests still pass (`./mk test`)
- [ ] New tests added for new or fixed behavior
  - [ ] Manually tested with `./mk web-dev APP=<app>`

--------------------------------------------------------------------------------

## Quality gate

```bash
./mk ci        # what CI runs
./mk ci-fast   # the pre-push subset
```

Run `./mk --list` for available tasks (plus tasks in `make/*.toml`).

- [ ] `./mk ci` passes locally on a clean tree

--------------------------------------------------------------------------------

## Documentation

- [ ] `CHANGELOG.md` updated under `## Unreleased`
- [ ] Rustdoc comments and READMEs updated if behavior or public APIs changed
  (run `./mk local:readme-write` if crate/app docs changed)
- [ ] `README.md` or `.github/CONTRIBUTING.md` updated if workflows changed

--------------------------------------------------------------------------------

## License

By submitting this pull request I confirm that my contribution is made available
under the terms of the **GNU Affero General Public License v3.0
(AGPL-3.0-only)**, as specified in the `LICENSE` file, and I certify the
[Developer Certificate of Origin](https://developercertificate.org/).
