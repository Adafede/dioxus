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
//!   Rust via `StyleBuilder`, `Theme`, `ColorScheme`, `Spacing`, and `Typography`
//! - **Document head helper**: [`DocumentHead`] manages
//!   page metadata, theme setup, and JSON-LD directly from Rust
//! - **Accessible components**: Semantic HTML landmarks, keyboard navigation,
//!   and high-contrast light/dark palettes
//!
//! # Example
//!
//! ```rust
//! use dioxus::prelude::*;
//! use ui::{Card, Footer, Header, Spacing, StyleBuilder};
//!
//! fn app() -> Element {
//!     rsx! {
//!         Header {
//!             title: "My App".to_string(),
//!         }
//!         div { style: StyleBuilder::with_capacity(64).padding(Spacing::LG).build(),
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

#![warn(missing_docs)]

mod a11y;
mod components;
mod document;
mod signals;
mod theme;
mod url;
mod webmcp;

pub use a11y::skip_link;
pub use components::button::{Button, ButtonVariant};
pub use components::card::Card;
pub use components::footer::Footer;
pub use components::header::Header;
pub use components::notice::{NoticeBar, NoticeTone};
pub use components::segmented_control::{SegmentedControl, SegmentedControlItem};
pub use components::simple_analytics_badge::SimpleAnalyticsBadge;
pub use components::upload_zone::UploadZone;
pub use document::DocumentHead;
pub use theme::{ColorScheme, Radius, Shadow, Spacing, StyleBuilder, Theme, Typography};
pub use webmcp::{EdamAnnotations, EdamTerm, WebMcpConfig};
