# lipid-selecto-rs

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-28-brightgreen)]()

**A modern, interactive web application for filtering and visualizing lipid mass
spectrometry data.**

`lipid-selecto-rs` analyzes mass spectrometry data (MGF or SMILES), classifies
molecules using LIPID MAPS-aligned SMARTS patterns, and lets you download the
lipid-only subset in the same format. Built with pure Rust using WebAssembly for
blazing-fast performance in the browser, with extensible rule configuration.

## Features

**Smart Lipid Classification (LIPID MAPS-aligned)**

- 30+ built-in SMARTS rules covering FA, GL, GP, SP lipid families
- Backbone-aware detection (glycerol vs sphingoid vs carboxylic acid)
- Chain-aware architecture analysis (DiAcyl, MonoAcyl, Plasmalogen, Ether)
- Acyclic-only filtering (rejects aromatic rings, sugars, steroids)
- Extensible rule system: add custom SMARTS patterns in YAML config
- All rules available as SMARTS fragments for custom development

**Flexible Input/Output Formats**

- Auto-detects input format: MGF (Mascot Generic) or SMILES list
- Auto-detects output format: preserves same format as input
- MGF in → MGF out \| SMILES in → SMILES list out
- Supports FORMULA= fallback when SMILES unavailable

**Interactive Analysis**

- Real-time class selection with live gallery/count updates
- Per-class summary with spectrum counts
- Checkbox filtering for flexible export
- Priority-based rule ordering (configure in YAML)

**Structure Visualization**

- 2D molecular structure rendering for each lipid
- On-the-fly depiction in the browser (no server required)
- Hover to see structure details

**Seamless Export**

- Download filtered MGF containing only selected lipid classes
- Download filtered SMILES list
- Preserves all original metadata
- No data loss or re-serialization drift

**Pure WebAssembly**

- Runs entirely in the browser (no server needed)
- Uses [`chematic`](https://crates.io/crates/chematic) for SMILES parsing
- Processes large files efficiently with cooperative multitasking

**100 Curated Example Lipids**

- Complete dataset covering all major lipid classes
- Real structures from LIPID MAPS
- Available for download, testing, or as reference

## Usage

### Online

Visit [lipid-selecto-rs.princelab.org](https://lipid-selecto-rs.princelab.org)
to use the web app.

### Locally

```bash
dx serve --package lipid-selecto-rs
```
Then open `http://localhost:8080` in your browser.

### Build for Deployment

```bash
dx build --release --platform web --package lipid-selecto-rs
```

## Input Formats

The application auto-detects format and preserves it for output.

### MGF (Mascot Generic Format)

Upload an MGF file with SMILES or FORMULA annotations:

```
BEGIN IONS
TITLE=spectrum_1
PEPMASS=256.2
CHARGE=1-
SMILES=CCCCCCCCCCCCCCCC(=O)O
END IONS

BEGIN IONS
TITLE=spectrum_2
PEPMASS=500.5
CHARGE=1+
FORMULA=C38H78O2
END IONS
```

**Requirements:** - Each spectrum must have a `TITLE=` field - Each spectrum
should have either: - A valid SMILES string in `SMILES=` field (preferred) - A
molecular formula in `FORMULA=` field (fallback)

**Output:** Filtered MGF with same structure and metadata preserved

### SMILES List

Upload a plain text file with SMILES strings (one per line):

```
CCCCCCCCCCCCCCCC(=O)O
CCCCCCCCCCCCCCCC(=O)OC(COP(=O)(O)OCC[N+](C)(C)C)OC(=O)CCCCCCCCCCCCCCCC
CCCCCCCCCCCCCCCCCC(=O)N[C@H](CO)[C@H](O)[C@H]=C[C@@H](O)CCCCCCCCCCCCCCCC
```

Optional: Tab-separated ID and description:
```
FA_16_0	CCCCCCCCCCCCCCCC(=O)O	Palmitic acid
PC_32_0	CCCCCCCCCCCCCCCC(=O)OC(COP(=O)(O)OCC[N+](C)(C)C)OC(=O)CCCCCCCCCCCCCCCC	PC 16:0/16:0
```

**Output:** Filtered SMILES list matching input format

## Classification Strategy

### Customization

Classification rules are now **fully configurable**. See
[`RULES_GUIDE.md`](./RULES_GUIDE.md) for:

- How to add custom lipid classes
- SMARTS pattern syntax and examples
- Rule priority configuration
- Troubleshooting matching issues

### Built-in Rules (LIPID MAPS-aligned)

The default rule set covers:

  | Family                        | Classes                                           | Count |
  | ----------------------------- | ------------------------------------------------- | ----- |
  | **FA** (Fatty Acyls)          | FA, MUFA, PUFA                                    | 3     |
  | **GL** (Glycerolipids)        | TG(AAA), DG(AA), MG(A)                            | 3     |
  | **GP** (Glycerophospholipids) | PC, PE, PS, PI, PG, PA, CL, LPC, LPE              | 9+    |
  | **SP** (Sphingolipids)        | Cer, SM, HexCer                                   | 3+    |
  | **Architectures**             | DiAcyl, MonoAcyl, Plasmalogen, AlkylAcyl, DiEther | 5     |

For complete reference, see [`RULES_GUIDE.md`](./RULES_GUIDE.md).

**Example dataset:** Download 100 curated SMILES from
`assets/example_lipids.smi`

### Performance

- **Acyclic gating** as first filter (rejects 90%+ non-lipids instantly)
- **Cooperative multitasking** for responsive UI on large files
- **Pre-computed class matches** to avoid redundant SMARTS evaluation
- **Efficient gallery rendering** with lazy loading for 100+ structures

## Dependencies

- **`chematic`** --- Pure-Rust cheminformatics for SMILES/SMARTS
- **`dioxus`** --- Rust frontend framework (compiles to WebAssembly)
- **`web-sys`** --- Bindings to browser APIs (file handling, DOM)

No external services or native binaries required.

## Notes

- **SMILES is preferred** for accurate classification. Formula-based detection
  is a fallback for unstructured spectra.
- **All processing happens in the browser** --- your data never leaves your
  computer.
- **Large files** (1000+ spectra) are handled efficiently with progress updates.
- **Structure diagrams** are generated on-the-fly; no external rendering service
  is required.

## Building

### Requirements

- Rust 1.95+
- Dioxus CLI (`dx`)

### Commands

```bash
# Development
dx serve --package lipid-selecto-rs

# Release build
dx build --release --platform web --package lipid-selecto-rs

# Run tests
cargo test -p lipid-selecto-rs

# Format code
cargo fmt -p lipid-selecto-rs

# Lint
cargo clippy -p lipid-selecto-rs -- -D warnings
```

## Testing

```bash
cargo test --lib -p lipid-selecto-rs
```

All 24+ unit tests pass, covering:

- SMILES parsing and classification
- MGF block extraction
- Gallery rendering
- SMARTS pattern matching
- Chemical class composition
