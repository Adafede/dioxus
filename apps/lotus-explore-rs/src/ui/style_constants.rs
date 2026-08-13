// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Comprehensive Design System for lotus-explore-rs
//!
//! Consolidates all UI element styling (buttons, forms, panels, notices, etc.) into a
//! centralized, reusable system. Eliminates duplication and ensures consistency across
//! the application.
//!
//! ## Architecture
//! - Color tokens (semantic, not literal)
//! - Spacing and sizing utilities
//! - Typography tokens
//! - Component style builders (buttons, forms, panels, notices, etc)
//! - Utility functions for dynamic styling (dark mode, etc)
//!
//! ## Philosophy
//! Rust-first approach using StyleBuilder for maintainability and type safety.
//! CSS in lotus_styles/ is reserved for global resets/utilities only.

// ============================================================================
// COLOR TOKENS
// ============================================================================

pub mod stat_stripe_colors {
    //! Stripe colors for statistic badges (compounds, taxa, references, entries).
    pub const COMPOUND: &str = "var(--wd-compound-stripe)";
    pub const TAXON: &str = "var(--wd-taxon-stripe)";
    pub const REFERENCE: &str = "var(--wd-reference-stripe)";
    pub const ENTRIES: &str = "var(--wd-entries-stripe)";
}

pub mod borders {
    //! Border colors and styles used across components.
    pub const RESULTS_BORDER: &str = "var(--results-border)";
}

pub mod backgrounds {
    //! Background color tokens.
    pub const SURFACE: &str = "var(--surface)";
}

pub mod text {
    //! Text color tokens.
    pub const PRIMARY: &str = "var(--text)";
    pub const SECONDARY: &str = "var(--text2)";
    pub const ACCENT: &str = "var(--accent)";
}

pub mod shadows {
    //! Shadow tokens.
    pub const SHADOW_XS: &str = "var(--shadow-xs)";
}

// ============================================================================
// SPACING TOKENS
// ============================================================================

pub mod spacing {
    //! Spacing values for padding, margins, and gaps.
    pub const STAT_BADGE_PAD: &str = "10px 12px";
    pub const STAT_VALUE_GAP: &str = "8px";
    pub const STAT_BADGE_GAP: &str = "4px";
    pub const STAT_BAR_GAP: &str = "10px";
    pub const PAGE_BRAND_GAP: &str = "8px 10px";
    pub const HEADER_META_GAP: &str = "12px";
    pub const QUERY_SUMMARY_PADDING: &str = "8px 14px 8px 32px";
}

// ============================================================================
// TYPOGRAPHY TOKENS
// ============================================================================

pub mod typography {
    //! Font size and weight tokens.
    pub const FONT_SIZE_STAT: &str = "var(--fs-stat)";
    pub const FONT_SIZE_0: &str = "var(--fs-0)";
    pub const FONT_WEIGHT_BOLD: &str = "800";
    pub const FONT_WEIGHT_SEMIBOLD: &str = "700";
    pub const LETTER_SPACING_TITLE: &str = "0.08em";
    pub const LETTER_SPACING_STAT: &str = "-0.02em";
    pub const LINE_HEIGHT_STAT: &str = "1.2";
}

// ============================================================================
// BUTTON STYLE BUILDERS
// ============================================================================

pub mod buttons {
    //! Reusable button style functions to avoid duplication across components.
    use ui::prelude::*;

    /// Standard button style: inline-flex with border, surface background, and text color.
    /// Used in notices, loading, search panel components.
    pub fn button_base_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--border)")
            .border_radius("4px")
            .property("min-height", "40px")
            .padding("8px 14px")
            .font_size("var(--fs-0)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("var(--surface)")
            .color("var(--text)")
            .box_shadow("var(--shadow-xs)")
            .property(
                "transition",
                "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
            )
            .build()
    }

    /// Transparent button variant: used for secondary actions like download status.
    pub fn button_transparent_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--border)")
            .border_radius("8px")
            .property("min-height", "40px")
            .padding("8px 14px")
            .font_size("var(--fs-0)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("transparent")
            .color("var(--text)")
            .property(
                "transition",
                "border-color .15s, background .15s, box-shadow .15s, transform .12s ease",
            )
            .build()
    }
}

pub mod shared {
    //! Legacy module - functions have been moved to more organized modules.
    //! Left for backward compatibility with imports in other files.

    pub use super::forms::hint_text_style;
    pub use super::forms::input_base_style;
    pub use super::forms::label_base_style;
    pub use super::forms::label_small_style;
    pub use super::notices::notice_value_style;
    pub use super::utilities::sr_only_style;
}

