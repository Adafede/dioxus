// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! MGF parsing for mgf-precursor-erro-rs.
//!
//! Split from one 1075-line file into responsibility modules:
//! - [`mass`] — exact-mass lookups (SMILES/formula → neutral mass) + numeric helpers.
//! - [`adduct`] — adduct-token parsing/normalization + `expected_precursor_mz`.
//! - [`block`] — streaming MGF `BEGIN IONS` block parser → [`PrecursorStats`].

pub const PROTON_MASS: f64 = 1.007_276_466_621;
pub const HYDROGEN_MASS: f64 = PROTON_MASS + ELECTRON_MASS;
pub const ELECTRON_MASS: f64 = 0.000_548_579_909_065;
pub const SODIUM_MASS: f64 = 22.989_769_67;
pub const POTASSIUM_MASS: f64 = 38.963_707;
pub const AMMONIUM_MASS: f64 = 18.033_823;

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
