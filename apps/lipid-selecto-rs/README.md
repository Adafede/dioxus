# lipid-selecto-rs

**A modern, interactive web application for filtering and visualizing lipid mass
spectrometry data.**

`lipid-selecto-rs` analyzes mass spectrometry data in MGF format, classifies
molecules as lipids using chemoinformatics-sound SMARTS patterns, and lets you
download the lipid-only subset. Built with pure Rust using WebAssembly for
blazing-fast performance in the browser.

## Features

**Smart Lipid Classification**

- Backbone-aware detection (glycerol vs sphingoid vs carboxylic acid)
- Chain-aware unsaturation analysis (distinguishes PUFA from MUFA)
- Acyclic-only filtering (rejects aromatic rings, sugars, steroids)
- Supports 25+ lipid classes: PC, PE, PS, PI, PG, PA, TG, DG, MG, Cer, SM, and
  more

**Interactive Analysis**

- Real-time class selection with live gallery/count updates
- Per-class summary with spectrum counts
- Checkbox filtering for flexible export

**Structure Visualization**

- 2D molecular structure rendering for each lipid
- On-the-fly depiction in the browser (no server required)
- Hover to see structure details

**Seamless Export**

- Download filtered MGF containing only selected lipid classes
- Preserves all original metadata
- No data loss or re-serialization drift

**Pure WebAssembly**

- Runs entirely in the browser (no server needed)
- Uses [`chematic`](https://crates.io/crates/chematic) for SMILES parsing
- Processes large files efficiently with cooperative multitasking

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

## Input Format

Upload an MGF (Mascot Generic Format) file with SMILES or FORMULA annotations:

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

**Requirements:**

- Each spectrum must have a `TITLE=` field
- Each spectrum should have either:
  - A valid SMILES string in `SMILES=` field (preferred)
  - A molecular formula in `FORMULA=` field (fallback)

## Classification Strategy

The classifier uses a three-level hierarchy:

**Level 1 --- Structural Family**

- FA (Fatty Acyls)
- GL (Glycerolipids)
- GP (Glycerophospholipids)
- SP (Sphingolipids)

**Level 2 --- Lipid Class**

- PC, PE, PS, PI, PG, PA (phospholipids)
- TG, DG, MG (glycerolipids)
- Cer, SM, HexCer (sphingolipids)
- FA, PUFA, MUFA (fatty acids)

**Level 3 --- Architecture**

- DiAcyl (2 ester groups)
- Plasmalogen (1Z-alkenyl ether + acyl)
- MonoAcyl (lyso compounds)
- AlkylAcyl (ether + acyl)

### Key Features

✓ **Acyclic-only filtering** --- Rejects any molecule with rings (aromatic,
sugar, steroid) ✓ **Backbone-aware** --- Distinguishes PC/LPC/SM by backbone
type + chain count ✓ **Chain-aware PUFA detection** --- Counts unsaturations per
chain (≥2 = PUFA) ✓ **SMARTS fragment library** --- Modular, maintainable
patterns aligned with LIPID MAPS

## Technical Details

### Library Architecture

See [`LIPID_ARCHITECTURE.md`](./LIPID_ARCHITECTURE.md) for detailed design
rationale:

- Modular SMARTS fragment cores
- Backbone detection strategy
- Chain analysis approach
- Production roadmap

### Supported Lipid Classes

The default library includes:

  | Family               | Classes                              |
  | -------------------- | ------------------------------------ |
  | Fatty Acyls          | FA, PUFA, MUFA                       |
  | Glycerolipids        | MG, DG, TG                           |
  | Glycerophospholipids | PC, PE, PS, PI, PG, PA, CL, LPC, LPE |
  | Sphingolipids        | Cer, SM, HexCer                      |

### Fallback Classification

When SMILES cannot be parsed, the app falls back to formula-based detection for: -
Fatty acids - Glycerolipids - Phospholipids

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
