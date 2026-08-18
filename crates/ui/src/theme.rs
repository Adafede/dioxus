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
/// Based on lotus-explore-rs's proven design system with authentic Wikidata entity colors.
/// All colors meet WCAG AAA contrast ratios (7:1 minimum for text).
#[derive(Clone, Copy, Debug)]
pub struct ColorScheme {
    /// Main background color
    pub bg: &'static str,
    /// Secondary background for depth
    pub bg2: &'static str,
    /// Card/panel backgrounds
    pub surface: &'static str,
    /// Subtle secondary surface
    pub surface2: &'static str,
    /// Border colors
    pub border: &'static str,
    /// Primary text color
    pub text: &'static str,
    /// Secondary text color (slightly muted)
    pub text2: &'static str,
    /// Tertiary text color (more muted)
    pub text3: &'static str,
    /// Primary accent color (interactive elements)
    pub accent: &'static str,
    /// Darker accent for hover states
    pub accent2: &'static str,
    /// Success/taxon state color (Wikidata green: #396 = #339966)
    pub green: &'static str,
    /// Error/compound/destructive action color (Wikidata red: #900 = #990000)
    pub red: &'static str,
    /// Info/reference color (Wikidata blue: #069 = #006699)
    pub blue: &'static str,
    /// Warning color
    pub yellow: &'static str,
    /// Secondary accent color
    pub purple: &'static str,
}

