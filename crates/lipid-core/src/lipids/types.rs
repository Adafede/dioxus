// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Lipid classification domain types.

use crate::family::LipidFamily;

/// Elemental composition extracted from a molecule or formula string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ElementCounts {
    /// Carbon count.
    pub carbon: u32,
    /// Hydrogen count.
    pub hydrogen: u32,
    /// Nitrogen count.
    pub nitrogen: u32,
    /// Oxygen count.
    pub oxygen: u32,
    /// Phosphorus count.
    pub phosphorus: u32,
    /// Sulfur count.
    pub sulfur: u32,
    /// Combined halogen count (`F`, `Cl`, `Br`, `I`).
    pub halogens: u32,
}

impl ElementCounts {
    /// Double-bond equivalents (`C - H/2 - X/2 + N/2 + 1`).
    #[must_use]
    pub fn double_bond_equivalent(&self) -> f64 {
        let c = f64::from(self.carbon);
        let h = f64::from(self.hydrogen);
        let n = f64::from(self.nitrogen);
        let x = f64::from(self.halogens);
        c - h / 2.0 - x / 2.0 + n / 2.0 + 1.0
    }

    /// Molecular formula formatted in Hill order (e.g. `C16H32O2`).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn formula_string(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        for (symbol, count) in [
            ("C", self.carbon),
            ("H", self.hydrogen),
            ("Cl", self.halogens),
            ("N", self.nitrogen),
            ("O", self.oxygen),
            ("P", self.phosphorus),
            ("S", self.sulfur),
        ] {
            if count == 1 {
                out.push_str(symbol);
            } else if count > 1 {
                let _ = write!(out, "{symbol}{count}");
            }
        }
        out
    }
}

/// Result of classifying a single spectrum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LipidClassification {
    /// Broad LIPID MAPS family assignment.
    pub family: LipidFamily,
    /// `true` when derived from parsed SMILES structure, `false` for formula fallback.
    pub derived_from_smiles: bool,
}
