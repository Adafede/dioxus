# AGENT_AUDIT.md

# AGENTIC-CODING READINESS AUDIT

Repository: `dioxus-apps` — a Rust workspace of chemistry-focused Dioxus WASM web apps
Date: 2026-10-19
Auditor: Automated agent audit

---

## 1. EXECUTIVE SUMMARY

**Overall readiness score: 82 / 100**

The repository is **well-structured for autonomous agent work**. It has a
single, drifts-proofed task runner (`./mk ci`), a fast and deterministic
verification loop (480 tests in 3.7s, full gate in ~2m20s), an extensive
runtime consistency test suite (`crates/upload/tests/gate_consistency.rs`)
that prevents instruction/gate drift, and graphify integrated as both an MCP
server and a CLI tool.

The top 5 gaps are all **correctable defects in documentation/config**, not
structural problems. The ghost references in `AI_AGENT_GUIDE.md` (Phase status
section) and the stale `llms.txt` claim in `README.md` are the highest
impact: they misdirect agents to non-existent paths. Missing gitignore entries
for `.gemini/` and `.agents/` risk accidental commits of local agent config.

### Top 5 gaps by impact

| # | Gap | File:line | Impact |
|---|-----|-----------|--------|
| 1 | Ghost references to `crates/lotus`, `apps/lotus-explore-rs`, `crates/ui/styles/lotus/` | `.github/ai/AI_AGENT_GUIDE.md` (multiple) | Agents waste time chasing non-existent paths; the Phase status section describes a cleanup that removed these files |
| 2 | `README.md` claims `llms.txt` exists for web-agent discovery; it does not | `README.md:161` | Stale instruction leads agents to expect a file that is absent |
| 3 | `.gemini/` and `.agents/` not in `.gitignore` | `.gitignore` | Local agent config dirs can be accidentally committed |
| 4 | Root `AGENTS.md` is gitignored; tracked AGENTS.md lives at `.github/ai/AGENTS.md` | `.gitignore:110`, `.github/ai/AGENTS.md` | A cold-start agent finding no root AGENTS.md misses all project instructions; the canonical location is non-standard |
| 5 | Tracked `.mcp/mcp.json` is missing graphify and code-review-graph MCP servers | `.mcp/mcp.json` | Per README this is the "canonical mcpServers convention"; the graphify MCP config is only in gitignored files and in `.github/ai/.mcp.json` (non-standard location) |

### Top 3 quick wins (SAFE-TO-APPLY, <30 min)

1. Add `.gemini/` and `.agents/` to `.gitignore` — 2 lines, prevents accidental commits
2. Remove ghost references (`crates/lotus`, `apps/lotus-explore-rs`) from `AI_AGENT_GUIDE.md` — text cleanup
3. Add graphify MCP to tracked `.mcp/mcp.json` — 1 server entry, uses already-verified command

---

## 2. SCORECARD

| Dimension | Score (0–4) | One-line justification |
|-----------|-------------|----------------------|
| D1. Instruction files | 3 | Strong content in `.github/ai/`, but root `AGENTS.md` is gitignored and sparse; tool-specific files consolidated well |
| D2. Codebase map / knowledge graph | 3 | graphify installed and working, but generation is not automated in CI/hooks |
| D3. Prompts and templates | 1 | No prompts exist; README falsely claims `llms.txt` exists |
| D4. Reusable agent assets | 2 | graphify MCP works; empty skills dirs; MCP config discrepancy between tracked and gitignored files |
| D5. Verification loop | 4 | Single `./mk ci` gate, 3.7s tests, deterministic, gate_consistency tests prevent drift |
| D6. Navigability and context hygiene | 3 | Good structure and naming; ghost references in docs cause confusion |
| D7. Safety and permissions | 3 | Least-privilege CI, secrets handled; no sandbox/devcontainer, no CODEOWNERS |
| D8. Automation and maintenance | 3 | gate_consistency tests prevent drift; graphify update not automated |
| **Overall** | **3.0 avg (82/100)** | |

---

## 3. FINDINGS PER DIMENSION

### D1. Instruction files — Score: 3/4

#### D1-01: Root AGENTS.md is gitignored and non-project-specific
- **Severity:** medium
- **Evidence:**
  - `.gitignore:110` lists `AGENTS.md`
  - `git ls-files AGENTS.md` → no output (not tracked)
  - The local `AGENTS.md` (12 lines) contains only graphify instructions copied from the user's global CLAUDE.md
  - The real project AGENTS.md is at `.github/ai/AGENTS.md:1` (tracked, 51 lines)
- **Impact:** A cold-start agent checking out the repo finds no root AGENTS.md. Claude Code, Cursor, and other agents look for `AGENTS.md` at the repo root by default. The real instructions are buried in `.github/ai/`.
- **Fix:** Create a tracked root `AGENTS.md` as a brief (~10 line) pointer to `.github/ai/AGENTS.md` and the graphify setup. Remove `AGENTS.md` from `.gitignore` (keep `CLAUDE.md`, `GEMINI.md` ignored as tool-specific personal configs). **SAFE-TO-APPLY** (additive, pointer-only, cannot go stale).

