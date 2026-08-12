# lotus-api

[![AGPL-3.0 license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-28-brightgreen)](https://github.com/adafede/dioxus/actions)

`lotus-api` — native HTTP API for LOTUS explorer search and export.

Wraps the `lotus` and `upload` shared crates behind Warp endpoints, providing
species/occurrence search, CSV/JSON/RDF export via `Query` or local SPARQL,
and runtime metrics.

### Run locally

```bash
LOTUS_API_BASE=http://localhost:3030 cargo run -p lotus-api
```

### Endpoints

| Method | Path                    | Description                              |
|--------|-------------------------|------------------------------------------|
| GET    | `/api/search`           | Search compounds with pagination         |
| GET    | `/api/export`           | Export results as CSV/JSON/RDF/Turtle  |
| GET    | `/api/stats`            | Dataset statistics                       |
| GET    | `/api/health`           | Health check                             |

### Environment variables

- `LOTUS_API_BASE` — base URL for the API server
- `HOST` — bind address (default: `0.0.0.0`)
- `PORT` — bind port (default: `3030`)

## License

`AGPL-3.0-only` — see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html) for details.