pub mod primary_buttons {
    //! Primary action button styles (Search, Add Row, Generate, etc).
    use ui::prelude::*;

    /// Primary button: medium size with primary background, used for search and main actions.
    pub fn button_primary_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--btn-primary-bg)")
            .border_radius("4px")
            .property("min-height", "40px")
            .padding("8px 14px")
            .font_size("var(--fs-0)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("var(--btn-primary-bg)")
            .color("#fff")
            .box_shadow("var(--shadow-xs)")
            .property(
                "transition",
                "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
            )
            .build()
    }

    /// Primary button small: compact size for curation actions (Add Row, etc).
    pub fn button_primary_sm_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--btn-primary-bg)")
            .border_radius("4px")
            .padding("6px 12px")
            .font_size("var(--fs-0)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("var(--btn-primary-bg)")
            .color("#fff")
            .build()
    }

    /// Primary button full width: for block-level actions (Generate QuickStatements, etc).
    pub fn button_primary_block_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--btn-primary-bg)")
            .border_radius("4px")
            .padding("8px 14px")
            .font_size("var(--fs-0)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("var(--btn-primary-bg)")
            .color("#fff")
            .property("width", "100%")
            .build()
    }

    /// Base button with standard surface background (used for secondary/general actions).
    pub fn button_sm_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--border)")
            .border_radius("4px")
            .property("min-height", "34px")
            .padding("5px 10px")
            .font_size("var(--fs-0)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("var(--surface)")
            .color("var(--text)")
            .box_shadow("var(--shadow-xs)")
            .property(
                "transition",
                "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
            )
            .build()
    }

    /// Extra-small button for compact UI elements (delete buttons in tables, etc).
    pub fn button_xs_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--border)")
            .border_radius("4px")
            .property("min-height", "30px")
            .padding("2px 8px")
            .font_size("var(--fs-label)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("var(--surface)")
            .color("var(--text)")
            .property("line-height", "1.2")
            .box_shadow("var(--shadow-xs)")
            .property(
                "transition",
                "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
            )
            .build()
    }

    /// Filters toggle button: mobile-only button for showing/hiding filters.
    /// Display controlled by CSS media queries (filters-toggle class).
    /// Rust styles handle appearance (colors, sizing, etc) — display is inline-flex like search button.
    pub fn button_filters_toggle_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--border)")
            .border_radius("4px")
            .property("min-height", "40px")
            .padding("8px 14px")
            .font_size("var(--fs-0)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("var(--btn-primary-bg)")
            .color("#fff")
            .box_shadow("var(--shadow-xs)")
            .property(
                "transition",
                "background .15s, box-shadow .15s, transform .12s ease",
            )
            .build()
    }

    /// Copy button: small secondary button for copying content to clipboard.
    pub fn button_copy_style() -> String {
        StyleBuilder::new()
            .property("margin-left", "6px")
            .font_family("var(--sans), system-ui, sans-serif")
            .font_weight("500")
            .property("letter-spacing", ".02em")
            .color("var(--text2)")
            .background_color("var(--surface)")
            .border("1px solid var(--border)")
            .border_radius("4px")
            .cursor("pointer")
            .property(
                "transition",
                "color .15s, background .15s, border-color .15s",
            )
            .property("vertical-align", "baseline")
            .build()
    }
}

// ============================================================================
// PANEL & CONTAINER STYLES
// ============================================================================

pub mod panels {
    //! Panel, card, and container styling for search panels, results, etc.
    use ui::prelude::*;

    /// Search panel container: border with subtle background.
    pub fn search_panel_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("12px")
            .padding("12px")
            .background_color("var(--surface)")
            .border_radius("4px")
            .build()
    }

    /// Query summary inline: small font with padding and icon space.
    pub fn query_summary_style() -> String {
        StyleBuilder::new()
            .font_size("var(--fs-0)")
            .padding("8px 14px 8px 32px")
            .property("position", "relative")
            .build()
    }
}

// ============================================================================
// NOTICE & ALERT STYLES
// ============================================================================

pub mod notices {
    //! Notice, alert, and status message styling.
    use ui::prelude::*;

