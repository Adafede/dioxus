# smarts-evoliposuction

Batch SMARTS evolution runner for lipid class classification using the
[`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution)
genetic algorithm.

`smarts-evoliposuction` is intentionally excluded from the root Cargo workspace
so its native bioinformatics, HTTP, and ZIP dependencies never affect the WASM
dependency graph. Always invoke Cargo with `--locked` (either from this
directory or from the workspace root with `--manifest-path`).

## Architecture

```text
smarts-evoliposuction/
├── Cargo.toml
├── Cargo.lock
└── src/
    ├── lib.rs         public API re-exports
    ├── main.rs        CLI binary (download / split / evolve / all)
    ├── download.rs    LMSD.sdf.zip download + TSV conversion
    ├── splitting.rs   TSV parsing + positive/negative SMILES splitting
    ├── evolve.rs      smarts-evolution wrapper with per-class error isolation
    ├── manifest.rs    manifest CSV reading/writing
    └── lipidsdl/      vendored SDF parser & async download module
```

Pipeline stages:

1. **Download**: fetches `LMSD.sdf.zip` from LIPID MAPS and converts it to TSV
   via `src/lipidsdl/`. Note that `download --dest LMSD.sdf.zip` writes the
   converted TSV to `dest.with_extension("tsv")` (i.e. `LMSD.sdf.tsv`).
2. **Split**: parses the TSV (`--input` defaults to `LMSD.tsv`) and writes
   balanced positive/negative SMILES files per `category`, `main_class`, and
   optionally `subclass`.
3. **Evolve**: runs `smarts-evolution` on each class manifest row, isolating
   parse/evolution errors per class. By default only `category` and `main_class`
   rows are processed; pass `--with-subclasses` to include subclasses.
4. **Results**: writes `smarts_results.csv` and per-class
   `smarts_results_tested_smarts/<slug>_tested_smarts.csv` sorted by MCC
   (descending), coverage score (descending), and generation (ascending).

## Build and Tests

Upstream `smarts-rs` `main` has moved to `smiles-rs` 0.2.1 while
`smarts-evolution` still depends on `smiles-rs` 0.1.5, so do not run unpinned
resolution updates (`cargo update`); pass `--locked`:

```bash
cargo clippy --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --locked
```

## CLI Usage

Run from this directory (`cargo run --release --locked -- ...`), or pass
`--manifest-path apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/Cargo.toml --locked`:

### All-in-one pipeline

```bash
cargo run --release --locked -- \
    all \
    --output-dir ./results \
    --generations 500 \
    --population 100
```

### Individual steps

```bash
# 1. Download + convert (writes LMSD.sdf.tsv next to --dest)
cargo run --release --locked -- download --dest LMSD.sdf.zip

# 2. Split into positive/negative SMILES pairs
cargo run --release --locked -- split --input LMSD.sdf.tsv --output-dir smiles_sets

# 3. Evolve SMARTS patterns from the manifest
cargo run --release --locked -- evolve \
    --manifest smiles_sets/manifest.csv \
    --output smarts_results.csv \
    --population 100 \
    --generations 500 \
    --stagnation 50 \
    --resume
```

### CLI flags

  | Flag                | Default                           | Command     | Description                                                   |
  | ---                 | ---                               | ---         | ---                                                           |
  | `--dest`            | `LMSD.sdf.zip`                    | Download    | Destination path for downloaded zip                           |
  | `--input`           | `LMSD.tsv`                        | Split       | Input TSV file path                                           |
  | `--output-dir`      | `smiles_sets` (Split) / `.` (All) | Split, All  | Output directory                                              |
  | `--population`      | `100`                             | All, Evolve | GA population size                                            |
  | `--generations`     | `500`                             | All, Evolve | Max generations per class                                     |
  | `--stagnation`      | `50`                              | Evolve      | Stop after N generations without MCC gain                     |
  | `--seed`            | `None`                            | All, Evolve | Fixed RNG seed for reproducibility                            |
  | `--resume`          | `false`                           | Evolve      | Skip classes already present in `--output`                    |
  | `--level`           | `None`                            | Evolve      | Filter to `category`, `main_class`, or `subclass`             |
  | `--match-timeout`   | `5`                               | All, Evolve | Per-SMARTS evaluation timeout in seconds                      |
  | `--smiles-cap`      | `2000`                            | Split       | Max SMILES per positive/negative set                          |
  | `--max-negatives`   | `5000`                            | Split       | Cap on negative SMILES per class                              |
  | `--no-tui`          | `false`                           | All, Evolve | Disable stderr progress output; run classes in parallel       |
  | `--with-subclasses` | `false`                           | All, Evolve | Include `subclass` rows (`category` + `main_class` otherwise) |

## Performance Tuning

Default parameters (`--population 100`, `--generations 500`, `--stagnation 50`,
`--smiles-cap 2000`, `--max-negatives 5000`, `--match-timeout 5`) balance search
depth and runtime:

### 1. Parallel batch mode (`--no-tui`)

Passing `--no-tui` suppresses stderr progress updates and lets `rayon` schedule
across classes concurrently:

```bash
cargo run --release --locked -- evolve --manifest smiles_sets/manifest.csv --no-tui --resume
```

### 2. Per-SMARTS evaluation timeout (`--match-timeout`)

`--match-timeout` defaults to 5 seconds (down from `smarts-evolution`'s
30-second default) to limit time spent on pathological patterns.

### 3. Training set caps (`--smiles-cap` / `--max-negatives`)

Lower caps speed up splitting and evaluation on large classes:

```bash
cargo run --release --locked -- split --input LMSD.sdf.tsv --smiles-cap 500 --max-negatives 1000
```

### 4. Level staging

Categories and main classes converge faster than finer subclasses. Run default
`category` + `main_class` first, then run `--level subclass --with-subclasses`:

```bash
cargo run --release --locked -- evolve --manifest smiles_sets/manifest.csv --output smarts_results.csv --resume
cargo run --release --locked -- evolve --manifest smiles_sets/manifest.csv --output smarts_results.csv --level subclass --with-subclasses --resume
```

### 5. Smaller GA budgets

To trade search depth for faster runs:

```bash
cargo run --release --locked -- evolve --manifest smiles_sets/manifest.csv --population 50 --generations 200 --resume
```

### 6. Incremental resume (`all`)

The `all` subcommand skips `download` if non-empty `LMSD.sdf.zip` already exists
(still converting to `LMSD.sdf.tsv` if missing), skips `split` when
`smiles_sets/manifest.csv` already exists, and resumes `evolve` against existing
`smarts_results.csv` entries.

## Results & CSV Output

  | Artifact                                                | Description                                                                                      |
  | ---                                                     | ---                                                                                              |
  | `smarts_results.csv`                                    | Best evolved SMARTS, MCC, coverage, generation count, and status per class                       |
  | `smarts_results_tested_smarts/<slug>_tested_smarts.csv` | Evaluated SMARTS history sorted by MCC descending, coverage descending, and generation ascending |

## Integration with `lipid-selecto-rs`

Evolved rules are curated offline from `smarts_results.csv` into static
`LmsdEntry` rows in `apps/lipid-selecto-rs/src/chemical_class/lmsd.rs` (exposed
internally inside `lipid-selecto-rs` via `lmsd_classes()` and `lmsd_all()`). The
WASM frontend does not read CSV files at runtime.

See [`../README.md`](../README.md) for the surrounding `lipidmaps/` directory
overview.

## Credits

Built on
[`smarts-evolution`](https://github.com/earth-metabolome-initiative/smarts-evolution),
[`smarts-rs`](https://github.com/earth-metabolome-initiative/smarts-rs), and
[`smiles-rs`](https://github.com/earth-metabolome-initiative/smiles-rs) by Luca
Cappelletti (@LucaCappelletti94).
