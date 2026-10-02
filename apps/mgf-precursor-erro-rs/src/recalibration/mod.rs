// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! MS2 fragment recalibration over MGF content.
//!
//! Split by responsibility:
//! - `types` — `CalibrationModel` / `Peak` definitions.
//! - `parsing` — MGF line parsing (pepmass directives + fragment lines).
//! - `calibration` — per-fragment calibration math.
//! - `generator` — MGF round-tripping + bulk recalibration orchestration.

mod calibration;
mod generator;
mod parsing;
mod types;

pub use calibration::recalibrate_fragment_mz;
pub use generator::{
    generate_recalibrated_mgf, recalibrate_fragments, write_fragments_as_is,
    write_recalibrated_fragments,
};
pub use parsing::{extract_pepmass_from_line, find_ms2_precursor_peak, is_fragment_line};
pub use types::{CalibrationModel, Peak};

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
// A test that indexes past its own literal fixture has already failed; the
// panic is the assertion. Anything the production path can get wrong is
// checked by a test that compares whole vectors, not by an index.
mod tests;
