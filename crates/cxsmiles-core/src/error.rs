// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Structured error types for CX-SMILES generation and expansion.

use thiserror::Error;

/// Error returned when parsing inputs, matching scaffolds, or expanding CX-SMILES.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CxError {
    /// Input list contained no non-blank SMILES lines.
    #[error("no parseable SMILES in input")]
    EmptyInput,

    /// An empty molecule group was passed to an internal builder.
    #[error("{0}: group is empty")]
    EmptyGroup(&'static str),

    /// A SMILES input failed to parse.
    #[error("could not parse '{smiles}': {detail}")]
    ParseFailed {
        /// Input SMILES line that failed.
        smiles: String,
        /// Underlying parser message.
        detail: String,
    },

    /// Canonicalised scaffold SMILES failed to re-parse.
    #[error("re-parse scaffold: {0}")]
    ScaffoldReparse(String),

    /// Query molecule had no embedding in the target molecule.
    #[error("no match of query into molecule")]
    NoScaffoldMatch,

    /// Shortest molecule had no embedding in the longest molecule.
    #[error("no MCS match of shortest into longest")]
    NoRepeatScaffoldMatch,

    /// No extra component could be selected as a repeat pattern.
    #[error("could not determine repeat pattern")]
    NoRepeatPattern,

    /// Requested repeat unit cannot fit inside the scaffold molecule.
    #[error("a repeat unit of {unit_size} atoms cannot lie inside a scaffold of {scaffold_atoms}")]
    RepeatTooLarge {
        /// Atom count of the candidate repeat unit.
        unit_size: usize,
        /// Atom count of the scaffold molecule.
        scaffold_atoms: usize,
    },

    /// Repeat search hit the fragment visit ceiling before finishing.
    #[error(
        "gave up looking for a {unit_size}-atom repeat unit in a scaffold of {scaffold_atoms} \
         atoms after {limit} fragments; the inputs differ by about {unit_size} atoms, which \
         makes the search exponential"
    )]
    RepeatBudgetExceeded {
        /// Atom count of the candidate repeat unit.
        unit_size: usize,
        /// Atom count of the scaffold molecule.
        scaffold_atoms: usize,
        /// Fragment search budget ceiling.
        limit: usize,
    },

    /// No internal connected subgraph matched the expected repeat element multiset.
    #[error("could not locate repeat unit in scaffold")]
    RepeatNotFound,

    /// Repeat expansion did not recover any homologue beyond the base scaffold.
    #[error("repeat expansion did not reproduce any homologue beyond the scaffold")]
    NoHomologueReproduced,

    /// All cluster members had identical atom counts, so repeating-unit mode does not apply.
    #[error("all inputs share the same atom count")]
    UniformAtomCounts,

    /// Combinatorial expansion exceeded `MAX_ARRANGEMENTS`.
    #[error(
        "{total} arrangements to expand, from {group_count} groups of {group_sizes:?} positions; \
         the limit is {limit}"
    )]
    TooManyArrangements {
        /// Total Cartesian arrangements requested.
        total: usize,
        /// Number of variable groups.
        group_count: usize,
        /// Position count per variable group.
        group_sizes: Vec<usize>,
        /// Maximum allowed arrangements.
        limit: usize,
    },

    /// A variable group had zero candidate attachment positions (or overflowed product).
    #[error("a variable group has no positions, so there is nothing to expand")]
    NoVariablePositions,

    /// Base scaffold exceeded the maximum atom limit for positional expansion.
    #[error(
        "a scaffold of {atoms} atoms cannot be expanded {arrangements} times; the limit is \
         {limit} atoms"
    )]
    ScaffoldTooLarge {
        /// Atom count of the scaffold.
        atoms: usize,
        /// Requested expansion count.
        arrangements: usize,
        /// Maximum atoms allowed per generated molecule.
        limit: usize,
    },

    /// Mixed-radix combination digit exceeded the group's position list.
    #[error("position {pos} is outside the {group_len} positions of its group")]
    InvalidPosition {
        /// Out-of-bounds position index.
        pos: usize,
        /// Number of available positions in the group.
        group_len: usize,
    },

    /// Number of repeat counts exceeded `MAX_ARRANGEMENTS`.
    #[error("{total} repeat counts to expand ({min} to {max}); the limit is {limit}")]
    RepeatRangeTooLarge {
        /// Number of repeat steps requested.
        total: usize,
        /// Minimum repeat count.
        min: usize,
        /// Maximum repeat count.
        max: usize,
        /// Maximum allowed repeat steps.
        limit: usize,
    },

    /// Multiplying `max * unit_len` overflowed `usize`.
    #[error(
        "a repeat range of {min} to {max} over a {unit_len}-atom unit overflows; the inputs are \
         not a molecule"
    )]
    ExpansionOverflow {
        /// Minimum repeat count.
        min: usize,
        /// Maximum repeat count.
        max: usize,
        /// Atom count of one repeat unit.
        unit_len: usize,
    },

    /// Widest expanded repeat molecule exceeded `MAX_MOLECULE_ATOMS`.
    #[error(
        "{widest} atoms in the largest expansion ({copies} copies of a {unit_len}-atom unit); the \
         limit is {limit} atoms"
    )]
    ExpansionTooLarge {
        /// Atom count of the largest expansion.
        widest: usize,
        /// Maximum repeat copies.
        copies: usize,
        /// Atom count of one repeat unit.
        unit_len: usize,
        /// Maximum allowed atoms in a single expansion.
        limit: usize,
    },
}
