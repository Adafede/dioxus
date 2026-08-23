//! Lipid classification domain types.
//!
//! `LipidClass` (enum + display label/color), `ElementCounts` (elemental
//! composition with formula-string + double-bond-equivalent helpers), and
//! `LipidClassification` (the full assembled result).  Classification *logic*
//! lives in [`super::classify`]; this module owns only the data.

use std::fmt::Write;

/// The broad lipid category a spectrum's molecule belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LipidClass {
    /// Free fatty acid (RCOOH) — the simplest fatty acyl.
    FattyAcyl,
    /// Di-/tri-acylglycerol (no phosphorus, ester-linked acyl chains).
    Glycerolipid,
    /// Glycerophospholipids & sphingomyelin (contain phosphorus).
    Glycerophospholipid,
    /// Ceramides, sphingoid bases, gangliosides and other sphingolipids.
    Sphingolipid,
    /// Steroids / sterols (fused tetracyclic skeleton).
    Sterol,
    /// A molecule that matched the structural formula but not a specific class.
    Other,
}

impl LipidClass {
    /// Human-readable label for UI display.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FattyAcyl => "Fatty acyl",
            Self::Glycerolipid => "Glycerolipid",
            Self::Glycerophospholipid => "Glycerophospholipid",
            Self::Sphingolipid => "Sphingolipid",
            Self::Sterol => "Sterol",
            Self::Other => "Lipid",
        }
    }

    /// LIPID MAPS category name with code, e.g. "Fatty Acyls [FA]".
    #[must_use]
    pub const fn lipidmaps_category(self) -> &'static str {
        match self {
            Self::FattyAcyl => "Fatty Acyls [FA]",
            Self::Glycerolipid => "Glycerolipids [GL]",
            Self::Glycerophospholipid => "Glycerophospholipids [GP]",
            Self::Sphingolipid => "Sphingolipids [SP]",
            Self::Sterol => "Sterol Lipids [ST]",
            Self::Other => "Other Lipids [-]",
        }
    }

    /// CSS background color used by the classification badge in the UI.
    #[must_use]
    pub const fn color(self) -> &'static str {
        match self {
            Self::FattyAcyl => "#2563eb",
            Self::Glycerolipid => "#0d9488",
            Self::Glycerophospholipid => "#7c3aed",
            Self::Sphingolipid => "#be185d",
            Self::Sterol => "#b45309",
            Self::Other => "#475569",
        }
    }
}

/// Elemental composition extracted from a molecule or a formula string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ElementCounts {
    pub carbon: u32,
    pub hydrogen: u32,
    pub nitrogen: u32,
    pub oxygen: u32,
    pub phosphorus: u32,
    pub sulfur: u32,
    pub halogens: u32,
}

impl ElementCounts {
    /// Double-bond equivalents (a.k.a. the index of hydrogen deficiency).
    ///
    /// `BE = C - H/2 - X/2 + N/2 + 1`
    #[must_use]
    pub fn double_bond_equivalent(&self) -> f64 {
        let c = f64::from(self.carbon);
        let h = f64::from(self.hydrogen);
        let n = f64::from(self.nitrogen);
        let x = f64::from(self.halogens);
        c - h / 2.0 - x / 2.0 + n / 2.0 + 1.0
    }

    /// Molecular formula string in Hill order (e.g. `C16H32O2`).
    #[must_use]
    pub fn formula_string(&self) -> String {
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
pub struct LipidClassification {
    pub class: LipidClass,
    pub counts: ElementCounts,
    pub formula: String,
    pub exact_mass: f64,
    /// Whether the classification came from a structural (SMILES) analysis.
    pub derived_from_smiles: bool,
}
