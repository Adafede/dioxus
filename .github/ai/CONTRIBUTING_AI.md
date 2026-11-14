# Contributing with AI Assistance

## Commit Discipline

- Use Conventional Commits with an imperative subject under 72 characters
  (`feat`, `fix`, `refactor`, `test`, `docs`, `ci`, `chore`, `build`), verified
  by `./mk commits`.
- Scope each commit to one logical change that builds and passes tests.
- Never add AI attribution lines, co-author trailers, or generated-by notes.
- Commit agent-facing files (`AGENTS.md`, `.github/ai/*`, `.mcp.json`) in their
  own `chore(agents): ...` commits, never mixed with application code or
  human-facing documentation.
- Keep local scratch notes or ledgers in `.agent-work/` (gitignored) and never
  stage them.

## Definition of Done

Before considering a change complete:

1. Target unit/integration tests, doctests (`./mk test`), and `wasm32` lints
   (`./mk local:lint-wasm`) pass after the final edit.
2. When `//!` crate documentation changes, run `./mk local:readme-write` and
   verify `./mk local:readme-check`.
3. When lint allows or coverage floors change, tighten only (`./mk ratchet`) and
   update [`docs/DEBT.md`](../../docs/DEBT.md) to match `mk.local` and
   `Cargo.toml`.
4. `./mk ci-fast` (or `./mk ci`) and `./mk commits` exit zero with a clean
   working tree.