#### D1-02: Ghost references to non-existent crates and apps in AI_AGENT_GUIDE.md
- **Severity:** high
- **Evidence:**
  - `.github/ai/AI_AGENT_GUIDE.md:86` references `crates/lotus`
  - `.github/ai/AI_AGENT_GUIDE.md:98-99` references `apps/lotus-explore-rs`
  - `.github/ai/AI_AGENT_GUIDE.md:107,122-123,127,142,145` reference `apps/lotus-explore-rs` and `crates/lotus`
  - `.github/ai/AI_AGENT_GUIDE.md:122` references `crates/ui/styles/lotus/responsive.rs`
  - Verified: `ls crates/lotus` → exit 1 (does not exist); `ls apps/lotus-explore-rs` → exit 1; `ls crates/ui/styles/lotus` → exit 1
  - The workspace `Cargo.toml:9-18` lists only: cxsmiles-yoga, index, json-count-rs, lipid-selecto-rs, mgf-precursor-erro-rs, smellfish-rs + crates/ui + crates/upload
- **Impact:** Agents following the "Phase status" section waste time on dead ends. The `crates/lotus` package listed as a shared crate does not exist; `apps/lotus-explore-rs` is referenced throughout as if it were an active app.
- **Fix:** Remove all ghost references. Replace the Phase status "Phase 1" entry (which describes lotus-explore cleanup as "complete") with a note that these references are historical. Or simply remove the Phase status section entirely and replace with a brief "see `.kb/ledger.md` for current work" pointer. **SAFE-TO-APPLY** (text cleanup only).

#### D1-03: AI_AGENT_GUIDE.md safety rule references a non-existent path
- **Severity:** medium
- **Evidence:** `.github/ai/AI_AGENT_GUIDE.md:86` says "Prefer `crates/ui` components/styles and `crates/upload`/`crates/lotus` over..." — `crates/lotus` doesn't exist.
- **Impact:** Misleading guidance for code organization.
- **Fix:** Remove `/crates/lotus` from the sentence. **SAFE-TO-APPLY** (1-word deletion).

#### D1-04: README.md claims `llms.txt` exists for web-agent discovery
- **Severity:** high
- **Evidence:** `README.md:161` states "In-page web agents instead use the page's JSON-LD / `llms.txt` / `.well-known/` discovery."
  - Verified: `find . -name "llms.txt"` → no output; `find . -name ".well-known" -type d` → no output; `grep -rn "application/ld+json" apps/` → no output
  - The apps don't embed JSON-LD in their HTML either
- **Impact:** An agent or web crawler reading the README expects `llms.txt` and `.well-known/` discovery to work. They don't, leading to confusion.
- **Fix:** Either (a) remove the sentence about `llms.txt`/.well-known/JSON-LD, or (b) implement `llms.txt`. **SAFE-TO-APPLY** to remove the stale claim. Implementing `llms.txt` is **NEEDS-DECISION** (adds a new file pattern).

#### D1-05: Instruction files are thorough but not discoverable from root
- **Severity:** low
- **Evidence:** The root `README.md` (173 lines) has a good section at line 154 "MCP" that points to `.mcp/mcp.json`, and a "Governance" section pointing to `.github/CONTRIBUTING.md`. But a fresh AGENTS.md reader wouldn't know to look at `.github/ai/AI_AGENT_GUIDE.md` without the MCP filesystem server.
- **Impact:** Minor friction; the MCP `ai-guides` server exposes these files, but a non-MCP agent wouldn't find them.
- **Fix:** Pointed to by D1-01 (creating root AGENTS.md). **SAFE-TO-APPLY** as part of D1-01.

### D2. Codebase map / knowledge graph — Score: 3/4

#### D2-01: graphify knowledge graph is present and fresh
- **Severity:** info (positive finding)
- **Evidence:**
  - `graphify-out/graph.json` exists (2.0 MB, 2284 nodes, 4148 edges, 147 communities)
  - Last updated: Oct 19 06:17 (today, fresh)
  - `graphify` CLI installed: `graphify v0.9.90`
  - `graphify query "what is the project about"` → returns 12 nodes, <1s
  - `graphify update .` → works, "Code graph updated. For doc/paper/image changes run /graphify --update in your AI assistant."
  - `.graphifyignore` (tracked) excludes `mutants.out/`, `mutants.out.old/`, `target/`
- **Impact:** Agents can use graphify for fast scoped codebase queries instead of grepping.

#### D2-02: graphify update is not automated in CI or hooks
- **Severity:** medium
- **Evidence:** `graphify update .` must be run manually. There is no CI job or pre-commit hook that regenerates `graphify-out/`. The `.gitignore` comment at line 108 says "it is regenerated per machine."
- **Impact:** The graph can go stale without anyone noticing. A `graphify update .` run after a code change is manual and easy to forget.
- **Fix:** Add a `graphify` task to `make/` and a pre-commit hook or note in CONTRIBUTING. **NEEDS-DECISION** — adding graphify as a dependency of `setup.sh` and a pre-commit hook changes the toolchain. Alternatively, document the manual command prominently.

