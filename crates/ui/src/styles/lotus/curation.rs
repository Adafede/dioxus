// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: curation.

use super::tokens::*;

fn curation_notice_and_input() -> String {
    format!(
        "/* Curation surface: only selectors that still need CSS. */\n\
         \n\
         .curation-wrap > .notice {{\n\
           margin-left: 0;\n\
           margin-right: 0;\n\
           margin-top: 0;\n\
         }}\n\
         \n\
         .curation-file-input::file-selector-button {{\n\
           margin-right: {};\n\
           border: 1px solid {};\n\
           border-radius: {};\n\
           padding: {} {};\n\
           min-height: 34px;\n\
           background: {};\n\
           color: {};\n\
           font-weight: 600;\n\
           cursor: pointer;\n\
           box-shadow: var(--shadow-xs);\n\
           transition: background .15s ease, border-color .15s ease, box-shadow .15s ease;\n\
         }}\n\
         \n\
         .curation-file-input::file-selector-button:hover {{\n\
           background: color-mix(in srgb, {} 12%, var(--accent));\n\
           border-color: {};\n\
         }}",
        FILE_BUTTON_PADDING_H,
        PANEL_BORDER,
        RADIUS_SM,
        FILE_BUTTON_PADDING_V,
        FILE_BUTTON_PADDING_H,
        SURFACE,
        TEXT,
        SURFACE,
        BORDER,
    )
}

fn curation_table() -> String {
    format!(
        ".curation-table {{\n\
           width: 100%;\n\
           border-collapse: collapse;\n\
           font-size: {};\n\
           table-layout: auto;\n\
           word-break: break-word;\n\
         }}\n\
         \n\
         .curation-table th,\n\
         .curation-table td {{\n\
           border-bottom: 1px solid {};\n\
           text-align: left;\n\
           vertical-align: top;\n\
           padding: {} {};\n\
         }}\n\
         \n\
         .curation-table thead {{\n\
           position: sticky;\n\
           top: 0;\n\
           z-index: 2;\n\
           background: {};\n\
         }}\n\
         \n\
         .curation-table tbody tr {{\n\
           transition: background .14s ease;\n\
           background: var(--row-bg, transparent);\n\
         }}\n\
         \n\
         .curation-table tbody tr:hover {{\n\
           background: color-mix(in srgb, {} 84%, {});\n\
         }}",
        FS_UI, BORDER, TABLE_CELL_PADDING_V, TABLE_CELL_PADDING_H, BG2, SURFACE2, BG2,
    )
}

fn curation_scroll_and_tables() -> String {
    format!(
        ".curation-table-scroll {{\n\
           border-radius: {};\n\
         }}\n\
         \n\
         .curation-table-scroll:focus-visible {{\n\
           outline: none;\n\
           border-color: color-mix(in srgb, {} 44%, {}) !important;\n\
           box-shadow: {} !important;\n\
         }}\n\
         \n\
         .curation-results-table {{\n\
           min-width: max-content;\n\
         }}\n\
         \n\
         .curation-queue-table {{\n\
           min-width: max-content;\n\
         }}",
        RADIUS_XL, ACCENT, PANEL_BORDER, RING,
    )
}

pub fn css() -> String {
    [
        curation_notice_and_input(),
        curation_table(),
        curation_scroll_and_tables(),
    ]
    .join("\n\n")
}
