# Lipid Classification Rules - User Guide

This document explains how to use and customize the lipid classification rules
in `lipid-selecto-rs`.

## Quick Start

The application comes with **30+ pre-configured LIPID MAPS-aligned rules**
covering:

- **Fatty Acids (FA)**: Saturated (FA), Monounsaturated (MUFA), Polyunsaturated
  (PUFA)
- **Glycerolipids (GL)**: Triglycerides (TG), Diglycerides (DG), Monoglycerides
  (MG)
- **Glycerophospholipids (GP)**: PC, PE, PS, PI, PG, PA, CL + Lyso and Ether
  variants
- **Sphingolipids (SP)**: Ceramides (Cer), Sphingomyelins (SM), Hexosylceramides
  (HexCer)

## Built-in Rules Reference

### Fatty Acyls (FA)

  | Rule   | Description                       | Example                         |
  | ------ | --------------------------------- | ------------------------------- |
  | `FA`   | Saturated or monounsaturated FA   | Palmitic (C16:0), Oleic (C18:1) |
  | `MUFA` | Monounsaturated (1 double bond)   | Oleic, Palmitoleic              |
  | `PUFA` | Polyunsaturated (≥2 double bonds) | Arachidonic, EPA, DHA           |

### Glycerolipids (GL)

  | Rule      | Description                     | Example           |
  | --------- | ------------------------------- | ----------------- |
  | `TG(AAA)` | Triacylglycerol (3 acyl groups) | Olein (C54H104O6) |
  | `DG(AA)`  | Diacylglycerol (2 acyl groups)  | C36H70O5          |
  | `MG(A)`   | Monoacylglycerol (1 acyl group) | 1-Oleoylglycerol  |

### Glycerophospholipids (GP)

  | Rule       | Description                         | Example                 |
  | ---------- | ----------------------------------- | ----------------------- |
  | `PC(AA)`   | Phosphatidylcholine (diacyl)        | DPPC, POPC              |
  | `PE(AA)`   | Phosphatidylethanolamine (diacyl)   | DPPE, POPE              |
  | `PS(AA)`   | Phosphatidylserine (diacyl)         | DPPS                    |
  | `PI(AA)`   | Phosphatidylinositol                | PdfIns                  |
  | `PG(AA)`   | Phosphatidylglycerol                | DPPG                    |
  | `PA(AA)`   | Phosphatidic acid                   | PA(16:0/16:0)           |
  | `LPC(A)`   | Lysophosphatidylcholine (mono)      | LPC(16:0)               |
  | `LPE(A)`   | Lysophosphatidylethanolamine (mono) | LPE(16:0)               |
  | `CL(AAAA)` | Cardiolipin (4 acyl groups)         | CL(16:0/16:0/16:0/16:0) |

### Sphingolipids (SP)

  | Rule         | Description             | Example            |
  | ------------ | ----------------------- | ------------------ |
  | `Cer(AS)`    | Ceramide (amide-linked) | Cer(d18:1/16:0)    |
  | `SM(AS)`     | Sphingomyelin           | SM(d18:1/16:0)     |
  | `HexCer(AS)` | Hexosylceramide         | GlcCer(d18:1/16:0) |

## Understanding Rule Structure

Each rule is defined by:

```yaml
lipid_classes:
  - name: "PC(AA)"              # Unique rule identifier
    family: "GP"                # Structural family (FA/GL/GP/SP)
    architecture: "DiAcyl"      # Molecular architecture
    description: "..."          # Human-readable description
    smarts: "[PX4]......"       # SMARTS pattern for matching
    color: "#7c3aed"           # Display color in UI
    priority: 10                # Matching priority (higher = checked first)
```

### SMARTS Pattern Explanation

SMARTS (Simplified Molecular Input Line Entry System) is a language for
describing chemical structures.

**Key symbols:**

- `[X]` - Atom with specific connectivity
- `(=[OX1])` - Double bond to oxygen with single connection
- `[OX2]` - Oxygen with two connections (ester/ether)
- `[#6]` - Any carbon
- `[!a]` - Not aromatic
- `[!R]` - Not in a ring
- `~` - Any bond type
- `,` - Logical OR
- `;` - Logical AND (implicit)

**Example breakdown:**

```
[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]
└─────────────────────────────┬──────────────────────────┘
  Phosphate group     Quaternary choline headgroup
  = Core of PC
```

## Adding Custom Rules

### Method 1: Edit the YAML Configuration (Recommended)

1. Open `assets/lipid_rules.yaml`
2. Add a new entry under `lipid_classes`:

```yaml
  - name: "My_Custom_Lipid"
    family: "GL"
    architecture: "DiAcyl"
    description: "My custom lipid rule"
    smarts: "[your_smarts_pattern_here]"
    color: "#ff0000"
    priority: 5
```

3. Save and restart the application
4. Your rule will appear in the class selector

### Method 2: Write SMARTS Patterns

If you're new to SMARTS, follow this workflow:

