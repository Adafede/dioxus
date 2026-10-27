# AGENTS.md

This is a Rust workspace of Dioxus WASM web apps for chemistry tooling.
See the full AI agent guide at `.github/ai/AI_AGENT_GUIDE.md`.

## Quick start

```bash
./mk setup     # once: install pinned toolchain and all tools
./mk check     # type-check
./mk dejadoc   # fail on duplicate doctests, functions or context blocks
./mk refquiem  # fail on names the git history removed
./mk lint      # clippy, all targets, all features, -D warnings
./mk test      # nextest + doctests (480 tests, ~4s)
./mk ci        # the full gate (same as CI)
./mk ci-fast   # the pre-push subset
```

## Where the real instructions live

| Topic | File |
|-------|------|
| Workspace map, change protocol, safety rules | `.github/ai/AI_AGENT_GUIDE.md` |
| Agent efficiency guide | `.github/ai/AI_EFFICIENCY_GUIDE.md` |
| Contributing with AI | `.github/ai/CONTRIBUTING_AI.md` |
| MCP config | `.mcp/mcp.json` |

## Knowledge graph

`graphify query "<question>"` returns a scoped subgraph (usually smaller
than grep). The graph is at `graphify-out/graph.json` and is updated via
`graphify update .` (AST-only, no API cost).

## Common task commands

```bash
cargo nextest run -p <crate>                 # tests for one crate
cargo clippy -p <crate> --locked -- -D warnings  # lint one crate
```
