// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Typed domain enums and value objects for natural-product screening.

use serde::{Deserialize, Deserializer};
use std::fmt;

/// Motif origin class reported by `motif-library.js`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceClass {
    /// Natural-product origin.
    Natural,
    /// Synthetic origin.
    Synthetic,
    /// Unclassified or unknown origin.
    #[default]
    Unknown,
}

impl SourceClass {
    /// Canonical lowercase label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Natural => "natural",
            Self::Synthetic => "synthetic",
            Self::Unknown => "unknown",
        }
    }
}

impl From<&str> for SourceClass {
    fn from(raw: &str) -> Self {
        match raw {
            "natural" => Self::Natural,
            "synthetic" => Self::Synthetic,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for SourceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SourceClass {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::from(raw.as_str()))
    }
}

/// Chemist checklist outcome status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckStatus {
    /// Criterion satisfied cleanly.
    Pass,
    /// Borderline or incomplete evidence.
    Warn,
    /// Negative or missing-evidence flag.
    Fail,
}

impl CheckStatus {
    /// CSS/CSV lowercase status string (`"pass"`, `"warn"`, `"fail"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

impl fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Ertl NP-likeness score together with its Morgan-fingerprint model coverage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpScore {
    /// Ertl NP-likeness score (approximately −5 to +5).
    pub score: f64,
    /// Fraction of molecule Morgan bits present in the model (`0.0..=1.0`).
    pub confidence: f64,
}

/// First 14-character skeleton connectivity layer of an `InChIKey`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct InChIKey14(String);

impl InChIKey14 {
    /// Extracts the 14-character connectivity layer from an `InChIKey` string.
    #[must_use]
    pub(crate) fn from_inchikey(inchikey: &str) -> Option<Self> {
        let trimmed = inchikey.trim();
        if trimmed.is_empty() {
            return None;
        }
        let layer = trimmed.split('-').next().unwrap_or(trimmed);
        if layer.is_empty() {
            None
        } else {
            Some(Self(layer.to_string()))
        }
    }

    /// Underlying connectivity layer string slice.
    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InChIKey14 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A matched LOTUS/Wikidata compound QID and whether it has taxon annotations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LotusCompound {
    /// Wikidata Q-identifier (for example `"Q60235"`).
    pub qid: String,
    /// Whether LOTUS/Wikidata returned at least one source taxon for this item.
    pub has_taxa: bool,
}
