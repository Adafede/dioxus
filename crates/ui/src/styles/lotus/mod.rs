// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus Knowledge Explorer design tokens.
//!
//! The CSS is now shipped as a static asset (`public/assets/lotus-explore.css`)
//! loaded via `<link rel="stylesheet">` in the generated `index.html` (see the
//! `[web.resource]` section in `Dioxus.toml`).  This module retains only the
//! token constants that are consumed by `StyleBuilder` at runtime for
//! per-element inline styles.

pub mod tokens;

pub use tokens::*;
