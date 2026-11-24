// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Shared signal state for `lipid-selecto-rs` UI components.

use dioxus::prelude::{ReadableExt, Signal};
use lipid_core::ClassMatchSet;

/// Grouped reactive filter signals shared across summary, gallery, and export panels.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct SummaryFilters {
    pub(super) selected_classes: Signal<ClassMatchSet>,
    pub(super) mz_min: Signal<f64>,
    pub(super) mz_max: Signal<f64>,
    pub(super) precursor_min: Signal<f64>,
    pub(super) precursor_max: Signal<f64>,
    pub(super) adduct_filter: Signal<String>,
}

impl SummaryFilters {
    pub(super) fn read_adduct(&self) -> String {
        self.adduct_filter.read().clone()
    }
}
