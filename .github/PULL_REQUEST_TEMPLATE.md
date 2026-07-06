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

- [ ] Existing tests still pass
      (`cargo test --workspace --all-targets --locked`)
- [ ] New tests added for new or fixed behavior
- [ ] Manually tested with `dx serve --package <app>` / `cargo run -p lotus-api`

--------------------------------------------------------------------------------

## Quality gate

```bash
prek run cargo-qa
prek run cargo-supply-chain
```

- [ ] `prek run cargo-qa` passes locally on a clean tree
- [ ] `prek run cargo-supply-chain` passes (or changes are dependency-free)

--------------------------------------------------------------------------------

## Documentation

- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] API docs updated (OpenAPI annotations, doc comments) if applicable
- [ ] `README.md` or `CONTRIBUTING.md` updated if behavior changed

--------------------------------------------------------------------------------

## License

By submitting this pull request I confirm that my contribution is made available
under the terms of the **GNU Affero General Public License v3.0
(AGPL-3.0-only)**, as specified in the `LICENSE` file, and I certify the
[Developer Certificate of Origin](https://developercertificate.org/).
