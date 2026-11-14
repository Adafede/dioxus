# LipidMaps Data & SMARTS Evolution

This directory holds tooling for downloading the LIPID MAPS Structure Database
(LMSD) and evolving SMARTS classification rules offline for `lipid-selecto-rs`.

## Directory Layout

```text
lipidmaps/
├── README.md                     this file
└── smarts-evoliposuction/        standalone CLI for SMARTS evolution (excluded from workspace)
    ├── Cargo.toml
    ├── Cargo.lock
    ├── README.md
    └── src/
        ├── download.rs           fetch LMSD.sdf.zip and convert to TSV
        ├── splitting.rs          split TSV into per-class +/- SMILES pairs
        ├── evolve.rs             run smarts-evolution GA per class
        ├── manifest.rs           CSV manifest I/O
        ├── main.rs               CLI binary (download / split / evolve / all)
        └── lipidsdl/             vendored SDF/TSV download & conversion module
```

Downloaded `LMSD.sdf.zip` and `LMSD.sdf.tsv` artifacts are gitignored and never
committed to the repository.

## Workflow

### 1. Evolve SMARTS from LIPID MAPS

Use `smarts-evoliposuction` (with `--locked` and `--manifest-path` from the repo
root, or `cargo run --release --locked` inside `smarts-evoliposuction/`):

```bash
cargo run --release --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --locked -- \
    all \
    --output-dir ./results \
    --generations 500 \
    --population 100
```

Stages:

1. **Download**: fetches `LMSD.sdf.zip` from
   `https://www.lipidmaps.org/files/?file=LMSD&ext=sdf.zip` and converts the SDF
   records into a 19-column TSV (`LMSD.sdf.tsv`).
2. **Split**: parses the TSV and creates deterministic balanced positive and
   negative SMILES files per LIPID MAPS category, main class, and (when enabled)
   subclass.
3. **Evolve**: runs
   [`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution)
   to search for compact SMARTS patterns maximizing Matthews Correlation
   Coefficient (MCC), isolating per-class failures.

`smarts_results.csv` records one row per class with columns:

  | Column | Description |
  | --- | --- |
  | `level` | `category`, `main_class`, or `subclass` |
  | `label` | Human-readable class label |
  | `main_class` | LIPID MAPS main class code (e.g. `FA01`) |
  | `subclass` | Subclass code (e.g. `FA0101`) |
  | `slug` | URL-safe identifier for resume tracking |
  | `positive_count` | Number of positive SMILES used |
  | `negative_count` | Number of negative SMILES used |
  | `best_smarts` | Evolved SMARTS pattern |
  | `best_mcc` | Best MCC score |
  | `best_coverage_score` | Positive-set coverage score |
  | `status` | `ok`, `empty_after_parse`, `read_error`, etc. |

### 2. Curated rules in `lipid-selecto-rs`

Selected SMARTS rules from LMSD evolution are stored statically in
`apps/lipid-selecto-rs/src/chemical_class/lmsd.rs` (`LMSD_ENTRIES`, exposed
crate-internally via `lmsd_classes()` and `lmsd_all()`). Each `ChemicalClass`
carries `name`, `smarts`, `color`, and `family` (such as `"Fatty Acyls"`), and
pre-compiles its SMARTS query at construction via `chematic::smarts`.

### 3. UI integration

Inside `apps/lipid-selecto-rs`, `chemical_class::lmsd_classes()` feeds the class
selector cards and the browser MGF/SMILES classification logic under
`src/parser/` and `src/app/`, grouping results in canonical LIPID MAPS family
order (`FA`, `GL`, `GP`, `SP`, `ST`, `PR`, `SL`, `PK`).

## Data Sources and External Dependencies

- **LMSD.sdf.zip**: `https://www.lipidmaps.org/files/?file=LMSD&ext=sdf.zip`
- [`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution) ---
  GA SMARTS search
- [`smarts-rs`](https://github.com/earth-metabolome-initiative/smarts-rs) ---
  SMARTS matching library
- [`smiles-rs`](https://github.com/earth-metabolome-initiative/smiles-rs) ---
  SMILES parser

See [`smarts-evoliposuction/README.md`](./smarts-evoliposuction/README.md) for
CLI options and `--locked` build requirements.