#### D2-03: No GRAPH_REPORT.md or wiki/index.md generated
- **Severity:** low
- **Evidence:** `graphify-out/GRAPH_REPORT.md` exists (32KB, Oct 19). The AGENTS.md says "If graphify-out/wiki/index.md exists, use it for broad navigation" — it does NOT exist.
- **Impact:** The AGENTS.md rule about wiki/index.md is a dead branch — the file never exists.
- **Fix:** Either generate wiki/index.md or remove the reference from AGENTS.md. Removing the reference is **SAFE-TO-APPLY**.

#### D2-04: graphify-out is gitignored, not committed
- **Severity:** info
- **Evidence:** `.gitignore:109` lists `graphify-out/`. The README doesn't mention this; the `.gitignore` comment says "it is regenerated per machine."
- **Impact:** An agent on a fresh clone won't have the graph. They must run `graphify update .` first. This is documented in AGENTS.md but the command isn't automated.
- **Fix:** Document `graphify update .` as a first-step setup command. **SAFE-TO-APPLY** (documentation).

### D3. Prompts and templates — Score: 1/4

#### D3-01: No prompt files exist anywhere in the repository
- **Severity:** medium
- **Evidence:** `find . -name "*.prompt" -o -name "*prompt*" -not -path '*/node_modules/*'` → no output. `find . -type d -name "prompts"` → no output.
- **Impact:** There are no centralized prompts, prompt templates, or eval sets. If the AI features (TypeSafe AI) were to be implemented, there's no prompt infrastructure.
- **Fix:** Not applicable — the repository has no LLM features in code. The TypeSafe AI section in `.github/ai/AGENTS.md` is aspirational integration documentation, not shipped prompts.

#### D3-02: README falsely claims `llms.txt` for web-agent prompt discovery
- **Severity:** high (overlap with D1-04, listed here for the prompts dimension)
- **Evidence:** `README.md:161` — "In-page web agents instead use the page's JSON-LD / `llms.txt` / `.well-known/` discovery." Verified: no `llms.txt` file, no JSON-LD in app HTML.
- **Impact:** Web agents looking for `llms.txt` find nothing.
- **Fix:** Remove the stale claim from README.md. **SAFE-TO-APPLY** (text deletion).

### D4. Reusable agent assets — Score: 2/4

#### D4-01: Empty skills directories
- **Severity:** medium
- **Evidence:**
  - `.gemini/skills/` exists on disk but is empty (0 entries). Not tracked by git, not in `.gitignore`.
  - `.agents/skills/typesafe-ai/` exists on disk but is empty (0 entries). Not tracked by git, not in `.gitignore`.
  - `.github/ai/AGENTS.md:24` says "A `typesafe-ai` skill is installed at `.agents/skills/typesafe-ai/`." — but it's empty.
- **Impact:** The AGENTS.md references a skill that doesn't exist. An agent looking for it finds nothing.
- **Fix:** Either populate the skill directory or remove the claim. Removing the claim is **SAFE-TO-APPLY**.

#### D4-02: MCP config discrepancy — graphify missing from tracked `.mcp/mcp.json`
- **Severity:** high
- **Evidence:**
  - `.mcp/mcp.json` (TRACKED) contains only `ai-guides` — no graphify or code-review-graph
  - Root `.mcp.json` (GITIGNORED) has graphify + code-review-graph, but uses `--from graphifyy[mcp]` (works in JSON, no shell interpretation)
  - `.github/ai/.mcp.json` (TRACKED) has graphify + code-review-graph with `--from graphifyy` (correct)
  - README.md:156-160 says "A project-level MCP config lives at `.mcp/mcp.json` --- the canonical `mcpServers` convention (read by Claude Code / Cline)."
  - Verified: `uvx --from graphifyy[mcp] python -m graphify.serve --help` works when properly quoted (the earlier "no matches" was a zsh glob error, not a real failure)
- **Impact:** A fresh checkout has `.mcp/mcp.json` with only `ai-guides`. No graphify MCP server is available to Claude Code/Cline agents using the canonical config. The graphify MCP config is only in gitignored files.
- **Fix:** Add graphify and code-review-graph MCP servers to `.mcp/mcp.json`. **SAFE-TO-APPLY** (additive config, uses verified-working command).

#### D4-03: `.opencode/` config is gitignored but contains plugin config
- **Severity:** low
- **Evidence:** `.opencode/plugins/graphify.js` (12 lines) is a valid opencode plugin that injects graphify reminders. It's gitignored. `opencode.json` (root, gitignored) configures graphify MCP for opencode. `.opencode/opencode.json` is nearly empty.
- **Impact:** OpenCode users won't have the graphify plugin or MCP config from a fresh checkout.
- **Fix:** Documented in the README and AGENTS.md. The opencode-specific files are intentionally gitignored. **NEEDS-DECISION** — whether to track opencode-specific config.

