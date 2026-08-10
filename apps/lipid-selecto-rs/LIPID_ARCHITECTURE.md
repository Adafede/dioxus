# Lipid Classification Architecture

## Overview

This document describes the three-level hierarchical lipid classification system
used in lipid-selecto-rs.

## Three-Level Hierarchy

### Level 1: Structural Family

Top-level LIPID MAPS categories representing the fundamental chemical structure:

- **FA** - Fatty Acyls
- **GL** - Glycerolipids
- **GP** - Glycerophospholipids
- **SP** - Sphingolipids
- **ST** - Sterol Lipids
- **PR** - Prenol Lipids
- **SL** - Saccharolipids
- **PK** - Polyketides

### Level 2: Lipid Class

The main categorical distinction within each family:

**Fatty Acyls:**

- FA (Fatty Acid)
- PUFA (Polyunsaturated Fatty Acid)
- MUFA (Monounsaturated Fatty Acid)

**Glycerolipids:**

- MG (Monoacylglycerol)
- DG (Diacylglycerol)
- TG (Triacylglycerol)

**Glycerophospholipids:**

- PC (Phosphatidylcholine)
- PE (Phosphatidylethanolamine)
- PS (Phosphatidylserine)
- PI (Phosphatidylinositol)
- PG (Phosphatidylglycerol)
- PA (Phosphatidic Acid)
- CL (Cardiolipin)
- LPC (Lysophosphatidylcholine)
- LPE (Lysophosphatidylethanolamine)

**Sphingolipids:**

- Cer (Ceramide)
- SM (Sphingomyelin)
- HexCer (Hexosylceramide)

### Level 3: Molecular Architecture

The sub-class distinguishing how the radyl groups are attached:

- **DiAcyl** - Two fatty acyl ester groups (standard)
- **AlkylAcyl** - Ether + acyl (1 ether, 1 ester)
- **AcylAlkyl** - Acyl + ether (1 ester, 1 ether)
- **Plasmalogen** - 1Z-alkenyl ether + acyl (plasmalogen)
- **DiEther** - Two ether-linked chains
- **MonoAcyl** - Single acyl group (lyso compounds)

### Examples

```
PC
├── PC(AA)      - Diacyl-PC
├── PC(O-)      - Alkyl-acyl-PC (1 ether, 1 acyl)
├── PC(P-)      - Plasmalogen-PC
├── LPC(A)      - Lysophosphatidylcholine
└── ...

PE
├── PE(AA)      - Diacyl-PE
├── PE(O-)      - Alkyl-acyl-PE
├── PE(P-)      - Plasmalogen-PE
├── LPE(A)      - Lysophosphatidylethanolamine
└── ...

TG
├── TG(AAA)     - Triacylglycerol
├── TG(AAB)     - Diacyl + 1 other
└── ...
```

## SMARTS Fragment Library

Rather than hardcoding full patterns, we define reusable SMARTS cores that
represent actual chemical structures:

### Functional Groups

- `acyl`: `[CX3](=[OX1])[#6]` --- Carbonyl + carbon
- `ester`: `[OX2][CX3](=[OX1])[#6]` --- Ester linkage (O-C(=O)-R)
- `amide`: `[NX3][CX3](=[OX1])[#6]` --- Amide linkage (N-C(=O)-R)

### Backbones

- `glycerol_3C`: `[CH2X4][CHX4][CH2X4]` --- 3-carbon glycerol core
- `phospho`: `[P;X4](=[OX1])` --- Phosphate group

### Headgroups

- `choline`: `[CH2X4][CH2X4][N+;X4]([CH3])([CH3])[CH3]` --- Quaternary choline
- `ethanolamine`: `[CH2X4][CH2X4][NX3;H2,H1,H0]` --- Primary/secondary amine

### Ring Systems

- `inositol`: `[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1` --- Cyclohexane ring

### Ether/Plasmalogen

