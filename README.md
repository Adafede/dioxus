# dioxus-apps

A Cargo workspace that hosts multiple small Dioxus web applications, each compiled independently to WASM and deployable to any static host.

It now also includes a small native HTTP API for the LOTUS explorer, documented with OpenAPI / Swagger UI.

## Structure

```
dioxus-apps/
├── Cargo.toml              ← workspace root (add new apps here)
├── Makefile                ← convenience targets
├── .github/workflows/      ← CI + per-app deploy actions
├── crates/
│   └── shared/             ← shared theme CSS, components, SPARQL client
└── apps/
    ├── lotus-api/          ← OpenAPI / Swagger service for LOTUS search + exports
    ├── lotus-explorer/     ← LOTUS Wikidata natural-product explorer
    └── hello-world/        ← minimal template for new apps
```

## Prerequisites

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.6 --locked
```

## Running an app locally

```bash
# from workspace root
dx serve --package lotus-explorer

# or from inside the app directory
cd apps/lotus-explorer
dx serve
```

## Running the LOTUS API locally

```bash
cargo run -p lotus-api
```

The server binds to `127.0.0.1:8787` by default. Override with env vars:

```bash
HOST=0.0.0.0 PORT=9000 cargo run -p lotus-api
```

Then open:

- `http://127.0.0.1:8787/docs`
- `http://127.0.0.1:8787/openapi.json`

## Deploying the LOTUS API

The API is a native HTTP server — it needs to run at a publicly reachable URL for the deployed WASM app to use it.
Without a running server the explorer falls back automatically to direct QLever / SPARQL queries (which is the default for the Codeberg Pages deployment).

The CI pipeline builds and pushes a ready-to-run container image on every push to `main`:

| Forge | Image |
|---|---|
| Codeberg | `codeberg.org/adafede/lotus-api:latest` |
| GitHub | `ghcr.io/adafede/lotus-api:latest` |

### Self-host with Docker (any VPS or home server)

```bash
docker run -d --restart unless-stopped \
  -p 8787:8787 \
  codeberg.org/adafede/lotus-api:latest
```

Then point the explorer at it by baking the public URL into the WASM build:

```bash
LOTUS_API_BASE=https://your-server.example.org \
  dx build --release --platform web -p lotus-explorer
```

### Build and run locally from source

```bash
# build from workspace root
docker build -f apps/lotus-api/Dockerfile -t lotus-api .
docker run -p 8787:8787 lotus-api
```

## Tight explorer ⇄ API integration

`lotus-explorer` can use `lotus-api` for search execution and export URL generation.
The API is **optional** — without it (the default for the public Codeberg Pages build) the explorer always falls back to direct QLever / SPARQL queries.

| Scenario | `api_base` resolution | API used? |
|---|---|---|
| Codeberg Pages (public) | none | ✗ direct SPARQL |
| Local dev (`localhost`) | auto-detected `http://127.0.0.1:8787` | ✓ if server is running |
| Custom build with `LOTUS_API_BASE` | compile-time env var | ✓ with self-hosted server |
| Runtime override `?api_base=…` | URL query param | ✓ with self-hosted server |

Example with an explicit runtime override:

```bash
dx serve --package lotus-explorer
# then open:
# http://localhost:8080/?api_base=http://127.0.0.1:8787
```

To bake a deployed API URL into the WASM bundle at build time:

```bash
LOTUS_API_BASE=https://your-server.example.org \
  dx build --release --platform web -p lotus-explorer
```

## API endpoints

`lotus-api` currently exposes:

- `GET /health`
- `POST /v1/search`
- `POST /v1/export-url`
- `GET /openapi.json`
- `GET /docs`

Example search request:

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

## Building for production

```bash
make build APP=lotus-explorer
# output → target/dx/lotus-explorer/release/web/public/
```

## Adding a new app

1. `cp -r apps/hello-world apps/my-new-app`
2. Edit `apps/my-new-app/Cargo.toml` — change `name`
3. Edit `apps/my-new-app/Dioxus.toml` — change `name` and `title`
4. Add `"apps/my-new-app"` to the workspace `members` list in `Cargo.toml`
5. `dx serve --package my-new-app`

## Deployment

Each app builds to `target/dx/<app-name>/release/web/public/` (HTML + WASM + JS).
Deploy any of them to:
- **GitHub Pages** — see `.github/workflows/deploy.yml`
- **Cloudflare Pages** — point build command to `make build APP=<name>`
- **Netlify / Vercel** — same as above

## Apps

| App | Description |
|-----|-------------|
| `lotus-api` | Native OpenAPI / Swagger service for LOTUS search execution and export URL generation |
| `lotus-explorer` | LOTUS Wikidata natural-product occurrence explorer (compound × taxon × reference) |
| `hello-world` | Minimal template — copy to start a new app |

## LOTUS URL Automation

`lotus-explorer` supports URL-driven execution and exports:

- `?execute=true` runs the query on page load (no file download)
- `?download=true&format=csv` runs the query and downloads CSV
- `?download=true&format=json` runs the query and downloads SPARQL Results JSON
- `?download=true&format=rdf` runs the query and downloads RDF (Turtle)

Accepted truthy values for `execute` / `download`: `true`, `1`, `yes`, `on`.

When both are present, `download=true` takes priority over `execute=true`.

## Continuous integration

GitHub Actions / Forgejo Actions validate and publish on every push to `main`:

- native `cargo check` / `cargo test` for `lotus-api`
- native `cargo check` / `cargo test` for `lotus-explorer`
- `wasm32-unknown-unknown` compile check for `lotus-explorer`
- `cargo clippy -- -D warnings` across the whole workspace
- build and push `lotus-api` Docker image to `codeberg.org` / `ghcr.io`

