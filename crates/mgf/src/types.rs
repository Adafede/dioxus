// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Spectrum data types produced by [`crate::MgfLineParser`].

/// One fragment ion peak `(m/z, optional intensity)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MgfPeak {
    /// Fragment ion `m/z`.
    pub mz: f64,
    /// Optional fragment intensity.
    pub intensity: Option<f64>,
}

/// Parsed metadata headers inside one `BEGIN IONS ... END IONS` block.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MgfHeader {
    /// Spectrum title from `TITLE=`, or fallback `NAME=` / `SCANS=`.
    pub title: Option<String>,
    /// Last seen precursor `m/z` (`PEPMASS` or `PRECURSOR_MZ`).
    pub latest_precursor_mz: Option<f64>,
    /// `CHARGE=` metadata value.
    pub charge: Option<String>,
    /// `ADDUCT=` metadata value.
    pub adduct: Option<String>,
    /// `IONMODE=` metadata value.
    pub ion_mode: Option<String>,
    /// `SMILES=` metadata value.
    pub smiles: Option<String>,
    /// `FORMULA=` metadata value.
    pub formula: Option<String>,
    /// `EXACTMASS=` numeric value.
    pub exact_mass: Option<f64>,
    /// `MOLECULEMASS=` numeric value.
    pub molecule_mass: Option<f64>,
}

impl MgfHeader {
    /// Returns the most recently observed precursor `m/z` from `PEPMASS` or `PRECURSOR_MZ`.
    #[must_use]
    pub const fn effective_precursor_mz(&self) -> Option<f64> {
        self.latest_precursor_mz
    }

    /// Returns `EXACTMASS` when present, falling back to `MOLECULEMASS`.
    #[must_use]
    pub const fn reference_mass(&self) -> Option<f64> {
        if self.exact_mass.is_some() {
            self.exact_mass
        } else {
            self.molecule_mass
        }
    }
}

/// One parsed `BEGIN IONS ... END IONS` spectrum record.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MgfBlock {
    /// 1-based index of the block within the parsed stream.
    pub index: usize,
    /// Parsed spectrum header metadata.
    pub header: MgfHeader,
    /// Parsed fragment peaks.
    pub peaks: Vec<MgfPeak>,
}
