//! Reusable UI components for the landing page.
//!
//! Demonstrates best practices for component design:
//! - Single responsibility per component
//! - Semantic HTML
//! - Proper ARIA annotations
//! - Type-safe props
//! - Comprehensive documentation

pub mod app_card;
pub mod footer;
pub mod header;

pub use app_card::AppCard;
pub use footer::Footer;
pub use header::Header;

/// Metadata for an application card.
///
/// Used by [`AppCard`] to render application information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppInfo {
    /// Unique identifier (used as React key).
    pub id: &'static str,
    /// Emoji and title (e.g., "🪷 LOTUS Wikidata Explorer").
    pub title: &'static str,
    /// Relative path to application (e.g., "./lotus-explorer/").
    pub path: &'static str,
    /// Short, clear description of functionality.
    pub description: &'static str,
}
