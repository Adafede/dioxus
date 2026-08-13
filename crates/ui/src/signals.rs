// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared signal-declaration abstraction.
//!
//! Across the workspace, component functions repeatedly declare local Dioxus
//! signals behind a `#[cfg(target_arch = "wasm32")]` / `#[cfg(not(...))]` fence,
//! differing only in the `mut` qualifier of the binding:
//!
//! ```text
//! #[cfg(target_arch = "wasm32")]
//! let file_name = use_signal(String::new);
//! #[cfg(not(target_arch = "wasm32"))]
//! let mut file_name = use_signal(String::new);
//! ```
//!
//! The web build only ever *reads* some signals, so the wasm arm binds without
//! `mut`; the native build calls `.set` on the same signals, requiring `mut`.
//! Duplicating this pair at every call site is purely mechanical repetition.
//!
//! The [`shared_signal!`] / [`shared_signals!`] macros collapse the pair into a
//! single declaration, emitting the identical cfg-gated arms so that
//! mutability-per-platform behaviour is preserved exactly. Usage:
//!
//! ```ignore
//! ui::shared_signal!(file_name, String::new);
//! ui::shared_signals! {
//!     metrics: || None::<PrecursorStats>,
//!     busy: || false,
//! }
//! ```

/// Declare a single local signal without duplicating the wasm/native
/// `#[cfg]` pair.
///
/// `$init` is forwarded verbatim to `dioxus::prelude::use_signal`, so it may
/// be a function item (`String::new`), a closure (`|| false`), or any
/// `FnOnce() -> T` expression.
#[macro_export]
macro_rules! shared_signal {
    ($name:ident, $init:expr $(,)?) => {
        #[cfg(target_arch = "wasm32")]
        let $name = ::dioxus::prelude::use_signal($init);
        #[cfg(not(target_arch = "wasm32"))]
        let mut $name = ::dioxus::prelude::use_signal($init);
    };
}

/// Declare several local signals in one call, collapsing each into its
/// wasm/native `#[cfg]` pair (see [`shared_signal!`]).
#[macro_export]
macro_rules! shared_signals {
    ( $( $name:ident : $init:expr ),* $(,)? ) => {
        $(
            #[cfg(target_arch = "wasm32")]
            let $name = ::dioxus::prelude::use_signal($init);
            #[cfg(not(target_arch = "wasm32"))]
            let mut $name = ::dioxus::prelude::use_signal($init);
        )*
    };
}
