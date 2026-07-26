//! Type-safe, theme-aware color and style definitions.
//!
//! This module demonstrates how to encode design tokens (colors, spacing, etc.)
//! in Rust types rather than magic strings or CSS variables. Benefits:
//! - Compile-time safety
//! - Easy refactoring
//! - Single source of truth
//! - Consistent theming

/// Light and dark color schemes inspired by Lotus Explorer.
///
/// Implements WCAG AAA contrast ratios:
/// - Light theme: Dark text (#1a1f2e) on light background (#f6f8fb)
/// - Dark theme: Light text (#f0f3f8) on dark background (#10141b)
#[derive(Clone, Copy, Debug)]
pub struct ColorScheme;

impl ColorScheme {
    /// Primary background (changes with system preference).
    pub const BG_PRIMARY_LIGHT: &'static str = "#f6f8fb";
    pub const BG_PRIMARY_DARK: &'static str = "#10141b";

    /// Secondary background (cards, panels).
    pub const BG_SECONDARY_LIGHT: &'static str = "#eef0f4";
    pub const BG_SECONDARY_DARK: &'static str = "#1a1f2a";

    /// Surface (elevated elements like cards).
    pub const SURFACE_LIGHT: &'static str = "#ffffff";
    pub const SURFACE_DARK: &'static str = "#1e2330";

    /// Primary text.
    pub const TEXT_PRIMARY_LIGHT: &'static str = "#1a1f2e";
    pub const TEXT_PRIMARY_DARK: &'static str = "#f0f3f8";

    /// Secondary text (muted, reduced prominence).
    pub const TEXT_SECONDARY_LIGHT: &'static str = "#4a5367";
    pub const TEXT_SECONDARY_DARK: &'static str = "#c5cad6";

    /// Tertiary text (hints, labels).
    pub const TEXT_TERTIARY_LIGHT: &'static str = "#6b7280";
    pub const TEXT_TERTIARY_DARK: &'static str = "#9099ab";

    /// Accent (interactive elements, links).
    pub const ACCENT_PRIMARY: &'static str = "#0b5cab";
    pub const ACCENT_SECONDARY: &'static str = "#084b8a";
    pub const ACCENT_TERTIARY: &'static str = "#0f7bc9";

    /// Border colors.
    pub const BORDER_PRIMARY_LIGHT: &'static str = "#d1d5db";
    pub const BORDER_PRIMARY_DARK: &'static str = "#3a3f49";

    /// Theme color for browser chrome.
    pub const THEME_COLOR_LIGHT: &'static str = "#f6f8fb";
    pub const THEME_COLOR_DARK: &'static str = "#10141b";
}
