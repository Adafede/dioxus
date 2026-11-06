# AGENTS.md

This is a Rust workspace of Dioxus WebAssembly applications and shared crates for chemistry and mass-spectrometry tooling.
See the full AI agent guide at [`.github/ai/AI_AGENT_GUIDE.md`](./.github/ai/AI_AGENT_GUIDE.md) and architecture overview at [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md).

## Quick start

```bash
./mk setup                     # Once: install pinned toolchain and all tools
./mk check                     # Fast inner loop: fmt, tombi, typos, clippy
./mk fix                       # Auto-fix formatting, TOML, typos, and clippy
./mk test-fast                 # Unit + integration tests via nextest (no doctests)
./mk test-fast -E 'test(name)' # Run one targeted test via nextest
./mk test                      # nextest + doctests (480 tests, ~4s)
./mk watch                     # Continuous feedback via bacon / cargo-watch
./mk echoes                    # Fail on duplicate doctests, functions, or context blocks
./mk refquiem                  # Fail on names the git history removed
./mk lint                      # clippy, all targets, all features, -D warnings
./mk mutants-diff              # Mutation testing on branch diff against base
./mk ci                        # Full gate (same as CI)
./mk ci-fast                   # Pre-push subset
```

## Where the instructions and debt ledger live

| Topic | File |
|-------|------|
| Workspace map, change protocol, safety rules | [`.github/ai/AI_AGENT_GUIDE.md`](./.github/ai/AI_AGENT_GUIDE.md) |
| Architecture and crate layout | [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) |
| Technical debt, lint allows, `deny.toml` skips, and ratchets | [`docs/DEBT.md`](./docs/DEBT.md) |
| Agent efficiency guide | [`.github/ai/AI_EFFICIENCY_GUIDE.md`](./.github/ai/AI_EFFICIENCY_GUIDE.md) |
| Contributing with AI | [`.github/ai/CONTRIBUTING_AI.md`](./.github/ai/CONTRIBUTING_AI.md) |
| MCP config | `.mcp/mcp.json` |

## Knowledge graph

`graphify query "<question>"` returns a scoped subgraph (usually smaller
than grep). The graph is at `graphify-out/graph.json` and is updated via
`graphify update .` (AST-only, no API cost).

## Common task commands

```bash
cargo nextest run -p <crate>                     # tests for one crate
cargo clippy -p <crate> --locked -- -D warnings  # lint one crate
```
