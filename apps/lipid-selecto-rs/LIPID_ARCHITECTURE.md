# Lipid Classification Architecture

## Overview

This document describes the hierarchical lipid classification system aligned
with LIPID MAPS standards, covering **all 8 major structural families** and 22
chemical classes.

## LIPID MAPS Categories (8 Families)

### Structural Families

The top level contains the 8 major LIPID MAPS categories:

  | Code   | Family               | Classes                              | Examples                              |
  | ------ | -------------------- | ------------------------------------ | ------------------------------------- |
  | **FA** | Fatty Acyls          | FA, MUFA, PUFA                       | Palmitic, Oleic, Arachidonic          |
  | **GL** | Glycerolipids        | TG(AAA), DG(AA), MG(A)               | Triglycerides, Diglycerides           |
  | **GP** | Glycerophospholipids | PC, PE, PS, PG, PI, PA, LPC, LPE, CL | Phosphatidylcholine, Cardiolipin      |
  | **SP** | Sphingolipids        | Cer(AS), SM(AS), HexCer(AS)          | Ceramides, Sphingomyelins             |
  | **ST** | Sterol Lipids        | ST                                   | Cholesterol, Progesterone             |
  | **PR** | Prenol Lipids        | PR                                   | Retinol (Vit A), α-Tocopherol (Vit E) |
  | **SL** | Saccharipolipids     | SL                                   | Lipid A, LPS Core                     |
  | **PK** | Polyketides          | PK                                   | Statins, Macrolides                   |

## Classification Rules by Family

### Fatty Acyls (FA)

Characterized by a carboxylic acid head and long hydrocarbon chain: - **FA** -
Saturated (no C=C double bonds) - **MUFA** - Monounsaturated (exactly 1 C=C) -
**PUFA** - Polyunsaturated (2+ C=C double bonds)

SMARTS: `[#6][#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]`

### Glycerolipids (GL)

Fatty acyl esters of glycerol (3-carbon backbone): - **TG(AAA)** - Triglyceride
(3 ester groups) - **DG(AA)** - Diglyceride (2 ester groups) - **MG(A)** -
Monoglyceride (1 ester group)

SMARTS core: `[CX4]([OX2][CX3](=[OX1])[#6])`

### Glycerophospholipids (GP)

Glycerol backbone with phosphate headgroup and fatty acyl chains: - **PC** -
Phosphatidylcholine (choline headgroup) - **PE** - Phosphatidylethanolamine
(amine headgroup) - **PS** - Phosphatidylserine (serine headgroup) - **PG** -
Phosphatidylglycerol (glycerol headgroup) - **PI** - Phosphatidylinositol
(inositol headgroup) - **PA** - Phosphatidic Acid (no headgroup) - **LPC** -
Lysophosphatidylcholine (monoacyl + phosphocholine) - **LPE** -
Lysophosphatidylethanolamine (monoacyl + phosphoethanolamine) - **CL** -
Cardiolipin (diphosphatidylglycerol with 4 acyl chains)

SMARTS core: `[PX4](=[OX1])([OX2])([OX2])`

### Sphingolipids (SP)

Sphingoid base backbone with amide-linked fatty acyl: - **Cer(AS)** - Ceramide
(sphingoid + acyl) - **SM(AS)** - Sphingomyelin (ceramide + phosphocholine) -
**HexCer(AS)** - Hexosylceramide (ceramide + hexose sugar)

SMARTS core: `[NX3][CX3](=[OX1])[CX4]`

### Sterol Lipids (ST)

Cholesterol and steroid-based molecules with 4-ring steroid core: - **ST** -
Sterols, Steroids, Cholesterol derivatives

SMARTS: `[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]` (4-ring steroid
core)

### Prenol Lipids (PR)

Isoprenoid-derived lipids (multiples of isoprene units): - **PR** - Carotenoids,
Retinoids, Tocopherols, Ubiquinones, Dolichols

SMARTS: `[#6]=[#6][#6]=[#6][#6]` (conjugated isoprenoid pattern)

### Saccharipolipids (SL)

Lipids with complex carbohydrate and phosphate linkages: - **SL** - Lipid A,
Lipopolysaccharides, Peptidoglycans

SMARTS: `[#6][OX2][PX4](=[OX1])[OX2][#6]` (phosphoester linkage)

### Polyketides (PK)

Large cyclic structures from iterative polyketide biosynthesis: - **PK** -
Statins, Macrolide Antibiotics, Tetracyclines

SMARTS: `[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R]...` (large cyclic with ketone)

## Chemical Class Details

### Fatty Acyls - Saturation Levels

```
Palmitic acid (FA, C16:0)
  CCCCCCCCCCCCCCCC(=O)O
  → 16 carbons, 0 double bonds → FA

Oleic acid (MUFA, C18:1)
  CCCCCCCC=CCCCCCCCCC(=O)O
  → 18 carbons, 1 double bond → MUFA

Arachidonic (PUFA, C20:4)
  CC=CCC=CCC=CCC=CCCCCCCC(=O)O
  → 20 carbons, 4 double bonds → PUFA
```

### Glycerophospholipids - Acyl Count

```
PC (Phosphatidylcholine)
  CCCCCCCCCCCCCCCC(=O)O-C(CO-P-OCC[N+(C)(C)C])-OC(=O)CCCCCCCC=C...
  → Glycerol + Phosphate + Choline + 2 acyl chains → PC

LPC (Lysophosphatidylcholine)
  CCCCCCCCCCCCCCCC(=O)O-C(CO-P-OCC[N+(C)(C)C])(-CO)-OH
  → Glycerol + Phosphate + Choline + 1 acyl chain → LPC (lyso)

CL (Cardiolipin)
  Complex: Two glycerophospholipid units connected by shared glycerol
  → 4 acyl chains total → CL
```

### Sphingolipids - Sugar Modification

```
Ceramide (Cer)
  R-NH-CO-R' with sphingoid backbone
  → Base ceramide

Sphingomyelin (SM)
  Ceramide + Phosphocholine headgroup

Hexosylceramide (HexCer)
  Ceramide + Hexose (glucose/galactose) sugar
```

## Color Scheme (CVD-Friendly Microshades)

  | Family                    | Color Palette       | Hex Codes                        |
  | ------------------------- | ------------------- | -------------------------------- |
  | Fatty Acyls (FA)          | cvd_orange          | #9D654C, #C17754, #F09163        |
  | Glycerolipids (GL)        | cvd_blue            | #098BD9, #56B4E9, #7DCCFF        |
  | Glycerophospholipids (GP) | cvd_green/turquoise | #4E7705–#DDFFA0, #148F77–#43BA8F |
  | Sphingolipids (SP)        | cvd_purple          | #7D3560, #A1527F, #CC79A7        |
  | Sterol Lipids (ST)        | purple              | #6a51a3                          |
  | Prenol Lipids (PR)        | orange              | #ff7f00                          |
  | Saccharipolipids (SL)     | blue                | #4292c6                          |
  | Polyketides (PK)          | green               | #238b45                          |

All palettes are colorblind-accessible (CVD-friendly).

## References

- LIPID MAPS: https://www.lipidmaps.org/
- LIPID MAPS Classification:
  https://www.lipidmaps.org/databases/lmsd/lipid_groups
- SMARTS: https://www.daylight.com/dayhtml/doc/theory/theory.smarts.html