    /// Base notice: flex container with border, padding, and gap.
    pub fn notice_base_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .align_items("flex-start")
            .gap("8px")
            .padding("10px 12px")
            .border_radius("4px")
            .border("1px solid")
            .font_size("var(--fs-0)")
            .build()
    }

    /// Notice in dark mode: dark background with light border and text.
    pub fn notice_dark_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .align_items("flex-start")
            .gap("8px")
            .padding("10px 12px")
            .border_radius("4px")
            .border("1px solid #444")
            .background_color("#1a1a1a")
            .color("#fff")
            .font_size("var(--fs-0)")
            .build()
    }

    /// Notice value text: word-break for long content with proper line height.
    pub fn notice_value_style() -> String {
        StyleBuilder::new()
            .color("inherit")
            .property("word-break", "break-word")
            .property("line-height", "1.4")
            .build()
    }

    /// Notice dismiss button: close icon with opacity and no border.
    pub fn notice_dismiss_style() -> String {
        StyleBuilder::new()
            .property("margin-left", "auto")
            .background_color("transparent")
            .border("0")
            .color("inherit")
            .cursor("pointer")
            .property("font-size", "18px")
            .property("line-height", "1")
            .padding("0 4px")
            .property("opacity", ".7")
            .build()
    }

    /// Share input field: flex input with readonly styling.
    pub fn share_input_style() -> String {
        StyleBuilder::new()
            .property("flex", "1 1 200px")
            .property("min-width", "min(200px, 100%)")
            .background_color("var(--surface)")
            .border("1px solid var(--border)")
            .border_radius("var(--radius-sm)")
            .color("var(--text)")
            .padding("4px 8px")
            .font_size("var(--fs-0)")
            .build()
    }
}

// ============================================================================
// FORM ELEMENT STYLES
// ============================================================================

pub mod forms {
    //! Form input, label, and control styling.
    use ui::prelude::*;

    /// Base input field: background, border, text color, padding, sizing.
    pub fn input_base_style() -> String {
        StyleBuilder::new()
            .background_color("var(--surface)")
            .border("1px solid var(--border)")
            .border_radius("4px")
            .color("var(--text)")
            .padding("9px 11px")
            .font_size("var(--fs-ui)")
            .property("width", "100%")
            .font_family("var(--sans)")
            .property("transition", "border-color .15s")
            .build()
    }

    /// Base form label: uppercase with specific font size and letter spacing.
    pub fn label_base_style() -> String {
        StyleBuilder::new()
            .font_size("var(--fs-0)")
            .font_weight("700")
            .color("var(--critical-text)")
            .property("text-transform", "uppercase")
            .property("letter-spacing", "0.08em")
            .build()
    }

    /// Small label: normal case, regular text color.
    pub fn label_small_style() -> String {
        StyleBuilder::new()
            .font_size("var(--fs-0)")
            .font_weight("700")
            .color("var(--text)")
            .property("text-transform", "none")
            .property("letter-spacing", "0")
            .build()
    }

    /// Hint text: smaller, secondary color.
    pub fn hint_text_style() -> String {
        StyleBuilder::new()
            .font_size("var(--fs-0)")
            .color("var(--text2)")
            .build()
    }

    /// Form section container: flex column with border and soft background.
    pub fn form_section_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("5px")
            .padding("10px 12px")
            .border("1px solid var(--panel-border)")
            .border_radius("12px")
            .background_color("var(--panel-bg-soft)")
            .build()
    }

    /// Range inputs container: flex row with items aligned to bottom.
    pub fn range_inputs_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .align_items("flex-end")
            .gap("8px")
            .build()
    }

    /// Range pair container: flex column for min/max value pair.
    pub fn range_pair_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("3px")
            .build()
    }
}

// ============================================================================
// UTILITY & ACCESSIBILITY STYLES
// ============================================================================

pub mod utilities {
    //! Utility styles for common patterns and accessibility needs.
    use ui::prelude::*;

    /// Screen-reader only text: hidden from visual display but readable by assistive tech.
    pub fn sr_only_style() -> String {
        StyleBuilder::new()
            .property("position", "absolute")
            .property("width", "1px")
            .property("height", "1px")
            .property("padding", "0")
            .property("margin", "-1px")
            .property("overflow", "hidden")
            .property("clip", "rect(0,0,0,0)")
            .property("white-space", "nowrap")
            .property("border", "0")
            .build()
    }
}

// ============================================================================
// DYNAMIC STYLING UTILITIES
// ============================================================================

pub mod header {
    //! Header and page title styling.
    use ui::prelude::*;

