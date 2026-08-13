// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Motion and animation preferences for accessibility.

use super::super::tokens::*;

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

pub fn css() -> String {
    [reduced_motion(), reduced_transparency()].join("\n\n")
}
