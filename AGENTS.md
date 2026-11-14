# AGENTS.md

Rust workspace of six Dioxus WebAssembly applications (`apps/`), two shared
crates (`crates/ui`, `crates/upload`), and one excluded standalone CLI binary
(`apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction`).

## Commands

```bash
./mk setup                                                                                       # Install pinned toolchain and CLI gates
./mk check                                                                                       # Fast loop: fmt, tombi, typos, clippy
./mk fix                                                                                         # Auto-fix fmt, tombi, typos, clippy
./mk test-fast                                                                                   # Workspace nextest suite (skips doctests)
./mk test                                                                                        # Workspace nextest suite + doctests
./mk lint                                                                                        # Host clippy (--all-targets --all-features -D warnings)
./mk wasm                                                                                        # Check every app for wasm32-unknown-unknown
./mk local:lint-wasm                                                                             # Clippy for wasm32 targets one package at a time
./mk local:readme-check                                                                          # Verify generated README.md files match //! docs
./mk local:readme-write                                                                          # Regenerate README.md files from //! docs + panache
./mk echoes                                                                                      # Reject duplicate doctests, functions, and blocks
./mk refquiem                                                                                    # Reject stale removed-item or untouched-branch-diff mentions
./mk mutants-diff                                                                                # Mutation tests scoped to branch Rust diff
./mk ci-fast                                                                                     # Pre-push CI subset
./mk ci                                                                                          # Full CI gate
cargo test --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --locked
```

## Documentation and Configuration Index

  | Topic                                      | File                                                                       |
  | ---                                        | ---                                                                        |
  | Workspace map, invariants, and pitfalls    | [`.github/ai/AI_AGENT_GUIDE.md`](./.github/ai/AI_AGENT_GUIDE.md)           |
  | Iteration economics and verification order | [`.github/ai/AI_EFFICIENCY_GUIDE.md`](./.github/ai/AI_EFFICIENCY_GUIDE.md) |
  | Commit discipline and definition of done   | [`.github/ai/CONTRIBUTING_AI.md`](./.github/ai/CONTRIBUTING_AI.md)         |
  | Workspace architecture (human-facing)      | [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md)                           |
  | Debt ledger and ratchet floors             | [`docs/DEBT.md`](./docs/DEBT.md)                                           |
  | MCP server configuration                   | [`.mcp.json`](./.mcp.json)                                                 |

## Knowledge Graph

`graphify query "<question>"` queries `graphify-out/graph.json`; run
`graphify update .` to refresh AST nodes without API calls.
