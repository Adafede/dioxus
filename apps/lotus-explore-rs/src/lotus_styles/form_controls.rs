// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: form_controls.

use super::tokens::*;

fn form_sections() -> String {
    format!(
        "/* Form controls pack: shared form primitives, structure editor, and search button. */\n\
         \n\
         .form-section {{ display:flex; flex-direction:column; gap:{}; padding:{} {}; border:1px solid {}; border-radius:{}; background:{}; }}\n\
         .form-section.nested {{ padding-left:{}; border-left:1px solid {}; margin-top:4px; }}\n\
         .form-label {{ font-size:{}; font-weight:700; color:{}; text-transform:uppercase; letter-spacing:0.08em; }}\n\
         .form-label.sm {{ font-size:{}; font-weight:700; color:{}; text-transform:none; letter-spacing:0; }}\n\
         .form-hint {{ font-size:{}; color:{}; }}\n\
         .radio-group {{ display:flex; gap:{}; }}\n\
         .radio-label {{ display:flex; align-items:center; gap:{}; font-size:{}; cursor:pointer; color:{}; }}\n\
         .radio-label input {{ accent-color:{}; }}\n\
         .range-input {{ width:100%; accent-color:{}; margin-top:4px; }}\n\
         .range-inputs {{ display:flex; align-items:flex-end; gap:{}; }}\n\
         .range-pair {{ display:flex; flex-direction:column; gap:3px; }}\n\
         .range-sep {{ color:{}; padding-bottom:8px; }}",
        SPACE_1,
        FORM_SECTION_PADDING_V,
        FORM_SECTION_PADDING_H,
        PANEL_BORDER,
        RADIUS_MD,
        PANEL_BG_SOFT,
        FORM_SECTION_PADDING_V,
        BORDER,
        FS_0,
        CRITICAL_TEXT,
        FS_0,
        TEXT,
        FS_0,
        TEXT2,
        GAP_LG,
        GAP_XXS,
        FS_0,
        TEXT2,
        ACCENT,
        ACCENT,
        GAP_SM,
        TEXT3,
    )
}

fn form_ranges() -> String {
    format!(
        ".formula-grid {{ display: grid; grid-template-columns: 1fr; gap: {}; }}\n\
         .formula-minmax-grid {{ display: grid; grid-template-columns: 1fr; gap: {}; }}\n\
         \n\
         .formula-minmax-grid .range-pair,\n\
         .formula-grid .range-pair {{ min-width: 0; }}\n\
         \n\
         .formula-minmax-grid .form-input.sm,\n\
         .formula-grid .form-input.sm {{\n\
           width: 100%;\n\
           min-width: 6ch;\n\
           padding-left: 6px;\n\
           padding-right: 6px;\n\
           font-variant-numeric: tabular-nums;\n\
         }}\n\
         \n\
         /* Make two-ended range filters responsive without affecting formula rows. */\n\
         .range-inputs--pair {{\n\
           display: grid;\n\
           grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);\n\
           align-items: end;\n\
           gap: {};\n\
         }}\n\
         \n\
         .range-inputs--pair .range-pair {{ min-width: 0; }}\n\
         .range-inputs--pair .form-input {{ width: 100%; }}",
        GAP_SM, GAP_XS, GAP_XS,
    )
}

fn form_structure() -> String {
    format!(
        "/* Normalize number input chrome for consistent borders on Safari/Firefox. */\n\
         input[type=\"number\"].form-input {{ appearance: textfield; }}\n\
         \n\
         input[type=\"number\"].form-input::-webkit-outer-spin-button,\n\
         input[type=\"number\"].form-input::-webkit-inner-spin-button {{ appearance: none; margin: 0; }}\n\
         \n\
         /* Structure section */\n\
         .form-textarea.mono, .mono {{ font-family: {}; }}",
        FONT_MONO,
    )
}

fn form_actions_and_responsive() -> String {
    format!(
        "/* Focus-visible styles for interactive elements */\n\
         button:focus-visible,\n\
         .sort-btn:focus-visible,\n\
         .notice-dismiss:focus-visible,\n\
         .primary-link:focus-visible,\n\
         .id-badge:focus-visible,\n\
         .filters-toggle:focus-visible {{\n\
           outline: 3px solid {};\n\
           outline-offset: 2px;\n\
         }}",
        ACCENT2,
    )
}

pub fn css() -> String {
    [
        form_sections(),
        form_ranges(),
        form_structure(),
        form_actions_and_responsive(),
    ]
    .join("\n\n")
}
