# Lipid Classification Rules - User Guide

This document explains the lipid classification rules in `lipid-selecto-rs`
covering **all 8 LIPID MAPS categories** with 22 chemical classes.

## Quick Start

The application comes with **22 chemical classes** organized into 8 structural
families:

- **Fatty Acyls (FA)**: Saturated (FA), Monounsaturated (MUFA), Polyunsaturated
  (PUFA)
- **Glycerolipids (GL)**: Triglycerides (TG), Diglycerides (DG), Monoglycerides
  (MG)
- **Glycerophospholipids (GP)**: PC, PE, PS, PI, PG, PA, CL, LPC, LPE
- **Sphingolipids (SP)**: Ceramides (Cer), Sphingomyelins (SM), Hexosylceramides
  (HexCer)
- **Sterol Lipids (ST)**: Cholesterol, Steroids, Sterol derivatives
- **Prenol Lipids (PR)**: Retinoids, Tocopherols, Ubiquinones, Isoprenoids
- **Saccharolipids (SL)**: Lipid A, Lipopolysaccharides
- **Polyketides (PK)**: Statins, Macrolides, Polyketide-derived metabolites

## Built-in Rules Reference

### Fatty Acyls (FA)

Characterized by a terminal carboxylic acid (`COOH`) and a long hydrocarbon
chain. Rules distinguish by degree of unsaturation:

  | Rule     | Criteria                   | SMARTS                                     | Example                        |
  | -------- | -------------------------- | ------------------------------------------ | ------------------------------ |
  | **FA**   | Saturated (0 double bonds) | `[#6]...[#6][#6][#6][#6][CX3](=[OX1])[OH]` | Palmitic C16:0, Stearic C18:0  |
  | **MUFA** | 1 C=C double bond          | `[#6]=[#6]...[CX3](=[OX1])[OH]`            | Oleic C18:1, Palmitoleic C16:1 |
  | **PUFA** | 2+ C=C double bonds        | `[#6]=[#6][#6]=[#6]...[CX3](=[OX1])[OH]`   | Arachidonic C20:4, EPA C20:5   |

### Glycerolipids (GL)

Fatty acyl esters of glycerol (3-carbon backbone). Rules distinguish by acyl
ester count:

  | Rule        | Acyl Count | SMARTS Core                                                                   | Example              |
  | ----------- | ---------- | ----------------------------------------------------------------------------- | -------------------- |
  | **TG(AAA)** | 3          | `[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]` | Triolein             |
  | **DG(AA)**  | 2          | `[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]`                         | 1,2-Dioleoylglycerol |
  | **MG(A)**   | 1          | `[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]`                                  | 1-Oleoylglycerol     |

### Glycerophospholipids (GP)

