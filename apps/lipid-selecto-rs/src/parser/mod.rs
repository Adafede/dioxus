// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! MGF parsing and chemical class matching.
//!
//! The parser walks an MGF file block-by-block (`BEGIN IONS ... END IONS`),
//! preserving each block's verbatim text so that the filtered MGF is a faithful
//! subset of the input (no re-serialization drift). For every block it extracts
//! SMILES and FORMULA metadata, then matches against user-defined chemical classes.
//!
//! Split by responsibility:
//! - `parsing`: `SpectrumBlock` (struct + classification methods) and block
//!   extraction.
//! - `analysis`: aggregation, summary, gallery, class matching across the
//!   collection, and filtered-MGF generation.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

mod analysis;
mod parsing;

pub(crate) use analysis::{Analysis, GalleryItem, Summary};
#[cfg(target_arch = "wasm32")]
pub(crate) use analysis::{build_analysis_from_classified, classify_blocks};
pub(crate) use parsing::SpectrumBlock;
#[cfg(target_arch = "wasm32")]
pub(crate) use parsing::extract_blocks_from_lines;
