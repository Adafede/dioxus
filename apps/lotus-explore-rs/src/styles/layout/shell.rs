// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: layout_shell.

use super::super::tokens::*;

fn app_frame() -> String {
    format!(
        "/* Layout shell pack: app frame, header/meta, notices, share bar, and sidebar shell. */\n\
         \n\
         .app-layout {{ display:flex; min-height:100dvh; height:100dvh; overflow:hidden; gap:{}; padding:{}; }}\n\
         .app-layout.no-sidebar {{ display:block; }}\n\
         \n\
         .sidebar {{\n\
           width:300px;\n\
           min-width:250px;\n\
           height:100%;\n\
           overflow-y:auto;\n\
           background:{};\n\
           border:1px solid {};\n\
           border-radius:{};\n\
           flex-shrink:0;\n\
           box-shadow:{};\n\
           display:flex;\n\
           flex-direction:column;\n\
           position: relative;\n\
           isolation: isolate;\n\
         }}\n\
         \n\
         .main-content {{\n\
           flex:1;\n\
           min-width:0;\n\
           height:100%;\n\
           overflow-y:auto;\n\
           display:flex;\n\
           flex-direction:column;\n\
           border:1px solid {};\n\
           border-radius:{};\n\
           background:{};\n\
           box-shadow:{};\n\
         }}\n\
         \n\
         .main-content.single-pane {{ width:100%; }}\n\
         \n\
         /* Perceived perf: skip off-screen paint work */\n\
         .welcome, .results-wrap, .query-panel, .ketcher-panel, .table-scroll {{\n\
           content-visibility: auto;\n\
           contain-intrinsic-size: 900px;\n\
         }}",
        LAYOUT_GAP,
        LAYOUT_GAP,
        PANEL_BG,
        PANEL_BORDER,
        RADIUS_LG,
        SHADOW_SM,
        PANEL_BORDER,
        RADIUS_LG,
        PANEL_BG,
        SHADOW_SM,
    )
}

fn page_header() -> String {
    format!(
        ".page-header {{\n\
           padding:{} {} {};\n\
           border-bottom:1px solid {};\n\
           background:color-mix(in srgb, {} 92%, {});\n\
           box-shadow:{};\n\
           position: sticky;\n\
           top: 0;\n\
           z-index: 3;\n\
           overflow: clip;\n\
         }}\n\
         \n\
         \n\
         .page-title-link,\n\
         .page-title-link:visited {{ color: inherit; text-decoration: none; }}\n\
         .page-title-link:hover {{ text-decoration: none; }}\n\
         \n\
         .lang-switch {{ margin-left:auto; display:flex; gap:{}; align-items:center; }}\n\
         .page-home-link {{ display: inline-flex; align-items: center; gap: 0; min-width: 0; }}\n\
         \n\
         .page-sub {{ font-size:{}; color:{}; margin-top:4px; }}\n\
         \n\
         .page-meta {{ display: contents; }}\n\
         .meta-item {{ display:inline-flex; align-items:center; gap:{}; white-space: normal; overflow-wrap: anywhere; line-height: 1.4; }}\n\
         .meta-key {{ text-transform:uppercase; letter-spacing:0.08em; font-weight:700; font-size: {}; color: {}; }}\n\
         .meta-val.mono {{ font-family:{}; color:{}; font-variant-numeric: tabular-nums; font-size: {}; }}\n\
         .meta-sep {{ color:{}; }}",
        PAGE_HEADER_PADDING_T,
        PAGE_HEADER_PADDING_H,
        PAGE_HEADER_PADDING_B,
        PANEL_BORDER,
        PANEL_BG_SOFT,
        SURFACE,
        SHADOW_XS,
        GAP_XS,
        FS_1,
        CRITICAL_MUTED,
        GAP_XS,
        FS_0,
        TEXT2,
        FONT_MONO,
        CRITICAL_TEXT,
        FS_0,
        TEXT3,
    )
}

