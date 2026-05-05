# dioxus-apps

A Cargo workspace that hosts multiple small Dioxus web applications, each compiled independently to WASM and deployable to any static host.

It now also includes a small native HTTP API for the LOTUS explorer, documented with OpenAPI / Swagger UI.

## Start here

Follow this order for fastest onboarding:

1. `Prerequisites`
2. `Quick start (fast path)`
3. `Running the LOTUS API locally` (optional)
4. `Tight explorer ⇄ API integration`
5. `API endpoints`
6. `Deploying the LOTUS API` (only when publishing)

## Prerequisites

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.7 --locked
```

Optional security tooling for full local parity with CI supply-chain checks:

```bash
cargo install cargo-deny --locked
cargo install cargo-audit --locked
```

## Quick start (fast path)

If you only want to run the explorer locally:

```bash
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.7 --locked
dx serve --package lotus-explorer
```

If you also want the optional local API integration:

```bash
cargo run -p lotus-api
```

Then open `http://localhost:8080/?api_base=http://127.0.0.1:8787`.

Without `lotus-api`, the explorer still works by falling back to direct QLever/SPARQL.

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

Use this together with `?api_base=...` in the explorer as described in `Tight explorer ⇄ API integration`.

## Tight explorer ⇄ API integration

`lotus-explorer` can use `lotus-api` for search execution and export URL generation.
The API is **optional** — without it (the default for the public Codeberg Pages build) the explorer always falls back to direct QLever / SPARQL queries.

| Scenario | `api_base` resolution | API used? |
|---|---|---|
| Codeberg Pages (public) | none | ✗ direct SPARQL |
| Local dev (`localhost`) | auto-detected `http://127.0.0.1:8787` | ✓ if server is running |
| Custom build with `LOTUS_API_BASE` | compile-time env var | ✓ with self-hosted server |
| Runtime override `?api_base=…` | URL query param | ✓ with self-hosted server |

For local API wiring, run `lotus-api` via `Running the LOTUS API locally` and open:

- `http://localhost:8080/?api_base=http://127.0.0.1:8787`

For build-time API wiring, see the `Deploying the LOTUS API` section.

## API endpoints

`lotus-api` currently exposes:

- `GET /health`
- `POST /v1/search`
- `POST /v1/export-url`
- `GET /openapi.json`
- `GET /docs`

For full request/response examples, see `apps/lotus-api/README.md` or open `http://127.0.0.1:8787/docs` locally.

## Deploying the LOTUS API

For full API runtime/deployment options, see `apps/lotus-api/README.md`.

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


## Building for production

```bash
make build APP=lotus-explorer
# output → target/dx/lotus-explorer/release/web/public/
```

In deployment, compression should happen at response time (via hosting/CDN/proxy
content negotiation with `Accept-Encoding`) rather than shipping static `.br`/`.gz`
sidecar files.

Run the same quality gate used by CI before release:

```bash
make qa
```

Run supply-chain checks (advisories, licenses, source policy):

```bash
make supply-chain
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

- workspace `cargo check --workspace --all-targets`
- workspace `cargo test --workspace --all-targets`
- `wasm32-unknown-unknown` compile check for `lotus-explorer`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo doc --workspace --no-deps`
- `cargo deny check advisories bans licenses sources`
- `cargo audit`
- build and push `lotus-api` Docker image to `codeberg.org` / `ghcr.io`

## Production checklist

- Run `make qa` on a clean branch.
- Build release web artifacts with `make build APP=<app>`.
- For API deployments, set `APP_ENV=production` and a strict `CORS_ALLOWED_ORIGINS` allowlist.
- Keep `Cargo.lock` committed and deploy from tagged revisions.

## Common setup mistakes

- Missing `wasm32-unknown-unknown` target leads to explorer build/serve failures.
- Using a different `dioxus-cli` version than `0.7.7` can cause unexpected behavior.
- Expecting API-backed behavior without a running API: explorer defaults to direct SPARQL.
- For public API deployments, set strict `CORS_ALLOWED_ORIGINS` (do not use `*`).

## Archive

A frozen version is archived on Zenodo: https://doi.org/10.5281/zenodo.5794106

## Governance

- Contribution guide: `CONTRIBUTING.md`
- Security policy: `SECURITY.md`
- Release process: `.github/RELEASE_CHECKLIST.md`
- Change history: `CHANGELOG.md`
- License: `LICENSE` (GNU AGPL v3.0)

