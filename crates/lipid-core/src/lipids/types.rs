// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Lipid classification domain types.

use crate::family::LipidFamily;

/// Elemental composition extracted from a molecule or formula string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ElementCounts {
    /// Carbon count.
    pub(crate) carbon: u32,
    /// Hydrogen count.
    pub(crate) hydrogen: u32,
    /// Nitrogen count.
    pub(crate) nitrogen: u32,
    /// Oxygen count.
    pub(crate) oxygen: u32,
    /// Phosphorus count.
    pub(crate) phosphorus: u32,
    /// Sulfur count.
    pub(crate) sulfur: u32,
    /// Combined halogen count (`F`, `Cl`, `Br`, `I`).
    pub(crate) halogens: u32,
}

impl ElementCounts {
    /// Double-bond equivalents (`C - H/2 - X/2 + (N + P)/2 + 1`).
    #[must_use]
    pub(crate) fn double_bond_equivalent(&self) -> f64 {
        let carbon = f64::from(self.carbon);
        let hydrogen = f64::from(self.hydrogen);
        let nitrogen = f64::from(self.nitrogen);
        let phosphorus = f64::from(self.phosphorus);
        let halogens = f64::from(self.halogens);
        carbon - hydrogen / 2.0 - halogens / 2.0 + f64::midpoint(nitrogen, phosphorus) + 1.0
    }
}

/// Result of classifying a single spectrum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LipidClassification {
    /// Broad LIPID MAPS family assignment.
    pub family: LipidFamily,
    /// `true` when derived from parsed SMILES structure, `false` for formula fallback.
    pub derived_from_smiles: bool,
}
