// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! MGF parsing for mgf-precursor-erro-rs.
//!
//! Split from one 1075-line file into responsibility modules:
//! - [`mass`] — exact-mass lookups (SMILES/formula → neutral mass) + numeric helpers.
//! - [`adduct`] — adduct-token parsing/normalization + `expected_precursor_mz`.
//! - [`block`] — streaming MGF `BEGIN IONS` block parser → [`PrecursorStats`].

// 2022 CODATA
pub const ELECTRON_MASS: f64 = 0.000_548_579_909_044_1;
// 2022 CODATA
pub const PROTON_MASS: f64 = 1.007_276_466_578_9;
// 2022 CODATA
pub const NEUTRON_MASS: f64 = 1.008_664_916_06;
// CIAAW
pub const HYDROGEN_MASS: f64 = 1.007_825_032_2;
// CIAAW
pub const HELIUM_MASS: f64 = 4.002_603_254_5;
// CIAAW
pub const NITROGEN_MASS: f64 = 14.003_074_004;
// calculated
pub const AMMONIUM_MASS: f64 = NITROGEN_MASS + (4_f64 * HYDROGEN_MASS) - ELECTRON_MASS;
// CIAAW
pub const SODIUM_MASS: f64 = 22.989_769_28;
// CIAAW
pub const POTASSIUM_MASS: f64 = 38.963_706_49;

pub(crate) mod adduct;
pub(crate) mod block;
pub(crate) mod mass;

pub use adduct::{
    adduct_class, adduct_family, expected_precursor_mz, is_excluded_adduct, is_supported_adduct,
    normalize_adduct_key, normalize_adduct_label, parse_adduct_charge_sign,
};
#[cfg(target_arch = "wasm32")]
pub use block::scan_blob_with_progress;
pub use block::{BlockParseState, process_block};
pub use mass::{
    decimal_precision, exact_mass_from_formula, exact_mass_from_smiles, round_to_precision,
    smiles_is_supported,
};
