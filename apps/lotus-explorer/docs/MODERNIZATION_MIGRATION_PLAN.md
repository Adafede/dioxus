# Modernization Migration Plan

This is the incremental execution plan for implementing `ARCHITECTURE_MODERNIZATION_BLUEPRINT.md` with low regression risk.

## Execution Principles

- small, reversible batches
- behavior-preserving refactors before behavior changes
- one feature boundary at a time
- test-first for extracted pure logic
- keep wasm and host targets green continuously

## Phase Plan

## Phase 0 - Baseline and Guardrails

### Goals
- Freeze baseline quality and perf checks.
- Standardize developer workflow.

### Tasks
- Add/update architecture docs and skills docs.
- Ensure CI checks include:
  - `cargo clippy --bin lotus-explorer --no-deps -- -D warnings`
  - `cargo test --bin lotus-explorer`
  - `cargo check --target wasm32-unknown-unknown`
- Define PR template checklist for boundary and perf concerns.

### Exit criteria
- docs accepted
- repeatable local and CI checks

## Phase 1 - Feature Boundary Hardening

### Goals
- Enforce clean module ownership for `explore` and `curation`.

### Tasks
- tighten `pub` visibility
- move residual business logic from components into feature services/domain
- ensure each feature has explicit facade exports

### Exit criteria
- no transport DTO leakage into component props
- reduced cross-feature imports

## Phase 2 - State and Lifecycle Modernization

### Goals
- predictable state flow with explicit ownership and narrow subscriptions

### Tasks
- audit local/shared/global state
- convert duplicated state into derived selectors
- harden request-token stale response handling

### Exit criteria
- selectors are narrow and feature-scoped
- async completion paths deterministic

## Phase 3 - Data Layer Normalization

### Goals
- strict API/repository/domain separation

### Tasks
- centralize DTO mapping in repository layer
- normalize errors using typed repository error enums
- add repository contract tests

### Exit criteria
- domain layer independent from transport structures

## Phase 4 - Rendering and Performance

### Goals
- remove unnecessary clones/allocations/rerenders on hot paths

### Tasks
- ensure view-model boundaries around heavy rendering zones
- trim reactive scopes and memo expensive derivations
- add perf instrumentation for table/search hotspots

### Exit criteria
- no known per-frame avoidable allocations in hot paths
- verified smoother interaction under larger datasets

## Phase 5 - Testing and Reliability Expansion

### Goals
- deterministic and comprehensive automated verification

### Tasks
- expand unit tests for pure logic boundaries
- add integration tests for feature lifecycles
- add regression tests for error/retry paths

### Exit criteria
- regressions caught by tests before manual QA

## Workstream Ownership Template

Use this for each modernization PR:

- `Problem`: what weakness is being removed
- `Boundary`: which architectural boundary is improved
- `Risk`: behavior/perf risk and mitigation
- `Validation`: exact checks and test scope
- `Follow-ups`: deferred items with rationale

## Suggested First 3 Epics

1. Explore feature facade and import tightening
2. Repository error normalization and DTO mapping isolation
3. Async lifecycle hardening (token cancellation and stale guards)

