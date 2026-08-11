# lotus

LOTUS domain models, SPARQL query construction, and result parsing for
Wikidata/QLever — the single source of truth consumed by `lotus-api` and
`lotus-explore-rs`.

## What belongs here

- **Domain types** (`models`): `SearchCriteria`, `CompoundEntry`,
  `DatasetStats`, `TaxonMatch`, sort state, Wikidata constants.
- **Query builders** (`queries`): pure functions that produce SPARQL strings.
  No I/O, no network.
- **Transport** (`transport`): platform-agnostic HTTP POST to any SPARQL/QLever
  endpoint with retries, content negotiation, and gateway-error detection.
- **LOTUS wrappers** (`sparql`): thin wrappers around `transport` that target
  the default QLever Wikidata endpoint and parse CSV results into typed rows.

## What does NOT belong here

- File upload or blob streaming → [`upload` crate](../../upload)
- UI components or styling → [`ui` crate](../../ui)
- App routing, state machines, or i18n → each app's own code