    /// Page header container with sticky positioning and bottom border.
    pub fn page_header_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("8px")
            .property("padding-left", "max(18px, env(safe-area-inset-left))")
            .property("padding-right", "max(18px, env(safe-area-inset-right))")
            .build()
    }

    /// Page brand section: flex row with title and language switcher.
    pub fn page_brand_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("row")
            .flex_wrap("wrap")
            .align_items("flex-start")
            .gap("8px 10px")
            .build()
    }

    /// Page title: main heading with min-width constraint.
    pub fn page_title_style() -> String {
        StyleBuilder::new()
            .property("min-width", "0")
            .property("flex", "1 1 260px")
            .font_size("var(--fs-4)")
            .property("margin", "0")
            .build()
    }

    /// Page title link: inline-flex with text overflow handling.
    pub fn page_title_link_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .property("max-width", "100%")
            .gap("8px")
            .text_decoration("none")
            .color("inherit")
            .build()
    }

    /// Page title text: proper wrapping and line height.
    pub fn page_title_text_style() -> String {
        StyleBuilder::new()
            .property("line-height", "1.1")
            .property("word-break", "break-word")
            .build()
    }

    /// Page subtitle: secondary color with smaller font.
    pub fn page_subtitle_style() -> String {
        StyleBuilder::new()
            .font_size("var(--fs-1)")
            .property("margin", "0")
            .color("var(--text2)")
            .build()
    }

    /// Archive note section: inline display.
    pub fn page_archive_note_style() -> String {
        StyleBuilder::new().display("inline").build()
    }

    /// Archive label: bold small-caps label.
    pub fn page_archive_label_style() -> String {
        StyleBuilder::new()
            .font_weight("700")
            .property("font-variant", "small-caps")
            .build()
    }

    /// Archive link: accent color, no wrap.
    pub fn page_archive_link_style() -> String {
        StyleBuilder::new()
            .text_decoration("none")
            .color("var(--accent)")
            .property("white-space", "nowrap")
            .build()
    }
}

pub mod table {
    //! Table header and sorting controls.
    use ui::prelude::*;

    /// Table header cell: uppercase label with border and padding.
    pub fn table_header_cell_style() -> String {
        StyleBuilder::new()
            .padding("9px 10px")
            .text_align("left")
            .font_size("var(--fs-label)")
            .font_weight("700")
            .color("var(--critical-muted)")
            .border_bottom("1px solid var(--results-border)")
            .property("white-space", "nowrap")
            .property("user-select", "none")
            .property("text-transform", "uppercase")
            .property("letter-spacing", "0.08em")
            .property("width", "auto")
            .property("min-width", "max-content")
            .build()
    }

    /// Header label text: block display with no-break constraint.
    pub fn header_label_style() -> String {
        StyleBuilder::new()
            .display("block")
            .property("min-width", "max-content")
            .property("white-space", "nowrap")
            .property("overflow", "visible")
            .property("text-overflow", "clip")
            .property("line-height", "1.2")
            .font_weight("inherit")
            .font_size("inherit")
            .property("text-transform", "inherit")
            .property("letter-spacing", "inherit")
            .build()
    }

    /// Sort button: transparent grid-based layout for label + icon.
    pub fn sort_button_style() -> String {
        StyleBuilder::new()
            .property("appearance", "none")
            .background_color("transparent")
            .border("0")
            .color("inherit")
            .font_family("inherit")
            .padding("0")
            .property("margin", "0")
            .cursor("pointer")
            .display("grid")
            .align_items("start")
            .property("grid-template-columns", "auto auto")
            .property("column-gap", "6px")
            .property("width", "100%")
            .property("min-width", "max-content")
            .build()
    }

    /// Sort icon: muted color, smaller font.
    pub fn sort_icon_style() -> String {
        StyleBuilder::new()
            .color("var(--text3)")
            .font_size("var(--fs-0)")
            .font_weight("700")
            .property("line-height", "1")
            .build()
    }
}

pub mod search_controls {
    //! Search form controls: radio groups, range inputs, text areas, etc.
    use ui::prelude::*;

    /// Radio group fieldset: flex wrap, no border/padding/margin.
    pub fn radio_group_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_wrap("wrap")
            .gap("14px")
            .property("border", "0")
            .property("padding", "0")
            .property("margin", "0")
            .build()
    }

    /// Radio label: flex with gap, cursor pointer, secondary text color.
    pub fn radio_label_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .align_items("center")
            .gap("6px")
            .font_size("var(--fs-0)")
            .cursor("pointer")
            .color("var(--text2)")
            .build()
    }

    /// Range input slider: full width with accent-color.
    pub fn range_input_style() -> String {
        StyleBuilder::new()
            .property("width", "100%")
            .property("accent-color", "var(--accent)")
            .property("margin-top", "4px")
            .build()
    }

    /// Textarea: surface background, border, full width, no resize.
    pub fn textarea_base_style() -> String {
        StyleBuilder::new()
            .background_color("var(--surface)")
            .border("1px solid var(--border)")
            .border_radius("4px")
            .color("var(--text)")
            .padding("9px 11px")
            .font_size("var(--fs-ui)")
            .property("width", "100%")
            .property("max-width", "100%")
            .property("resize", "none")
            .font_family("var(--sans)")
            .property("transition", "border-color .15s")
            .build()
    }

    /// Threshold section: column with left border and top margin.
    pub fn threshold_section_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("5px")
            .padding("10px")
            .property("border-left", "1px solid var(--border)")
            .property("margin-top", "4px")
            .build()
    }

    /// Kind pill badge: inline-block with background color and uppercase text.
    pub fn kind_pill_style(kind: &str) -> String {
        let background = match kind {
            "smiles" => "var(--accent2)",
            "mol2000" => "#c97a2b",
            "mol3000" => "#2b8f57",
            _ => "var(--text3)",
        };
        StyleBuilder::new()
            .display("inline-block")
            .padding("1px 7px")
            .border_radius("999px")
            .font_size("var(--fs-micro)")
            .font_weight("700")
            .property("letter-spacing", "1px")
            .property("text-transform", "uppercase")
            .property("margin-right", "6px")
            .color("var(--text)")
            .background_color(background)
            .build()
    }
}

