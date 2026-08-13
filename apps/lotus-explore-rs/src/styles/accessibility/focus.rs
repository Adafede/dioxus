// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Focus and keyboard indicators for accessibility.

use super::super::tokens::*;

fn skip_link() -> String {
    format!(
        "/* Skip link focus styles */\n\
         .skip-link:focus {{\n\
           top: 0;\n\
           outline: {} solid {};\n\
           outline-offset: {};\n\
         }}\n\
         .skip-link:hover {{\n\
           text-decoration: underline;\n\
         }}",
        FOCUS_OUTLINE_WIDTH, TEXT, FOCUS_OUTLINE_OFFSET,
    )
}

fn high_contrast() -> String {
    "@media (prefers-contrast: more) {\n\
     :root {\n\
       --border: #7a879a;\n\
       --text3: #334155;\n\
       --critical-muted: #223247;\n\
     }\n\
     \n\
     .notice,\n\
     .query-panel,\n\
     .table-scroll,\n\
     .ketcher-panel {\n\
       border-width: 2px;\n\
       box-shadow: none;\n\
     }\n\
     \n\
     .sort-th,\n\
     .th-static,\n\
     .form-label,\n\
     .footer-label,\n\
     .notice-label,\n\
     .stat-label,\n\
     .meta-key {\n\
       color: var(--text);\n\
     }\n\
     \n\
     .meta-val,\n\
     .page-sub,\n\
     .welcome-lead,\n\
     .form-hint {\n\
       color: var(--text);\n\
     }\n\
   }"
    .to_string()
}

fn forced_colors() -> String {
    "@media (forced-colors: active) {\n\
     .btn,\n\
     .search-btn,\n\
     .notice,\n\
     .results-toolbar,\n\
     .table-scroll,\n\
     .query-panel,\n\
     .stat-badge,\n\
     .id-badge {\n\
       border: 1px solid CanvasText;\n\
       box-shadow: none;\n\
       forced-color-adjust: auto;\n\
     }\n\
     \n\
     .btn:focus-visible,\n\
     .search-btn:focus-visible,\n\
     .sort-btn:focus-visible,\n\
     .notice-dismiss:focus-visible,\n\
     .id-badge:focus-visible,\n\
     .copy-btn:focus-visible {\n\
       outline: 2px solid Highlight;\n\
       outline-offset: 2px;\n\
     }\n\
   }"
    .to_string()
}

pub fn css() -> String {
    [skip_link(), high_contrast(), forced_colors()].join("\n\n")
}
