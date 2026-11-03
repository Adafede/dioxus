// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Shared types for the `app` module.
//!
//! These types cross-cut rendering, gallery preparation, and download
//! logic — extracting them to a dedicated module keeps every other file
//! focused on a single responsibility.

use dioxus::prelude::Signal;

/// Grouped mutable filter signals for the summary panel.
///
/// Bundling the signals into a struct keeps the [`summary`](super::components::summary)
/// function signature readable and avoids passing twelve individual arguments.
#[derive(Clone)]
pub(super) struct SummaryFilters {
    pub selected_classes: Signal<Vec<String>>,
    pub mz_min: Signal<f64>,
    pub mz_max: Signal<f64>,
    pub precursor_min: Signal<f64>,
    pub precursor_max: Signal<f64>,
    pub adduct_filter: Signal<String>,
}
