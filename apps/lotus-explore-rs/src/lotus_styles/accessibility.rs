// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: accessibility.

use super::tokens::*;

fn skip_link() -> String {
    format!(
        "/* Accessibility-specific styles and media queries.\n\
           Extracted from style.css for maintainability and cacheable separately. */\n\
         \n\
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

fn reduced_motion() -> String {
    "@media (prefers-reduced-motion: reduce) {\n\
     .sidebar::before,\n\
     .page-header::before {\n\
       animation: none;\n\
     }\n\
     \n\
     .data-row:hover,\n\
     .id-badge:hover {\n\
       transform: none;\n\
     }\n\
     \n\
     *, *::before, *::after {\n\
       animation-duration: 0.01ms !important;\n\
       animation-iteration-count: 1 !important;\n\
       transition-duration: 0.01ms !important;\n\
       scroll-behavior: auto !important;\n\
     }\n\
     \n\
     .sidebar,\n\
     .main-content,\n\
     .page-header {\n\
       backdrop-filter: none;\n\
     }\n\
   }"
    .to_string()
}

fn reduced_transparency() -> String {
    format!(
        "@media (prefers-reduced-transparency: reduce) {{\n\
           .sidebar,\n\
           .main-content,\n\
           .page-header {{\n\
             backdrop-filter: none;\n\
             background: {};\n\
           }}\n\
         }}",
        BG2,
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
    [
        skip_link(),
        reduced_motion(),
        reduced_transparency(),
        high_contrast(),
        forced_colors(),
    ]
    .join("\n\n")
}
