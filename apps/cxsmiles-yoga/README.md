# CX-SMILES Yoga

Generate a **CX-SMILES** representation from a list of related structures and validate it by re-expansion.

A CX-SMILES encodes positional isomers (`m:` blocks) or variable-length repeats (`Sg:n:` blocks) over a shared scaffold derived from the input list via maximum common substructure.

## How it works

The UI (`src/app.rs` and `src/app/*`) calls [`crates/cxsmiles-core`](../../crates/cxsmiles-core/README.md) for CX-SMILES generation and [`crates/chem-depict`](../../crates/chem-depict/README.md) for local 2D SVG rendering.

```
 input SMILES list   ──▶  parse & sort canonically, then cluster by ECFP4/Tanimoto (threshold = 0.3)
                               └─ isolates the largest chemically related cluster
                        ──▶  try variable-length repeat detection (Sg:n: block)
                        ──▶  if no clean repeat holds, compute MCS & extract positional groups (m: block)
                        ──▶  serialize canonical base SMILES and CX extension fields
                        ──▶  round-trip: enumerate the CX-SMILES and measure recall (covered / total)
```

### Construct families

  | Family                          | CX-SMILES construct                         | Example                                                                                   |
  | ------------------------------- | ------------------------------------------- | ----------------------------------------------------------------------------------------- |
  | Positional isomers              | `m:<starIdx>:<pos1>.<pos2>.…`               | `c2ccc(cc2)-c1ccccc1.[*]Cl \|m:12:0.2.5\|` (monochlorobiphenyl isomers)                   |
  | Variable-length repeat          | `Sg:n:<atomIndices>:n:ht`                   | `OC(=O)C(F)(F)C(F)F \|Sg:n:3,4,5:n:ht\|` (PFAS) / `CCCCCCC \|Sg:n:3:n:ht\|` (alkyl chain) |
  | Best-effort (incomplete recall) | `m:` or `Sg:n:` from the largest cluster    | six structural isomers (`CC(C)(C)O` … `COCCC`)                                            |

### Recall

The round-trip step expands the generated CX-SMILES into concrete canonical structures and reports `covered / total`. When all inputs re-expand (`covered == total`), the construct is preserved as `Positional` or `Repeating`; otherwise it is downgraded to `BestEffort`.

### 2D depiction

Molecule structures render inline in the browser using `chem-depict` (`chematic::depict`), with no remote requests or external JavaScript runtime.

## Development

```bash
# Run domain and UI unit tests
cargo nextest run -p cxsmiles-core -p cxsmiles-yoga

# Serve in the browser
dx serve --package cxsmiles-yoga

# Production bundle
dx build --profile wasm-release --platform web --package cxsmiles-yoga
```

## Limitations

- **Aromatic equivalence.** Checking "Enumerate equivalent aromatic positions" expands each `m:` site across atoms in the same `BondOrder::Aromatic` connected component matching `(atomic_number, degree, formal_charge, implicit_H > 0)`. This local signature invariant avoids substituting quaternary ring junctions or charged heteroatoms but is not a full graph-automorphism computation.
- **Serialization.** The base SMILES and `m:` / `Sg:n:` extension strings are formatted directly inside `cxsmiles-core`.
- **Fingerprint search.** Comparing candidates against external fingerprint databases is out of scope.

## Dependencies

- `cxsmiles-core` --- CX-SMILES clustering, MCS generation, and round-trip engine.
- `chem-depict` --- size-guarded local SVG structure depiction.
- [`dioxus`](https://dioxuslabs.com) `0.7` --- UI framework.
- [`ui`](../../crates/ui/README.md) --- shared UI components and theme tokens.
- `web-sys` --- clipboard API (`Clipboard`, `Navigator`, `Window` features).
- `wasm-bindgen-futures` & `gloo-timers` --- async clipboard Promise resolution and feedback timer.