1. **Draw your molecule** in ChemDraw or similar tool
2. **Get the SMILES** (e.g., from PubChem)
3. **Convert to SMARTS** using a SMARTS editor:
   - Use online tools like: https://www.daylight.com/dayweb/dayphtml/smarts.html
   - Or import SMILES and manually craft patterns

Example: To detect **linoleic acid** (C18:2):

```smiles
CC(C)CC(N)C(=O)O    # too restrictive
CC=CCC=CCCCCCCCC(=O)O  # linoleic acid (exact)
```

Generalize to pattern:

```smarts
[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6][CX3](=[OX1])[OH]
```

### Common SMARTS Fragments

Reuse these building blocks:

```
Acyl ester:    [OX2][CX3](=[OX1])[#6]
Amide:         [NX3][CX3](=[OX1])[#6]
Phosphate:     [PX4](=[OX1])
Choline head:  [NX4+]([CH3])([CH3])[CH3]
Ethanolamine:  [CH2X4][CH2X4][NX3]
Long chain:    [#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]...  (≥8 carbons)
Inositol ring: [C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1
```

## Supported Input/Output Formats

### MGF (Mascot Generic Format)

**Input:**

```mgf
BEGIN IONS
TITLE=spectrum_1
PEPMASS=512.35
CHARGE=1+
SMILES=CCCCCCCCCCCCCCCC(=O)OC(COP(=O)(O)OCC[N+](C)(C)C)OC(=O)CCCCCCCCCCCCCCCC
END IONS

BEGIN IONS
TITLE=spectrum_2
PEPMASS=256.24
CHARGE=1-
FORMULA=C16H32O2
END IONS
```

**Output:** Filtered MGF containing only selected spectra (format preserved)

### SMILES List

**Input:**

```
CCCCCCCCCCCCCCCC(=O)O
CC=CCC=CCC=CCCCCCCC(=O)O
CCCCCCCCCCCCCCCC(=O)OC(COP(=O)(O)OCC[N+](C)(C)C)OC(=O)CCCCCCCCCCCCCCCC
```

**Output:** Filtered SMILES list (one per line)

## Example Dataset

The application includes **100 curated SMILES** covering all lipid classes:

- **18 Fatty Acids**: Saturated, MUFA, PUFA variants
- **14 Triglycerides**: Various chain compositions
- **10 Diglycerides & Monoglycerides**
- **16 Phospholipids**: PC, PE, PS, PI, PG, PA + lyso + plasmalogen + ether
  variants
- **14 Sphingolipids**: Ceramides, sphingomyelins, hexosylceramides
- **2 Cardiolipins**
- **Plus additional architectural variants**

Access via: `lipid_selecto_rs::examples::EXAMPLE_LIPIDS`

## Performance Tips

1. **Priority ordering**: Higher priority rules are checked first (default:
   10→1)
   - Set `priority: 10` for specific lipids you expect frequently
   - Set `priority: 1` for catch-all or rare classes

2. **Rule specificity**: More specific SMARTS patterns are faster
   - Avoid overly broad patterns (e.g., `[#6]` everywhere)
   - Use backbone/headgroup constraints to narrow matches

3. **Large files**: If processing 1000+ spectra:
   - The browser-based mode uses cooperative multitasking (see progress bar)
   - CLI mode (future release) will support batch processing

## Troubleshooting

### Rule not matching expected molecules

1. **Check SMARTS syntax**: Use an online SMARTS editor to verify
2. **Test on known molecules**: Compare with ChemSpider or PubChem
3. **Check priority**: Lower-priority rules won't match if higher rules match
   first
4. **Verify acyclic check**: All rules reject molecules with rings

### False positives

If non-lipids are being matched:

1. **Add stricter headgroup requirement** (e.g., require both phosphate AND
   choline)
2. **Require long aliphatic chains** (≥8 carbons) using `[#6;!a;!R]~...`
3. **Exclude cyclic/aromatic motifs** using `[!R]` or `[!a]`

### Performance issues

1. Simplify SMARTS patterns (remove redundant atoms)
2. Use backbone-first patterns (phosphate/glycerol before chain analysis)
3. Lower the priority of rarely-matched rules

## Alignment with LIPID MAPS

Rules are curated to match the **LIPID MAPS classification system**:

- **Structural families**: FA, GL, GP, SP (+ future: ST, PR, SL, PK)
- **Architectures**: DiAcyl, MonoAcyl, Plasmalogen, AlkylAcyl, DiEther
- **Chain naming**: Fatty acids denoted as `C#:#` (carbons:double bonds)

References: - LIPID MAPS: https://www.lipidmaps.org/ - LIPID MAPS
classification: https://www.lipidmaps.org/databases/lmsd/lipid_groups - SMARTS
tutorial: https://www.daylight.com/dayhtml/doc/theory/theory.smarts.html

## Contributing Rule Improvements

If you discover issues or improvements:

1. Test your SMARTS pattern on known lipids
2. Include the pattern, test molecules, and reasoning
3. Consider adding examples to the `examples.rs` file
4. File an issue or PR with documentation

--------------------------------------------------------------------------------

**Questions?** Check the main README.md for API usage and building instructions.
