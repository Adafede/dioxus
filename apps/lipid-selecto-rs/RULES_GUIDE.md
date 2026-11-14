# Lipid Classification Rules - User Guide

This document explains the SMARTS classification rules compiled into
`lipid-selecto-rs`:

- **Runtime classification (`lmsd_classes` in `src/chemical_class/lmsd.rs`)**:
  86 LIPID MAPS Structure Database (LMSD) main classes across all 8 LIPID MAPS
  categories (`FA`, `GL`, `GP`, `SP`, `ST`, `PR`, `SL`, `PK`).
- **Coarse family reference rules (`#[cfg(test)]` in
  `src/chemical_class/defaults.rs`)**: 22 broad family-level SMARTS patterns
  covering the 8 categories, used in unit tests for coarse lipid class
  comparisons.

## Coarse Family Overview (`src/chemical_class/defaults.rs`)

- **Fatty Acyls (FA)**: Saturated (`FA`), Monounsaturated (`MUFA`),
  Polyunsaturated (`PUFA`)
- **Glycerolipids (GL)**: Triglycerides (`TG(AAA)`), Diglycerides (`DG(AA)`),
  Monoglycerides (`MG(A)`)
- **Glycerophospholipids (GP)**: `PC(AA)`, `PE(AA)`, `PS(AA)`, `PI(AA)`,
  `PG(AA)`, `PA(AA)`, `LPC(A)`, `LPE(A)`, `CL(AAAA)`
- **Sphingolipids (SP)**: Ceramides (`Cer(AS)`), Sphingomyelins (`SM(AS)`),
  Hexosylceramides (`HexCer(AS)`)
- **Sterol Lipids (ST)**: Sterols/Steroids (`ST`)
- **Prenol Lipids (PR)**: Conjugated isoprenoids (`PR`)
- **Saccharolipids (SL)**: Phosphoester saccharolipids (`SL`)
- **Polyketides (PK)**: Large cyclic ketones (`PK`)

### Coarse SMARTS Reference

#### Fatty Acyls (FA)

  | Rule     | Criteria            | SMARTS                                                    | Example                       |
  | ---      | ---                 | ---                                                       | ---                           |
  | **FA**   | Saturated           | `[#6][#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]`       | Palmitic C16:0, Stearic C18:0 |
  | **MUFA** | 1 C=C double bond   | `[#6][#6]=[#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]`  | Oleic C18:1                   |
  | **PUFA** | 2+ C=C double bonds | `[#6][#6]=[#6][#6][#6]=[#6][#6][#6][#6][CX3](=[OX1])[OH]` | Arachidonic C20:4             |

#### Glycerolipids (GL)

  | Rule        | Acyl Count | SMARTS                                                                        | Example              |
  | ---         | ---        | ---                                                                           | ---                  |
  | **TG(AAA)** | 3          | `[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]` | Triolein             |
  | **DG(AA)**  | 2          | `[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]`                         | 1,2-Dioleoylglycerol |
  | **MG(A)**   | 1          | `[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]`                                  | 1-Oleoylglycerol     |

#### Glycerophospholipids (GP)

  | Rule         | Headgroup      | Acyls | SMARTS                                                                              |
  | ---          | ---            | ---   | ---                                                                                 |
  | **PC(AA)**   | Choline        | 2     | `[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]`                              |
  | **PE(AA)**   | Ethanolamine   | 2     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]`                           |
  | **PS(AA)**   | Serine         | 2     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]`           |
  | **PI(AA)**   | Inositol       | 2     | `[PX4](=[OX1])([OX2])([OX2])[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1`            |
  | **PG(AA)**   | Glycerol       | 2     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]`           |
  | **PA(AA)**   | Phosphate      | 2     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]`                        |
  | **LPC(A)**   | Choline        | 1     | `[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][N+;X4]` |
  | **LPE(A)**   | Ethanolamine   | 1     | `[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][NX3]`   |
  | **CL(AAAA)** | Diphosphatidyl | 4     | `[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2])[CH2X4][OX2]`                       |

