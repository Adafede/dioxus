// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Lipid classification domain types.
//!
//! `LipidClass` (enum + display label/color), `ElementCounts` (elemental
//! composition with formula-string + double-bond-equivalent helpers), and
//! `LipidClassification` (the full assembled result).  Classification *logic*
//! lives in [`super::classify`]; this module owns only the data.

/// The broad lipid category a spectrum's molecule belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) enum LipidClass {
    /// Free fatty acid (RCOOH) — the simplest fatty acyl.
    FattyAcyl,
    /// Di-/tri-acylglycerol (no phosphorus, ester-linked acyl chains).
    Glycerolipid,
    /// Glycerophospholipids & sphingomyelin (contain phosphorus).
    Glycerophospholipid,
    /// Ceramides, sphingoid bases, gangliosides and other sphingolipids.
    Sphingolipid,
}

impl LipidClass {
    /// LIPID MAPS category name with code, e.g. "Fatty Acyls \[FA]".
    #[must_use]
    pub(crate) const fn lipidmaps_category(self) -> &'static str {
        match self {
            Self::FattyAcyl => "Fatty Acyls [FA]",
            Self::Glycerolipid => "Glycerolipids [GL]",
            Self::Glycerophospholipid => "Glycerophospholipids [GP]",
            Self::Sphingolipid => "Sphingolipids [SP]",
        }
    }
}

/// Elemental composition extracted from a molecule or a formula string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) struct ElementCounts {
    /// Number of carbon atoms.
    pub carbon: u32,
    /// Number of hydrogen atoms.
    pub hydrogen: u32,
    /// Number of nitrogen atoms.
    pub nitrogen: u32,
    /// Number of oxygen atoms.
    pub oxygen: u32,
    /// Number of phosphorus atoms.
    pub phosphorus: u32,
    /// Number of sulfur atoms.
    pub sulfur: u32,
    /// Count of halogen atoms (F, Cl, Br, I).
    pub halogens: u32,
}

impl ElementCounts {
    /// Double-bond equivalents (a.k.a. the index of hydrogen deficiency).
    ///
    /// `BE = C - H/2 - X/2 + N/2 + 1`
    #[must_use]
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn double_bond_equivalent(&self) -> f64 {
        let c = f64::from(self.carbon);
        let h = f64::from(self.hydrogen);
        let n = f64::from(self.nitrogen);
        let x = f64::from(self.halogens);
        c - h / 2.0 - x / 2.0 + n / 2.0 + 1.0
    }

    /// Molecular formula string in Hill order (e.g. `C16H32O2`).
    #[cfg(all(test, target_arch = "wasm32"))]
    #[must_use]
    pub(crate) fn formula_string(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        if self.carbon > 0 {
            out.push('C');
            if self.carbon > 1 {
                let _ = write!(out, "{}", self.carbon);
            }
        }
        let h = self.hydrogen;
        if h == 1 {
            out.push('H');
        } else if h > 1 {
            let _ = write!(out, "H{h}");
        }
        for (symbol, count) in [
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

/// A full result of classifying one spectrum, ready for display.
#[derive(Clone, Debug)]
pub(crate) struct LipidClassification {
    /// The broad lipid class assignment.
    pub class: LipidClass,
    /// Whether the classification came from a structural (SMILES) analysis.
    #[cfg_attr(not(all(test, target_arch = "wasm32")), allow(dead_code))]
    pub derived_from_smiles: bool,
}
