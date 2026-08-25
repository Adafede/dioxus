// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! `lotus-explore-rs` — LOTUS Knowledge Explorer.
//!
//! A linked open data (LOAD) explorer for the LOTUS compound-taxon-reference
//! knowledge graph from Wikidata, queried via SPARQL.  Powered by the `lotus`
//! shared crate and the QLever SPARQL endpoint.
//!
//! # Quick start
//!
//! ```bash
//! dx serve --package lotus-explore-rs
//! ```
//!
//! To also run the optional API:
//!
//! ```bash
//! cargo run --locked -p lotus-api
//! ```
//!
//! Then open `http://localhost:8080/?api_base=http://127.0.0.1:8787`.
//!
//! Without `lotus-api`, the explorer falls back to direct QLever/SPARQL queries.
//!
//! # Architecture
//!
//! See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) for the full architectural
//! overview.
//!
//! # Engineering skills
//!
//! - [`SKILLS.md`](./SKILLS.md)
//! - [`docs/skills/SUGGESTIONS.md`](./docs/skills/SUGGESTIONS.md)
//!
//! # Curation share links
//!
//! - [`docs/CURATION_SHARE_LINKS.md`](./docs/CURATION_SHARE_LINKS.md)
//!
//! # Development testing
//!
//! Run logging format tests during telemetry work:
//!
//! ```bash
//! cargo test --locked -p lotus-explore-rs utils::logging::tests
//! ```
//!
//! # Setup: external assets
//!
//! RDKit.js and citation.js are loaded from CDN on demand by the curation
//! workflow when their respective operations first need them (no local
//! download needed). The @citation-js/plugin-quickstatements output formatter
//! is registered inline with citation.js after it loads.
//! All document `<head>` metadata, scripts, and styles are managed in Rust
//! via `ui::document::DocumentHead` — see `src/document_head.rs`.
//!
//! Ketcher (115 MB) must be fetched before serving or deploying:
//!
//! ```bash
//! ./scripts/fetch-ketcher.sh
//! ```
//!
//! # Citation
//!
//! - Paper (DOI): <https://doi.org/10.7554/eLife.70780>
//! - BibTeX: [`public/docs/references.bib`](./public/docs/references.bib)
//!
//! # Site metadata
//!
//! `public/llms.txt`, `public/humans.txt`, `public/robots.txt`,
//! `public/.well-known/security.txt`, `public/_headers`, and
//! `public/site.webmanifest` are generated from
//! [`metadata/site-metadata.json`](./metadata/site-metadata.json).
//!
//! # Explorer ⇄ API integration
//!
//! | Scenario                | `api_base` source                     | API used            |
//! | ----------------------- | ------------------------------------- | ------------------- |
//! | Codeberg Pages (public) | none                                  | ✗ direct SPARQL     |
//! | Local dev               | auto-detected `http://127.0.0.1:8787` | ✓ if server running |
//! | Build-time              | `LOTUS_API_BASE` env var              | ✓                   |
//! | Runtime override        | `?api_base=…` query param             | ✓                   |
//!
//! # URL automation
//!
//! URL-driven execution and exports:
//!
//! - `?execute=true` --- run query on load
//! - `?download=true&format=csv` --- download CSV
//! - `?download=true&format=json` --- download SPARQL Results JSON
//! - `?download=true&format=rdf` --- download RDF (Turtle)
//!
//! When both `download` and `execute` are present, `download` takes priority.
//!
//! # Archive
//!
//! A frozen version is archived on Zenodo: <https://doi.org/10.5281/zenodo.5794106>

#![allow(non_snake_case)] // Dioxus component naming convention
#![allow(
    // High-noise lints: false positives or antipatterns in WASM web UI context
    clippy::future_not_send,
    clippy::unused_async,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    // Patterns legitimate in complex UI codebases:
    clippy::too_many_lines,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::match_same_arms,
    // Type coercion lints disabled due to UI framework requirements
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    // Code organization patterns intentional in this codebase
    clippy::struct_field_names,
    clippy::struct_excessive_bools,
    clippy::fn_params_excessive_bools,
    clippy::redundant_pub_crate,
    // Performance lints with narrow optimization windows
    clippy::trivially_copy_pass_by_ref,
    clippy::duration_suboptimal_units,
    clippy::redundant_clone,
    clippy::redundant_closure_for_method_calls,
    // Stylistic choices preferred in this codebase
    clippy::manual_let_else,
    clippy::if_not_else,
    clippy::or_fun_call,
    clippy::no_effect_underscore_binding,
    clippy::semicolon_if_nothing_returned,
    clippy::unreadable_literal,
    clippy::uninlined_format_args,
    clippy::format_push_string,
    // Safe patterns disabled
    clippy::needless_pass_by_value,
    clippy::needless_pass_by_ref_mut,
    clippy::significant_drop_tightening,
    clippy::single_match_else,
    clippy::single_option_map,
    clippy::unnested_or_patterns,
    clippy::default_trait_access,
    clippy::explicit_iter_loop,
    clippy::derive_partial_eq_without_eq,
    clippy::ignored_unit_patterns,
    clippy::large_types_passed_by_value,
    clippy::manual_string_new,
    clippy::option_as_ref_cloned,
    clippy::float_cmp,
    clippy::unused_self,
    clippy::checked_conversions
)]

mod api;
mod app;
mod app_state;
mod components;
mod core;
mod curation;
mod document_head;
mod download;
mod export;
mod features;
mod hooks;
mod i18n;
mod models;
mod pages;
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
mod tests;

fn main() {
    let level = if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    console_log::init_with_level(level).ok();
    launch(app::shell::AppRoot);
}
