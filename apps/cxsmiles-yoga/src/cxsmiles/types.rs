// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Public result types for CX-SMILES generation.
//!
//! These types describe a generated CX-SMILES construct and its round-trip
//! confidence. They are the stable public surface of the [`crate::cxsmiles`]
//! module — everything else is an internal helper.

/// High-level classification of a generated CX-SMILES construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Construct {
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
pub(crate) struct Coverage {
    pub(crate) covered: usize,
    pub(crate) total: usize,
}

impl Coverage {
    #[must_use]
    pub(crate) fn fraction(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.covered as f64 / self.total as f64
        }
    }
}

/// Confidence descriptor surfaced to the UI.
#[derive(Debug, Clone)]
pub(crate) struct Confidence {
    pub(crate) coverage: Coverage,
    /// `true` when the fit was clean: 100% round-trip coverage.
    pub(crate) clean: bool,
}

/// A floating (pendant) group, described for serialisation and expansion.
///
/// # Nothing reads this but the tests
///
/// `build_positional` fills it in and [`CxResult::floating`] carries it, and the
/// only consumer anywhere in the workspace is the test suite in
/// [`super::mod`]: the UI renders `cx_smiles` and `construct` and never looks at
/// `floating`. So in a build without tests these fields are written and dropped.
///
/// The alternative to the `allow` is deleting the field and the type, which
/// would leave the tests asserting on nothing the product produces — the emitted
/// `m:` field is what they should be checking instead. That is a change in what
/// the app reports, not a lint fix, so it is raised rather than made here.
/// `not(test)` is the exact set of builds in which the lint fires; nothing here
/// is wasm-specific.
#[allow(dead_code)] // see above
#[derive(Debug, Clone)]
pub(crate) struct FloatingPart {
    /// Equivalent base atom indices where this group may attach.
    pub(crate) equiv: Vec<usize>,
    /// Raw SMILES of the floating fragment (e.g. `[*O`, `[*]C(=O)C`).
    pub(crate) fragment_smiles: String,
    /// `true` when emitted as the second half of a split (double-`m`) group.
    pub(crate) split: bool,
}

/// A repeating unit marked with `Sg:n:`.
#[derive(Debug, Clone)]
pub(crate) struct RepeatUnit {
    /// Atom indices (into the base SMILES) of one copy of the repeat unit.
    pub(crate) atoms: Vec<usize>,
    pub(crate) min: usize,
    pub(crate) max: usize,
}

/// The result of generating a CX-SMILES from a SMILES list.
#[derive(Debug, Clone)]
/// See [`FloatingPart`] for why `floating` is suppressed: it is written by
/// `build_positional` and read by nothing outside the test suite.
#[allow(dead_code)] // see above
pub(crate) struct CxResult {
    pub(crate) cx_smiles: String,
    pub(crate) construct: Construct,
    pub(crate) scaffold_smiles: String,
    pub(crate) floating: Vec<FloatingPart>,
    pub(crate) confidence: Confidence,
    /// All distinct molecules enumerated from the generated CX-SMILES
    /// (canonical SMILES) — used by the UI to depict candidates.
    pub(crate) enumerated: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct CxError(pub(crate) String);
impl std::fmt::Display for CxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for CxError {}

/// Convenience alias for a CX-SMILES generation attempt.
pub(crate) type CxResult_ = Result<CxResult, CxError>;