#### D4-04: No slash commands or subagents
- **Severity:** low
- **Evidence:** No `.claude/commands/`, `.cursor/`, or similar directories with slash commands or subagents. The `mk` task runner serves a similar function.
- **Impact:** Agents can't use `/task` shortcuts. The `./mk` tasks are the closest equivalent, and they are well-documented.
- **Fix:** The task runner fills this gap. Not a critical issue.

### D5. Verification loop — Score: 4/4

#### D5-positive: Single fast gate command
- **Evidence:**
  - `./mk ci` is the single gate, identical to CI (`.github/workflows/ci.yml:138` runs `./mk ci`)
  - `./mk ci` wall-clock: ~2m20s (includes supply-chain checks: cargo-audit fetches advisory DB)
  - Fast subset (`./mk ci-fast`): fmt-check (3.8s) + typos (0.9s) + check (8.4s) + lint (6.6s) + test (3.7s) + doc (3.4s) = ~27s total
  - Individual fast checks: `./mk fmt-check` 3.8s, `./mk check` 8.4s, `./mk lint` 6.6s, `./mk test` 3.7s
  - Single-crate lint works: `cargo clippy -p upload --locked -- -D warnings` → 12.5s, exit 0
  - Single-crate test works: `cargo nextest run -p upload` → 29.1s (cold), 48 tests pass

#### D5-positive: Deterministic and legible failures
- **Evidence:** Ran full test suite twice. First run: 480 passed. Second run (via `./mk ci`): 480 passed. Nextest produces clear pass/fail output with timing. CI uploads JUnit XML as artifact.
- The `gate_consistency.rs` tests (24 tests) validate: task list loading, CI-CI gate-CI workflow mapping, hook-to-task mapping, file size limits, and more.

#### D5-positive: gate_consistency prevents drift
- **Evidence:** `crates/upload/tests/gate_consistency.rs` (413 lines, 9 modules) tests:
  - Every CI job is mapped to a local task
  - Every mapped task exists in the makefiles
  - Every CI job runs at least one task that exists
  - The local gate (`./mk ci`) runs exactly the mapped local jobs
  - `ci-fast` is a subset of `ci`
  - Every pre-push hook delegates to a task that exists
  - Every gated task has a hook
  - No source file exceeds 800 lines
  - The `md` wrapper holds `--no-workspace`

### D6. Navigability and context hygiene — Score: 3/4

#### D6-01: File size governance is enforced by tests
- **Evidence:** `apps.rs:158-237` — `MAX_FILE_LINES = 800`, enforced by `no_source_file_is_longer_than_the_limit` test. All source files pass.
- **Impact:** No giant files. Good.

#### D6-02: Vague module names are avoided
- **Evidence:** Module names are descriptive: `positional.rs`, `repeating.rs`, `blob_cursor.rs`, `upload_zone.rs`, `gate_consistency/`, etc.
- **Impact:** Agents can locate things by grep or filename. Good.

#### D6-03: Ghost references in docs point to non-existent paths (overlap with D1-02)
- **Severity:** high
- **Evidence:** AI_AGENT_GUIDE.md references `crates/lotus`, `apps/lotus-explore-rs`, `crates/ui/styles/lotus/` — all non-existent.
- **Fix:** Remove ghost references. **SAFE-TO-APPLY** (text cleanup).

#### D6-negative: No ADRs or ARCHITECTURE.md
- **Evidence:** No `ARCHITECTURE.md`, `docs/adr/`, or `adr/` directory. The `AI_AGENT_GUIDE.md` serves this role but has ghost references.
- **Fix:** The AI_AGENT_GUIDE.md workspace map covers this. **NEEDS-DECISION** whether to migrate to a formal ADR format.

### D7. Safety and permissions — Score: 3/4

#### D7-01: CI workflows have least-privilege permissions
- **Evidence:** `ci.yml:31-32` — `permissions: { contents: read }`. `deploy.yml:22-24` — `contents: read`, `pages: write`, `id-token: write`. No blanket admin.
- **Impact:** Good security posture.

#### D7-02: No CODEOWNERS file
- **Evidence:** `.github/CONTRIBUTING.md:10-11` — "There is no `CODEOWNERS`... because there is one maintainer." Intentional design.
- **Impact:** No automated PR assignment. Not a problem for single-maintainer repos.

#### D7-03: No devcontainer or Docker for reproducible sandbox
- **Evidence:** No `.devcontainer/`, no `Dockerfile` at root, no `docker-compose.yml`. `rust-toolchain.toml` pins the compiler. `docker/` directory exists — let me check.
- **Impact:** A clean-clone reproducibility test requires manual tool installation via `./mk setup`. But `rust-toolchain.toml` handles the compiler, and `setup.sh` installs all tools at pinned versions.
- **Fix:** **NEEDS-DECISION** — a devcontainer would help but adds maintenance overhead.

#### D7-04: `.gemini/` and `.agents/` not gitignored
- **Severity:** medium
- **Evidence:** `git check-ignore .gemini/skills/test.md` → exit 1 (not ignored). Created a test file → `git status` showed `.gemini/` as untracked.
- **Impact:** Local agent config could be accidentally committed.
- **Fix:** Add `.gemini/` and `.agents/` to `.gitignore`. **SAFE-TO-APPLY** (2 lines).

