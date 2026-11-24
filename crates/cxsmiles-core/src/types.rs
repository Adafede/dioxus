// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Public domain result types and options for CX-SMILES generation.

use crate::graph::usize_to_f64;

/// High-level classification of a generated CX-SMILES construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructKind {
    /// A pendant group moves to equivalent attachment points (`m:` blocks).
    Positional,
    /// Inputs differ in the repeat count of an internal sub-unit (`Sg:n:`).
    Repeating,
    /// Inputs could not be reconciled into one clean construct; a best-effort
    /// representation is returned instead.
    BestEffort,
}

/// Round-trip report: how many original inputs re-appear after expanding CX-SMILES.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coverage {
    /// Number of input molecules present in the expanded enumeration.
    pub covered: usize,
    /// Total number of valid input molecules evaluated.
    pub total: usize,
}

impl Coverage {
    /// Fraction of evaluated inputs recovered by round-trip enumeration (`0.0`–`1.0`).
    #[must_use]
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            usize_to_f64(self.covered) / usize_to_f64(self.total)
        }
    }

    /// Returns `true` when every evaluated input (`total > 0`) round-tripped cleanly.
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.total > 0 && self.covered >= self.total
    }
}

/// Round-trip confidence summary surfaced to callers and UI components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Confidence {
    /// Exact numerator/denominator coverage counts.
    pub coverage: Coverage,
    /// `true` when 100% of evaluated inputs round-tripped cleanly.
    pub clean: bool,
}

impl Confidence {
    /// Builds a [`Confidence`] value directly from [`Coverage`].
    #[must_use]
    pub const fn from_coverage(coverage: Coverage) -> Self {
        Self {
            clean: coverage.is_clean(),
            coverage,
        }
    }
}

/// Attachment target of a floating pendant group during enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentTarget {
    /// Attaches to any of the listed equivalent scaffold atom indices.
    Variable(Vec<usize>),
    /// Attaches to another floating group's attachment atom (index into `defs`).
    Fixed(usize),
}

/// A floating (pendant) group described for serialisation, expansion, and UI display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloatingPart {
    /// Equivalent base scaffold atom indices where this group may attach.
    pub equiv: Vec<usize>,
    /// Raw fragment SMILES (e.g. `[*]O`, `[*]C(=O)C`).
    pub fragment_smiles: String,
    /// `true` when emitted as the second half of a split (double-`m`) group.
    pub split: bool,
}

/// A repeating sub-unit annotated with `Sg:n:`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatUnit {
    /// Atom indices (into the base SMILES) of one copy of the repeat unit.
    pub atoms: Vec<usize>,
    /// Minimum repeat count across inputs.
    pub min: usize,
    /// Maximum repeat count across inputs.
    pub max: usize,
}

/// Configuration knobs controlling CX-SMILES generation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CxOptions {
    /// Expand candidate `m:` sites across symmetry-equivalent aromatic ring carbons.
    pub expand_aromatic: bool,
}

/// Result of generating a CX-SMILES representation from a SMILES list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CxResult {
    /// Generated SMILES + CX extension block (e.g. `scaffold.[*]Cl |m:12:0.2.3|`).
    pub cx_smiles: String,
    /// Detected structural construct family.
    pub construct: ConstructKind,
    /// Canonical SMILES of the shared scaffold.
    pub scaffold_smiles: String,
    /// Variable pendant fragments when `construct` uses `m:` blocks.
    pub floating: Vec<FloatingPart>,
    /// Detected repeating sub-unit when `construct` uses `Sg:n:`.
    pub repeat_unit: Option<RepeatUnit>,
    /// Round-trip confidence metrics against the input list.
    pub confidence: Confidence,
    /// Distinct canonical SMILES enumerated from the generated CX-SMILES.
    pub enumerated: Vec<String>,
}
