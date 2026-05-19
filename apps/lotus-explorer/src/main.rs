// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(non_snake_case)]

mod api;
mod app;
mod app_state;
mod components;
mod core;
mod curation;
mod data;
mod download;
mod export;
mod features;
mod hooks;
mod i18n;
mod models;
mod perf;
mod queries;
mod repositories;
mod services;
mod sparql;
mod state;
mod ui;
mod utils;

use dioxus::prelude::*;
#[cfg(test)]
use download::DownloadFormat;

fn main() {
    let level = if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    console_log::init_with_level(level).ok();
    launch(app::shell::AppRoot);
}

#[cfg(test)]
fn is_supported_download_format(fmt: &str) -> bool {
    DownloadFormat::from_str(fmt).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_download_formats_include_documented_values() {
        assert!(is_supported_download_format("csv"));
        assert!(is_supported_download_format("json"));
        assert!(is_supported_download_format("ndjson"));
        assert!(is_supported_download_format("rdf"));
        assert!(!is_supported_download_format("ttl"));
    }
}
