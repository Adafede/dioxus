//! `smellfish-rs` — literature-backed NP-likeness scoring.
//!
//! Scores natural-product-likeness of SMILES structures using machine-learned
//! features, `Query` enrichment, and RDKit.js chemistry descriptors.
//!
//! # Run locally
//!
//! ```bash
//! dx serve --package smellfish-rs
//! ```
//!
//! # Build for deployment
//!
//! ```bash
//! dx build --release --platform web --package smellfish-rs
//! ```
//!
//! # Tests
//!
//! ```bash
//! cargo test --lib -p smellfish-rs
//! ```

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
    clippy::checked_conversions,
    // Library crate exposures
    clippy::too_long_first_doc_paragraph,
    missing_debug_implementations,
    clippy::duplicated_attributes,
)]
#![cfg_attr(test, allow(unused_imports))]

pub mod app;
#[cfg(any(test, target_arch = "wasm32"))]
pub mod csv;
pub mod document_head;
#[cfg(any(test, target_arch = "wasm32"))]
pub mod evidence;
pub mod literature;
pub mod model;
#[cfg(target_arch = "wasm32")]
pub mod pipeline;
#[cfg(target_arch = "wasm32")]
pub mod qlever;
#[cfg(target_arch = "wasm32")]
pub mod rdkit;
pub mod styles;

pub use app::app;
