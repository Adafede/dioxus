// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Size-guarded `chematic` SMILES parsing and local 2D SVG depiction.
//!
//! Enforces SMILES byte-length (`16_384`) and atom-count (`512`) bounds before
//! calling `chematic::smiles` or `chematic::depict`. When built with `panic = "unwind"`
//! (host targets and `[profile.wasm-release]`), `std::panic::catch_unwind` recovers from
//! internal parser or layout panics; `panic = "abort"` builds rely on those pre-guards.
//!
//! # Example
//!
//! ```rust
//! use chem_depict::{parse_smiles_safe, render_smiles_svg};
//!
//! assert!(parse_smiles_safe("CCO").is_ok());
//! assert!(parse_smiles_safe("   ").is_err());
//! let svg = render_smiles_svg("CCO");
//! assert!(svg.trim_start().starts_with("<svg"));
//! ```

#![warn(missing_docs)]

mod error;
mod parse;
mod svg;

pub use error::ChemParseError;
pub use parse::parse_smiles_safe;
pub use svg::{
    empty_compact_svg, render_molecule_svg_compact, render_smiles_svg, render_smiles_svg_compact,
    render_smiles_svg_highlighted,
};

#[cfg(test)]
mod tests;
