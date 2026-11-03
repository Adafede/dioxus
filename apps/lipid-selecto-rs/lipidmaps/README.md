# LipidMaps Data & SMARTS Evolution

This directory contains the LIPID MAPS lipid data and tooling for evolving
SMARTS patterns that classify lipids with perfect precision.

## Directory Layout

```
lipidmaps/
├── README.md                     ← you are here
├── LMSD.sdf.zip                  ← raw LIPID MAPS Structure Database (download)
├── smarts-evoliposuction/        ← GA runner: evolve SMARTS from LIPID MAPS data
│   ├── src/
│   │   ├── download.rs           ← fetch LMSD.sdf.zip → convert to TSV
│   │   ├── splitting.rs          ← split TSV into per-class +/− SMILES pairs
│   │   ├── evolve.rs             ← run smarts-evolution GA per class
│   │   ├── manifest.rs           ← CSV manifest I/O
│   │   └── main.rs               ← CLI binary
│   └── Cargo.toml
└── sparql_lipids.ttl             ← RDF/Turtle LIPID MAPS ontology (if present)
```

## The Full Workflow

The pipeline has three stages:

### 1. Evolve SMARTS from LIPID MAPS

Use the `smarts-evoliposuction` binary to evolve high-quality SMARTS patterns
from the LIPID MAPS Structure Database (LMSD).

```bash
# All-in-one pipeline — download, split, and evolve:
cd smarts-evoliposuction
cargo run --release -- \
    all \
    --output-dir ./results \
    --generations 500 \
    --population 200
```

**What this does:**

1. **Download** --- fetches `LMSD.sdf.zip` from
   `https://www.lipidmaps.org/files/?file=LMSD&ext=sdf.zip` and converts it to a
   clean TSV with 19 standard columns (LM_ID, NAME, SYSTEMATIC_NAME, MAIN_CLASS,
   SUB_CLASS, EXACT_MASS, FORMULA, INCHI, SMILES, etc.).

2. **Split** --- parses the TSV and, for each LIPID MAPS main class and
   subclass, creates balanced positive (matching the class) and negative (not
   matching) `.smiles` file pairs. This mirrors the
   `split_for_smarts_evolution.py` script but in idiomatic Rust with
   deterministic seeded sampling.

3. **Evolve** --- for each class pair, runs the
   [`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution)
   genetic algorithm, which searches for the shortest SMARTS pattern that
   maximizes the Matthews Correlation Coefficient (MCC) on the positive/negative
   set. Errors are caught per-class so one failure doesn't abort the batch.

The result is `smarts_results.csv` --- one row per class with columns:

| Column                | Description                                   |
| --------------------- | --------------------------------------------- |
| `level`               | `main_class` or `subclass`                    |
| `label`               | Human-readable class label                    |
| `main_class`          | LIPID MAPS main class code (e.g. `FA01`)      |
| `subclass`            | Subclass code (e.g. `FA0101`)                 |
| `slug`                | URL-safe identifier for resume tracking       |
| `positive_count`      | Number of positive SMILES used                |
| `negative_count`      | Number of negative SMILES used                |
| `best_smarts`         | The evolved SMARTS pattern                    |
| `best_mcc`            | MCC score of the best result                  |
| `best_coverage_score` | Coverage of the pattern on the positive set   |
| `status`              | `ok`, `empty_after_parse`, `read_error`, etc. |

### 2. Use the evolved SMARTS in lipid-selecto-rs

The evolved SMARTS from `smarts_results.csv` populate the LMSD subclass definitions in `chemical_class::lmsd_classes()`:

```rust
use lipid_selecto_rs::chemical_class::{ChemicalClass, lmsd_classes};

let classes: &[ChemicalClass] = lmsd_classes();
assert!(classes.iter().any(|c| c.category == "FA"));
```

The evolved rules are **pre-compiled** at load time (via
`chematic::smarts::parse_smarts`) so they're fast for runtime matching. Higher
`best_mcc` values indicate tighter, more selective patterns.

### 3. UI integration

The `ChemicalClass` table in `chemical_class::lmsd` is wired into the UI through two entry points:

- **"Available Lipid Classes" card** --- Displays all LMSD rules from `lmsd_classes()`.
  Each card shows the rule name, description, and family tag.

- **Classification pipeline** --- The `ChemicalClass` system (used by the
  browser analysis pipeline in `parser/analysis.rs`) matches molecules against
  `lmsd_classes()`. The Results UI groups matches by LIPID MAPS family (FA → GL → GP →
  SP → ST → PR → SL → PK) and sorts alphabetically within each family.

### LIPID MAPS Family Hierarchy

All eight LIPID MAPS structural families are represented:

| Rank | Code | Family               | Classes                                                                  |
| ---- | ---- | -------------------- | ------------------------------------------------------------------------ |
| 0    | FA   | Fatty Acyls          | FA, MUFA, PUFA                                                           |
| 1    | GL   | Glycerolipids        | TG(AAA), DG(AA), MG(A)                                                   |
| 2    | GP   | Glycerophospholipids | PC(AA), PE(AA), PS(AA), PI(AA), PG(AA), PA(AA), LPC(A), LPE(A), CL(AAAA) |
| 3    | SP   | Sphingolipids        | Cer(AS), SM(AS), HexCer(AS)                                              |
| 4    | ST   | Sterol Lipids        | ST                                                                       |
| 5    | PR   | Prenol Lipids        | PR                                                                       |
| 6    | SL   | Saccharolipids       | SL                                                                       |
| 7    | PK   | Polyketides          | PK                                                                       |

## Data Sources

- **LMSD.sdf.zip**: Downloaded from
  `https://www.lipidmaps.org/files/?file=LMSD&ext=sdf.zip`

## External Dependencies

This project builds on the work of **Luca Cappelletti** (@LucaCappelletti94),
Earth Metabolome Initiative:

- [`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution) ---
  Genetic algorithm for evolving SMARTS patterns
- [`smarts-rs`](https://github.com/earth-metabolome-initiative/smarts-rs) ---
  High-performance SMARTS matching library
- [`smiles-rs`](https://github.com/earth-metabolome-initiative/smiles-rs) ---
  SMILES string parser

See [`smarts-evoliposuction/README.md`](./smarts-evoliposuction/README.md) for
the evoliposuction binary's build instructions and CLI reference.
