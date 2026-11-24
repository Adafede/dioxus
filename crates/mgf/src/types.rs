// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Explicit MGF parser options and spectrum data types.

/// Controls how header key names (e.g., `SMILES=`, `PEPMASS=`) are matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderCase {
    /// Match MGF metadata keys ASCII case-insensitively.
    CaseInsensitive,
    /// Match MGF metadata keys only in canonical uppercase (`SMILES`, `PEPMASS`, etc.).
    CaseSensitive,
}

/// Controls what happens to an unclosed `BEGIN IONS` block at end-of-stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnterminatedBlockPolicy {
    /// Discard an unclosed block at EOF.
    Drop,
    /// Emit an unclosed block at EOF.
    Flush,
}

/// Controls how fragment peak lines inside a spectrum block are validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeakParseMode {
    /// Accept any whitespace-containing non-header line whose leading float is `> 0.0`.
    RequirePositiveMz,
    /// Require at least two whitespace-separated floats (`mz` and `intensity`).
    RequireMzAndIntensity,
}

/// Configuration for [`crate::MgfLineParser`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MgfOptions {
    /// Strip a UTF-8 BOM (`\u{feff}`) from the first fed line.
    pub strip_bom: bool,
    /// Header key matching rule.
    pub header_case: HeaderCase,
    /// Parse `KEY=VALUE` tokens occurring on the `BEGIN IONS` line itself.
    pub inline_begin_headers: bool,
    /// Policy for blocks left open at EOF.
    pub unterminated_block: UnterminatedBlockPolicy,
    /// Retain verbatim input lines per block for filtered export.
    pub retain_raw_lines: bool,
    /// Validation mode for MS2 fragment peak lines.
    pub peak_mode: PeakParseMode,
}

impl MgfOptions {
    /// Standard MGF parsing preset (strips BOM, case-insensitive keys, drops unclosed EOF blocks).
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            strip_bom: true,
            header_case: HeaderCase::CaseInsensitive,
            inline_begin_headers: false,
            unterminated_block: UnterminatedBlockPolicy::Drop,
            retain_raw_lines: false,
            peak_mode: PeakParseMode::RequirePositiveMz,
        }
    }

    /// Lipid-selection preset (supports inline `BEGIN IONS KEY=VAL`, retains raw lines, flushes EOF).
    #[must_use]
    pub const fn lipid_selection() -> Self {
        Self {
            strip_bom: false,
            header_case: HeaderCase::CaseSensitive,
            inline_begin_headers: true,
            unterminated_block: UnterminatedBlockPolicy::Flush,
            retain_raw_lines: true,
            peak_mode: PeakParseMode::RequireMzAndIntensity,
        }
    }
}

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
    /// Precursor `m/z` from `PEPMASS=`.
    pub pepmass: Option<f64>,
    /// Precursor `m/z` from `PRECURSOR_MZ=`.
    pub precursor_mz: Option<f64>,
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
    /// Verbatim lines (excluding trailing newlines) when `retain_raw_lines` is enabled.
    pub raw_lines: Vec<String>,
}
