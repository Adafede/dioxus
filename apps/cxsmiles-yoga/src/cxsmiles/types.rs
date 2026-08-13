// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Public result types for CX-SMILES generation.
//!
//! These types describe a generated CX-SMILES construct and its round-trip
//! confidence. They are the stable public surface of the [`crate::cxsmiles`]
//! module — everything else is an internal helper.

/// High-level classification of a generated CX-SMILES construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Construct {
    /// A pendant group moves to equivalent attachment points (`m:` blocks).
    Positional,
    /// The inputs differ only in the repeat count of a sub-unit (`Sg:n:`).
    Repeating,
    /// The set could not be reconciled into one clean construct (e.g.
    /// constitutional isomers). A best-effort result is still produced.
    BestEffort,
}

/// Round-trip report: how many original inputs re-appear after expanding CX-SMILES.
#[derive(Debug, Clone)]
pub struct Coverage {
    pub covered: usize,
    pub total: usize,
}

impl Coverage {
    #[must_use]
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.covered as f64 / self.total as f64
        }
    }
}

/// Confidence descriptor surfaced to the UI.
#[derive(Debug, Clone)]
pub struct Confidence {
    pub coverage: Coverage,
    /// `true` when the fit was clean: 100% round-trip coverage.
    pub clean: bool,
}

/// A floating (pendant) group, described for serialisation and expansion.
#[derive(Debug, Clone)]
pub struct FloatingPart {
    /// Atom index of the `*` in the *base* SMILES.
    pub star_idx: usize,
    /// Equivalent base atom indices where this group may attach.
    pub equiv: Vec<usize>,
    /// Raw SMILES of the floating fragment (e.g. `[*O`, `[*]C(=O)C`).
    pub fragment_smiles: String,
    /// `true` when emitted as the second half of a split (double-`m`) group.
    pub split: bool,
}

/// A repeating unit marked with `Sg:n:`.
#[derive(Debug, Clone)]
pub struct RepeatUnit {
    /// Atom indices (into the base SMILES) of one copy of the repeat unit.
    pub atoms: Vec<usize>,
    pub min: usize,
    pub max: usize,
}

/// The result of generating a CX-SMILES from a SMILES list.
#[derive(Debug, Clone)]
pub struct CxResult {
    pub cx_smiles: String,
    pub base_smiles: String,
    pub construct: Construct,
    pub scaffold_smiles: String,
    pub floating: Vec<FloatingPart>,
    pub repeating: Option<RepeatUnit>,
    pub confidence: Confidence,
    /// All distinct molecules enumerated from the generated CX-SMILES
    /// (canonical SMILES) — used by the UI to depict candidates.
    pub enumerated: Vec<String>,
}

#[derive(Debug)]
pub struct CxError(pub String);
impl std::fmt::Display for CxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for CxError {}

/// Convenience alias for a CX-SMILES generation attempt.
pub type CxResult_ = Result<CxResult, CxError>;
