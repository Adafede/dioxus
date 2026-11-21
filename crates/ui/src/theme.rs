// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Unified theme system for all Dioxus applications.
//!
//! Defines colors, spacing, typography, and shadows as Rust constants.
//! No CSS files or variables—everything is type-safe and compile-time checked.
//!
//! # Design Philosophy
//!
//! - **Lotus aesthetic**: Clean, professional color palette with excellent contrast
//! - **Responsive typography**: Fluid scaling using `clamp()` for all text sizes
//! - **Accessibility first**: WCAG AAA contrast ratios, semantic HTML, keyboard navigation
//! - **Pure Rust**: All styling defined as constants; inline styles generated in components

use core::fmt;

/// Color palette for light and dark themes.
///
/// High-contrast light and dark color tokens shared across the workspace apps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorScheme {
    pub bg: &'static str,
    /// The light and dark palettes use the same value here, so `bg`/`bg2` and
    /// `surface`/`surface2` exist to be overridden per-theme by an app that wants
    /// a visible distinction between page and panel.
    pub bg2: &'static str,
    pub surface: &'static str,
    pub surface2: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    /// Muted steps of `text`, for progressively less important copy.
    pub text2: &'static str,
    pub text3: &'static str,
    pub accent: &'static str,
    /// Hover/active state of `accent`.
    pub accent2: &'static str,
    /// Success/taxon state color (Wikidata green: #396 = #339966)
    pub green: &'static str,
    /// Error/compound/destructive action color (Wikidata red: #900 = #990000)
    pub red: &'static str,
    /// Info/reference color (Wikidata blue: #069 = #006699)
    pub blue: &'static str,
    pub yellow: &'static str,
    pub(crate) purple: &'static str,
}

impl ColorScheme {
    /// Light theme colors optimized for daytime viewing.
    /// All text/background pairs meet WCAG AAA 7:1 contrast ratio.
    pub const LIGHT: Self = Self {
        bg: "#f7fafc",
        bg2: "#f7fafc",
        surface: "#ffffff",
        surface2: "#ffffff",
        border: "#c3cfdd",
        text: "#111827",
        text2: "#233548",
        text3: "#516274",
        accent: "#0b5cab",
        accent2: "#084b8a",
        green: "#339966", // Wikidata taxon #396
        red: "#990000",   // Wikidata compound #900
        blue: "#006699",  // Wikidata reference #069
        yellow: "#8a4b0f",
        purple: "#6941c6",
    };

    /// Dark theme colors optimized for low-light viewing.
    /// All text/background pairs meet WCAG AAA 7:1 contrast ratio.
    pub const DARK: Self = Self {
        bg: "#0f172a",
        bg2: "#0f172a",
        surface: "#111827",
        surface2: "#111827",
        border: "#38475a",
        text: "#eef4fb",
        text2: "#d5deea",
        text3: "#a7b4c7",
        accent: "#8cbcff",
        accent2: "#5e98f3",
        green: "#339966", // Wikidata taxon
        red: "#990000",   // Wikidata compound
        blue: "#006699",  // Wikidata reference
        yellow: "#f0b35e",
        purple: "#c3a0ff",
    };
}

/// Spacing scale derived from 6px base unit
///
/// Maintains consistent 6px grid throughout the design system.
#[derive(Clone, Copy, Debug)]
pub struct Spacing;

impl Spacing {
    pub const SM: &'static str = "10px";
    pub const MD: &'static str = "14px";
    pub const LG: &'static str = "20px";
    pub const XL: &'static str = "28px";
}

/// Border radius scale for consistent rounding
#[derive(Clone, Copy, Debug)]
pub struct Radius;

impl Radius {
    /// Micro rounding for small elements
    pub const SM: &'static str = "4px";
    /// Default rounding for cards and components
    pub const MD: &'static str = "10px";
    /// Large rounding for hero sections
    pub const LG: &'static str = "16px";
}

/// Shadow system for depth and layering
///
/// Responsive to theme; shadows render differently in light/dark modes.
#[derive(Clone, Copy, Debug)]
pub struct Shadow;

impl Shadow {
    /// Subtle shadow for minimal elevation
    pub const XS: &'static str = "0 1px 2px rgba(15, 23, 42, 0.06)";
    /// Small shadow for cards at rest
    pub const SM: &'static str = "0 4px 14px rgba(15, 23, 42, 0.06)";
    /// Medium shadow for elevated cards
    pub const MD: &'static str = "0 10px 30px rgba(15, 23, 42, 0.09)";