fn notices() -> String {
    format!(
        "/* Notices */\n\
         .notice {{\n\
           margin:{} {} 0;\n\
           padding:{} {};\n\
           display:flex;\n\
           align-items:center;\n\
           gap:{};\n\
           border-radius:{};\n\
           font-size:{};\n\
           border:1px solid {};\n\
           background:{};\n\
           box-shadow:{};\n\
           transition: background {} ease, border-color {} ease, box-shadow {} ease;\n\
         }}\n\
         \n\
         /* Notices that are direct children of the results pane span full width. */\n\
         .main-content > .notice {{\n\
           margin-left: 0;\n\
           margin-right: 0;\n\
           padding-left: {};\n\
           padding-right: {};\n\
           border-radius: 0;\n\
           border-left: 0;\n\
           border-right: 0;\n\
         }}\n\
         \n\
         .results-wrap > .notice {{ margin: 0; }}\n\
         .notice:hover {{ box-shadow: {}; }}\n\
         \n\
         .notice-label {{\n\
           display:inline-flex;\n\
           align-items:center;\n\
           text-transform:uppercase;\n\
           letter-spacing:1px;\n\
           font-size:{};\n\
           font-weight:700;\n\
           line-height:1.4;\n\
           padding:2px 6px;\n\
           border-radius:3px;\n\
           flex-shrink:0;\n\
         }}\n\
         \n\
         .notice-value {{ flex:1; color:inherit; word-break:break-word; line-height:1.4; }}\n\
         \n\
         .notice-copy-field {{\n\
           min-width: min(220px, 100%);\n\
           max-width: 100%;\n\
           background: {};\n\
           border: 1px solid {};\n\
           border-radius: {};\n\
           color: {};\n\
           padding: 4px 8px;\n\
         }}\n\
         \n\
         .notice-info {{ border-color:color-mix(in srgb, {} 34%, {}); background:color-mix(in srgb, {} 9%, {}); }}\n\
         .notice-info .notice-label {{ background:color-mix(in srgb, {} 16%, {}); color:color-mix(in srgb, {} 86%, {}); }}\n\
         .notice-warn {{ border-color:color-mix(in srgb, {} 34%, {}); background:color-mix(in srgb, {} 8%, {}); }}\n\
         .notice-warn .notice-label {{ background:color-mix(in srgb, {} 16%, {}); color:color-mix(in srgb, {} 88%, {}); }}\n\
         .notice-error {{ border-color:color-mix(in srgb, {} 34%, {}); background:color-mix(in srgb, {} 8%, {}); }}\n\
         .notice-error .notice-label {{ background:color-mix(in srgb, {} 16%, {}); color:color-mix(in srgb, {} 88%, {}); }}",
        MARGIN_NOTICE_V,
        MARGIN_NOTICE_H,
        NOTICE_PADDING_V,
        NOTICE_PADDING_H,
        GAP_MD,
        RADIUS,
        FS_0,
        PANEL_BORDER,
        PANEL_BG_SOFT,
        PANEL_SHADOW,
        TRANSITION_TIMING,
        TRANSITION_TIMING,
        TRANSITION_TIMING,
        MARGIN_NOTICE_H,
        MARGIN_NOTICE_H,
        SHADOW_SM,
        FS_LABEL,
        SURFACE,
        BORDER,
        RADIUS_SM,
        TEXT,
        ACCENT,
        RESULTS_BORDER,
        ACCENT,
        PANEL_BG_SOFT,
        ACCENT,
        SURFACE,
        ACCENT,
        TEXT,
        YELLOW,
        RESULTS_BORDER,
        YELLOW,
        PANEL_BG_SOFT,
        YELLOW,
        SURFACE,
        YELLOW,
        TEXT,
        RED,
        RESULTS_BORDER,
        RED,
        PANEL_BG_SOFT,
        RED,
        SURFACE,
        RED,
        TEXT,
    )
}

