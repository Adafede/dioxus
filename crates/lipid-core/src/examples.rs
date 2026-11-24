// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Curated dataset of 204 example lipids covering all eight LIPID MAPS categories.

mod fatty_acyls;
mod glycerolipids;
mod glycerophospholipids;
mod polyketides;
mod prenol_lipids;
mod saccharolipids;
mod sphingolipids;
mod sterol_lipids;

const FAMILIES: &[&[(&str, &str)]] = &[
    fatty_acyls::LIPIDS,
    glycerolipids::LIPIDS,
    glycerophospholipids::LIPIDS,
    sphingolipids::LIPIDS,
    sterol_lipids::LIPIDS,
    prenol_lipids::LIPIDS,
    saccharolipids::LIPIDS,
    polyketides::LIPIDS,
];

/// Returns the 204 curated example lipids formatted as `"SMILES\tname"` lines.
#[must_use]
pub fn example_smiles() -> Vec<String> {
    FAMILIES
        .iter()
        .copied()
        .flatten()
        .map(|(id, smiles)| format!("{smiles}\t{id}"))
        .collect()
}
