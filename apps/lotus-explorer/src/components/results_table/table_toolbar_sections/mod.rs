// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Sub-components for the ResultsTable toolbar sections.
//!
//! Each sub-module owns exactly one visual concern:
//! * [`query_panel`] — collapsible SPARQL query viewer with copy button
//! * [`stat_bar`] — dataset statistics badges and the capped-rows notice
//! * [`download_actions`] — download button group (CSV, JSON, RDF, metadata) and QLever link

mod download_actions;
mod query_panel;
mod stat_bar;

pub use download_actions::DownloadActionsGroup;
pub use query_panel::QueryPanel;
pub use stat_bar::{CappedRowsNotice, StatBar};
