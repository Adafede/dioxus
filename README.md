# dioxus-apps

A Cargo workspace hosting multi-app web experiences compiled to WASM. **lotus-explorer** explores the LOTUS natural products knowledge graph from Wikidata via SPARQL. **lotus-pubchem-tree** builds PubChem classification JSON trees from LOTUS/Wikidata data. **lotus-api** provides a native HTTP API for advanced search, export, and tree-generation workflows.

## Prerequisites

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.9 --locked
```

## Quick start

```bash
dx serve --package lotus-explorer
dx serve --package lotus-pubchem-tree
```

To also run the optional API:

```bash
cargo run -p lotus-api
```

Then open `http://localhost:8080/?api_base=http://127.0.0.1:8787`.

Without `lotus-api`, the explorer falls back to direct QLever/SPARQL queries.

## Structure

```
dioxus-apps/
├── Cargo.toml              ← workspace root
├── Makefile                ← convenience targets
├── .github/                ← CI, deploy, governance
├── crates/
│   └── shared/             ← SPARQL client, LOTUS models
└── apps/
    ├── lotus-api/          ← OpenAPI service for LOTUS search and exports
    ├── lotus-explorer/     ← LOTUS Wikidata natural-product explorer
    ├── lotus-pubchem-tree/ ← LOTUS PubChem tree generator frontend
    └── hello-world/        ← minimal template for new apps
```

## Running apps locally

```bash
dx serve --package lotus-explorer
dx serve --package lotus-pubchem-tree
cargo run -p lotus-api
```

The API binds to `127.0.0.1:8787`. Override with `HOST` and `PORT` env vars.

Open `http://127.0.0.1:8787/docs` for the Swagger UI.

## Explorer ⇄ API integration

| Scenario | `api_base` source | API used |
|---|---|---|
| Codeberg Pages (public) | none | ✗ direct SPARQL |
| Local dev | auto-detected `http://127.0.0.1:8787` | ✓ if server running |
| Build-time | `LOTUS_API_BASE` env var | ✓ |
| Runtime override | `?api_base=…` query param | ✓ |

## API endpoints

- `GET /health`
- `GET /metrics`
- `POST /v1/search`
- `POST /v1/export-url`
- `GET /v1/export-file/{cache_key}/{format}`
- `GET /openapi.json`
- `GET /docs`

## Building for production

```bash
make build APP=lotus-explorer
```

Output: `target/dx/lotus-explorer/release/web/public/`

Quality gate:

```bash
make qa
make supply-chain
```

## Deploying the API

The CI pipeline builds and pushes a container image on every push to `main`:

| Forge | Image |
|---|---|
| Codeberg | `codeberg.org/adafede/lotus-api:latest` |
| GitHub | `ghcr.io/adafede/lotus-api:latest` |

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
  dx build --release --platform web -p lotus-explorer
```

## Adding a new app

1. `cp -r apps/hello-world apps/my-new-app`
2. Edit `apps/my-new-app/Cargo.toml` — change `name`
3. Edit `apps/my-new-app/Dioxus.toml` — change `name` and `title`
4. Add `"apps/my-new-app"` to `members` in `Cargo.toml`
5. `dx serve --package my-new-app`

## URL automation

`lotus-explorer` supports URL-driven execution and exports:

- `?execute=true` — run query on load
- `?download=true&format=csv` — download CSV
- `?download=true&format=json` — download SPARQL Results JSON
- `?download=true&format=rdf` — download RDF (Turtle)

When both `download` and `execute` are present, `download` takes priority.

## Continuous integration

On every push to `main`:

- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- `cargo check -p lotus-explorer --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo doc --workspace --no-deps`
- `cargo deny check advisories bans licenses sources`
- `cargo audit`
- Docker image build and push for `lotus-api`

## AI and agent docs

- [`AI_AGENT_GUIDE.md`](./AI_AGENT_GUIDE.md)
- [`PROJECT_METADATA.json`](./PROJECT_METADATA.json)
- [`apps/lotus-explorer/SKILLS.md`](./apps/lotus-explorer/SKILLS.md)

## Governance

- Contributing: [`.github/CONTRIBUTING.md`](./.github/CONTRIBUTING.md)
- AI contributions: [`.github/CONTRIBUTING_AI.md`](./.github/CONTRIBUTING_AI.md)
- Security: [`.github/SECURITY.md`](./.github/SECURITY.md)
- Release process: [`.github/RELEASE_CHECKLIST.md`](./.github/RELEASE_CHECKLIST.md)
- Change history: [`CHANGELOG.md`](./CHANGELOG.md)
- License: `LICENSE` (GNU AGPL v3.0)

## Archive

A frozen version is archived on Zenodo: https://doi.org/10.5281/zenodo.5794106
