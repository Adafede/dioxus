# lipid-selecto-rs

`lipid-selecto-rs` is a Dioxus web app that selects **lipid** spectra from an
uploaded MGF file (an MGF that already carries a `SMILES=` / `FORMULA=` per
spectrum) and lets you download the resulting **lipid-only MGF**.

It uses the pure‑Rust [`chematic`](https://crates.io/crates/chematic) toolkit to
parse each spectrum's SMILES into a molecular graph, then classifies the
molecule as a lipid using a combination of:

- a **structural / substructure check** (a long aliphatic carbon chain of ≥ 8
  atoms, which is the universal hallmark of a fatty‑acyl chain) **plus** a
  polar head group (carboxylic acid, ester, amide, phosphate, sulfate or a
  sphingoid amino‑alcohol),
- a **formula signature** for steroids / sterols (the fused tetracyclic skeleton
  that lacks a classic polar head group),

covering fatty acyls, glycerolipids, glycerophospholipids, sphingolipids and
sterols. Molecules flagged as cofactors / adducts without a long aliphatic
chain (e.g. ATP, NAD⁺, glucose‑6‑phosphate, choline) are correctly **kept out**.

After selection the app:

- reports a per‑class summary,
- offers the **filtered MGF for download**, and
- renders a 2D structure diagram for each selected lipid.

## Run locally

```bash
dx serve --package lipid-selecto-rs
```

## Build for the website

```bash
dx build --release --platform web --package lipid-selecto-rs
```

## Notes

- SMILES is preferred for classification; when a SMILES is absent or cannot be
  parsed, the `FORMULA=` value is used as a fallback.
- Structure diagrams are generated on‑the‑fly in the browser with a force‑
  directed layout; no external service or native library is required.
