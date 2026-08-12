# lotus-api

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-28-brightgreen)](https://github.com/adafede/dioxus/actions)

`lotus-api` --- native HTTP API for LOTUS explorer search and export.

Wraps the `lotus` and `upload` shared crates behind Warp endpoints, providing
species/occurrence search, CSV/JSON/RDF export via `Query` or local SPARQL, and
runtime metrics.

## Run locally

```bash
LOTUS_API_BASE=http://localhost:3030 cargo run -p lotus-api
```

## Endpoints

- `GET /health`
- `GET /metrics`
- `POST /v1/search`
- `POST /v1/export-url`
- `GET /v1/export-file/{cache_key}/{format}`
- `GET /openapi.json`
- `GET /docs`

## Environment variables

- `LOTUS_API_BASE` --- base URL for the API server
- `HOST` --- bind address (default: `127.0.0.1`)
- `PORT` --- bind port (default: `8787`)

## Deploying the API

The CI pipeline builds and pushes a container image on every push to `main`:

  | Forge    | Image                                   |
  | -------- | --------------------------------------- |
  | Codeberg | `codeberg.org/adafede/lotus-api:latest` |
  | GitHub   | `ghcr.io/adafede/lotus-api:latest`      |

Self-host:

```bash
docker run -d --restart unless-stopped \
  -e APP_ENV=production \
  -e CORS_ALLOWED_ORIGINS=https://your-origin.example.org \
  -p 8787:8787 \
  codeberg.org/adafede/lotus-api:latest
```

Build-time WASM wiring:

```bash
LOTUS_API_BASE=https://your-server.example.org \
  dx build --release --platform web --package lotus-explore-rs
```

## License

`AGPL-3.0-only` --- see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html)
for details.