#### Sphingolipids (SP)

  | Rule           | Headgroup      | SMARTS                                                     |
  | ---            | ---            | ---                                                        |
  | **Cer(AS)**    | Amide          | `[NX3][CX3](=[OX1])[CX4]`                                  |
  | **SM(AS)**     | Phosphocholine | `[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]`                  |
  | **HexCer(AS)** | Hexose sugar   | `[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]` |

#### Sterol Lipids (ST), Prenol Lipids (PR), Saccharolipids (SL), Polyketides (PK)

  | Rule   | Category       | SMARTS                                                                                             |
  | ---    | ---            | ---                                                                                                |
  | **ST** | Sterol Lipids  | `[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]`                                             |
  | **PR** | Prenol Lipids  | `[#6]=[#6][#6]=[#6][#6]`                                                                           |
  | **SL** | Saccharolipids | `[#6][OX2][PX4](=[OX1])[OX2][#6]`                                                                  |
  | **PK** | Polyketides    | `[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R]1` |

## Understanding SMARTS Patterns

**SMARTS (SMiles ARbitrary Target Specification)** describes substructures for
pattern matching (distinct from **SMILES**, Simplified Molecular Input Line
Entry System, which encodes full molecules).

  | Symbol             | Meaning                                           |
  | ---                | ---                                               |
  | `[X]`              | Atom with total connection count                  |
  | `[#6]`             | Any carbon atom (`[#7]` nitrogen, `[#8]` oxygen)  |
  | `[CX3]` / `[CX4]`  | Aliphatic carbon with 3 / 4 connections           |
  | `(=[OX1])`         | Double-bonded oxygen                              |
  | `[OX2]`            | Oxygen with 2 connections (ether or ester)        |
  | `[NX3]` / `[NX4+]` | 3-connected nitrogen / quaternary ammonium cation |
  | `~` / `=`          | Any bond / double bond                            |
  | `,` / `;` / `!`    | OR / AND / NOT logical operators                  |
  | `[!R]` / `[!a]`    | Non-ring / non-aromatic atom                      |

## Updating or Adding Rules

Rules are compiled statically into Rust (`src/chemical_class/lmsd.rs`) rather
than loaded from external YAML files at runtime:

1. Either evolve updated SMARTS patterns from LIPID MAPS using
   [`lipidmaps/smarts-evoliposuction`](./lipidmaps/smarts-evoliposuction/README.md)
   or craft a pattern compatible with `chematic::smarts`.
2. Add or update an `LmsdEntry` in `LMSD_ENTRIES` inside
   `src/chemical_class/lmsd.rs`. Each entry specifies `family`, `code`,
   `description`, `smarts`, and LMSD `count` (ordered by descending database
   count within each family).
3. Verify compilation, SMARTS parsing, and unit tests:

```bash
cargo nextest run -p lipid-selecto-rs --locked
```

## Supported Input and Output Formats

### MGF (Mascot Generic Format)

Streams records bounded by `BEGIN IONS` / `END IONS` and preserves all spectrum
metadata when writing the filtered subset. Reads `SMILES=` lines or `FORMULA=`
fallback headers when SMILES is absent.

### SMILES List

Reads one SMILES entry per line (supporting tab/whitespace-delimited labels) and
outputs matching lines verbatim.

## Example Dataset

`src/examples/` contains **204 curated example lipids** split across 8 family
modules (`fatty_acyls.rs`, `glycerolipids.rs`, `glycerophospholipids.rs`,
`sphingolipids.rs`, `sterol_lipids.rs`, `prenol_lipids.rs`, `saccharolipids.rs`,
`polyketides.rs`), accessed internally via `examples::example_smiles()`.

## References

- LIPID MAPS: <https://www.lipidmaps.org/>
- LIPID MAPS Classification:
  <https://www.lipidmaps.org/databases/lmsd/lipid_groups>
- Daylight SMARTS Theory:
  <https://www.daylight.com/dayhtml/doc/theory/theory.smarts.html>
- Lipid Classification Nomenclature:
  <https://www.ncbi.nlm.nih.gov/pmc/articles/PMC2920091/>
