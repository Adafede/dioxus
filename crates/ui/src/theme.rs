// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Unified theme system for all Dioxus applications.

use core::fmt;

/// Light/dark theme selector for UI components.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Theme {
    /// High-contrast daytime palette.
    #[default]
    Light,
    /// Low-light dark palette.
    Dark,
}

impl Theme {
    /// Returns the palette matching this theme.
    #[must_use]
    pub const fn colors(self) -> ColorScheme {
        match self {
            Self::Light => ColorScheme::LIGHT,
            Self::Dark => ColorScheme::DARK,
        }
    }

    /// Returns the card shadow token appropriate for this theme.
    #[must_use]
    pub(crate) const fn card_shadow(self) -> &'static str {
        match self {
            Self::Light => Shadow::SM,
            Self::Dark => Shadow::SM_DARK,
        }
    }
}

/// Color palette for light and dark themes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorScheme {
    /// Primary page background.
    pub bg: &'static str,
    /// Secondary background.
    pub bg2: &'static str,
    /// Primary card/surface background.
    pub surface: &'static str,
    /// Elevated/nested surface background.
    pub surface2: &'static str,
    /// Hairline border color.
    pub border: &'static str,
    /// Primary body text color.
    pub text: &'static str,
    /// Secondary text color.
    pub text2: &'static str,
    /// Muted/helper text color.
    pub text3: &'static str,
    /// Primary brand/action accent color.
    pub accent: &'static str,
    /// Hover/active accent color.
    pub accent2: &'static str,
    /// Positive status color.
    pub green: &'static str,
    /// Error/danger status color.
    pub red: &'static str,
    /// Informational status color.
    pub blue: &'static str,
    /// Warning status color.
    pub yellow: &'static str,
    /// Secondary category highlight color.
    pub(crate) purple: &'static str,
}

impl ColorScheme {
    /// Light theme colors optimized for daytime viewing.
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
        green: "#166534",
        red: "#990000",
        blue: "#006699",
        yellow: "#8a4b0f",
        purple: "#6941c6",
    };

    /// Dark theme colors optimized for low-light viewing.
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
        green: "#4ade80",
        red: "#f87171",
        blue: "#60a5fa",
        yellow: "#f0b35e",
        purple: "#c3a0ff",
    };
}

/// Spacing scale derived from a 6px base unit.
#[derive(Clone, Copy, Debug)]
pub struct Spacing;

impl Spacing {
    /// Small spacing (`10px`).
    pub const SM: &'static str = "10px";
    /// Medium spacing (`14px`).
    pub const MD: &'static str = "14px";
    /// Large spacing (`20px`).
    pub const LG: &'static str = "20px";
    /// Extra-large spacing (`28px`).
    pub const XL: &'static str = "28px";
}

/// Border radius scale for consistent rounding.
#[derive(Clone, Copy, Debug)]
pub struct Radius;

impl Radius {
    /// Small border radius (`4px`).
    pub const SM: &'static str = "4px";
    /// Medium border radius (`10px`).
    pub const MD: &'static str = "10px";
    /// Large border radius (`16px`).
    pub const LG: &'static str = "16px";
}

/// Shadow system for depth and layering.
#[derive(Clone, Copy, Debug)]
pub struct Shadow;

impl Shadow {
    /// Subtle hairline shadow.
    pub const XS: &'static str = "0 1px 2px rgba(15, 23, 42, 0.06)";
    /// Standard light-mode card shadow.
    pub const SM: &'static str = "0 4px 14px rgba(15, 23, 42, 0.06)";
    /// Elevated panel shadow.
    pub const MD: &'static str = "0 10px 30px rgba(15, 23, 42, 0.09)";
    pub(crate) const SM_DARK: &'static str = "0 4px 14px rgba(0, 0, 0, 0.35)";
}

/// Typography scale with responsive fluid sizing.
#[derive(Clone, Copy, Debug)]
pub struct Typography;

impl Typography {
    /// Small label font size.
    pub const LABEL: &'static str = "clamp(0.6875rem, 0.66rem + 0.14vw, 0.75rem)";
    /// Compact UI control font size.
    pub const UI: &'static str = "clamp(0.8125rem, 0.785rem + 0.16vw, 0.875rem)";
    /// Body copy font size.
    pub const BODY: &'static str = "clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem)";
    /// Section heading (`h2`) font size.
    pub const H2: &'static str = "clamp(1.125rem, 1.02rem + 0.6vw, 1.5rem)";
    /// Page title (`h1`) font size.
    pub const H1: &'static str = "clamp(1.375rem, 1.1rem + 0.85vw, 1.85rem)";
    /// Standard body line height.
    pub const LINE_HEIGHT: &'static str = "1.5";
    /// Sans-serif font stack.
    pub const SANS: &'static str = "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', roboto, 'Helvetica Neue', arial, sans-serif";
}

/// Motion settings shared by components.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Interaction;

impl Interaction {
    pub(crate) const TRANSITION_DEFAULT: &'static str = "200ms ease-in-out";
}

/// Style string builder for inline CSS attributes.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleBuilder {
    css: String,
}

macro_rules! style_props {
    ($( $method:ident => $css:literal ),* $(,)?) => {
        $(
            #[doc = concat!("Appends CSS property `", $css, "`.")]
            pub fn $method(self, value: &str) -> Self {
                self.property($css, value)
            }
        )*
    };
}

impl StyleBuilder {
    /// Creates an empty style builder without initial heap allocation.
    pub const fn new() -> Self {
        Self { css: String::new() }
    }

    /// Creates a style builder with a pre-allocated byte capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            css: String::with_capacity(capacity),
        }
    }

    fn push_separator_and_name(&mut self, name: &str) {
        if !self.css.is_empty() {
            self.css.push_str("; ");
        }
        self.css.push_str(name);
        self.css.push_str(": ");
    }

    /// Appends an arbitrary CSS `name: value` declaration.
    pub fn property(mut self, name: &str, value: &str) -> Self {
        self.push_separator_and_name(name);
        self.css.push_str(value);
        self
    }

    /// Appends an arbitrary CSS `name: value` declaration formatted directly into the buffer.
    pub fn property_fmt(mut self, name: &str, args: fmt::Arguments<'_>) -> Self {
        self.push_separator_and_name(name);
        let _ = fmt::Write::write_fmt(&mut self.css, args);
        self
    }

    /// Pre-configured body copy typography base (`font-size`, `font-family`, `color`, `margin: 0`, `line-height`).
    pub(crate) fn body_text_style(color: &str) -> Self {
        Self::with_capacity(140)
            .font_size(Typography::BODY)
            .font_family(Typography::SANS)
            .color(color)
            .margin("0")
            .line_height(Typography::LINE_HEIGHT)
    }

    /// Pre-configured wrapping centered flex row container.
    pub(crate) fn flex_row_wrap(capacity: usize, gap: &str) -> Self {
        Self::with_capacity(capacity)
            .display("flex")
            .flex_direction("row")
            .flex_wrap("wrap")
            .align_items("center")
            .gap(gap)
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

    /// Consumes the builder and returns the assembled CSS string.
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