pub mod search_buttons {
    //! Search action buttons and their states (active, dirty, etc).
    use ui::prelude::*;

    /// Search button when dirty (form has changes): emphasized background with accent.
    pub fn search_button_dirty_style() -> String {
        StyleBuilder::new()
            .display("inline-flex")
            .align_items("center")
            .justify_content("center")
            .gap("8px")
            .border("0")
            .border_radius("4px")
            .property("min-height", "40px")
            .padding("11px 16px")
            .font_size("var(--fs-ui)")
            .font_weight("700")
            .cursor("pointer")
            .background_color("color-mix(in srgb, var(--btn-primary-bg) 90%, var(--accent))")
            .color("var(--text)")
            .box_shadow("var(--shadow-xs)")
            .property(
                "transition",
                "background .15s, box-shadow .15s, transform .12s ease",
            )
            .build()
    }

    /// Search button conditional: returns dirty style if true, base style if false.
    pub fn search_button_state(dirty: bool) -> String {
        if dirty {
            search_button_dirty_style()
        } else {
            super::buttons::button_base_style()
        }
    }
}

pub mod panel_containers {
    //! Panel and container layouts: flex stacks, section cards, nested panels, etc.
    use ui::prelude::*;

    /// Flex column stack: vertical layout with padding and gap.
    pub fn panel_stack_style(padding: &str, gap: &str) -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap(gap)
            .padding(padding)
            .build()
    }

    /// Section card: bordered container with soft background and rounded corners.
    pub fn section_card_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("5px")
            .padding("10px 12px")
            .border("1px solid var(--panel-border)")
            .border_radius("12px")
            .background_color("var(--panel-bg-soft)")
            .build()
    }

    /// Ketcher panel: dialog container for structure editor with border and shadow.
    pub fn ketcher_panel_style() -> String {
        StyleBuilder::new()
            .property("margin", "0")
            .border("1px solid var(--panel-border)")
            .border_radius("var(--radius)")
            .background_color("var(--panel-bg-soft)")
            .box_shadow("var(--panel-shadow)")
            .property(
                "transition",
                "background .15s ease, border-color .15s ease, box-shadow .15s ease",
            )
            .build()
    }

    /// Iframe container: full width with specific height constraints.
    pub fn iframe_style() -> String {
        StyleBuilder::new()
            .property("width", "100%")
            .property("height", "min(78vh, 820px)")
            .property("min-height", "600px")
            .border("1px solid var(--border)")
            .border_radius("4px")
            .background_color("var(--surface)")
            .build()
    }

    /// Ketcher content wrapper: padding and gap for nested content.
    pub fn ketcher_wrap_style() -> String {
        panel_stack_style("0 14px 14px", "10px")
    }
}

pub mod stats {
    //! Statistic badge, bar, and value styling for dataset statistics.
    use super::{StatStripe, spacing, text, typography};
    use ui::prelude::*;

    /// Stat badge container: flex column with border, stripe, and shadow.
    pub fn stat_badge_style(stripe: StatStripe) -> String {
        let stripe_color = stripe.as_color();
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap(spacing::STAT_BADGE_GAP)
            .property("min-width", "0")
            .padding(spacing::STAT_BADGE_PAD)
            .border_radius("12px")
            .property(
                "border",
                &format!("1px solid {}", super::borders::RESULTS_BORDER),
            )
            .background_color(super::backgrounds::SURFACE)
            .box_shadow(super::shadows::SHADOW_XS)
            .property("position", "relative")
            .property("overflow", "hidden")
            .property("flex", "1 1 0")
            .property("border-left", &format!("3px solid {stripe_color}"))
            .build()
    }

