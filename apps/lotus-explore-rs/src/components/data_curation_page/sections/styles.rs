// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Curation/table style builders: pure `StyleBuilder` → CSS strings,
//! grouped here so the section components stay under 500 lines.

use ui::prelude::*;

pub(super) fn curation_card_style() -> String {
    StyleBuilder::new()
        .property("display", "flex")
        .property("flex-direction", "column")
        .property("gap", "10px")
        .property("padding", "12px")
        .property("border", "1px solid var(--panel-border)")
        .property("border-radius", "var(--radius)")
        .property("background", "var(--panel-bg-soft)")
        .property("box-shadow", "var(--panel-shadow)")
        .build()
}

pub(super) fn curation_form_grid_style() -> String {
    StyleBuilder::new()
        .property("display", "grid")
        .property("grid-template-columns", "1fr")
        .property("gap", "8px")
        .build()
}

pub(super) fn curation_actions_style(space_between: bool) -> String {
    let mut style = StyleBuilder::new()
        .property("display", "flex")
        .property("flex-wrap", "wrap")
        .property("gap", "8px")
        .property("align-items", "center");
    if space_between {
        style = style.property("justify-content", "space-between");
    }
    style.build()
}

pub(super) fn curation_hint_style() -> String {
    StyleBuilder::new()
        .property("font-size", "var(--fs-0)")
        .property("color", "var(--text)")
        .build()
}

pub(super) fn curation_textarea_style(min_height: &str) -> String {
    StyleBuilder::new()
        .property("min-height", min_height)
        .property("font-family", "var(--mono)")
        .property("border-radius", "8px")
        .property("resize", "none")
        .build()
}

pub(super) fn curation_file_input_style() -> String {
    StyleBuilder::new()
        .property("color", "var(--text2)")
        .property("max-width", "100%")
        .property("font-size", "var(--fs-0)")
        .build()
}

pub(super) fn curation_notice_value_style() -> String {
    StyleBuilder::new()
        .color("inherit")
        .property("word-break", "break-word")
        .property("line-height", "1.4")
        .build()
}

pub(super) fn curation_table_scroll_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("min-width", "0")
        .property("overflow-x", "auto")
        .property("overflow-y", "visible")
        .property("border", "1px solid var(--panel-border)")
        .property("background", "var(--panel-bg-soft)")
        .property("box-shadow", "var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

pub(super) fn queue_table_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("border-collapse", "collapse")
        .property("font-size", "var(--fs-ui)")
        .property("table-layout", "auto")
        .property("word-break", "break-word")
        .build()
}

pub(super) fn queue_action_col_style() -> String {
    StyleBuilder::new()
        .property("width", "110px")
        .property("min-width", "110px")
        .build()
}

pub(super) fn queue_index_col_style() -> String {
    StyleBuilder::new().property("min-width", "3ch").build()
}

pub(super) fn queue_smiles_col_style() -> String {
    StyleBuilder::new()
        .property("min-width", "220px")
        .property("max-width", "320px")
        .build()
}

pub(super) fn row_stripe_style(idx: usize) -> String {
    let background = if idx.is_multiple_of(2) {
        "color-mix(in srgb, var(--surface) 94%, transparent)"
    } else {
        "color-mix(in srgb, var(--surface) 88%, transparent)"
    };

    StyleBuilder::new()
        .property("transition", "background .14s ease")
        .property("--row-bg", background)
        .build()
}
