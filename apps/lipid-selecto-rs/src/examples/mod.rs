// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Collection of 204 example SMILES covering all LIPID MAPS classes from real data.
//! Generated from `LipidMaps` LMSD dataset.
//! Covers all 8 categories: FA, GL, GP, SP, ST, PR, SL, PK

mod fatty_acyls;
mod glycerolipids;
mod glycerophospholipids;
mod polyketides;
mod prenol_lipids;
mod saccharolipids;
mod sphingolipids;
mod sterol_lipids;

// LIPID MAPS order. A slice of slices rather than one flat array: a `const`
// cannot be built from other consts, so composing the families needs this
// shape. `example_smiles` is the only consumer either way.
#[cfg(target_arch = "wasm32")]
pub(crate) const FAMILIES: &[&[(&str, &str, &str)]] = &[
    fatty_acyls::LIPIDS,
    glycerolipids::LIPIDS,
    glycerophospholipids::LIPIDS,
    sphingolipids::LIPIDS,
    sterol_lipids::LIPIDS,
    prenol_lipids::LIPIDS,
    saccharolipids::LIPIDS,
    polyketides::LIPIDS,
];

/// Convert example list to query format (just SMILES + description lines separated by newlines).
#[must_use]
#[cfg(target_arch = "wasm32")]
pub(crate) fn example_smiles() -> Vec<String> {
    // `copied()` before `flatten`: `FAMILIES.iter()` yields `&&[T]`, and `&&[T]`
    // does not satisfy `IntoIterator`, so the bound on `flatten` is not met
    // without taking the reference back off.
    FAMILIES
        .iter()
        .copied()
        .flatten()
        .map(|(id, smiles, _)| format!("{id}\t{smiles}"))
        .collect()
}