    /// Stat value text: large bold font with tabular numbers.
    pub fn stat_value_style() -> String {
        StyleBuilder::new()
            .font_size(typography::FONT_SIZE_STAT)
            .font_weight(typography::FONT_WEIGHT_BOLD)
            .color(text::PRIMARY)
            .property("font-variant-numeric", "tabular-nums")
            .property("letter-spacing", typography::LETTER_SPACING_STAT)
            .property("min-width", "0")
            .property("flex", "0 1 auto")
            .property("line-height", typography::LINE_HEIGHT_STAT)
            .build()
    }

    /// Stat secondary value: smaller font, tabular numbers, overflow wrap.
    pub fn stat_secondary_style() -> String {
        StyleBuilder::new()
            .font_size(typography::FONT_SIZE_0)
            .font_weight(typography::FONT_WEIGHT_SEMIBOLD)
            .color(text::PRIMARY)
            .property("font-variant-numeric", "tabular-nums")
            .property("min-width", "0")
            .property("max-width", "100%")
            .property("overflow-wrap", "anywhere")
            .property("flex", "0 0 auto")
            .build()
    }

    /// Stat bar container: auto-fit grid with responsive columns.
    pub fn stat_bar_style() -> String {
        StyleBuilder::new()
            .display("grid")
            .property(
                "grid-template-columns",
                "repeat(auto-fit, minmax(120px, 1fr))",
            )
            .gap(spacing::STAT_BAR_GAP)
            .align_items("stretch")
            .property("width", "100%")
            .property("min-width", "0")
            .build()
    }

    /// Stat value row: flex wrap with baseline alignment.
    pub fn stat_value_row_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .property("flex-wrap", "wrap")
            .align_items("baseline")
            .gap(spacing::STAT_VALUE_GAP)
            .property("min-width", "0")
            .property("width", "100%")
            .justify_content("center")
            .build()
    }

    /// Stat label: uppercase with centered text.
    pub fn stat_label_style() -> String {
        StyleBuilder::new()
            .font_size(typography::FONT_SIZE_0)
            .color(text::SECONDARY)
            .property("text-transform", "uppercase")
            .property("letter-spacing", typography::LETTER_SPACING_TITLE)
            .font_weight(typography::FONT_WEIGHT_SEMIBOLD)
            .property("width", "100%")
            .text_align("center")
            .build()
    }
}

pub mod table_cells {
    //! Table cell styling for different column types (taxon, reference, numeric, etc).
    use ui::prelude::*;

    /// Taxon cell container: soft background with inset shadow border.
    pub fn taxon_cell_style() -> String {
        StyleBuilder::new()
            .padding("8px 12px")
            .border_radius("10px")
            .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
            .property(
                "box-shadow",
                "inset 3px 0 0 rgb(51 153 102 / 42%), inset 0 0 0 1px var(--results-border)",
            )
            .property("min-width", "0")
            .build()
    }

    /// Cell primary text: italic font weight 500.
    pub fn cell_primary_style() -> String {
        StyleBuilder::new()
            .font_weight("500")
            .property("font-style", "italic")
            .build()
    }

    /// ID badge: inline-block with monospace font and soft background.
    pub fn id_badge_style() -> String {
        StyleBuilder::new()
            .display("inline-block")
            .font_size("var(--fs-micro)")
            .padding("1px 5px")
            .border_radius("3px")
            .font_weight("600")
            .text_decoration("none")
            .property("line-height", "1.5")
            .border("1px solid transparent")
            .font_family("var(--mono)")
            .property("max-width", "100%")
            .property("white-space", "normal")
            .property("overflow-wrap", "anywhere")
            .property(
                "transition",
                "transform .12s ease, box-shadow .12s ease, filter .12s ease",
            )
            .background_color("var(--wd-taxon-soft-bg)")
            .color("var(--wd-taxon)")
            .property("border-color", "var(--wd-taxon-soft-border)")
            .build()
    }

    /// Primary link in cell: block display with word break.
    pub fn primary_link_style() -> String {
        StyleBuilder::new()
            .color("var(--text)")
            .property("display", "block")
            .property("line-height", "1.4")
            .property("overflow-wrap", "break-word")
            .property("word-break", "break-word")
            .property("white-space", "normal")
            .build()
    }

