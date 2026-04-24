# lotus-api

OpenAPI/Swagger service for programmatic LOTUS explorer access.

## What it provides

- `POST /v1/search`: run a LOTUS search with filters and get JSON rows + stats.
- `POST /v1/export-url`: generate direct QLever CSV/JSON/RDF export URLs.
- `GET /openapi.json`: OpenAPI schema.
- `GET /docs`: Swagger UI.
- `GET /health`: liveness probe.

This service reuses query and parser logic from `apps/lotus-explorer` and is intended as a thin API layer for integrations.

## Run

```bash
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

- CORS is enabled for all origins by default (easy frontend integration).
- For production, restrict CORS and place this service behind your existing reverse proxy.

