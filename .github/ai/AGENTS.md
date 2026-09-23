## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community
structure, and cross-file relationships.

When the user types `/graphify`, use the installed graphify skill or
instructions before doing anything else.

Rules: - For codebase questions, first run `graphify query "<question>"` when
graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for
relationships and `graphify explain "<concept>"` for focused concepts. These
return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep
output. - Dirty graphify-out/ files are expected after hooks or incremental
updates; dirty graph files are not a reason to skip graphify. Only skip graphify
if the task is about stale or incorrect graph output, or the user explicitly
says not to use it. - If graphify-out/wiki/index.md exists, use it for broad
navigation instead of raw source browsing. - Read graphify-out/GRAPH_REPORT.md
only for broad architecture review or when query/path/explain do not surface
enough context. - After modifying code, run `graphify update .` to keep the
graph current (AST-only, no API cost).

## TypeSafe AI

A `typesafe-ai` skill is installed at `.agents/skills/typesafe-ai/`.

**Available primitives:** - **Choice**: select one option from a defined set
(useful for lipid class categorization, format detection) - **Noul**:
probability of a yes/no condition (useful for validating whether a compound name
is a natural product) - **Score**: probability-weighted position on ordered
levels (useful for confidence scoring, ranking)

**Integration opportunities in this codebase:**

1. **lipid-selecto-rs**: After rule-based classification of a lipid species, use
   TypeSafe Choice to verify the LIPID MAPS category/family assignment using the
   compound name and matched adducts as state.

2. **smellfish-rs**: Use TypeSafe Noul to validate whether a motif's source
   class annotation ("Natural", "Synthetic", "Unclassified") is consistent with
   the compound's structural features --- useful for evidence verification
   before scoring.

3. **mgf-precursor-erro-rs**: Use TypeSafe Score to grade the quality of
   precursor mass accuracy on a 1-5 scale, combining with existing numeric
   thresholds.

4. **cxsmiles-yoga**: Use TypeSafe Choice to route ambiguous CXSMILES patterns
   to the correct stereochemistry handler.

**API key handling:** TypeSafe SDK calls require a server-side proxy or
environment variable `TYPESAFE_API_KEY`. Never embed keys in WASM client code.
