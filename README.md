# dioxus-apps

A Cargo workspace for reproducible Dioxus web apps, pinned by
`rust-toolchain.toml`.

**lotus-explorer** explores the LOTUS compounds knowledge graph from Wikidata
via SPARQL. **lotus-api** provides a native HTTP API for advanced search and
export. **index** is the accessible landing page. **jsoncount** counts non-null
fields in uploaded JSON files. **mgf-precursor-erro-rs** analyzes uploaded MGF
files and reports precursor mass errors in Da and ppm.

## Prerequisites

```bash
rustup toolchain install 1.95.0 --profile minimal
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
```

The repo pins Rust 1.95.0, `clippy`, `rustfmt`, and `wasm32-unknown-unknown` in
`rust-toolchain.toml`.

## Quick start

```bash
dx serve --package lotus-explorer
```

To also run the optional API:

```bash
cargo run --locked -p lotus-api
```

Then open `http://localhost:8080/?api_base=http://127.0.0.1:8787`.

Without `lotus-api`, the explorer falls back to direct QLever/SPARQL queries.

## Structure

```
dioxus-apps/
├── Cargo.toml                ← workspace root
├── rust-toolchain.toml       ← pinned compiler, components, target
├── prek.toml                 ← repo hooks and quality gate
├── .github/                  ← CI, deploy, governance
├── apps/
│   ├── index/                ← accessible landing page
│   ├── jsoncount/            ← upload a JSON file and count non-null values
│   ├── mgf-precursor-erro-rs/ ← upload an MGF file and explore mass errors
│   ├── lotus-api/            ← OpenAPI service for LOTUS search and exports
│   └── lotus-explorer/       ← LOTUS Wikidata natural-product explorer
└── crates/
    ├── shared/               ← SPARQL client, LOTUS models
    └── ui/                   ← shared accessibility-focused UI helpers
```

## Running apps locally

```bash
dx serve --package lotus-explorer
cargo run --locked -p lotus-api
dx serve --package json-count-rs
dx serve --package index
dx serve --package mgf-precursor-erro-rs
```

The API binds to `127.0.0.1:8787`. Override with `HOST` and `PORT` env vars.

Open `http://127.0.0.1:8787/docs` for the Swagger UI.

## Explorer ⇄ API integration

  | Scenario                | `api_base` source                     | API used            |
  | ----------------------- | ------------------------------------- | ------------------- |
  | Codeberg Pages (public) | none                                  | ✗ direct SPARQL     |
  | Local dev               | auto-detected `http://127.0.0.1:8787` | ✓ if server running |
  | Build-time              | `LOTUS_API_BASE` env var              | ✓                   |
  | Runtime override        | `?api_base=…` query param             | ✓                   |

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
dx build --release --package lotus-explorer
dx build --release --package json-count-rs
dx build --release --package index
dx build --release --package mgf-precursor-erro-rs
```

Output lands under `target/dx/<package>/release/web/public/`.

## Quality gate and local checks

Install the repo hooks once:

```bash
cargo install prek --locked
prek install
```

Run the repo quality gate manually:

```bash
prek run cargo-qa
```

Equivalent checks for the old Makefile targets:

```bash
prek run cargo-fmt-check        # fmt-check
prek run cargo-check-workspace  # check
prek run cargo-test-workspace   # test
prek run cargo-clippy-workspace # clippy
prek run cargo-doc-workspace    # doc
prek run cargo-qa               # qa
prek run cargo-deny             # deny
prek run cargo-audit            # audit
prek run cargo-supply-chain     # supply-chain
prek run cargo-machete          # machete
prek run cargo-license          # license
prek run cargo-strict           # strict
prek run cargo-tree-d           # tree-d
prek run cargo-outdated         # outdated
```

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
  dx build --release --platform web --package lotus-explorer
```

## Adding a new app

1. Copy the app directory that best matches your target stack.
2. Edit `Cargo.toml` and `Dioxus.toml` to set `name` and `title`.
3. Add `"apps/my-new-app"` to `members` in `Cargo.toml`.
4. `dx serve --package my-new-app`

## URL automation

`lotus-explorer` supports URL-driven execution and exports:

- `?execute=true` --- run query on load
- `?download=true&format=csv` --- download CSV
- `?download=true&format=json` --- download SPARQL Results JSON
- `?download=true&format=rdf` --- download RDF (Turtle)

When both `download` and `execute` are present, `download` takes priority.

## Continuous integration

On every push to `main`:

- `cargo check --workspace --all-targets --locked`
- `cargo test --workspace --all-targets --locked`
- `cargo check -p lotus-explorer --target wasm32-unknown-unknown --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo doc --workspace --no-deps --locked`
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
- Release process:
  [`.github/RELEASE_CHECKLIST.md`](./.github/RELEASE_CHECKLIST.md)
- Change history: [`CHANGELOG.md`](./CHANGELOG.md)
- License: `LICENSE` (GNU AGPL v3.0)

## Archive

A frozen version is archived on Zenodo: https://doi.org/10.5281/zenodo.5794106
