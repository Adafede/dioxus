// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Input and export format detection for lipid mass-spectrometry files.

use std::path::Path;

/// Supported input and export formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LipidFormat {
    /// Mascot Generic Format (`.mgf`) with SMILES or formula metadata.
    Mgf,
    /// Plain-text SMILES list (`.smi` / `.smiles` / `.smi.txt`), optionally tab/space titled.
    Smiles,
}

impl LipidFormat {
    /// Detect input format from file name or extension.
    #[must_use]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = path.as_ref();
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.len() >= 8
            && name
                .as_bytes()
                .get(name.len() - 8..)
                .is_some_and(|suffix| suffix.eq_ignore_ascii_case(b".smi.txt"))
        {
            return Some(Self::Smiles);
        }
        let ext = path.extension()?.to_str()?;
        if ext.eq_ignore_ascii_case("mgf") {
            Some(Self::Mgf)
        } else if ext.eq_ignore_ascii_case("smi") || ext.eq_ignore_ascii_case("smiles") {
            Some(Self::Smiles)
        } else {
            None
        }
    }

    /// Canonical file extension (`"mgf"` or `"smi"`).
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Mgf => "mgf",
            Self::Smiles => "smi",
        }
    }
}
