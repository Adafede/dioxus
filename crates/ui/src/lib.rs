//! Unified UI design system for Dioxus applications.
//!
//! Provides a complete, type-safe design system with reusable components,
//! theme constants, and styling utilities—all defined in pure Rust.
//!
//! # Design Philosophy
//!
//! - **No external CSS files**: All styling defined as Rust constants via [`theme`]
//! - **Type-safe theming**: Compile-time checked colors, spacing, typography
//! - **Accessible components**: WCAG AAA contrast, keyboard navigation, semantic HTML
//! - **Lotus aesthetic**: Clean, professional design inspired by lotus-explorer
//! - **Zero runtime overhead**: All styles inline, no dynamic CSS generation
//!
//! # Example
//!
//! ```ignore
//! use dioxus::prelude::*;
//! use ui::prelude::*;
//! use ui::theme::{ColorScheme, Spacing};
//!
//! fn app() -> Element {
//!     let colors = ColorScheme::LIGHT;
//!
//!     rsx! {
//!         Header {
//!             title: "My App".to_string(),
//!         }
//!         div { style: "padding: {}", Spacing::LG,
//!             Card {
//!                 title: "Content".to_string(),
//!                 "Body text here"
//!             }
//!         }
//!         Footer {}
//!     }
//! }
//! ```

pub mod components;
pub mod theme;

pub mod prelude {
    pub use crate::components::{Button, Card, Footer, Header};
    pub use crate::theme::{
        ColorScheme, Interaction, Radius, Shadow, Spacing, StyleBuilder, Typography,
    };
}
