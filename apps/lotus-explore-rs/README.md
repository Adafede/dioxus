# lotus-explore-rs

[![AGPL-3.0 license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-315-brightgreen)](https://github.com/adafede/dioxus/actions)

`lotus-explore-rs` — LOTUS Knowledge Explorer.

A linked open data (LOAD) explorer for the LOTUS compound-taxon-reference
knowledge graph from Wikidata, queried via SPARQL.  Powered by the `lotus`
shared crate and the QLever SPARQL endpoint.

### Architecture

See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) for the full architectural
overview.

### Engineering skills

- [`SKILLS.md`](./SKILLS.md)
- [`docs/skills/SUGGESTIONS.md`](./docs/skills/SUGGESTIONS.md)

### Curation share links

- [`docs/CURATION_SHARE_LINKS.md`](./docs/CURATION_SHARE_LINKS.md)

### Development testing

Run logging format tests during telemetry work:

```bash
cargo test --locked -p lotus-explore-rs utils::logging::tests
```

### Setup: external assets

RDKit.js and citation-js are loaded from CDN (no local download needed).
Ketcher (115 MB) must be fetched before serving:

```bash
./scripts/fetch-ketcher.sh
```

### Citation

- Paper (DOI): <https://doi.org/10.7554/eLife.70780>
- BibTeX: [`public/docs/references.bib`](./public/docs/references.bib)

### Site metadata

`public/llms.txt`, `public/humans.txt`, `public/robots.txt`,
`public/.well-known/security.txt`, `public/_headers`, and
`public/site.webmanifest` are generated from
[`metadata/site-metadata.json`](./metadata/site-metadata.json).

## License

`AGPL-3.0-only` — see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html) for details.
