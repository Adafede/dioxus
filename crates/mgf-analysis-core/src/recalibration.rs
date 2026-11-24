// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Precursor-driven MS2 fragment recalibration.

mod calibration;
mod generator;
mod parsing;
mod types;

pub use calibration::recalibrate_fragment_mz;
pub use generator::{
    generate_recalibrated_mgf, recalibrate_fragments, write_fragments_as_is,
    write_recalibrated_fragments,
};
pub use parsing::{
    closest_ms2_precursor_mz, extract_pepmass_from_line, find_ms2_precursor_peak, is_fragment_line,
};
pub use types::{CalibrationModel, Peak, ShrinkageLambda};
