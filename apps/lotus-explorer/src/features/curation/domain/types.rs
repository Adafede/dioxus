// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurationInputRow {
    pub name: String,
    pub smiles: String,
    pub taxon: Option<String>,
    pub doi: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CurationStatus {
    ExistingComplete,
    ExistingNeedsUpdates,
    NewCompound,
    PendingDependencies,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurationResultRow {
    pub input: CurationInputRow,
    pub canonical_smiles: Option<String>,
    pub inchikey: Option<String>,
    pub inchi: Option<String>,
    pub formula: Option<String>,
    pub exact_mass: Option<f64>,
    pub mass_warning: Option<String>,
    pub wikidata_qid: Option<String>,
    pub status: CurationStatus,
    pub note: String,
    pub dependency_blocks: Vec<String>,
    pub quickstatements: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickStatementsBundle {
    pub dependencies: std::sync::Arc<str>,
    pub main: std::sync::Arc<str>,
}

impl Default for QuickStatementsBundle {
    fn default() -> Self {
        Self {
            dependencies: std::sync::Arc::<str>::from(""),
            main: std::sync::Arc::<str>::from(""),
        }
    }
}

#[derive(Debug)]
pub enum CurationError {
    InvalidInput(String),
    Http(String),
    Parse(String),
}

impl fmt::Display for CurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "{msg}"),
            Self::Http(msg) => write!(f, "{msg}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}