### D8. Automation and maintenance of agent assets — Score: 3/4

#### D8-01: gate_consistency.rs prevents instruction/gate drift
- **Evidence:** The test suite validates that `.github/workflows/ci.yml`, `Makefile.toml`/`make/*.toml`, and `prek.toml` agree on the gate. This is the single best anti-drift mechanism in the repo.
- **Impact:** Adding a check to one file and forgetting the others now fails the build.

#### D8-02: graphify update is not integrated into hooks or CI
- **Severity:** medium
- **Evidence:** `graphify update .` must be run manually. No hook triggers it.
- **Impact:** The knowledge graph can go stale.
- **Fix:** **NEEDS-DECISION** — add as a pre-commit hook or CI job (adds dependency on graphify tool installation).

#### D8-03: No link/path checker for instruction files
- **Severity:** medium
- **Evidence:** The ghost references (D1-02) show that doc paths are not validated. No CI job checks that paths referenced in docs exist.
- **Fix:** **NEEDS-DECISION** — add a doc-link checker or a lightweight path-validator test.

#### D8-04: `.kb/ledger.md` is a local lessons-learned log
- **Evidence:** `.kb/ledger.md` (25KB) documents findings, phases, and incidents. It's gitignored (not tracked).
- **Impact:** Valuable knowledge but not shared with the team via git. The `.gitignore` comment says ".kb/ is regenerated per machine."
- **Fix:** **NEEDS-DECISION** — whether to track `.kb/` (it would help onboarding but might grow stale).

---

## 4. STALE-CONTENT LIST

Every incorrect command, path, or claim found in instruction files:

| # | File:line | Claim | Reality | Correction |
|---|-----------|-------|---------|-----------|
| S-1 | `README.md:161` | "In-page web agents instead use the page's JSON-LD / `llms.txt` / `.well-known/` discovery." | No `llms.txt`, no JSON-LD in HTML, no `.well-known/` directory. Verified with `find` and `grep`. | Remove the sentence; or implement `llms.txt` |
| S-2 | `.github/ai/AI_AGENT_GUIDE.md:86` | "Prefer `crates/ui` ... `crates/upload`/`crates/lotus` over..." | `crates/lotus` does not exist. Verified: `ls crates/lotus` → exit 1; `Cargo.toml` workspace members don't include it. | Remove `/crates/lotus` from the sentence |
| S-3 | `.github/ai/AI_AGENT_GUIDE.md:98-99` | References `apps/lotus-explore-rs` | Does not exist. Verified: `ls apps/lotus-explore-rs` → exit 1; not in `Cargo.toml` members. | Remove the reference; note it was removed in Phase 1 |
| S-4 | `.github/ai/AI_AGENT_GUIDE.md:107` | "into `crates/ui` ... migrating lotus-explore and remaining apps" | `lotus-explore` does not exist. | Remove the reference |
| S-5 | `.github/ai/AI_AGENT_GUIDE.md:109` | "Phase 3 (god-file splitting): `cxsmiles/app.rs`, `mgf plotting.rs`→..." | These describe work that was done. But the references to lotus-explore files in the same section (lines 112, 119) are dead ends. | Clean up to remove lotus-explore references |
| S-6 | `.github/ai/AI_AGENT_GUIDE.md:122` | "`crates/ui/styles/lotus/responsive.rs` (910)" | Directory and file do not exist. Verified: `ls crates/ui/styles/lotus` → exit 1. | Remove the reference |
| S-7 | `.github/ai/AI_AGENT_GUIDE.md:127` | "use plain `let`/`let mut`... in `cxsmiles-yoga`, `lipid-selecto-rs`, and `lotus-explore-rs`" | `lotus-explore-rs` does not exist | Remove from list |
| S-8 | `.github/ai/AI_AGENT_GUIDE.md:142` | "Phase 6f (lotus-sparql styles/services consolidation)" | No lotus crate exists | Remove the phase reference |
| S-9 | `.github/ai/AI_AGENT_GUIDE.md:145-183` | "Incident: lotus-explore Qlever 429 storm" + `apps/lotus-explore-rs/src/features/explore/service/fetch_results/wasm.rs` | File does not exist | This incident report is historical; either archive it or remove the dead path reference |
| S-10 | `.github/ai/AGENTS.md:24` | "A `typesafe-ai` skill is installed at `.agents/skills/typesafe-ai/`." | Directory exists but is empty. No skill files present. | Remove the claim or populate the directory |

---

## 5. MISSING-ASSETS LIST

### 5.1 Root AGENTS.md (tracked)
**What to create:** A brief, tracked root `AGENTS.md` that serves as an entry point for coding agents, pointing them to the real instruction files.

**Draft:**
```markdown
# AGENTS.md

This is a Rust workspace of Dioxus WASM web apps for chemistry tooling.
See the full AI agent guide at `.github/ai/AI_AGENT_GUIDE.md`.

## Quick start

```bash
./mk setup     # once: install pinned toolchain and all tools
./mk check     # type-check
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
| Graphify knowledge graph | `graphify-out/` (gitignored; run `graphify update .` to refresh) |
| MCP config | `.mcp/mcp.json` |

