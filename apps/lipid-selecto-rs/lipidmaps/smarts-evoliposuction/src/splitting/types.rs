// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Public data types and configuration for SMILES dataset splitting.

use std::fmt;

/// Column names to read from the input CSV.
#[derive(Debug, Clone)]
pub struct ColumnNames {
    pub smiles: String,
    pub category: String,
    pub main_class: String,
    pub subclass: String,
}

impl Default for ColumnNames {
    fn default() -> Self {
        Self {
            smiles: "SMILES".to_string(),
            category: "CATEGORY".to_string(),
            main_class: "MAIN_CLASS".to_string(),
            subclass: "SUB_CLASS".to_string(),
        }
    }
}

/// Where subclass negatives are drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubclassNegatives {
    #[default]
    Siblings,
    Global,
}

/// Balancing / sampling knobs for [`crate::splitting::split_dataset`].
#[derive(Debug, Clone)]
pub struct SplitConfig {
    pub min_positive: usize,
    pub neg_ratio: f64,
    pub max_negatives: usize,
    pub subclass_negatives: SubclassNegatives,
    pub seed: u64,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            min_positive: 20,
            neg_ratio: 1.0,
            max_negatives: 5000,
            subclass_negatives: SubclassNegatives::default(),
            seed: 42,
        }
    }
}

/// Taxonomic hierarchy level for a split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    Category,
    MainClass,
    Subclass,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Category => f.write_str("category"),
            Self::MainClass => f.write_str("main_class"),
            Self::Subclass => f.write_str("subclass"),
        }
    }
}

/// One row of the input dataset after parsing.
#[derive(Debug, Clone)]
pub struct DatasetRow {
    pub smiles: String,
    pub category: String,
    pub main_class: String,
    pub subclass: String,
}

/// A parsed, in-memory dataset built with [`crate::splitting::parse_csv`].
#[derive(Debug, Clone, Default)]
pub struct Dataset {
    pub rows: Vec<DatasetRow>,
}

/// CSV parsing errors returned by [`crate::splitting::parse_csv`].
#[derive(Debug, thiserror::Error)]
pub enum SplitError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("missing column(s) in CSV header: {}", .0.join(", "))]
    MissingColumns(Vec<String>),
}

/// Positive/negative SMILES pair for one `CATEGORY`, `MAIN_CLASS`, or `SUBCLASS` label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassSplit {
    pub level: Level,
    pub label: String,
    pub category: String,
    pub main_class: String,
    pub subclass: String,
    pub slug: String,
    pub positive: Vec<String>,
    pub negative: Vec<String>,
}

impl ClassSplit {
    #[must_use]
    pub const fn positive_count(&self) -> usize {
        self.positive.len()
    }

    #[must_use]
    pub const fn negative_count(&self) -> usize {
        self.negative.len()
    }
}

/// Class omitted because it contained fewer than `min_positive` rows.
#[derive(Debug, Clone)]
pub struct SkippedClass {
    pub level: Level,
    pub label: String,
    pub count: usize,
}

/// Complete dataset split result across all hierarchy levels.
#[derive(Debug, Clone, Default)]
pub struct SplitResult {
    pub classes: Vec<ClassSplit>,
    pub skipped: Vec<SkippedClass>,
}
