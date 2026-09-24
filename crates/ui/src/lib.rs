// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Unified UI design system for Dioxus applications.
//!
//! Provides a complete, type-safe design system with reusable components,
//! theme constants, and styling utilities—all defined in pure Rust.
//!
//! # Design Philosophy
//!
//! - **CSS bundled as static assets**: App-wide CSS lives in external files
//!   (e.g. `lotus-explore.css`) loaded via `<link rel="stylesheet">` in the
//!   generated `index.html` for optimal mobile performance
//! - **Type-safe theming**: Compile-time checked colors, spacing, typography
//!   via `StyleBuilder`, `ColorScheme`, `Spacing` and `Typography`
//! - **Accessible components**: WCAG AAA contrast, keyboard navigation, semantic HTML
//! - **Lotus aesthetic**: Clean, professional design inspired by lotus-explore-rs
//!
//! # Example
//!
//! ```ignore
//! use dioxus::prelude::*;
//! use ui::prelude::*;
//! use ui::theme::{ColorScheme, Spacing, StyleBuilder};
//!
//! fn app() -> Element {
//!     let colors = ColorScheme::LIGHT;
//!
//!     rsx! {
//!         Header {
//!             title: "My App".to_string(),
//!         }
//!         div { style: StyleBuilder::new().padding(Spacing::LG).build(),
//!             Card {
//!                 title: "Content".to_string(),
//!                 "Body text here"
//!             }
//!         }
//!         Footer {}
//!     }
//! }
//! ```

mod common;
mod components;
mod document;
mod signals;
/// Theme design tokens and the [`StyleBuilder`](theme::StyleBuilder).
///
/// Public because apps consume these items via qualified paths such as
/// `ui::theme::StyleBuilder`.
pub mod theme;
mod webmcp;

/// Convenience re-exports of the most commonly used UI primitives.
pub mod prelude {
    pub use crate::common::{SKIP_LINK_STYLE, skip_link, skip_link_main};
    pub use crate::components::{
        Button, ButtonVariant, Card, Footer, Header, NoticeBar, NoticeTone, SegmentedControl,
        SegmentedControlItem, UploadZone,
    };
    pub use crate::document::DocumentHead;
    pub use crate::theme::{
        ColorScheme, Interaction, Radius, Shadow, Spacing, StyleBuilder, Typography,
    };
    pub use crate::webmcp::{WebMcpConfig, capabilities_script};
}