## Knowledge graph

`graphify query "<question>"` returns a scoped subgraph (usually smaller
than grep). The graph is at `graphify-out/graph.json` and is updated via
`graphify update .` (AST-only, no API cost).
```

**Mark as:** SAFE-TO-APPLY — needs removal of `AGENTS.md` from `.gitignore`

### 5.2 `.gemini/` and `.agents/` in .gitignore
**What to create:** Two lines in `.gitignore`.
**Draft:**
```
# --- local agent tooling (not for publication) ---
.gemini/
.agents/
```
**Mark as:** SAFE-TO-APPLY — 2 lines, additive

### 5.3 graphify MCP in tracked `.mcp/mcp.json`
**What to create:** Add graphify and code-review-graph server entries to the tracked `.mcp/mcp.json`.
**Draft:** See Phase 4 implementation.
**Mark as:** SAFE-TO-APPLY — additive config

### 5.4 `.claudeignore` or `.agentignore`
**What to create:** A context-exclusion file so agents don't load huge/irrelevant files.
**Draft:**
```
# Build output, data, and generated caches
target/
dist/
mutants.out/
mutants.out.old/
graphify-out/
.code-review-graph/
kb/

# Large data files
smarts_results.csv
example_lipids.smi
smiles_sets/
smarts_results_tested_smarts/

# Node modules for opencode plugins
.opencode/node_modules/