- `ether`: `[#6][OX2][#6]` --- C-O-C ether linkage
- `plasmalogen_ether`: `[#6][OX2][CHX3]=[CHX3][#6]` --- Vinylether (1Z-alkenyl)

## Classification Strategy

### Step 1: Acyclic Gating

- First check: Does molecule contain rings? → Reject if yes
- Lipids are acyclic (no aromatic rings, no sugar rings, no steroids)

### Step 2: Backbone Detection

- Identify key structural elements using SMARTS
- Determine structural family (FA, GL, GP, SP)

### Step 3: Chain Analysis

- For each radyl group attachment (ester/ether/amide):
  - Traverse the hydrocarbon chain
  - Count carbons
  - Count and position unsaturations
  - Detect modifications (OH, OOH, epoxide)
  - Distinguish ether vs. plasmalogen

### Step 4: Classification

- Combine backbone type + chain composition
- Assign class (PC, PE, TG, etc.)
- Assign architecture (DiAcyl, Plasmalogen, MonoAcyl, etc.)

## Important Distinctions

### FA vs PUFA vs MUFA

**Do NOT** rely solely on SMARTS matching for unsaturation classification.

**Wrong approach:**
```
molecule.HasSubstructMatch(PUFA_SMARTS)  // finds 2+ C=C sequence
→ classified as PUFA
```

**Correct approach:**
```
acyl_chains = analyze_chains(molecule)
for chain in acyl_chains:
    if len(chain.double_bonds) >= 2:
        → PUFA
    elif len(chain.double_bonds) == 1:
        → MUFA
    else:
        → FA
```

### PC vs LPC vs SM

**Do NOT** classify based on phosphocholine headgroup alone.

**Wrong approach:**

```
if has_phosphocholine():
    classify_as(PC)  // Also matches LPC and SM!
```

**Correct approach:** \`\`\` backbone = identify_backbone() // glycerol vs.
sphingoid acyl_count = count_acyl_esters()

if backbone == glycerol: if acyl_count == 2: → PC elif acyl_count == 1: → LPC
elif backbone == sphingoid: → SM \`\`\`

### Ether vs Acyl vs Plasmalogen

**Glycerophospholipid with:**

- 2 ester linkages (C-O-C(=O)-R) → DiAcyl-PC, DiAcyl-PE
- 1 ether (C-O-C) + 1 ester → Alkyl-acyl-PC (O- designation)
- 1 vinyl-ether (C-O-C=C) + 1 ester → Plasmalogen-PC (P- designation)

## Future Enhancements

### YAML/JSON Specification

For production use, define the classifier as machine-readable specifications:

```yaml
lipid_classes:
  - name: PC
    family: GP
    smarts_components:
      backbone:
        - glycerol_3C
        - phospho
      headgroup: choline
      acyl_count: 2
    architectures:
      - DiAcyl
      - AlkylAcyl
      - Plasmalogen
      - DiEther
      - MonoAcyl
```

This allows:

- Maintaining patterns without code changes
- Version control of classification rules
- Easy addition of new classes/architectures
- Integration with LIPID MAPS
- Machine-readable documentation

### Chain-Aware Pattern Composition

Build complex patterns from chain analysis:

```python
def build_pc_pattern(architecture):
    backbone = glycerol_3C_with_phosphocholine
    if architecture == DiAcyl:
        return f"{backbone}[OX2]{acyl}[OX2]{acyl}"
    elif architecture == Plasmalogen:
        return f"{backbone}[OX2]{plasmalogen_ether}[OX2]{acyl}"
    else:
        # ...
```

This keeps SMARTS maintainable while supporting diverse architectures.

## References

- LIPID MAPS: https://www.lipidmaps.org/
- LIPID MAPS Classification:
  https://www.lipidmaps.org/databases/lmsd/lipid_groups
- SMILES/SMARTS: https://www.daylight.com/dayhtml/doc/theory/theory.smarts.html
