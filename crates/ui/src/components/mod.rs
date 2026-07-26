//! Reusable Dioxus components for unified UI across all applications.
//!
//! All components use pure Rust styling via [`crate::theme`], no external CSS files.

pub mod header;
pub mod footer;
pub mod card;
pub mod button;

pub use header::Header;
pub use footer::Footer;
pub use card::Card;
pub use button::Button;
