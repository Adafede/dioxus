# lotus-explorer

A linked open data (LOAD) explorer for the LOTUS compound-taxon-reference
knowledge graph from Wikidata, queried via SPARQL.

## Architecture

- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md)

## Engineering skills

- [`SKILLS.md`](./SKILLS.md)
- [`docs/skills/SUGGESTIONS.md`](./docs/skills/SUGGESTIONS.md)

## Curation share links

- [`docs/CURATION_SHARE_LINKS.md`](./docs/CURATION_SHARE_LINKS.md)

## Development testing

Run logging format tests during telemetry work:

```bash
cargo test --locked -p lotus-explorer utils::logging::tests
```

## Citation

- Paper (DOI): <https://doi.org/10.7554/eLife.70780>
- BibTeX: [`public/docs/references.bib`](./public/docs/references.bib)

## Site metadata

`public/llms.txt`, `public/humans.txt`, `public/robots.txt`,
`public/.well-known/security.txt`, `public/_headers`, and
`public/site.webmanifest` are generated from
[`metadata/site-metadata.json`](./metadata/site-metadata.json).