    /// Badge row container: flex wrap with gap.
    pub fn badge_row_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .property("flex-wrap", "wrap")
            .gap("4px")
            .property("margin-top", "4px")
            .property("overflow", "visible")
            .property("min-width", "0")
            .build()
    }

    /// Reference cell container: flex column with padding and border.
    pub fn reference_cell_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("4px")
            .padding("8px 12px")
            .border_radius("10px")
            .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
            .property(
                "box-shadow",
                "inset 3px 0 0 rgb(185 65 104 / 42%), inset 0 0 0 1px var(--results-border)",
            )
            .property("min-width", "0")
            .build()
    }

    /// Reference ID badge: inline-block with reference styling.
    pub fn reference_id_badge_style() -> String {
        StyleBuilder::new()
            .display("inline-block")
            .font_size("var(--fs-micro)")
            .padding("1px 5px")
            .border_radius("3px")
            .font_weight("600")
            .text_decoration("none")
            .property("line-height", "1.5")
            .border("1px solid transparent")
            .font_family("var(--mono)")
            .property("max-width", "100%")
            .property("white-space", "normal")
            .property("overflow-wrap", "anywhere")
            .property(
                "transition",
                "transform .12s ease, box-shadow .12s ease, filter .12s ease",
            )
            .background_color("var(--wd-reference-soft-bg)")
            .color("var(--wd-reference)")
            .property("border-color", "var(--wd-reference-soft-border)")
            .build()
    }

    /// Numeric value cell: standard table cell styling.
    pub fn table_cell_style() -> String {
        StyleBuilder::new()
            .padding("8px 12px")
            .property("vertical-align", "top")
            .build()
    }

    /// N/A text for missing values: muted color.
    pub fn na_style() -> String {
        StyleBuilder::new().color("var(--text3)").build()
    }

    /// Formula text: monospace font.
    pub fn formula_style() -> String {
        StyleBuilder::new()
            .font_family("var(--mono)")
            .font_size("var(--fs-0)")
            .build()
    }

    /// Structure cell container: flex column with padding.
    pub fn structure_cell_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("4px")
            .padding("8px 12px")
            .border_radius("10px")
            .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
            .property(
                "box-shadow",
                "inset 3px 0 0 rgb(102 102 153 / 42%), inset 0 0 0 1px var(--results-border)",
            )
            .property("min-width", "0")
            .build()
    }

    /// Compound cell container: similar to other cell containers.
    pub fn compound_cell_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
            .gap("4px")
            .padding("8px 12px")
            .border_radius("10px")
            .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
            .property(
                "box-shadow",
                "inset 3px 0 0 rgb(201 122 43 / 42%), inset 0 0 0 1px var(--results-border)",
            )
            .property("min-width", "0")
            .build()
    }
}

pub mod downloads {
    //! Download toolbar and action button styling.
    use ui::prelude::*;

    /// Toolbar actions container: flex wrap with space-between.
    pub fn toolbar_actions_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .property("flex-wrap", "wrap")
            .gap("8px")
            .align_items("center")
            .justify_content("space-between")
            .property("min-width", "0")
            .build()
    }

    /// Download group container: flex wrap with items.
    pub fn dl_group_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .property("flex-wrap", "wrap")
            .gap("8px")
            .property("min-width", "0")
            .property("max-width", "100%")
            .align_items("stretch")
            .build()
    }

    /// Spinner small: animated circular loader.
    pub fn spinner_sm_style() -> String {
        StyleBuilder::new()
            .property("width", "14px")
            .property("height", "14px")
            .border("2px solid color-mix(in srgb, var(--text) 30%, transparent)")
            .property("border-top-color", "var(--text)")
            .border_radius("50%")
            .property("animation", "spin .7s linear infinite")
            .property("display", "inline-block")
            .build()
    }

    /// Download button small: flex button with border and transition.
    pub fn button_small_style() -> String {
        StyleBuilder::new()
            .display("flex")
            .align_items("center")
            .justify_content("center")
            .gap("6px")
            .border("1px solid var(--border)")
            .border_radius("8px")
            .padding("8px 12px")
            .font_size("var(--fs-0)")
            .font_weight("600")
            .cursor("pointer")
            .background_color("transparent")
            .color("var(--text)")
            .property("flex", "1 1 auto")
            .property("min-width", "0")
            .property("white-space", "nowrap")
            .property(
                "transition",
                "border-color .15s, background .15s, box-shadow .15s, transform .12s ease",
            )
            .build()
    }
}

pub mod query {
    //! Query panel and query display styling.
    use super::spacing;
    use ui::prelude::*;

    /// Query summary toggle: cursor pointer with gradient background.
    pub fn query_summary_style() -> String {
        StyleBuilder::new()
            .cursor("pointer")
            .padding(spacing::QUERY_SUMMARY_PADDING)
            .font_size("var(--fs-0)")
            .color("var(--text2)")
            .property("user-select", "none")
            .property("letter-spacing", "0.04em")
            .font_weight("600")
            .property("list-style", "none")
            .property("position", "relative")
            .property("transition", "color .15s ease, background .15s ease")
            .property(
                "background",
                "linear-gradient(135deg, transparent 0%, rgba(255,255,255,0.02) 100%)",
            )
            .build()
    }

