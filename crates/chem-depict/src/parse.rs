// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Panic-guarded SMILES parser wrapper.

use std::panic::{AssertUnwindSafe, catch_unwind};

use chematic::core::Molecule;
use chematic::smiles::parse;

use crate::error::ChemParseError;

/// Maximum accepted SMILES byte length before invoking `chematic::smiles::parse`.
pub(crate) const MAX_SMILES_LEN: usize = 16_384;

fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|msg| (*msg).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic payload".to_string())
}

/// Parses a SMILES string, catching any internal parser panic.
///
/// # Errors
///
/// Returns [`ChemParseError::Empty`] on blank input, [`ChemParseError::InvalidSmiles`]
/// when `chematic::smiles::parse` returns `Err`, or [`ChemParseError::ParserPanicked`]
/// if the parser panics.
pub fn parse_smiles_safe(smiles: &str) -> Result<Molecule, ChemParseError> {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return Err(ChemParseError::Empty);
    }
    if trimmed.len() > MAX_SMILES_LEN {
        return Err(ChemParseError::InvalidSmiles {
            smiles: trimmed.chars().take(64).collect(),
            message: format!(
                "SMILES length {} exceeds maximum of {MAX_SMILES_LEN} bytes",
                trimmed.len()
            ),
        });
    }

    match catch_unwind(AssertUnwindSafe(|| parse(trimmed))) {
        Ok(Ok(mol)) => Ok(mol),
        Ok(Err(err)) => Err(ChemParseError::InvalidSmiles {
            smiles: trimmed.to_string(),
            message: err.to_string(),
        }),
        Err(payload) => Err(ChemParseError::ParserPanicked(panic_payload_message(
            payload.as_ref(),
        ))),
    }
}
