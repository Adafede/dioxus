// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Pure-Rust CX-SMILES generation engine (`m:` positional blocks and `Sg:n:` repeats).
//!
//! Pipeline (`generate` / `generate_with_options`):
//! 1. Parse non-empty SMILES lines into molecules.
//! 2. Cluster candidates by ECFP4/Tanimoto similarity.
//! 3. Detect repeating units (`Sg:n:`) or fall back to MCS positional diffs (`m:`).
//! 4. Expand and canonicalise the resulting CX-SMILES to compute round-trip recall.
//!
//! # Example
//!
//! ```rust
//! use cxsmiles_core::{ConstructKind, generate};
//!
//! let inputs = vec![
//!     "Clc1ccccc1-c2ccccc2".to_string(),
//!     "Clc1cccc(-c2ccccc2)c1".to_string(),
//!     "Clc1ccc(-c2ccccc2)cc1".to_string(),
//! ];
//! let res = generate(&inputs).expect("valid biphenyl series");
//! assert_eq!(res.construct, ConstructKind::Positional);
//! assert!(res.confidence.clean);
//! assert!(res.cx_smiles.contains("m:"));
//! ```

#![warn(missing_docs)]

mod aromatic;
mod error;
mod fragment;
mod graph;
mod parse;
mod pipeline;
mod positional;
mod repeating;
mod roundtrip;
mod splice;
mod types;

pub use error::CxError;
pub use pipeline::{generate, generate_with_options};
pub use types::{
    AttachmentTarget, Confidence, ConstructKind, Coverage, CxOptions, CxResult, FloatingPart,
    RepeatUnit,
};

#[cfg(test)]
#[expect(clippy::unwrap_used)]
#[expect(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
mod tests;