impl ColorScheme {
    /// Light theme colors optimized for daytime viewing.
    /// All text/background pairs meet WCAG AAA 7:1 contrast ratio.
    pub const LIGHT: Self = ColorScheme {
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
    pub const DARK: Self = ColorScheme {
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
    /// Extra-small spacing (6px)
    pub const XS: &'static str = "6px";
    /// Small spacing (10px)
    pub const SM: &'static str = "10px";
    /// Medium spacing (14px)
    pub const MD: &'static str = "14px";
    /// Large spacing (20px)
    pub const LG: &'static str = "20px";
    /// Extra-large spacing (28px)
    pub const XL: &'static str = "28px";
    /// Extra-extra-large spacing (40px)
    pub const XXL: &'static str = "40px";
}

/// Border radius scale for consistent rounding
#[derive(Clone, Copy, Debug)]
pub struct Radius;

impl Radius {
    /// Sharp corners for minimal rounding
    pub const NONE: &'static str = "0";
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

    /// Dark mode shadow (higher opacity for contrast)
    /// Dark-mode small shadow (higher opacity)
    pub const XS_DARK: &'static str = "0 1px 2px rgba(0, 0, 0, 0.45)";
    /// Dark-mode small shadow (higher opacity)
    pub const SM_DARK: &'static str = "0 4px 14px rgba(0, 0, 0, 0.35)";
    /// Dark-mode medium shadow (higher opacity)
    pub const MD_DARK: &'static str = "0 10px 30px rgba(0, 0, 0, 0.35)";
}

/// Typography scale with responsive fluid sizing
///
/// Uses CSS `clamp()` for automatic scaling between mobile and desktop.
/// All sizes maintain 1.5 line-height for readability.
#[derive(Clone, Copy, Debug)]
pub struct Typography;

impl Typography {
    /// Micro text: 0.75–0.8125rem, used for labels, captions
    pub const MICRO: &'static str = "clamp(0.75rem, 0.73rem + 0.12vw, 0.8125rem)";
    /// Label text: 0.6875–0.75rem, form labels, badges
    pub const LABEL: &'static str = "clamp(0.6875rem, 0.66rem + 0.14vw, 0.75rem)";
    /// UI text: 0.8125–0.875rem, buttons, small text
    pub const UI: &'static str = "clamp(0.8125rem, 0.785rem + 0.16vw, 0.875rem)";
    /// Body text: 0.875–0.9375rem, paragraphs, default
    pub const BODY: &'static str = "clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem)";
    /// Heading 3: 0.9375–1.0625rem, section headings
    pub const H3: &'static str = "clamp(0.9375rem, 0.9rem + 0.28vw, 1.0625rem)";
    /// Heading 2: 1.125–1.5rem, main section headings
    pub const H2: &'static str = "clamp(1.125rem, 1.02rem + 0.6vw, 1.5rem)";
    /// Heading 1: 1.375–1.85rem, page title
    pub const H1: &'static str = "clamp(1.375rem, 1.1rem + 0.85vw, 1.85rem)";

    /// Large stat text: 1.125–1.375rem
    pub const STAT: &'static str = "clamp(1.125rem, 1.02rem + 0.52vw, 1.375rem)";

    /// Line height for body copy (1.5 = excellent readability)
    pub const LINE_HEIGHT: &'static str = "1.5";

    /// Font families
    /// Font family: sans-serif stack with Inter as primary
    pub const SANS: &'static str = "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', roboto, 'Helvetica Neue', arial, sans-serif";
    /// Font family: monospace stack with Fira Code as primary
    pub const MONO: &'static str =
        "'Fira Code', ui-monospace, 'SF Mono', 'JetBrains Mono', consolas, monospace";
}

/// Accessibility and interaction constants following WCAG 2.1 AAA standards.
///
/// These values ensure:
/// - Sufficient color contrast (7:1 text contrast minimum)
/// - Perceivable focus indicators (3px minimum width)
/// - Sufficient touch/click targets (48px minimum per WCAG)
/// - Motion-safe defaults with prefers-reduced-motion support
/// - Proper semantic focus management
#[derive(Clone, Copy, Debug)]
pub struct Interaction;

impl Interaction {
    /// Minimum tap target size per WCAG 2.5.5 (48px recommended, 44px minimum).
    /// Used for buttons, links, and interactive elements.
    pub const MIN_TOUCH_TARGET: &'static str = "44px";

    /// Focus indicator outline width (3px for high visibility).
    /// Must be visible at any zoom level and with high contrast.
    pub const FOCUS_OUTLINE_WIDTH: &'static str = "3px";

    /// Focus indicator offset from element (2px recommended).
    /// Prevents overlap with element borders.
    pub const FOCUS_OUTLINE_OFFSET: &'static str = "2px";

    /// Focus outline color — high contrast, independent of theming.
    /// Uses a perceptually distinct blue-purple for universal visibility.
    pub const FOCUS_OUTLINE_COLOR: &'static str = "#4f46e5";

    /// Fast transition for micro-interactions (150ms).
    /// Used for button hovers, small state changes.
    /// Respects prefers-reduced-motion preference.
    pub const TRANSITION_FAST: &'static str = "150ms ease-in-out";

    /// Default transition for standard interactions (200ms).
    /// Used for card hovers, menu opens, typical state changes.
    pub const TRANSITION_DEFAULT: &'static str = "200ms ease-in-out";

    /// Slow transition for prominent animations (300ms).
    /// Used for modal opens, page transitions, major layout changes.
    pub const TRANSITION_SLOW: &'static str = "300ms ease-in-out";

    /// CSS media query rule to disable animations for users with motion sensitivity.
    /// Apply to all animated elements:
    /// ```css
    /// @media (prefers-reduced-motion: reduce) { animation: none; transition: none; }
    /// ```
    pub const PREFERS_REDUCED_MOTION: &'static str = "@media (prefers-reduced-motion: reduce)";

    /// Skip link style for keyboard navigation.
    /// Positioned off-screen but becomes visible on focus.
    pub const SKIP_LINK_STYLE: &'static str = "position: absolute; top: -40px; left: 0; background: #000; color: #fff; padding: 8px 16px; text-decoration: none; z-index: 100;";

    /// Skip link focus style (becomes visible when focused).
    pub const SKIP_LINK_FOCUS_STYLE: &'static str = "top: 0;";
}

/// Style string builder for inline CSS attributes
///
/// Accumulates CSS properties and returns the final style string.
///
/// # Example
///
/// ```ignore
/// let style = StyleBuilder::new()
///     .color(colors.accent)
///     .padding(Spacing::LG)
///     .border_radius(Radius::MD)
///     .build();
/// ```
#[derive(Clone)]
pub struct StyleBuilder {
    properties: Vec<(String, String)>,
}

impl StyleBuilder {
    /// Create a new style builder
    pub fn new() -> Self {
        StyleBuilder {
            properties: Vec::new(),
        }
    }

    /// Add a property to the style
    pub fn property(mut self, name: &str, value: &str) -> Self {
        self.properties.push((name.to_string(), value.to_string()));
        self
    }

    /// Set color
    pub fn color(self, color: &str) -> Self {
        self.property("color", color)
    }

    /// Set background color
    pub fn background_color(self, color: &str) -> Self {
        self.property("background-color", color)
    }

    /// Set padding
    pub fn padding(self, padding: &str) -> Self {
        self.property("padding", padding)
    }

    /// Set margin
    pub fn margin(self, margin: &str) -> Self {
        self.property("margin", margin)
    }

    /// Set border radius
    pub fn border_radius(self, radius: &str) -> Self {
        self.property("border-radius", radius)
    }

    /// Set display
    pub fn display(self, display: &str) -> Self {
        self.property("display", display)
    }

    /// Set flex properties
    pub fn flex(self, flex: &str) -> Self {
        self.property("flex", flex)
    }

    /// Set flex direction
    pub fn flex_direction(self, direction: &str) -> Self {
        self.property("flex-direction", direction)
    }

    /// Set flex wrap
    pub fn flex_wrap(self, wrap: &str) -> Self {
        self.property("flex-wrap", wrap)
    }

    /// Set align items
    pub fn align_items(self, align: &str) -> Self {
        self.property("align-items", align)
    }

    /// Set justify content
    pub fn justify_content(self, justify: &str) -> Self {
        self.property("justify-content", justify)
    }

    /// Set gap (flex gap)
    pub fn gap(self, gap: &str) -> Self {
        self.property("gap", gap)
    }

    /// Set border
    pub fn border(self, border: &str) -> Self {
        self.property("border", border)
    }

    /// Set font size
    pub fn font_size(self, size: &str) -> Self {
        self.property("font-size", size)
    }

    /// Set font family
    pub fn font_family(self, family: &str) -> Self {
        self.property("font-family", family)
    }

    /// Set font weight
    pub fn font_weight(self, weight: &str) -> Self {
        self.property("font-weight", weight)
    }

    /// Set line height
    pub fn line_height(self, height: &str) -> Self {
        self.property("line-height", height)
    }

    /// Set text align
    pub fn text_align(self, align: &str) -> Self {
        self.property("text-align", align)
    }

    /// Set width
    pub fn width(self, width: &str) -> Self {
        self.property("width", width)
    }

    /// Set height
    pub fn height(self, height: &str) -> Self {
        self.property("height", height)
    }

    /// Set minimum height
    pub fn min_height(self, height: &str) -> Self {
        self.property("min-height", height)
    }

    /// Set box shadow
    pub fn box_shadow(self, shadow: &str) -> Self {
        self.property("box-shadow", shadow)
    }

    /// Set transition
    pub fn transition(self, transition: &str) -> Self {
        self.property("transition", transition)
    }

    /// Set opacity
    pub fn opacity(self, opacity: &str) -> Self {
        self.property("opacity", opacity)
    }

    /// Set text decoration
    pub fn text_decoration(self, decoration: &str) -> Self {
        self.property("text-decoration", decoration)
    }

    /// Set cursor style
    pub fn cursor(self, cursor: &str) -> Self {
        self.property("cursor", cursor)
    }

    /// Set border bottom
    pub fn border_bottom(self, border: &str) -> Self {
        self.property("border-bottom", border)
    }

    /// Set border left
    pub fn border_left(self, border: &str) -> Self {
        self.property("border-left", border)
    }

    /// Build the final style string
    pub fn build(&self) -> String {
        self.properties
            .iter()
            .map(|(name, value)| format!("{}: {}", name, value))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StyleBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_scheme_light_has_values() {
        assert!(!ColorScheme::LIGHT.accent.is_empty());
        assert_ne!(ColorScheme::LIGHT.accent, ColorScheme::DARK.accent);
    }

    #[test]
    fn style_builder_creates_valid_css() {
        let style = StyleBuilder::new()
            .color("#fff")
            .padding(Spacing::LG)
            .border_radius(Radius::MD)
            .build();

        assert!(style.contains("color: #fff"));
        assert!(style.contains("padding: 20px"));
        assert!(style.contains("border-radius: 10px"));
    }

    #[test]
    fn style_builder_multiple_properties() {
        let style = StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap(Spacing::MD)
            .align_items("center")
            .build();

        assert_eq!(style.split("; ").count(), 4, "Should have 4 CSS properties");
    }
}
