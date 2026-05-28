// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(non_snake_case)] // Dioxus component naming convention
#![allow(
    // High-noise lints: false positives or anti-patterns in WASM web UI context
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