fn share_bar() -> String {
    format!(
        "/* Share bar */\n\
         .share-bar {{\n\
           display: flex;\n\
           flex-flow: row wrap;\n\
           align-items: center;\n\
           gap: {} {};\n\
           margin: {} {} 0;\n\
           padding: {} {};\n\
           border: 1px solid {};\n\
           border-radius: {};\n\
           background: color-mix(in srgb, {} 92%, {});\n\
           box-shadow: {};\n\
           font-size: {};\n\
           transition: background {} ease, border-color {} ease, box-shadow {} ease;\n\
         }}\n\
         \n\
         .curation-wrap .share-bar {{ margin: 0; }}\n\
         \n\
         .share-bar-label {{\n\
           text-transform: uppercase;\n\
           letter-spacing: 0.08em;\n\
           font-weight: 700;\n\
           font-size: {};\n\
           color: {};\n\
           flex-shrink: 0;\n\
           white-space: nowrap;\n\
         }}\n\
         \n\
         .share-bar-input {{\n\
           flex: 1;\n\
           min-width: min(200px, 100%);\n\
           background: {};\n\
           border: 1px solid {};\n\
           border-radius: {};\n\
           color: {};\n\
           padding: {} {};\n\
           font-size: {};\n\
         }}\n\
         \n\
         .share-bar-input:focus {{\n\
           outline: none;\n\
           border-color: {};\n\
         }}",
        GAP_XXS,
        GAP_SM,
        MARGIN_NOTICE_V,
        MARGIN_NOTICE_H,
        SHARE_BAR_PADDING_V,
        SHARE_BAR_PADDING_H,
        PANEL_BORDER,
        RADIUS_MD,
        PANEL_BG_SOFT,
        SURFACE,
        PANEL_SHADOW,
        FS_0,
        TRANSITION_TIMING,
        TRANSITION_TIMING,
        TRANSITION_TIMING,
        FS_0,
        TEXT2,
        SURFACE,
        BORDER,
        RADIUS_SM,
        TEXT,
        SHARE_BAR_INPUT_PADDING_V,
        SHARE_BAR_INPUT_PADDING_H,
        FS_0,
        ACCENT,
    )
}

fn search_panel() -> String {
    format!(
        "/* Search panel shell */\n\
         .search-panel {{ align-self:stretch; padding:{} {}; display:flex; flex-direction:column; gap:{}; background:{}; flex:0 0 auto; box-sizing:border-box; min-width:240px; overflow-y:auto; max-height:calc(100vh - 200px); margin-top:auto; }}\n\
         .search-panel-body {{ display:flex; flex-direction:column; gap:{}; }}\n\
         .filters-toggle {{ display:none !important; }}\n\
         .sidebar-logo-wrap {{ padding:{} 8px 8px; display:flex; justify-content:center; border-top:1px solid {}; margin-top:auto; }}\n\
         \n\
         .sidebar-logo {{ display:block; width:128px; height:128px; }}\n\
         .view-switch [role=\"group\"] {{ background: transparent !important; border-color: {} !important; }}\n\
         .lang-switch [role=\"group\"] {{ background: transparent !important; border-color: {} !important; }}\n\
         @media (max-width: 768px) {{\n\
           .filters-toggle {{ display:inline-flex !important; min-height: 40px; }}\n\
           .search-panel-body {{ display:none !important; }}\n\
           .sidebar.mobile-open .search-panel-body {{ display:flex !important; }}\n\
         }}",
        SEARCH_PANEL_PADDING_V,
        SEARCH_PANEL_PADDING_H,
        GAP_LG,
        PANEL_BG,
        GAP_MD,
        SPACE_1,
        BORDER,
        BORDER,
        BORDER,
    )
}

fn footer() -> String {
    "/* Footer responsive grid sizing */\n\
     @media (min-width: 640px) {\n\
       footer > div {\n\
         grid-template-columns: 1.2fr 1fr !important;\n\
       }\n\
     }"
    .to_string()
}

pub fn css() -> String {
    [
        app_frame(),
        page_header(),
        notices(),
        share_bar(),
        search_panel(),
        footer(),
    ]
    .join("\n\n")
}