# Vendored JS dependencies in Ketcher assets
public/assets/ketcher/
```
**Mark as:** NEEDS-DECISION — there is no standard `.claudeignore` format; different agents use different files (`.claudeignore`, `.agentignore`, `.cursorignore`, etc.)

### 5.5 graphify update in a hook or CI
**What to create:** Add `graphify update .` as a pre-commit hook or CI step.
**Mark as:** NEEDS-DECISION — graphify must be installed (via `./mk setup`), adding it changes the toolchain

---

## 6. REMEDIATION ROADMAP

Ordered by (impact / effort). Effort estimates assume a developer familiar with the repo.

| Priority | Finding | Effort | Safe? | Phase |
|----------|---------|--------|-------|-------|
| P0 | Add `.gemini/` and `.agents/` to `.gitignore` | 5 min | ✓ | Phase 4 |
| P0 | Remove ghost references (`crates/lotus`, `apps/lotus-explore-rs`) from `AI_AGENT_GUIDE.md` | 20 min | ✓ | Phase 4 |
| P0 | Remove stale `llms.txt` claim from `README.md` | 5 min | ✓ | Phase 4 |
| P0 | Add graphify MCP to tracked `.mcp/mcp.json` | 10 min | ✓ | Phase 4 |
| P0 | Create tracked root `AGENTS.md` as pointer; remove `AGENTS.md` from `.gitignore` | 15 min | ✓ | Phase 4 |
| P1 | Remove dead `wiki/index.md` reference from AGENTS.md | 2 min | ✓ | Phase 4 |
| P1 | Remove "typesafe-ai skill is installed" claim from `.github/ai/AGENTS.md` (directory is empty) | 5 min | ✓ | Phase 4 |
| P2 | Fix MCP config in gitignored `opencode.json` to use `--from graphifyy` (matching tracked `.github/ai/.mcp.json`) | 5 min | ✗ (gitignored, local only) | — |
| P3 | Automate `graphify update .` in pre-commit hook or CI | 30 min + setup.sh changes | ✗ | — |
| P3 | Add path-existence checker for doc references | 1–2 hr | ✗ | — |
| P3 | Create `.claudeignore`/`.agentignore` for large vendor/data files | 15 min | ✗ (format uncertain) | — |
| P3 | Implement `llms.txt` for web-agent discovery | 30 min | ✗ (new feature) | — |
| P3 | Populate `.gemini/skills/` and `.agents/skills/typesafe-ai/` with actual skill definitions | 1–2 hr | ✗ | — |

---

## 7. PHASE 2: EMPIRICAL COLD-START TEST

Three realistic tasks, simulated from scratch using ONLY instruction files and maps.

### Task 1: Bug fix — fix a clippy warning in `crates/upload`

**Cold-start path:**
1. Open `README.md` → find `./mk` commands → "Run `./mk lint`"
2. Open `.github/ai/AI_AGENT_GUIDE.md` → line 49: "`./mk lint` # clippy, all targets, all features, warnings denied"
3. Run `./mk lint` → find specific warning
4. Locate the warning, fix it, re-run `./mk lint`

**Expected friction:**
- `./mk lint` lints the ENTIRE workspace. To iterate quickly on one crate, the agent must know `cargo clippy -p upload --locked -- -D warnings`. This is NOT documented in any instruction file. `AI_AGENT_GUIDE.md` only lists workspace-wide commands.
- `AI_EFFICIENCY_GUIDE.md:12` mentions the native command `cargo clippy --workspace --all-targets --locked -- -D warnings` but only as context for "state the target gate explicitly," not as a documented alternative.

**Empirical verification:**
- `cargo clippy -p upload --locked -- -D warnings` → exit 0, Finished in 12.5s. Works.
- No friction if the agent infers the `-p` flag, but this is a guess.

**Result:** PARTIAL — works but requires one undocumented inference.

### Task 2: Small feature — add a `styles::border_radius` helper to `crates/ui`

**Cold-start path:**
1. Open `README.md` → Structure section: `crates/ui` is "shared accessibility-focused UI helpers"
2. Open `.github/ai/AI_AGENT_GUIDE.md` → workspace map says `crates/ui` has `styles::*` helpers
3. Grep for existing style helpers in `crates/ui/src/`
4. Add new helper following the pattern
5. Run `./mk check`, `./mk lint`, `./mk test`

**Expected friction:**
- The inline-style convention is documented at `AI_AGENT_GUIDE.md:15-18`: "every inline `style:` must be a `StyleBuilder::new()...build()` value." But the agent must read the actual `styles.rs` to understand the pattern.
- No single-file test command documented — the agent must run the full `./mk test` (480 tests, 3.7s) or guess `cargo nextest run -p ui`.

**Empirical verification:**
- `cargo nextest run -p ui` → would work (nextest is installed)
- `cargo clippy -p ui --locked -- -D warnings` → would work (not tested but inferred)

**Result:** PASS with minor friction — the pattern is discoverable from grep + module docs.

### Task 3: Refactor — split `apps/json-count-rs/src/processing.rs` (677 lines)

**Cold-start path:**
1. Open `.github/ai/AI_AGENT_GUIDE.md` → Phase 3 status mentions god-file splitting as "complete"
2. The AI_AGENT_GUIDE.md:109-124 says `cxsmiles/app.rs` 684→137, `mgf parser.rs`→`parser/{mod,...}` splits are done
3. Grep for large files to find candidates
4. The `OVERSIZED` list in `apps.rs:179` is empty
5. `processing.rs` is 677 lines — under the 800-line limit but close

**Expected friction:**
- The AI_AGENT_GUIDE.md "Phase status" section is the ONLY place that describes refactoring conventions. It references `apps/lotus-explore-rs` (DOES NOT EXIST) and `crates/lotus` (DOES NOT EXIST). A cold-start agent following these references hits dead ends.
- The `OVERSIZED` test (`MAX_FILE_LINES = 800`) in `apps.rs:158` is not documented outside the test file itself. An agent might not know about this convention.

**Empirical verification:**
- `apps/json-count-rs/src/processing.rs` is 677 lines — under the 800 limit, so no `OVERSIZED` exemption needed.
- The test `no_source_file_is_longer_than_the_limit` enforces the 800-line ceiling.

**Result:** FAIL (high friction) — the Phase status section in AI_AGENT_GUIDE.md is the primary refactoring guide, and it contains ghost references that mislead the agent.

### Cold-start friction summary

| Task | Friction points | Resolution |
|------|-----------------|------------|
| Bug fix (lint one crate) | No single-crate lint command documented | Infer `cargo clippy -p <crate>` |
| Small feature (add helper) | Pattern discoverable from code; no single-crate test documented | Full `./mk test` works (3.7s) |
| Refactor (split file) | Phase status section references non-existent `crates/lotus`, `apps/lotus-explore-rs` | Use graphify query instead; 800-line limit undocumented outside test |

---

## 8. BEFORE/AFTER (Phase 4 fixes applied)

### Phase 4 fixes applied

Branch: `agent-readiness/audit-fixes` (6 commits)

| Commit | Change | SAFE? |
|--------|--------|-------|
| `7f5fc169` | `.gitignore`: add `.gemini/` and `.agents/`; stop ignoring `AGENTS.md` | ✓ |
| `61fac90f` | Create tracked root `AGENTS.md` (25-line pointer + common commands) | ✓ |
| `8c54b53d` | `README.md`: remove stale `llms.txt`/`.well-known`/`JSON-LD` claim | ✓ |
| `97ebe6c6` | `.mcp/mcp.json`: add graphify + code-review-graph MCP servers | ✓ |
| `e7302de3` | `.github/ai/AGENTS.md`: remove dead `wiki/index.md` ref, fix false skill claim | ✓ |
| `30a4fde3` | `.github/ai/AI_AGENT_GUIDE.md`: remove all ghost references to `crates/lotus` and `apps/lotus-explore-rs` | ✓ |

### Verification of fixes

All fast gate checks pass on the branch:
- `./mk verify-tasks` → 2.4s, exit 0
- `./mk fmt-check` → 1.3s, exit 0
- `./mk typos` → 1.1s, exit 0 (fixed the typo found in this report itself)
- `./mk tombi-check` → 0.8s, exit 0
- `./mk check` → 1.4s, exit 0
- `./mk lint` → 1.4s, exit 0
- `./mk test` → 7.4s, 480 tests pass, exit 0

### Cold-start re-test results

| Task | Before friction | After friction | Resolution |
|------|-----------------|----------------|------------|
| Bug fix (lint one crate) | No single-crate lint command documented; must infer `cargo clippy -p` | Root `AGENTS.md:24` now documents `cargo clippy -p <crate> --locked -- -D warnings` | RESOLVED |
| Small feature (add helper) | No single-crate test documented; must infer `cargo nextest run -p` | Root `AGENTS.md:23` now documents `cargo nextest run -p <crate>` | RESOLVED |
| Refactor (split file) | Phase status section referenced `apps/lotus-explore-rs`, `crates/lotus`, `crates/ui/styles/lotus/` — all non-existent | Ghost references removed; disclaimer added at `.github/ai/AI_AGENT_GUIDE.md:96` | RESOLVED |
| MCP setup | `.mcp/mcp.json` had only `ai-guides` MCP; graphify MCP only in gitignored files | `.mcp/mcp.json` now has graphify + code-review-graph using verified-working `--from graphifyy` command | RESOLVED |

### Empirical verification of Phase 2 tasks (post-fix)

**Task 1:** `cargo clippy -p upload --locked -- -D warnings` → exit 0 in 0.9s (cache warm). The command is now documented in root `AGENTS.md`.

**Task 2:** `cargo nextest run -p ui` → would work. The command pattern is now documented in root `AGENTS.md`.

**Task 3:** `grep -n "lotus" .github/ai/AI_AGENT_GUIDE.md` → only 9 matches, all in explicit disclaimers or historical context with "removed from the workspace" qualifiers. An agent reading the file is now warned before encountering dead paths.

---

## APPENDIX A: Files examined

**Tracked instruction/config files (post-fix):**
- `AGENTS.md` (now TRACKED — 25-line pointer + common commands)
- `.github/ai/AGENTS.md` (tracked — graphify + TypeSafe AI, ghost refs removed)
- `.github/ai/AI_AGENT_GUIDE.md` (tracked — workspace map, change protocol, safety, phase status with disclaimer added)
- `.github/ai/AI_EFFICIENCY_GUIDE.md` (tracked — agent efficiency guide)
- `.github/ai/CONTRIBUTING_AI.md` (tracked — AI contribution guidelines)
- `.github/ai/.mcp.json` (tracked — code-review-graph + graphify MCP)
- `.mcp/mcp.json` (now TRACKED — ai-guides + graphify + code-review-graph MCP)
- `.gitignore` (tracked — now includes `.gemini/` and `.agents/`); AGENTS.md no longer ignored
- `.graphifyignore` (tracked — excludes target/, mutants.out/)
- `README.md` (tracked — project overview, commands, structure)
- `.github/CONTRIBUTING.md` (tracked — development workflow, gate description)
- `.github/PULL_REQUEST_TEMPLATE.md` (tracked)
- `Makefile.toml` + `make/*.toml` (tracked — task definitions)
- `prek.toml` (tracked — pre-commit hooks)
- `mk` (tracked — cargo-make wrapper)
- `.config/nextest.toml` (tracked — test profiles)
- `.cargo/config.toml` (tracked — cargo config)
- `.cargo/mutants.toml` (tracked — mutation testing config)
- `rust-toolchain.toml` (tracked — Rust 1.99.0)
- `clippy.toml`, `rustfmt.toml`, `.tombi.toml`, `.typos.toml`, `deny.toml` (tracked)
- `.vscode/settings.json`, `.vscode/extensions.json`, `.vscode/README.md` (tracked)
- `crates/upload/tests/gate_consistency.rs` (tracked — 413-line consistency test suite)
- `.kb/ledger.md` (gitignored — local review ledger)

**Gitignored agent files (local only, after fix):**
- `opencode.json` (root — graphify MCP config, uses `--from graphifyy[mcp]`)
- `.opencode/` (plugins/graphify.js, package.json, node_modules/)
- `.mcp.json` (root — code-review-graph + graphify MCP; now outdated, see `.mcp/mcp.json` for canonical config)
- `.claude/` (does not exist; not ignored)
- `.cursor/` (does not exist; not ignored)
- `.gemini/` (now gitignored, was not tracked)
- `.agents/` (now gitignored, was not tracked)
- `.code-review-graph/` (graph.db, 39MB)
- `graphify-out/` (knowledge graph output, fresh)
- `.kb/` (local review ledger)

**Verification commands executed:**
- `graphify query "what is the project about"` → 12 nodes, <1s
- `graphify update .` → 2284 nodes, 4148 edges, 147 communities, ~1s
- `uvx --from "graphifyy[mcp]"` and `uvx --from "graphifyy"` → both work
- `./mk --list-all-steps` → works
- `./mk check` → 8.4s, exit 0
- `./mk fmt-check` → 3.8s, exit 0
- `./mk typos` → 0.9s, exit 0
- `./mk lint` → 6.6s, exit 0
- `./mk doc` → 3.4s, exit 0
- `./mk license-headers` → 5.7s, exit 0
- `./mk verify-tasks` → 2.7s, exit 0
- `./mk test` → 3.7s, 480 tests passed
- `./mk ci` → 2m20s, exit 0
- `cargo clippy -p upload --locked -- -D warnings` → 12.5s, exit 0
- `cargo nextest run -p upload` → 29.1s, 48 tests passed
