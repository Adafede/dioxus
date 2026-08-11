//! Format detection and preservation for input/output files.
//!
//! Detects whether input is MGF (Mascot Generic Format) or SMILES list,
//! preserves the format, and outputs in the same format as the input.

#![allow(clippy::module_name_repetitions)]

use std::path::Path;

/// Supported input/output formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LipidFormat {
    /// MGF (Mascot Generic Format) - mass spectrometry data with SMILES/formula metadata
    Mgf,
    /// SMILES - plain text list of SMILES strings (one per line, with optional IDs)
    Smiles,
}

impl LipidFormat {
    /// Detect format from file extension.
    #[must_use]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = path.as_ref();
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "mgf" => Some(Self::Mgf),
            "smi" | "smiles" | "smi.txt" => Some(Self::Smiles),
            _ => None,
        }
    }

    /// Detect format from file contents by examining the first non-empty lines.
    #[must_use]
    pub fn detect_from_content(content: &str) -> Option<Self> {
        let trimmed = content.trim();

        // Check for MGF format: look for "BEGIN IONS" block
        if trimmed.contains("BEGIN IONS") && trimmed.contains("END IONS") {
            return Some(Self::Mgf);
        }

        // Check for SMILES format: should start with SMILES-like patterns
        // SMILES patterns contain: C, c, N, n, O, o, P, p, S, s, F, Cl, Br, I, =, #, [, ], (, ), /, \, @, +, -, digits
        // Most lines should be either pure SMILES or "ID\tSMILES" or "SMILES\tID"
        for line in trimmed.lines().take(10) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Check if line looks like SMILES (contains C, N, O, or aromatic notation)
            let has_smiles_chars = line.contains('C')
                || line.contains('c')
                || line.contains('N')
                || line.contains('n')
                || line.contains('O')
                || line.contains('o')
                || line.contains('[')
                || line.contains(']')
                || line.contains('=')
                || line.contains('#');

            if has_smiles_chars {
                return Some(Self::Smiles);
            }
        }

        None
    }

    /// Get the file extension for this format.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Mgf => "mgf",
            Self::Smiles => "smi",
        }
    }

    /// Get a descriptive label for this format.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mgf => "MGF (Mass Spectrometry)",
            Self::Smiles => "SMILES (Text)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mgf_by_extension() {
        let format = LipidFormat::from_path("data.mgf");
        assert_eq!(format, Some(LipidFormat::Mgf));
    }

    #[test]
    fn detects_smiles_by_extension() {
        let format = LipidFormat::from_path("compounds.smi");
        assert_eq!(format, Some(LipidFormat::Smiles));
    }

    #[test]
    fn detects_mgf_by_content() {
        let mgf_content = "BEGIN IONS\nTITLE=spectrum_1\nEND IONS";
        let format = LipidFormat::detect_from_content(mgf_content);
        assert_eq!(format, Some(LipidFormat::Mgf));
    }

    #[test]
    fn detects_smiles_by_content() {
        let smiles_content = "CCCCCCCCCCCCCCCC(=O)O\nCC(C)CC(N)C(=O)O";
        let format = LipidFormat::detect_from_content(smiles_content);
        assert_eq!(format, Some(LipidFormat::Smiles));
    }

    #[test]
    fn returns_correct_extensions() {
        assert_eq!(LipidFormat::Mgf.extension(), "mgf");
        assert_eq!(LipidFormat::Smiles.extension(), "smi");
    }
}
