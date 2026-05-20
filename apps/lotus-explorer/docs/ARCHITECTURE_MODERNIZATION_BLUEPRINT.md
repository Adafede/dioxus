# Architecture Modernization Blueprint

This document is the principal-engineering blueprint for evolving `apps/lotus-explorer` from a working app into a production-grade, scalable, collaboration-friendly Rust/Dioxus system.

## Scope and Intent

This is not a cosmetic refactor plan. The target state is:

- strong feature boundaries
- explicit state ownership
- transport/domain separation
- predictable async flows
- measurable rendering performance
- robust error model
- high testability
- AI-assisted contributor readiness

## Target Architecture

```text
src/
  app/
  components/
    common/
    layout/
    forms/
    domain_specific/
  features/
    explore/
    curation/
    ...
  hooks/
  services/
  state/
  models/
  repositories/
  api/
  utils/
  config/
  tests/
```

## Architectural Rules (Non-negotiable)

1. UI components do rendering orchestration only.
2. Business logic lives in feature services/domain modules.
3. API DTOs never leak directly into UI/component props.
4. Shared state is normalized, typed, and minimal.
5. Every side effect has one owner and one boundary.
6. Async paths are cancellation-safe and race-aware.
7. Expensive computations are memoized or moved out of render paths.

## Major Improvement Workstreams

## 1) Feature Boundary Enforcement

### Problem analysis
Cross-cutting concerns can drift between components, feature services, and ad hoc utility modules.

### Why current approach is weak
When ownership is implicit, regressions appear as hidden coupling, prop drilling, and duplicate logic.

### Refactoring strategy
Adopt strict feature-oriented decomposition:

- `features/<feature>/domain` -> pure domain models/rules
- `features/<feature>/services` -> orchestration/use-cases
- `features/<feature>/repositories` -> data access contracts
- `features/<feature>/state` -> feature state machine/controller
- `components/domain_specific/<feature>` -> view layer only

### New architecture/design
Each feature exports a constrained public API via `mod.rs`, keeping internals private.

### Example implementation
```rust
// features/explore/mod.rs
pub use state::controller::ExploreController;
pub use domain::types::{SearchCriteria, SearchResult};

mod domain;
mod services;
mod repositories;
mod state;
```

### Performance/maintainability impact
- lower cognitive load
- fewer accidental imports
- easier replacement of infrastructure pieces

### Tradeoffs
- initial migration touches many module paths
- requires discipline in `pub` visibility

## 2) State Flow and Reactive Scope Hardening

### Problem analysis
Wide reactive scopes can trigger unnecessary rerenders and create hidden coupling.

### Why current approach is weak
Global-ish subscriptions increase rendering churn and make behavior less predictable.

### Refactoring strategy
- classify state as local, feature-shared, or app-shared
- prefer derived state over duplicated storage
- keep selectors narrow and colocated with consumers

### New architecture/design
Controllers expose typed selectors/actions; components subscribe to minimal slices.

### Example implementation
```rust
#[derive(Clone, Copy)]
pub struct ExploreSelectors {
    pub entries_len: usize,
    pub sort_state: SortState,
}

pub fn select_explore_view(state: &ExploreState) -> ExploreSelectors {
    ExploreSelectors {
        entries_len: state.result.entries.len(),
        sort_state: state.result.sort,
    }
}
```

### Performance/maintainability impact
- fewer rerender cascades
- easier debugging of state transitions

### Tradeoffs
- slightly more selector boilerplate

## 3) Data Layer Separation (API -> Repository -> Domain)

### Problem analysis
Transport concerns can leak into feature logic, making tests brittle and error handling inconsistent.

### Why current approach is weak
If DTOs and HTTP errors flow straight into UI/service logic, architecture hardens around infrastructure.

### Refactoring strategy
- keep API DTOs in `api/`
- map DTOs to domain types in repositories
- normalize errors in repository boundary

### New architecture/design
```text
api client -> dto mapper -> repository -> domain/service -> UI
```

### Example implementation
```rust
#[derive(thiserror::Error, Debug)]
pub enum ExploreRepoError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("domain: {0}")]
    Domain(String),
}
```

### Performance/maintainability impact
- better caching and retries at repository boundary
- stable service APIs independent of transport layer

### Tradeoffs
- adds explicit mapping code

## 4) Async Concurrency and Cancellation Discipline

### Problem analysis
Concurrent searches and retries can race without explicit token/cancellation ownership.

### Why current approach is weak
Race bugs are subtle and can appear as stale UI commits.

### Refactoring strategy
- token every async request path
- cancel or ignore stale completions centrally
- keep retry policy in dedicated coordinator modules

### New architecture/design
Feature orchestrator owns request lifecycle and stale-check policy.

### Example implementation
```rust
if response.request_token != state.active_request_token {
    return; // stale completion ignored
}
```

### Performance/maintainability impact
- deterministic state commits
- reduced stale network noise in UI

### Tradeoffs
- requires strict test coverage of lifecycle logic

## 5) Rendering Performance and Allocation Hygiene

### Problem analysis
Large result tables are sensitive to per-frame allocations and broad reactive subscriptions.

### Why current approach is weak
Small inefficiencies multiply under scroll-heavy usage.

### Refactoring strategy
- precompute pure view models
- keep virtualization windows as index ranges, not materialized vectors
- keep row text bundles and formatting pure + testable

### New architecture/design
`entries + sort + virtualization -> pure render model -> slim render component`

### Example implementation
```rust
for idx in order[start_row..end_row].iter().copied() {
    // render visible row only
}
```

### Performance/maintainability impact
- lower allocation pressure
- smoother scroll under larger datasets

### Tradeoffs
- requires careful bounds correctness tests

## 6) Error Model and Observability Upgrade

### Problem analysis
Inconsistent error forms reduce reliability and operability.

### Why current approach is weak
String-only errors and ad hoc logging reduce diagnosability.

### Refactoring strategy
- typed errors with context at each boundary
- user-facing and telemetry-facing error forms separated
- event naming and timing conventions standardized

### New architecture/design
- domain error enums for user semantics
- infrastructure errors wrapped with context
- structured event keys

### Performance/maintainability impact
- faster root-cause analysis
- safer retry/fallback decisions

### Tradeoffs
- more explicit conversions between error types

## 7) Testability and Deterministic Verification

### Problem analysis
Untested logic in UI or side-effect-heavy modules is costly to change.

### Why current approach is weak
Behavior can regress without obvious compile-time feedback.

### Refactoring strategy
- maximize pure modules and test them directly
- use trait-based injection for repository/services where useful
- add deterministic async tests for lifecycle/retry

### New architecture/design
- unit tests for pure transformations
- integration tests for feature flows
- thin component behavior tests for render correctness

### Performance/maintainability impact
- safer refactors
- lower regression risk

### Tradeoffs
- broader test matrix and maintenance overhead

## Governance and Quality Gates

Every modernization PR should satisfy:

- `cargo clippy -- -D warnings`
- focused tests for touched feature(s)
- full `lotus-explorer` binary tests
- wasm target check
- architecture note if boundaries changed

See:

- `docs/MODERNIZATION_MIGRATION_PLAN.md`
- `docs/skills/README.md`

