// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Canonical LIPID MAPS family metadata, palettes, and display helpers.
//!
//! [`LipidFamily`] is the single source of truth for the eight LIPID MAPS
//! categories in canonical hierarchy order (`FA -> GL -> GP -> SP -> ST -> PR -> SL -> PK`).

pub(crate) const FA_PALETTE: [&str; 5] = ["#4E7705", "#6D9F06", "#97CE2F", "#BDEC6F", "#DDFFA0"];
pub(crate) const GL_PALETTE: [&str; 5] = ["#098BD9", "#56B4E9", "#7DCCFF", "#BCE1FF", "#E7F4FF"];
pub(crate) const GP_PALETTE: [&str; 5] = ["#7D3560", "#A1527F", "#CC79A7", "#E794C1", "#EFB6D6"];
pub(crate) const SP_PALETTE: [&str; 5] = ["#9D654C", "#C17754", "#F09163", "#FCB076", "#FFD5AF"];
pub(crate) const ST_PALETTE: [&str; 5] = ["#238b45", "#41ab5d", "#74c476", "#a1d99b", "#c7e9c0"];
pub(crate) const PR_PALETTE: [&str; 5] = ["#4292c6", "#6baed6", "#9ecae1", "#c6dbef", "#eff3ff"];
pub(crate) const SL_PALETTE: [&str; 5] = ["#6a51a3", "#807dba", "#9e9ac8", "#bcbddc", "#dadaeb"];
pub(crate) const PK_PALETTE: [&str; 5] = ["#ff7f00", "#fe9929", "#fdae6b", "#fec44f", "#feeda0"];

/// The eight top-level LIPID MAPS categories in canonical hierarchy order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LipidFamily {
    /// Free fatty acids, esters, amides, eicosanoids, and conjugates (`FA`).
    FattyAcyls = 0,
    /// Mono-/di-/tri-radylglycerols and glycosylglycerols (`GL`).
    Glycerolipids = 1,
    /// Glycerophospholipids (`GP`).
    Glycerophospholipids = 2,
    /// Ceramides, sphingoid bases, and glycosphingolipids (`SP`).
    Sphingolipids = 3,
    /// Sterols, steroids, and bile acids (`ST`).
    SterolLipids = 4,
    /// Isoprenoids, quinones, and polyprenols (`PR`).
    PrenolLipids = 5,
    /// Acylaminosugars and acyltrehaloses (`SL`).
    Saccharolipids = 6,
    /// Flavonoids, macrolides, and polyketides (`PK`).
    Polyketides = 7,
}

impl LipidFamily {
    /// All eight LIPID MAPS families in canonical hierarchy order.
    pub const ALL: [Self; 8] = [
        Self::FattyAcyls,
        Self::Glycerolipids,
        Self::Glycerophospholipids,
        Self::Sphingolipids,
        Self::SterolLipids,
        Self::PrenolLipids,
        Self::Saccharolipids,
        Self::Polyketides,
    ];

    /// Canonical sort rank (`0..8`).
    #[must_use]
    pub const fn rank(self) -> usize {
        self as usize
    }

    /// Two-letter LIPID MAPS family code (e.g. `"FA"`).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::FattyAcyls => "FA",
            Self::Glycerolipids => "GL",
            Self::Glycerophospholipids => "GP",
            Self::Sphingolipids => "SP",
            Self::SterolLipids => "ST",
            Self::PrenolLipids => "PR",
            Self::Saccharolipids => "SL",
            Self::Polyketides => "PK",
        }
    }

    /// Broad LIPID MAPS display name (e.g. `"Fatty Acyls"`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FattyAcyls => "Fatty Acyls",
            Self::Glycerolipids => "Glycerolipids",
            Self::Glycerophospholipids => "Glycerophospholipids",
            Self::Sphingolipids => "Sphingolipids",
            Self::SterolLipids => "Sterol Lipids",
            Self::PrenolLipids => "Prenol Lipids",
            Self::Saccharolipids => "Saccharolipids",
            Self::Polyketides => "Polyketides",
        }
    }

    /// Display name with trailing bracketed code (e.g. `"Fatty Acyls [FA]"`).
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::FattyAcyls => "Fatty Acyls [FA]",
            Self::Glycerolipids => "Glycerolipids [GL]",
            Self::Glycerophospholipids => "Glycerophospholipids [GP]",
            Self::Sphingolipids => "Sphingolipids [SP]",
            Self::SterolLipids => "Sterol Lipids [ST]",
            Self::PrenolLipids => "Prenol Lipids [PR]",
            Self::Saccharolipids => "Saccharolipids [SL]",
            Self::Polyketides => "Polyketides [PK]",
        }
    }

    /// Fallback `[XX00]` class code used when classifying from molecular formula alone.
    #[must_use]
    pub(crate) const fn fallback_code(self) -> &'static str {
        match self {
            Self::FattyAcyls => "[FA00]",
            Self::Glycerolipids => "[GL00]",
            Self::Glycerophospholipids => "[GP00]",
            Self::Sphingolipids => "[SP00]",
            Self::SterolLipids => "[ST00]",
            Self::PrenolLipids => "[PR00]",
            Self::Saccharolipids => "[SL00]",
            Self::Polyketides => "[PK00]",
        }
    }

    /// Five-shade microshades color palette assigned to this family.
    #[must_use]
    pub const fn palette(self) -> &'static [&'static str; 5] {
        match self {
            Self::FattyAcyls => &FA_PALETTE,
            Self::Glycerolipids => &GL_PALETTE,
            Self::Glycerophospholipids => &GP_PALETTE,
            Self::Sphingolipids => &SP_PALETTE,
            Self::SterolLipids => &ST_PALETTE,
            Self::PrenolLipids => &PR_PALETTE,
            Self::Saccharolipids => &SL_PALETTE,
            Self::Polyketides => &PK_PALETTE,
        }
    }
}
