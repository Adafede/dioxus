// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Unified UI design system for Dioxus applications.
//!
//! Provides a complete, type-safe design system with reusable components,
//! theme constants, and styling utilities—all defined in pure Rust.
//!
//! # Design Philosophy
//!
//! - **Pure-Rust theme tokens**: Colors, spacing, and typography defined in
//!   Rust via `StyleBuilder`, `ColorScheme`, `Spacing`, and `Typography`
//! - **Document head helper**: [`DocumentHead`](prelude::DocumentHead) manages
//!   page metadata, theme setup, and JSON-LD directly from Rust
//! - **Accessible components**: Semantic HTML landmarks, keyboard navigation,
//!   and high-contrast light/dark palettes
//!
//! # Example
//!
//! ```rust
//! use dioxus::prelude::*;
//! use ui::prelude::*;
//!
//! fn app() -> Element {
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
//!
//! let mut dom = VirtualDom::new(app);
//! dom.rebuild_in_place();
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
    pub use crate::common::{SKIP_LINK_STYLE, skip_link};
    pub use crate::components::{
        Button, ButtonVariant, Card, Footer, Header, NoticeBar, NoticeTone, SegmentedControl,
        SegmentedControlItem, SimpleAnalyticsBadge, UploadZone,
    };
    pub use crate::document::DocumentHead;
    pub use crate::theme::{
        ColorScheme, Interaction, Radius, Shadow, Spacing, StyleBuilder, Typography,
    };
    /// `capabilities_script` stays crate-private: apps opt in by passing
    /// `webmcp: Some(..)` to [`DocumentHead`], which builds the script itself.
    pub use crate::webmcp::{EdamAnnotations, EdamTerm, WebMcpConfig};
}
