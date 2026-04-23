# dioxus-apps

A Cargo workspace that hosts multiple small Dioxus web applications, each compiled independently to WASM and deployable to any static host.

## Structure

```
dioxus-apps/
├── Cargo.toml              ← workspace root (add new apps here)
├── Makefile                ← convenience targets
├── .github/workflows/      ← CI + per-app deploy actions
├── crates/
│   └── shared/             ← shared theme CSS, components, SPARQL client
└── apps/
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

