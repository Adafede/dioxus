// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! MGF parsing for mgf-precursor-erro-rs.
//!
//! Split from one 1075-line file into responsibility modules:
//! - `mass` — exact-mass lookups (SMILES/formula → neutral mass) + numeric helpers.
//! - `adduct` — adduct-token parsing/normalization + `expected_precursor_mz`.
//! - `block` — streaming MGF `BEGIN IONS` block parser → `PrecursorStats`.

// 2022 CODATA
pub(crate) const ELECTRON_MASS: f64 = 0.000_548_579_909_044_1;
// 2022 CODATA
pub(crate) const PROTON_MASS: f64 = 1.007_276_466_578_9;
// CIAAW
pub(crate) const HYDROGEN_MASS: f64 = 1.007_825_032_2;
// CIAAW
pub(crate) const NITROGEN_MASS: f64 = 14.003_074_004;
// calculated
pub(crate) const AMMONIUM_MASS: f64 = NITROGEN_MASS + (4_f64 * HYDROGEN_MASS) - ELECTRON_MASS;
// CIAAW
pub(crate) const SODIUM_MASS: f64 = 22.989_769_28;
// CIAAW
pub(crate) const POTASSIUM_MASS: f64 = 38.963_706_49;

pub(crate) mod adduct;
pub(crate) mod block;
pub(crate) mod mass;

#[cfg(target_arch = "wasm32")]
pub(crate) use block::scan_blob_with_progress;
