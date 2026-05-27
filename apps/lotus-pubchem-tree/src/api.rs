// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

mod client;
mod config;
mod dto;
mod error;

pub use client::{build_pubchem_trees, fetch_pubchem_dataset};
pub use config::{api_base_url, resolve_api_url};
pub use dto::{
    DataStatsDto, DownloadArtifactDto, PreviewNodeDto, PreviewTreeDto, PubchemBuildResponse,
    TreeSummaryDto,
};
pub use error::ApiClientError;