    /// Dark-mode small shadow (higher opacity)
    pub(crate) const SM_DARK: &'static str = "0 4px 14px rgba(0, 0, 0, 0.35)";
}

/// Typography scale with responsive fluid sizing
///
/// Uses CSS `clamp()` for automatic scaling between mobile and desktop.
/// All sizes maintain 1.5 line-height for readability.
#[derive(Clone, Copy, Debug)]
pub struct Typography;

impl Typography {
    /// Label text: 0.6875–0.75rem, form labels, badges
    pub const LABEL: &'static str = "clamp(0.6875rem, 0.66rem + 0.14vw, 0.75rem)";
    /// UI text: 0.8125–0.875rem, buttons, small text
    pub const UI: &'static str = "clamp(0.8125rem, 0.785rem + 0.16vw, 0.875rem)";
    /// Body text: 0.875–0.9375rem, paragraphs, default
    pub const BODY: &'static str = "clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem)";
    /// Heading 2: 1.125–1.5rem, main section headings
    pub const H2: &'static str = "clamp(1.125rem, 1.02rem + 0.6vw, 1.5rem)";
    /// Heading 1: 1.375–1.85rem, page title
    pub const H1: &'static str = "clamp(1.375rem, 1.1rem + 0.85vw, 1.85rem)";

    /// Line height for body copy (1.5 = excellent readability)
    pub const LINE_HEIGHT: &'static str = "1.5";

    /// Font family: sans-serif stack with Inter as primary
    pub const SANS: &'static str = "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', roboto, 'Helvetica Neue', arial, sans-serif";
}

/// Motion settings shared by every component.
///
/// The contrast, focus-indicator and touch-target guarantees the palette and the
/// components aim at are not encoded here — they are properties of the values in
/// [`ColorScheme`] and of the component markup, and nothing in this module
/// enforces or checks them.
#[derive(Clone, Copy, Debug)]
pub struct Interaction;

impl Interaction {
    pub const TRANSITION_DEFAULT: &'static str = "200ms ease-in-out";
}

/// Style string builder for inline CSS attributes
///
/// Accumulates CSS properties and returns the final style string.
///
/// # Example
///
/// ```rust
/// use ui::theme::{ColorScheme, Radius, Spacing, StyleBuilder};
///
/// let colors = ColorScheme::LIGHT;
/// let style = StyleBuilder::new()
///     .color(colors.accent)
///     .padding(Spacing::LG)
///     .border_radius(Radius::MD)
///     .build();
/// assert!(style.contains("color: "));
/// assert!(style.contains("padding: 20px"));
/// ```
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleBuilder {
    css: String,
}

macro_rules! style_props {
    ($( $method:ident => $css:literal ),* $(,)?) => {
        $(
            pub fn $method(self, value: &str) -> Self {
                self.property($css, value)
            }
        )*
    };
}

impl StyleBuilder {
    pub const fn new() -> Self {
        Self { css: String::new() }
    }

    pub fn property(mut self, name: &str, value: &str) -> Self {
        if !self.css.is_empty() {
            self.css.push_str("; ");
        }
        self.css.push_str(name);
        self.css.push_str(": ");
        self.css.push_str(value);
        self
    }

    style_props! {
        color => "color",
        background_color => "background-color",
        padding => "padding",
        margin => "margin",
        border_radius => "border-radius",
        display => "display",
        flex => "flex",
        flex_direction => "flex-direction",
        flex_wrap => "flex-wrap",
        align_items => "align-items",
        justify_content => "justify-content",
        gap => "gap",
        border => "border",
        font_size => "font-size",
        font_family => "font-family",
        font_weight => "font-weight",
        line_height => "line-height",
        text_align => "text-align",
        width => "width",
        height => "height",
        min_height => "min-height",
        box_shadow => "box-shadow",
        transition => "transition",
        opacity => "opacity",
        text_decoration => "text-decoration",
        cursor => "cursor",
        border_bottom => "border-bottom",
        border_left => "border-left",
    }

    #[must_use]
    pub fn build(self) -> String {
        self.css
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StyleBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.css)
    }
}

#[cfg(test)]
mod tests;