Glycerol backbone with phosphate group and variable headgroup. Rules distinguish
by: - Headgroup type (choline, ethanolamine, serine, inositol, glycerol, none) -
Acyl ester count (2 = normal, 1 = lyso, 4 = cardiolipin)

  | Rule         | Headgroup      | Acyls | SMARTS Core                                                               | Example                 |
  | ------------ | -------------- | ----- | ------------------------------------------------------------------------- | ----------------------- |
  | **PC(AA)**   | Choline        | 2     | `[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]`                    | DPPC, POPC              |
  | **PE(AA)**   | Ethanolamine   | 2     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]`                 | DPPE, POPE              |
  | **PS(AA)**   | Serine         | 2     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]` | DPPS                    |
  | **PI(AA)**   | Inositol       | 2     | `[PX4](=[OX1])([OX2])([OX2])[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1`  | PdfIns                  |
  | **PG(AA)**   | Glycerol       | 2     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]` | DPPG                    |
  | **PA(AA)**   | None (just P)  | 2     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]`              | PA(16:0/16:0)           |
  | **LPC(A)**   | Choline        | 1     | `[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]` + phosphate                  | LPC(16:0)               |
  | **LPE(A)**   | Ethanolamine   | 1     | `[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]` + phosphate                  | LPE(16:0)               |
  | **CL(AAAA)** | Diphosphatidyl | 4     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2])[CH2X4][OX2]` × 2         | CL(16:0/16:0/16:0/16:0) |

### Sphingolipids (SP)

Sphingoid base (long-chain amine) with amide-linked fatty acid. Rules
distinguish by headgroup attachment:

  | Rule           | Headgroup         | SMARTS Core                                                | Example            |
  | -------------- | ----------------- | ---------------------------------------------------------- | ------------------ |
  | **Cer(AS)**    | None (just amide) | `[NX3][CX3](=[OX1])[CX4]`                                  | Cer(d18:1/16:0)    |
  | **SM(AS)**     | Phosphocholine    | `[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]`                  | SM(d18:1/16:0)     |
  | **HexCer(AS)** | Hexose sugar      | `[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]` | GlcCer(d18:1/16:0) |

### Sterol Lipids (ST)

4-Ring steroid core (cholestane skeleton) with lipophilic side chain.

  | Rule   | Structure      | SMARTS Core                                            | Example                   |
  | ------ | -------------- | ------------------------------------------------------ | ------------------------- |
  | **ST** | Steroid/Sterol | `[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]` | Cholesterol, β-Sitosterol |

### Prenol Lipids (PR)

Isoprenoid-derived (isoprene repeat units). Pattern: repeating C=C conjugation.

  | Rule   | Characteristic         | SMARTS Core              | Example                               |
  | ------ | ---------------------- | ------------------------ | ------------------------------------- |
  | **PR** | Conjugated isoprenoids | `[#6]=[#6][#6]=[#6][#6]` | Retinol (Vit A), α-Tocopherol (Vit E) |

### Saccharolipids (SL)

Lipids with complex carbohydrate core and phosphate/acyl modifications.

  | Rule   | Structure     | SMARTS Core                       | Example           |
  | ------ | ------------- | --------------------------------- | ----------------- |
  | **SL** | Lipid A / LPS | `[#6][OX2][PX4](=[OX1])[OX2][#6]` | Lipid A, LPS core |

### Polyketides (PK)

Large cyclic structures with ketone groups (produced by polyketide synthase).

  | Rule   | Structure      | SMARTS Core                                         | Example                    |
  | ------ | -------------- | --------------------------------------------------- | -------------------------- |
  | **PK** | Macrolide ring | `[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R]...(14+ atoms)` | Atorvastatin, Erythromycin |

## Understanding SMARTS Patterns

**SMARTS (Simplified Molecular Input Line Entry System)** describes chemical
structures and substructures using a simple syntax.

### Common Symbols

  | Symbol     | Meaning                                  |
  | ---------- | ---------------------------------------- |
  | `[X]`      | Atom with specific connectivity          |
  | `[#6]`     | Any carbon atom                          |
  | `[#7]`     | Any nitrogen atom                        |
  | `[#8]`     | Any oxygen atom                          |
  | `[CX3]`    | Carbon with 3 connections                |
  | `[CX4]`    | Carbon with 4 connections                |
  | `(=[OX1])` | Double-bonded oxygen (single attachment) |
  | `[OX2]`    | Oxygen with 2 connections (ether/ester)  |
  | `[NX3]`    | Nitrogen with 3 connections              |
  | `[NX4+]`   | Quaternary nitrogen (charged)            |
  | `~`        | Any bond (single, double, aromatic)      |
  | `=`        | Double bond                              |
  | `,`        | OR operator                              |
  | `;`        | AND operator                             |
  | `!`        | NOT operator                             |
  | `[!R]`     | NOT in a ring                            |
  | `[!a]`     | NOT aromatic                             |

### Example Breakdown

**PC(AA) headgroup:**

```smarts
[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]
│     │      │      │      │
│     │      │      │      └─ Quaternary choline: N+(CH3)3
│     │      │      └─ Two phosphate ester linkages
│     │      └─ Phosphate P=O
│     └─ Phosphorus with 4 connections
└─ Match starts at phosphorus
```

**TG(AAA) backbone:**

```smarts
[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]
│      │    │       │      │   │    │       │      │   │    │       │      │
│      └─ 3 × ester linkage to carbonyls ──────────────┘   └─ Each acyl chain
│         All attached to central carbon (glycerol C)
└─ Central C with 4 connections (one C, three O)
```

## Why Rules Matter

The classification depends on precise SMARTS patterns because:

1. **FA vs MUFA vs PUFA**: Must count C=C double bonds correctly
   - `FA`: No unsaturation
   - `MUFA`: Exactly 1 C=C
   - `PUFA`: 2+ C=C in conjugation

2. **PC vs LPC vs SM**: Must count ester linkages AND identify backbone
   - `PC`: Glycerol + 2 esters + phosphocholine
   - `LPC`: Glycerol + 1 ester + phosphocholine (lyso)
   - `SM`: Sphingoid + 1 amide + phosphocholine

3. **TG vs DG vs MG**: Must count ester linkages exactly
   - Each ester = one acyl group attachment
   - Remaining OH groups indicate mono vs di vs tri

4. **Cer vs SM vs HexCer**: Must detect headgroup attachment
   - `Cer`: Just the amide linkage
   - `SM`: Amide + phosphocholine
   - `HexCer`: Amide + sugar moiety

## Testing Your Rules

To verify a rule works correctly:

1. **Test Positives**: Load examples of known lipids in that class
   - Expected: All marked as matching

2. **Test Negatives**: Load examples of similar but different lipids
   - PC should NOT match SM or LPC
   - MUFA should NOT match PUFA
   - TG should NOT match DG

3. **Check False Positives**: Load complex molecules
   - Steroids should NOT match if using overly broad patterns
   - All examples should match ONLY their intended class

## References

- **LIPID MAPS**: https://www.lipidmaps.org/
- **LIPID MAPS Classification**:
  https://www.lipidmaps.org/databases/lmsd/lipid_groups
- **SMARTS Documentation**:
  https://www.daylight.com/dayhtml/doc/theory/theory.smarts.html
- **Lipid Nomenclature**: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC2920091/

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
