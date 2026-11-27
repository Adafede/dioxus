// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Shared signal state for `lipid-selecto-rs` UI components.

use dioxus::prelude::{ReadableExt, Signal};
use lipid_core::{ClassMatchSet, FilterCriteria};

/// Maximum number of structure cards rendered in the interactive gallery grid.
pub(super) const MAX_GALLERY_ITEMS: usize = 96;

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
    pub(super) fn with_criteria<R>(&self, f: impl FnOnce(&FilterCriteria<'_>) -> R) -> R {
        let adduct = self.adduct_filter.read();
        let criteria = FilterCriteria {
            selected_classes: *self.selected_classes.read(),
            mz_min: *self.mz_min.read(),
            mz_max: *self.mz_max.read(),
            precursor_min: *self.precursor_min.read(),
            precursor_max: *self.precursor_max.read(),
            adduct: &adduct,
        };
        f(&criteria)
    }
}
