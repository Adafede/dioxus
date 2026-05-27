# lotus-pubchem-tree

Dioxus frontend for the LOTUS PubChem Tree Generator workflow.

## What it does

- Fetches the LOTUS-linked Wikidata dataset needed for PubChem tree generation
- Builds three tree variants:
  - biological taxonomy tree
  - chemical tree from Wikidata `P279`
  - chemical tree from NPClassifier
- Shows truncated previews for quick inspection
- Exposes download links for PubChem-format and full metadata JSON artifacts

## Architecture

This app follows the same workspace layering as `apps/lotus-explorer`:

- `src/app` — app shell and bootstrap wiring
- `src/api` — typed backend client and DTOs
- `src/components` — rendering-only components
- `src/services` — error presentation
- `src/models` / `src/app_state` — UI domain state

The heavy tree generation runs in `apps/lotus-api`, which reuses shared Rust domain logic from `crates/shared/src/lotus/pubchem_tree.rs`.

## Run locally

```bash
cargo run -p lotus-api

dx serve --package lotus-pubchem-tree
```

If the API is not auto-detected, open:

```text
http://localhost:8080/?api_base=http://127.0.0.1:8787
```

