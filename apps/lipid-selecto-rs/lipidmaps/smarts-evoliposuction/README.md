# smarts-evoliposuction

Batch SMARTS evolution runner for lipid class classification using the
[`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution)
genetic algorithm.

Evolves high-precision SMARTS patterns directly from LIPID MAPS data, then loads
those patterns into [`lipid-selecto-rs`](../..) for use in the interactive web
classification UI.

## Architecture

```
smarts-evoliposuction/
├── src/
│   ├── lib.rs        — public API re-exports
│   ├── main.rs       — CLI binary (download / split / evolve / all)
│   ├── splitting.rs  — CSV parsing + positive/negative SMILES splitting
│   ├── evolve.rs     — smarts-evolution wrapper with per-class error isolation
│   ├── manifest.rs   — manifest CSV reading/writing
│   └── download.rs   — LMSD.sdf.zip download + TSV conversion
```

**Pipeline:**

1. **Download** --- fetches `LMSD.sdf.zip` from LIPID MAPS and converts to TSV
   via [`lipidsdl`](../../../crates/lipidsdl).
2. **Split** --- parses the TSV and creates balanced positive/negative SMILES
   file pairs for each LIPID MAPS `CATEGORY` (8 major families), `MAIN_CLASS`,
   and `SUB_CLASS`. Categories correspond to the 8 major colors used in the
   lipid-selecto-rs UI.
3. **Evolve** --- runs `smarts-evolution`'s genetic algorithm on each pair,
   catching parse failures and evolution errors per-class so the batch continues
   even if one class fails. By default only `category` and `main_class` rows are
   evolved; use `--with-subclasses` to include `subclass` rows.
4. **Results** --- writes `smarts_results.csv` with one row per class containing
   the best-evolved `best_smarts` pattern, MCC score, and coverage metrics.
   Per-class `_tested_smarts.csv` files contain all evaluated SMARTS sorted by
   MCC, coverage, and generation.

## CLI Usage

### All-in-one pipeline

```bash
cargo run --release -p smarts-evoliposuction -- \
    all \
    --output-dir ./results \
    --generations 500 \
    --population 200
```

### Individual steps

```bash
# Download + convert
cargo run --release -p smarts-evoliposuction -- download --dest LMSD.sdf.zip

# Split into positive/negative SMILES pairs
cargo run --release -p smarts-evoliposuction -- split --input LMSD.sdf.tsv --output-dir smiles_sets

# Evolve SMARTS patterns from the manifest
cargo run --release -p smarts-evoliposuction -- evolve \
    --manifest smiles_sets/manifest.csv \
    --output smarts_results.csv \
    --population 200 \
    --generations 500 \
    --stagnation 50 \
    --resume
```

### CLI flags

  | Flag                | Default | Command     | Description                                                            |
  | ------------------- | ------- | ----------- | ---------------------------------------------------------------------- |
  | `--population`      | `200`   | All, Evolve | GA population size (individuals per generation)                        |
  | `--generations`     | `500`   | All, Evolve | Max generations per class                                              |
  | `--stagnation`      | `50`    | Evolve      | Stop after N generations with no MCC improvement                       |
  | `--seed`            | `None`  | All, Evolve | Fixed RNG seed for reproducibility                                     |
  | `--resume`          | `false` | Evolve      | Skip classes already in the output CSV                                 |
  | `--level`           | `None`  | Evolve      | Filter to `category`, `main_class`, or `subclass` rows                 |
  | `--match-timeout`   | `5`     | All, Evolve | Per-SMARTS evaluation timeout in seconds (0 = none)                    |
  | `--smiles-cap`      | `2000`  | Split       | Max SMILES per positive/negative set after shuffling                   |
  | `--max-negatives`   | `5000`  | Split       | Hard cap on negatives per class                                        |
  | `--no-tui`          | `false` | All, Evolve | Disable progress output; run classes in parallel                       |
  | `--with-subclasses` | `false` | All, Evolve | Include subclass-level evolution (default: category + main_class only) |

## Performance Tuning

The default parameters (`population=100`, `generations=500`, `stagnation=50`,
`smiles-cap=2000`, `max-negatives=5000`) are tuned for good quality with
reasonable runtimes. The following optimizations are **already built into the
CLI** --- no manual intervention required:

### 1. Parallel across classes (built-in, zero quality impact)

`evolve_step` uses `rayon` to evolve all classes concurrently by default. The
`smarts-evolution` library already parallelises *within* each generation via
rayon; `smarts-evoliposuction` additionally parallelises *across* classes.
Rayon's work-stealing scheduler shares the same thread pool, so total thread
count stays at `num_cpus()` --- no over-subscription.

**Use `--no-tui` for parallel batch mode** (no progress output, runs all classes
concurrently):

```bash
# Sequential with text progress (default — prints generation/best MCC to stderr)
cargo run --release -p smarts-evoliposuction -- evolve --manifest manifests/manifest.csv --resume

# Parallel batch (no progress output, all classes at once)
cargo run --release -p smarts-evoliposuction -- evolve --manifest manifests/manifest.csv --no-tui --resume
```

### 2. Reduced match time limit (built-in, zero quality impact)

The default `smarts-evolution` per-SMARTS `match_time_limit` is **30 seconds**.
We now set it to **5 seconds** (`--match-timeout`). MCC is a binary true/false
metric per molecule, so 5 seconds is plenty for patterns matching ≤1000
molecules --- you either match or you don't. The 30s limit was wasting time on
complex patterns that time out and contribute no fitness information.

### 3. Evaluation set limits (quality vs. speed tradeoff)

SMILES sets are capped at **2000** per side (`--smiles-cap`) and negatives are
capped at **5000** per class (`--max-negatives`). For most classes, \~500
positives + \~1000 negatives is sufficient --- MCC saturates well before these
counts because the metric is dominated by true/false positive/negative counts,
not raw volume. However, for very large classes (e.g. fatty acyls with thousands
of members), using more examples improves convergence.

To reduce runtime, lower these caps:

````bash
cargo run --release -p smarts-evoliposuction -- split --smiles-cap 500 --max-negatives 1000

### 4. Start with categories and main classes, then add subclasses (workflow)

Categories (the 8 LIPID MAPS families, e.g. `Fatty Acyls`) and main classes
(e.g. `FA01`) have thousands of lipids and converge quickly. Subclasses
(e.g. `FA01`) have fewer members, harder negatives, and take much longer.
By default, `evolve` and `all` only process `category` + `main_class` rows.
Run subclasses separately with `--with-subclasses`:

```bash
# Step 1: category + main classes (default — fast, broad coverage)
cargo run --release -p smarts-evoliposuction -- evolve --manifest manifest.csv --output results.csv --resume

# Step 2: subclasses (slower, more specific)
cargo run --release -p smarts-evoliposuction -- evolve --manifest manifest.csv --output results.csv --level subclass --with-subclasses --resume

# Or filter to a single level:
cargo run --release -p smarts-evoliposuction -- evolve --manifest manifest.csv --level category --resume
````

### Parameter Reference

  | Parameter         | Default | Low/budget | High/quality | What it affects                                        |
  | ----------------- | ------- | ---------- | ------------ | ------------------------------------------------------ |
  | `--population`    | `100`   | `50`       | `200`        | Genetic diversity; higher = fewer local optima         |
  | `--generations`   | `500`   | `200`      | `1000`       | Evolution time; higher = more convergence              |
  | `--stagnation`    | `50`    | `30`       | `100`        | Early stopping patience; higher = more time at plateau |
  | `--smiles-cap`    | `2000`  | `500`      | `5000`       | Training set size per side; higher = more signal       |
  | `--max-negatives` | `5000`  | `1000`     | `10000`      | Negative set cap per class                             |
  | `--match-timeout` | `5`     | `3`        | `10`         | Per-SMARTS eval timeout; safety net only               |

**Recommendation:** Increase `--population` first (diversity bottleneck), then
`--generations` (convergence). Training set sizes (`--smiles-cap`) mainly matter
for large categories/families with thousands of lipids.

### 5. Reduce population and generations (quality trade-off)

`population_size=200` is generous. For well-separated chemical classes,
`population=50` often finds MCC > 0.9 just as reliably, cutting runtime by 4×.
Similarly, `generations=200` instead of 500 still allows convergence while
reducing time:

```bash
cargo run --release -p smarts-evoliposuction -- evolve --manifest manifest.csv --population 50 --generations 200 --resume
```

### 7. Incremental pipeline (skip existing)

The `all` command automatically skips steps that already have outputs:

- **Download**: skips if `LMSD.sdf.zip` already exists and is non-empty (still
  converts to TSV if the TSV is missing)
- **Split**: skips if `smiles_sets/manifest.csv` already exists
- **Evolve**: uses `--resume` by default; skips classes already in
  `smarts_results.csv`

This makes re-runs fast --- just re-run `all` after a crash or partial
completion:

```bash
cargo run --release -p smarts-evoliposuction -- all --output-dir ./results
```

The `all` pipeline always resumes --- it skips download, split, and
already-completed classes automatically.

## Results & CSV output

Two CSV files are produced per class:

  | File                       | Description                                                                                                                        |
  | -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
  | `smarts_results.csv`       | One row per class — best SMARTS, MCC, coverage, generations, status, and link to tested-SMARTS CSV                                 |
  | `<slug>_tested_smarts.csv` | All evaluated SMARTS for that class with metrics (`smarts`, `mcc`, `smarts_len`, `coverage_score`, `limit_exceeded`, `generation`) |

**Tested-SMARTS capture (both modes):**

Both progress and parallel modes use the `TestedSmartsObserver` which intercepts
every genome evaluation during evolution. This captures **all** evaluated SMARTS ---
including duplicates and ties --- with their generation numbers. No
deduplication is performed, so the per-class CSV contains the complete history
of every SMARTS evaluated.

The per-class CSV is sorted by: 1. MCC (descending) --- most selective patterns
first 2. `coverage_score` (descending) --- broader coverage preferred on ties 3.
`generation` (ascending) --- earlier discoveries preferred on ties

**Progress mode (default)**: Sequential, one class at a time. Prints text
progress to stderr (generation, best MCC, best SMARTS). Safe in any terminal.

**Parallel mode (`--no-tui`)**: All classes evolved concurrently via rayon. No
stderr progress output (to avoid interleaved spam). Use `evolve` (not `all`) if
you want to see per-class progress.

## Integration with lipid-selecto-rs

The evolved SMARTS CSV is the single output file. Load it into
`lipid-selecto-rs` via `LipidRuleLibrary::add_evolved_rules_from_csv()`:

```rust
use lipid_selecto_rs::rules::LipidRuleLibrary;

let mut library = LipidRuleLibrary::defaults();
library.add_evolved_rules_from_csv("smarts_results.csv")?;
```

Rules with `status == "ok"` and a non-empty `best_smarts` field are loaded and
pre-compiled for fast matching. Each rule's `description` field includes the MCC
score, coverage, and generation count so you can see which patterns are most
selective.

See [`../../README.md`](../../README.md) (parent `lipidmaps/` README) for the
full end-to-end workflow documentation.

## External Dependencies

This project builds on the excellent work of **Luca Cappelletti**
(@LucaCappelletti94), Earth Metabolome Initiative:

- [`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution) ---
  Genetic algorithm for evolving SMARTS patterns
- [`smarts-rs`](https://github.com/earth-metabolome-initiative/smarts-rs) ---
  High-performance SMARTS matching library
- [`smiles-parser`](https://github.com/earth-metabolome-initiative/smiles-parser) ---
  SMILES string parser

Credit: Luca Cappelletti, Earth Metabolome Initiative.