    /// Query summary chevron: positioned absolute with rotation transform.
    pub fn query_summary_chevron_style(is_open: bool) -> String {
        let (rotation_deg, color) = if is_open {
            ("90deg", "var(--accent)")
        } else {
            ("0deg", "var(--text3)")
        };

        let transform = format!("translateY(-50%) rotate({})", rotation_deg);

        StyleBuilder::new()
            .property("position", "absolute")
            .property("left", "12px")
            .property("top", "50%")
            .property("transition", "transform .2s ease")
            .property("font-size", "14px")
            .property("line-height", "1")
            .color(color)
            .property("transform", &transform)
            .build()
    }

    /// Query panel container: soft background with border and shadow.
    pub fn query_panel_style() -> String {
        StyleBuilder::new()
            .background_color("var(--panel-bg-soft)")
            .border("1px solid var(--panel-border)")
            .border_radius("var(--radius)")
            .box_shadow("var(--panel-shadow)")
            .property(
                "transition",
                "background .15s ease, border-color .15s ease, box-shadow .15s ease",
            )
            .build()
    }

    /// Query body content: positioned relative with bottom rounded corners.
    pub fn query_body_style() -> String {
        StyleBuilder::new()
            .property("position", "relative")
            .border_radius("0 0 var(--radius) var(--radius)")
            .property("overflow", "hidden")
            .build()
    }

    /// Query text display: monospace font with left border accent.
    pub fn query_text_style() -> String {
        StyleBuilder::new()
            .padding("12px 16px")
            .property("margin", "0")
            .font_family("var(--mono)")
            .font_size("var(--fs-0)")
            .color("var(--text)")
            .background_color("var(--bg2)")
            .property("border-left", "3px solid var(--wd-entries)")
            .property("white-space", "pre-wrap")
            .property("word-break", "break-word")
            .property("max-height", "320px")
            .property("overflow", "auto")
            .build()
    }
}

pub mod theme {
    //! Theme and dynamic styling utilities (dark mode detection, etc).

    /// Detect if dark mode is active based on system preferences.
    /// Must be called at component render time to detect changes.
    #[cfg(target_arch = "wasm32")]
    pub fn is_dark_mode() -> bool {
        if let Ok(window) = web_sys::window().ok_or("no window") {
            if let Ok(media) = window.match_media("(prefers-color-scheme: dark)") {
                if let Some(media_query) = media {
                    return media_query.matches();
                }
            }
        }
        false
    }

    /// Fallback for non-WASM builds (always returns false).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_dark_mode() -> bool {
        false
    }
}

// ============================================================================
// ENUM TYPES FOR TYPE-SAFE SELECTION
// ============================================================================

/// Type-safe stat stripe colors (instead of string parameters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatStripe {
    Compound,
    Taxon,
    Reference,
    Entries,
}

impl StatStripe {
    /// Get the CSS color variable for this stripe.
    pub fn as_color(&self) -> &'static str {
        match self {
            Self::Compound => stat_stripe_colors::COMPOUND,
            Self::Taxon => stat_stripe_colors::TAXON,
            Self::Reference => stat_stripe_colors::REFERENCE,
            Self::Entries => stat_stripe_colors::ENTRIES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StatStripe;
    use crate::ui::style_constants::{spacing, stat_stripe_colors};

    #[test]
    fn stat_stripe_colors_are_nonempty() {
        assert!(!stat_stripe_colors::COMPOUND.is_empty());
        assert!(!stat_stripe_colors::TAXON.is_empty());
        assert!(!stat_stripe_colors::REFERENCE.is_empty());
        assert!(!stat_stripe_colors::ENTRIES.is_empty());
    }

    #[test]
    fn spacing_tokens_parse() {
        // Verify format is valid for CSS
        assert!(spacing::STAT_BADGE_GAP.contains("px"));
    }

    #[test]
    fn stat_stripe_enum_colors_match() {
        assert_eq!(
            StatStripe::Compound.as_color(),
            stat_stripe_colors::COMPOUND
        );
        assert_eq!(StatStripe::Taxon.as_color(), stat_stripe_colors::TAXON);
        assert_eq!(
            StatStripe::Reference.as_color(),
            stat_stripe_colors::REFERENCE
        );
        assert_eq!(StatStripe::Entries.as_color(), stat_stripe_colors::ENTRIES);
    }
}
