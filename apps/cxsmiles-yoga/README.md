# CX-SMILES Yoga

Generate a single **CX-SMILES** from a list of related structures and validate
it back by re-expansion.

A CX-SMILES encodes *structural uncertainty*: a positional isomer (`m:` block)
or a variable-length repeat (`Sg:n:` block). Hand-crafting these does not scale,
so this app derives them automatically from a candidate structure list using the
**maximum common substructure** as the shared scaffold.

--------------------------------------------------------------------------------

## How it works

The UI (`src/app.rs` + `src/app/*`) is intentionally thin. All chemistry lives
in the headless domain crate [`crates/cxsmiles-core`](../../crates/cxsmiles-core/README.md)
and 2D SVG depiction lives in [`crates/chem-depict`](../../crates/chem-depict/README.md).

```
 input SMILES list   ──▶  parse & ECFP4/Tanimoto clustering (τ = 0.3)
                               └─ rejects unrelated structures so they are not
                                  forced into one nonsensical CX-SMILES
                        ──▶  Maximum Common Substructure
                        ──▶  diff group vs. MCS → classify each variable region
                               • discrete equivalents  →  m:   block
                               • variable-length repeat →  Sg:n: block
                        ──▶  serialize base SMILES and CX extension fields
                        ──▶  round-trip: enumerate the CX-SMILES and check that
                               it re-expands to (a superset of) the inputs
                               └─ surfaced as a confidence % in the UI
```

### Construct families

  | Family                          | CX-SMILES construct                         | Example                                                                                   |
  | ------------------------------- | ------------------------------------------- | ----------------------------------------------------------------------------------------- |
  | Positional isomers              | `m:<starIdx>:<pos1>.<pos2>.…`               | `…Cl* \|m:13:0.2.3\|` for Cl on a biphenyl at three positions                             |
  | Variable-length repeat          | `Sg:n:<atomRange>:n:ht`                     | `OC(=O)C(F)(F)C(F)F \|Sg:n:3,4,5:n:ht\|` (PFAS) / `CCCCCCC \|Sg:n:3:n:ht\|` (alkyl chain) |
  | Best-effort (no clean scaffold) | `m:` from the MCS, flagged lower-confidence | the six PFOS constitutional isomers                                                       |

### Confidence

The round-trip stage enumerates the generated CX-SMILES back into concrete
structures and counts how many of the original inputs are recovered (**coverage =
covered / total**). When every input re-expands, confidence is *clean* (100%);
otherwise the construct is flagged as lower-confidence --- this is how the "no
clean shared scaffold" constitutional-isomer case is surfaced.

### 2D depiction is fully local

All molecule rendering uses the local `chem-depict` SVG engine (in-process 2D
layout) --- no RDKit.js, no remote service, no network calls. The scaffold and
each enumerated candidate are drawn inline in the results panel.

--------------------------------------------------------------------------------

## Try it

```bash
# Run the domain and app test suites
cargo nextest run -p cxsmiles-core -p cxsmiles-yoga
```

The headless core is exercised by unit and property suites in `crates/cxsmiles-core`,
covering the six seed fixtures --- biphenyl-Cl, moving -OCH₃, the double-`m:` acetyl
case, the PFAS repeat, the alkyl repeat, and the six constitutional PFOS isomers ---
plus parse/write round-trip and ceiling tests.

### Run in the browser

```bash
./mk web-dev APP=cxsmiles-yoga
# Production bundle (uses --profile wasm-release):
./mk web-build APP=cxsmiles-yoga
```

--------------------------------------------------------------------------------

## Limitations & out of scope

- **Aromatic equivalence.** The "Enumerate equivalent aromatic positions"
  checkbox expands `m:` attachment lists across symmetry-equivalent positions
  within each aromatic ring component of the shared scaffold (via graph
  automorphisms restricted to `BondOrder::Aromatic` components).
- **CX-SMILES hand-rolled.** Standard CX parsers do not handle arbitrary `m:` /
  `Sg:n:` constructs alongside wildcard-atom indexing, so the base scaffold and
  extension fields are serialized directly inside `cxsmiles-core`.
- **Hashing / fingerprint library search.** Comparing a generated CX-SMILES
  against an external fingerprint library (e.g., ECFP4/Tanimoto
  nearest-neighbour lookup) to score or rank candidates is out of scope for this
  app; only intra-input ECFP4 clustering is performed locally.

--------------------------------------------------------------------------------

## Dependencies

- `cxsmiles-core` --- CX-SMILES clustering, MCS generation, and round-trip engine.
- `chem-depict` --- panic-guarded local SVG structure depiction.
- [`dioxus`](https://dioxuslabs.com) `0.7` --- UI framework.
- [`ui`](../../crates/ui/README.md) --- shared UI components and theme tokens.
- `web-sys` --- clipboard API (`Window` + `Navigator` + `Clipboard` features).
- `gloo-timers` --- async clipboard copy feedback timer.
- [`crates/ui`](../../crates/ui) + [`crates/upload`](../../crates/upload) ---
  shared UI components and upload/download helpers.
- [`rustc-hash`](https://crates.io/crates/rustc-hash) --- `FxHashMap` for match
  maps.
