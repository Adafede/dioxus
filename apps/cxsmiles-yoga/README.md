# CX-SMILES Yoga

Generate a single **CX-SMILES** from a list of related structures and validate
it back by re-expansion.

A CX-SMILES encodes *structural uncertainty*: a positional isomer (`m:` block)
or a variable-length repeat (`Sg:n:` block). Hand-crafting these does not scale,
so this app derives them automatically from a candidate structure list using the
**maximum common substructure** as the shared scaffold.

--------------------------------------------------------------------------------

## How it works

The UI (`src/app.rs`) is intentionally thin. All chemistry lives in a headless,
unit-tested module (`src/cxsmiles/`) so it can be reused outside the browser.

```
 input SMILES list   ──▶  parse (chematic::smiles)
                        ──▶  cluster (ECFP4 + Tanimoto, τ = 0.3)
                              └─ rejects unrelated structures so they are not
                                 forced into one nonsensical CX-SMILES
                        ──▶  Maximum Common Substructure (chematic::smarts::find_mcs)
                        ──▶  diff group vs. MCS → classify each variable region
                              • discrete equivalents  →  m:   block
                              • variable-length repeat →  Sg:n: block
                        ──▶  serialize to a CX-SMILES string
                              (m: /Sg:n: are hand-rolled — chematic's CX reader
                               does not support these constructs)
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

All molecule rendering uses `chematic::depict` (`depict_svg` /
`depict_svg_highlighted`, which run an in-process 2D layout) --- no RDKit.js, no
remote service, no network calls. The scaffold and each enumerated candidate are
drawn inline in the results panel.

--------------------------------------------------------------------------------

## Try it

```bash
# Run the test suite (covers all six fixtures + a parse/write round-trip)
cargo nextest run -p cxsmiles-yoga
```

The headless core is exercised by the unit tests in `src/cxsmiles/`, covering
the six seed fixtures --- biphenyl-Cl, moving -OCH₃, the double-`m:` acetyl
case, the PFAS repeat, the alkyl repeat, and the six constitutional PFOS isomers ---
plus a parse/write round-trip.

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
- **CX-SMILES hand-rolled.** `chematic::cx` (`CxSmiles` / `parse_cxsmiles` /
  `write_cxsmiles`) does not support `m:` / `Sg:n:` constructs, and `chematic`'s
  SMILES writer does not emit `*` for wildcard atoms, so the base scaffold and
  extension fields are serialized directly.
- **Hashing / fingerprint library search.** Comparing a generated CX-SMILES
  against an external fingerprint library (e.g., ECFP4/Tanimoto
  nearest-neighbour lookup) to score or rank candidates is out of scope for this
  app; only intra-input ECFP4 clustering is performed locally.

--------------------------------------------------------------------------------

## Dependencies

- [`chematic`](https://crates.io/crates/chematic) --- SMILES parsing/writing,
  SMARTS/MCS (`find_mcs`, `find_matches`), ECFP4 fingerprints, 2D depiction.
- [`dioxus`](https://dioxuslabs.com) `0.7` --- UI framework.
- `web-sys` --- clipboard API (`Window` + `Navigator` + `Clipboard` features).
- `gloo-timers` --- async clipboard copy feedback timer.
- [`crates/ui`](../../crates/ui) + [`crates/upload`](../../crates/upload) ---
  shared UI components and upload/download helpers.
- [`rustc-hash`](https://crates.io/crates/rustc-hash) --- `FxHashMap` for match
  maps.
