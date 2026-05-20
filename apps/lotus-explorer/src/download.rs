// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared download helpers for browser/native targets, including format handling & deduplication.

use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use crate::models::SearchCriteria;
use crate::perf;

mod coordinator;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadFormat {
    Csv,
    Json,
    Rdf,
}

impl DownloadFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        let normalized = s.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "csv" => Some(Self::Csv),
            "json" | "ndjson" => Some(Self::Json),
            "rdf" => Some(Self::Rdf),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    pub fn log_name(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    pub fn timer_label(&self) -> &'static str {
        match self {
            Self::Csv => "LOTUS:download_csv",
            Self::Json => "LOTUS:download_json",
            Self::Rdf => "LOTUS:download_rdf",
        }
    }

    pub fn trigger_timer_label(&self) -> String {
        format!("{}_trigger", self.timer_label())
    }
}

/// Execute a download in the given format using direct query export.
pub async fn execute_download(
    format: DownloadFormat,
    #[cfg(target_arch = "wasm32")] criteria: std::sync::Arc<SearchCriteria>,
    query: Arc<str>,
    filename: String,
) -> Result<(), String> {
    let dl_timer = perf::start_timer(format.timer_label());
    log::info!("event=download format={} state=started", format.log_name());

    #[cfg(target_arch = "wasm32")]
    {
        return wasm::execute_download_wasm(format, criteria, query, filename, dl_timer).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        native::execute_download_native(format, query, filename, dl_timer).await
    }
}

pub fn trigger_download(filename: &str, mime: &str, content_or_url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::trigger_download(filename, mime, content_or_url);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        native::trigger_download(filename, mime, content_or_url);
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
pub fn trigger_download_url(filename: &str, url: &str) {
    wasm::trigger_download_url(filename, url);
}

#[cfg(test)]
mod tests {
    use super::DownloadFormat;

    #[test]
    fn parse_download_format_supports_documented_aliases() {
        assert_eq!(DownloadFormat::from_str("csv"), Some(DownloadFormat::Csv));
        assert_eq!(DownloadFormat::from_str("json"), Some(DownloadFormat::Json));
        assert_eq!(
            DownloadFormat::from_str("ndjson"),
            Some(DownloadFormat::Json)
        );
        assert_eq!(DownloadFormat::from_str("rdf"), Some(DownloadFormat::Rdf));
        assert_eq!(
            DownloadFormat::from_str(" JSON "),
            Some(DownloadFormat::Json)
        );
        assert_eq!(DownloadFormat::from_str("RDF"), Some(DownloadFormat::Rdf));
        assert_eq!(DownloadFormat::from_str("ttl"), None);
    }
}
