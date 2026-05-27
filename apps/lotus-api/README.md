# lotus-api

OpenAPI/Swagger service for programmatic LOTUS explorer access.

## What it provides

- `POST /v1/search`: run a LOTUS search with filters and get JSON rows + stats.
- `POST /v1/export-url`: generate direct QLever CSV/JSON/RDF export URLs.
- `POST /v1/pubchem-tree/fetch`: fetch and cache source data for PubChem tree generation.
- `POST /v1/pubchem-tree/build`: build tree previews + download artifact links from a session.
- `GET /v1/pubchem-tree/download/{session_id}/{artifact}`: download generated JSON artifact.
- `GET /openapi.json`: OpenAPI schema.
- `GET /docs`: Swagger UI.
- `GET /health`: liveness probe.

This service reuses query and parser logic from `apps/lotus-explorer` and is intended as a thin API layer for integrations.

## Run

```bash
cargo run -p lotus-api
```

### Runtime configuration

`lotus-api` reads runtime settings from environment variables:

- `HOST` (default: `127.0.0.1`)
- `PORT` (default: `8787`)
- `DEFAULT_LIMIT` (default: `500`, clamped to service max)
- `APP_ENV` (`development` by default; set to `production` in deployments)
- `CORS_ALLOWED_ORIGINS` (comma-separated list, required when `APP_ENV=production`)

Example production-like run:

```bash
APP_ENV=production \
CORS_ALLOWED_ORIGINS="https://explorer.example.org" \
HOST=0.0.0.0 \
PORT=8787 \
cargo run -p lotus-api
```

Then open:

- `http://127.0.0.1:8787/docs`
- `http://127.0.0.1:8787/openapi.json`

## Example request

```bash
curl -sS http://127.0.0.1:8787/v1/search \
  -H 'content-type: application/json' \
  -d '{
    "taxon": "Gentiana lutea",
    "formula_exact": "C20H28O2",
    "c_min": 1,
    "c_max": 300,
    "limit": 100
  }'
```

## Notes

- In development, CORS allows all origins for easy local integration.
- In production (`APP_ENV=production`), startup fails unless `CORS_ALLOWED_ORIGINS` is explicitly configured.
- Responses are content-negotiated and compressed (Brotli/Gzip) when clients send `Accept-Encoding`.
- Keep this service behind your reverse proxy and TLS termination layer.

