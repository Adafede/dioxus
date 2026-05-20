# Async, Data Layer, and Error Skills

- Async lifecycle safety — every request path has a stable identity and stale responses are ignored centrally.
- Data layer normalization — API clients return DTOs, repositories map to domain types, and services consume domain types only.
- Typed error boundaries — use contextual `thiserror`-style errors and derive user-facing messages separately.

