// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Spacing primitives: padding, margin, gap.

use super::super::tokens::*;
use ui::theme::StyleBuilder;

// ============================================================================
// PADDING PRIMITIVES
// ============================================================================

/// No padding: `padding: 0`
pub fn padding_none() -> String {
    StyleBuilder::new().padding("0").build()
}

/// Extra-small padding from tokens: XS spacing
pub fn padding_xs() -> String {
    StyleBuilder::new().padding(SPACING_XS).build()
}

/// Small padding from tokens: SM spacing
pub fn padding_sm() -> String {
    StyleBuilder::new().padding(SPACING_SM).build()
}

/// Medium padding from tokens: MD spacing
pub fn padding_md() -> String {
    StyleBuilder::new().padding(SPACING_MD).build()
}

/// Large padding from tokens: LG spacing
pub fn padding_lg() -> String {
    StyleBuilder::new().padding(SPACING_LG).build()
}

/// Button padding from tokens
pub fn padding_button() -> String {
    StyleBuilder::new()
        .padding(&format!("{} {}", BUTTON_PADDING_Y, BUTTON_PADDING_X))
        .build()
}

/// Input padding from tokens
pub fn padding_input() -> String {
    StyleBuilder::new()
        .padding(&format!("{} {}", INPUT_PADDING_Y, INPUT_PADDING_X))
        .build()
}

/// Cell padding from tokens
pub fn padding_cell() -> String {
    StyleBuilder::new()
        .padding(&format!("{} {}", CELL_PADDING_Y, CELL_PADDING_X))
        .build()
}

// ============================================================================
// MARGIN PRIMITIVES
// ============================================================================

/// No margin: `margin: 0`
pub fn margin_none() -> String {
    StyleBuilder::new().margin("0").build()
}

/// Extra-small margin from tokens
pub fn margin_xs() -> String {
    StyleBuilder::new().margin(SPACING_XS).build()
}

/// Small margin from tokens
pub fn margin_sm() -> String {
    StyleBuilder::new().margin(SPACING_SM).build()
}

/// Medium margin from tokens
pub fn margin_md() -> String {
    StyleBuilder::new().margin(SPACING_MD).build()
}

// ============================================================================
// GAP PRIMITIVES
// ============================================================================

/// Extra-small gap from tokens
pub fn gap_xs() -> String {
    StyleBuilder::new().gap(GAP_XS).build()
}

/// Extra-small (6px) gap from tokens
pub fn gap_xxs() -> String {
    StyleBuilder::new().gap("6px").build()
}

/// Small gap from tokens
pub fn gap_sm() -> String {
    StyleBuilder::new().gap(GAP_SM).build()
}

/// Medium gap from tokens
pub fn gap_md() -> String {
    StyleBuilder::new().gap(GAP_MD).build()
}

/// Large gap from tokens
pub fn gap_lg() -> String {
    StyleBuilder::new().gap(GAP_LG).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_xs_uses_token() {
        assert!(padding_xs().contains("padding"));
    }
}
