// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: base.

use super::tokens::*;

// ─────────────────────────────────────────────────────────────────────────────
// RESET & DESIGN TOKENS - CSS variables and base styles
// ─────────────────────────────────────────────────────────────────────────────

fn reset_and_tokens() -> String {
    format!(
        "/* ─────────────────────────────────────────────────────────────────────────────\n\
           LOTUS Knowledge Explorer — design tokens + base + app layout\n\
           Previously injected at runtime via `dangerous_inner_html`. Now assembled\n\
           from smaller Rust strings so the browser still caches the result.\n\
           ───────────────────────────────────────────────────────────────────────── */\n\
         \n\
         /* Reset & base */\n\
         *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}\n\
         html, body {{ height: 100%; }}\n\
         \n\
         html, body, #main {{\n\
           width: 100%;\n\
           max-width: 100%;\n\
           overflow-x: hidden;\n\
         }}\n\
         \n\
         img, svg, canvas, video {{\n\
           max-width: 100%;\n\
           height: auto;\n\
         }}\n\
         \n\
         /* Design tokens */\n\
         :root {{\n\
           color-scheme: light dark;\n\
         \n\
           --bg:        #f7fafc;\n\
           --bg2:       #f7fafc;\n\
           --surface:   #ffffff;\n\
           --surface2:  #ffffff;\n\
           --border:    #c3cfdd;\n\
           --text:      #111827;\n\
           --text2:     #233548;\n\
           --text3:     #516274;\n\
           --accent:    #0b5cab;\n\
           --accent2:   #084b8a;\n\
           --btn-primary-bg: #0b5cab;\n\
           --btn-primary-hover-bg: #084b8a;\n\
           --green:     #1f7a4d;\n\
           --red:       #b42318;\n\
           --yellow:    #8a4b0f;\n\
           --purple:    #6941c6;\n\
           --radius:    {};\n\
           --radius-sm: {};\n\
           --shadow-xs: 0 1px 2px rgb(15 23 42 / 6%);\n\
           --shadow-sm: 0 4px 14px rgb(15 23 42 / 6%);\n\
           --shadow-md: 0 10px 30px rgb(15 23 42 / 9%);\n\
           --mono:      'Fira Code', ui-monospace, sfmono-regular, 'JetBrains Mono', consolas, monospace;\n\
           --sans:      'Inter', -apple-system, blinkmacsystemfont, 'Segoe UI', roboto, 'Helvetica Neue', arial, sans-serif;\n\
           --fs-0:      clamp(0.75rem, 0.725rem + 0.17vw, 0.875rem);\n\
           --fs-1:      clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem);\n\
           --fs-2:      clamp(0.9375rem, 0.9rem + 0.28vw, 1.0625rem);\n\
           --fs-3:      clamp(1.125rem, 1.02rem + 0.6vw, 1.5rem);\n\
           --fs-4:      clamp(1.375rem, 1.1rem + 0.85vw, 1.85rem);\n\
           --fs-body:   clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem);\n\
           --fs-label:  clamp(0.6875rem, 0.66rem + 0.14vw, 0.75rem);\n\
           --fs-micro:  clamp(0.75rem, 0.73rem + 0.12vw, 0.8125rem);\n\
           --fs-ui:     clamp(0.8125rem, 0.785rem + 0.16vw, 0.875rem);\n\
           --fs-stat:   clamp(1.125rem, 1.02rem + 0.52vw, 1.375rem);\n\
           --tap-target-min: 40px;\n\
           --space-1:   6px;\n\
           --space-2:   10px;\n\
           --space-3:   14px;\n\
           --space-4:   20px;\n\
           --space-5:   28px;\n\
           --glass:     rgb(255 255 255 / 82%);\n\
           --ring:      0 0 0 3px rgb(11 92 171 / 22%);\n\
           --critical-text: #172535;\n\
           --critical-muted: #33475c;\n\
           --panel-bg: var(--surface);\n\
           --panel-bg-soft: var(--surface);\n\
           --panel-border: color-mix(in srgb, var(--border) 82%, transparent);\n\
           --results-border: var(--panel-border);\n\
           --panel-shadow: var(--shadow-xs);\n\
         \n\
           /* Wikidata colour palette */\n\
           --wd-compound:  #990000;\n\
           --wd-taxon:     #339966;\n\
           --wd-reference: #006699;\n\
           --wd-entries:   #484848;\n\
           --footer-wd-taxon: color-mix(in srgb, var(--wd-taxon) 77%, #000);\n\
           --footer-wd-compound: var(--wd-compound);\n\
           --footer-wd-reference: var(--wd-reference);\n\
           --footer-wd-entries: color-mix(in srgb, var(--wd-entries) 77%, #000);\n\
           --wd-compound-footer: var(--footer-wd-compound);\n\
           --wd-taxon-footer: var(--footer-wd-taxon);\n\
           --wd-reference-footer: var(--footer-wd-reference);\n\
           --wd-entries-footer: var(--footer-wd-entries);\n\
           --wd-compound-soft-bg: color-mix(in srgb, var(--wd-compound) 12%, var(--surface));\n\
           --wd-compound-soft-border: color-mix(in srgb, var(--wd-compound) 34%, var(--results-border));\n\
           --wd-compound-soft-border-weak: color-mix(in srgb, var(--wd-compound) 30%, var(--results-border));\n\
           --wd-taxon-soft-bg: color-mix(in srgb, var(--wd-taxon) 12%, var(--surface));\n\
           --wd-taxon-soft-border: color-mix(in srgb, var(--wd-taxon) 34%, var(--results-border));\n\
           --wd-reference-soft-bg: color-mix(in srgb, var(--wd-reference) 14%, var(--surface));\n\
           --wd-reference-soft-border: color-mix(in srgb, var(--wd-reference) 34%, var(--results-border));\n\
           --wd-reference-soft-border-weak: color-mix(in srgb, var(--wd-reference) 30%, var(--results-border));\n\
         \n\
           /* Stats palette follows the footer color logic so it stays aligned with the theme. */\n\
           --stat-compound-bg: color-mix(in srgb, var(--footer-wd-compound) 12%, var(--surface));\n\
           --stat-compound-border: color-mix(in srgb, var(--footer-wd-compound) 34%, var(--border));\n\
           --stat-compound-stripe: var(--footer-wd-compound);\n\
           --stat-taxon-bg: color-mix(in srgb, var(--footer-wd-taxon) 12%, var(--surface));\n\
           --stat-taxon-border: color-mix(in srgb, var(--footer-wd-taxon) 34%, var(--border));\n\
           --stat-taxon-stripe: var(--footer-wd-taxon);\n\
           --stat-reference-bg: color-mix(in srgb, var(--footer-wd-reference) 12%, var(--surface));\n\
           --stat-reference-border: color-mix(in srgb, var(--footer-wd-reference) 34%, var(--border));\n\
           --stat-reference-stripe: var(--footer-wd-reference);\n\
           --stat-total-bg: color-mix(in srgb, var(--footer-wd-entries) 12%, var(--surface));\n\
           --stat-total-border: color-mix(in srgb, var(--footer-wd-entries) 34%, var(--border));\n\
           --stat-total-stripe: var(--footer-wd-entries);\n\
         }}\n\
         \n\
         @media (prefers-color-scheme: dark) {{\n\
           :root {{\n\
             --bg:        #0f172a;\n\
             --bg2:       #0f172a;\n\
             --surface:   #111827;\n\
             --surface2:  #111827;\n\
             --border:    #38475a;\n\
             --text:      #eef4fb;\n\
             --text2:     #d5deea;\n\
             --text3:     #a7b4c7;\n\
             --accent:    #8cbcff;\n\
             --accent2:   #5e98f3;\n\
             --btn-primary-bg: #0b5cab;\n\
             --btn-primary-hover-bg: #285fcc;\n\
             --green:     #4cc38a;\n\
             --red:       #ff8a80;\n\
             --yellow:    #f0b35e;\n\
             --purple:    #c3a0ff;\n\
             --shadow-xs: 0 1px 2px rgb(0 0 0 / 45%);\n\
             --shadow-sm: 0 4px 14px rgb(0 0 0 / 35%);\n\
             --shadow-md: 0 10px 30px rgb(0 0 0 / 35%);\n\
             --glass:     rgb(22 27 34 / 78%);\n\
             --ring:      0 0 0 3px rgb(140 188 255 / 28%);\n\
             --critical-text: #e8edf5;\n\
             --critical-muted: #d0d9e5;\n\
             --footer-wd-taxon: var(--wd-taxon);\n\
             --footer-wd-compound: color-mix(in srgb, var(--wd-compound) 67%, #fff);\n\
             --footer-wd-reference: color-mix(in srgb, var(--wd-reference) 77%, #fff);\n\
             --footer-wd-entries: color-mix(in srgb, var(--wd-entries) 77%, #fff);\n\
         \n\
             /* Same footer-driven palette in dark mode so stats stay aligned with the app theme. */\n\
             --stat-compound-bg: color-mix(in srgb, var(--footer-wd-compound) 18%, var(--surface));\n\
             --stat-compound-border: color-mix(in srgb, var(--footer-wd-compound) 38%, var(--border));\n\
             --stat-compound-stripe: var(--footer-wd-compound);\n\
             --stat-taxon-bg: color-mix(in srgb, var(--footer-wd-taxon) 18%, var(--surface));\n\
             --stat-taxon-border: color-mix(in srgb, var(--footer-wd-taxon) 38%, var(--border));\n\
             --stat-taxon-stripe: var(--footer-wd-taxon);\n\
             --stat-reference-bg: color-mix(in srgb, var(--footer-wd-reference) 18%, var(--surface));\n\
             --stat-reference-border: color-mix(in srgb, var(--footer-wd-reference) 38%, var(--border));\n\
             --stat-reference-stripe: var(--footer-wd-reference);\n\
             --stat-total-bg: color-mix(in srgb, var(--footer-wd-entries) 20%, var(--surface));\n\
             --stat-total-border: color-mix(in srgb, var(--footer-wd-entries) 40%, var(--border));\n\
             --stat-total-stripe: var(--footer-wd-entries);\n\
           }}\n\
         }}",
        RADIUS, RADIUS_SM,
    )
}

fn data_theme_light() -> String {
    "/* Light mode override via data-theme attribute */\n\
     [data-theme=\"light\"] {\n\
       color-scheme: light;\n\
       --bg:        #f7fafc;\n\
       --bg2:       #f7fafc;\n\
       --surface:   #ffffff;\n\
       --surface2:  #ffffff;\n\
       --border:    #c3cfdd;\n\
       --text:      #111827;\n\
       --text2:     #233548;\n\
       --text3:     #516274;\n\
       --accent:    #0b5cab;\n\
       --accent2:   #084b8a;\n\
       --btn-primary-bg: #0b5cab;\n\
       --btn-primary-hover-bg: #084b8a;\n\
       --green:     #1f7a4d;\n\
       --red:       #b42318;\n\
       --yellow:    #8a4b0f;\n\
       --purple:    #6941c6;\n\
       --shadow-xs: 0 1px 2px rgb(15 23 42 / 6%);\n\
       --shadow-sm: 0 4px 14px rgb(15 23 42 / 6%);\n\
       --shadow-md: 0 10px 30px rgb(15 23 42 / 9%);\n\
       --glass:     rgb(255 255 255 / 82%);\n\
       --ring:      0 0 0 3px rgb(11 92 171 / 22%);\n\
       --critical-text: #172535;\n\
       --critical-muted: #33475c;\n\
       --footer-wd-taxon: color-mix(in srgb, var(--wd-taxon) 77%, #000);\n\
       --footer-wd-compound: var(--wd-compound);\n\
       --footer-wd-reference: var(--wd-reference);\n\
       --footer-wd-entries: color-mix(in srgb, var(--wd-entries) 77%, #000);\n\
       --stat-compound-bg: color-mix(in srgb, var(--footer-wd-compound) 12%, var(--surface));\n\
       --stat-compound-border: color-mix(in srgb, var(--footer-wd-compound) 34%, var(--border));\n\
       --stat-compound-stripe: var(--footer-wd-compound);\n\
       --stat-taxon-bg: color-mix(in srgb, var(--footer-wd-taxon) 12%, var(--surface));\n\
       --stat-taxon-border: color-mix(in srgb, var(--footer-wd-taxon) 34%, var(--border));\n\
       --stat-taxon-stripe: var(--footer-wd-taxon);\n\
       --stat-reference-bg: color-mix(in srgb, var(--footer-wd-reference) 12%, var(--surface));\n\
       --stat-reference-border: color-mix(in srgb, var(--footer-wd-reference) 34%, var(--border));\n\
       --stat-reference-stripe: var(--footer-wd-reference);\n\
       --stat-total-bg: color-mix(in srgb, var(--footer-wd-entries) 12%, var(--surface));\n\
       --stat-total-border: color-mix(in srgb, var(--footer-wd-entries) 34%, var(--border));\n\
       --stat-total-stripe: var(--footer-wd-entries);\n\
     }\n\
     [data-theme=\"light\"] body {\n\
       background: var(--bg);\n\
       color: var(--text);\n\
     }"
    .to_string()
}

fn data_theme_dark() -> String {
    "/* Dark mode override via data-theme attribute */\n\
     [data-theme=\"dark\"] {\n\
       color-scheme: dark;\n\
       --bg:        #0f172a;\n\
       --bg2:       #0f172a;\n\
       --surface:   #111827;\n\
       --surface2:  #111827;\n\
       --border:    #38475a;\n\
       --text:      #eef4fb;\n\
       --text2:     #d5deea;\n\
       --text3:     #a7b4c7;\n\
       --accent:    #8cbcff;\n\
       --accent2:   #5e98f3;\n\
       --btn-primary-bg: #0b5cab;\n\
       --btn-primary-hover-bg: #285fcc;\n\
       --green:     #4cc38a;\n\
       --red:       #ff8a80;\n\
       --yellow:    #f0b35e;\n\
       --purple:    #c3a0ff;\n\
       --shadow-xs: 0 1px 2px rgb(0 0 0 / 45%);\n\
       --shadow-sm: 0 4px 14px rgb(0 0 0 / 35%);\n\
       --shadow-md: 0 10px 30px rgb(0 0 0 / 35%);\n\
       --glass:     rgb(22 27 34 / 78%);\n\
       --ring:      0 0 0 3px rgb(140 188 255 / 28%);\n\
       --critical-text: #e8edf5;\n\
       --critical-muted: #d0d9e5;\n\
       --footer-wd-taxon: var(--wd-taxon);\n\
       --footer-wd-compound: color-mix(in srgb, var(--wd-compound) 67%, #fff);\n\
       --footer-wd-reference: color-mix(in srgb, var(--wd-reference) 77%, #fff);\n\
       --footer-wd-entries: color-mix(in srgb, var(--wd-entries) 77%, #fff);\n\
       \n\
       /* Same footer-driven palette in dark mode so stats stay aligned with the app theme. */\n\
       --stat-compound-bg: color-mix(in srgb, var(--footer-wd-compound) 18%, var(--surface));\n\
       --stat-compound-border: color-mix(in srgb, var(--footer-wd-compound) 38%, var(--border));\n\
       --stat-compound-stripe: var(--footer-wd-compound);\n\
       --stat-taxon-bg: color-mix(in srgb, var(--footer-wd-taxon) 18%, var(--surface));\n\
       --stat-taxon-border: color-mix(in srgb, var(--footer-wd-taxon) 38%, var(--border));\n\
       --stat-taxon-stripe: var(--footer-wd-taxon);\n\
       --stat-reference-bg: color-mix(in srgb, var(--footer-wd-reference) 18%, var(--surface));\n\
       --stat-reference-border: color-mix(in srgb, var(--footer-wd-reference) 38%, var(--border));\n\
       --stat-reference-stripe: var(--footer-wd-reference);\n\
       --stat-total-bg: color-mix(in srgb, var(--footer-wd-entries) 20%, var(--surface));\n\
       --stat-total-border: color-mix(in srgb, var(--footer-wd-entries) 40%, var(--border));\n\
       --stat-total-stripe: var(--footer-wd-entries);\n\
     }\n\
     [data-theme=\"dark\"] body {\n\
       background: var(--bg);\n\
       color: var(--text);\n\
     }"
    .to_string()
}

fn global_base() -> String {
    format!(
        "body {{\n\
           background: {};\n\
           color: {};\n\
           font-family: {};\n\
           font-size: {};\n\
           line-height: 1.52;\n\
           min-height: 100vh;\n\
           text-size-adjust: 100%;\n\
           -webkit-font-smoothing: antialiased;\n\
           -moz-osx-font-smoothing: grayscale;\n\
           font-feature-settings: 'cv02', 'cv03', 'cv04', 'cv11';\n\
           font-optical-sizing: auto;\n\
         }}\n\
         \n\
         fieldset {{ background: transparent; border: none; padding: 0; margin: 0; }}\n\
         legend {{ background: transparent; color: {}; padding: 0; }}\n\
         \n\
         a {{ color: {}; text-decoration: none; transition: color {} ease; }}\n\
         a:hover {{ text-decoration: underline; }}\n\
         \n\
         .page-archive-link,\n\
         .notice a:not(.copy-btn),\n\
         .curation-hint a,\n\
         .footer-link,\n\
         .welcome-inline-link {{\n\
           text-decoration: underline;\n\
           text-decoration-thickness: 0.08em;\n\
           text-underline-offset: 0.14em;\n\
         }}\n\
         \n\
         .page-archive-link:hover,\n\
         .notice a:not(.copy-btn):hover,\n\
         .curation-hint a:hover,\n\
         .footer-link:hover,\n\
         .welcome-inline-link:hover {{\n\
           text-decoration-thickness: 0.11em;\n\
         }}\n\
         ::selection {{ background: color-mix(in srgb, {} 22%, transparent); color: {}; }}\n\
         \n\
         :focus-visible {{\n\
           outline: {} solid {};\n\
           outline-offset: {};\n\
           border-radius: {};\n\
         }}\n\
         \n\
         .sr-only {{\n\
           position: absolute !important;\n\
           width: 1px; height: 1px;\n\
           padding: 0; margin: -1px;\n\
           overflow: hidden; clip: rect(0,0,0,0);\n\
           white-space: nowrap; border: 0;\n\
         }}\n\
         \n\
         @keyframes spin    {{ to {{ transform: rotate(360deg); }} }}\n\
         \n\
         @keyframes fadeIn  {{ from {{ opacity:0; transform:translateY(4px) }} to {{ opacity:1; transform:none }} }}\n\
         \n\
         ::-webkit-scrollbar {{ width:6px; height:6px; }}\n\
         ::-webkit-scrollbar-track {{ background: transparent; }}\n\
         ::-webkit-scrollbar-thumb {{ background: {}; border-radius:3px; }}\n\
         ::-webkit-scrollbar-thumb:hover {{ background: {}; }}",
        BG,
        TEXT,
        FONT_SANS,
        FS_BODY,
        TEXT,
        ACCENT,
        TRANSITION_TIMING,
        ACCENT,
        TEXT,
        FOCUS_OUTLINE_WIDTH,
        ACCENT,
        FOCUS_OUTLINE_OFFSET,
        RADIUS_SM,
        BORDER,
        TEXT3,
    )
}

fn controls_and_forms() -> String {
    format!(
        "/* Forms */\n\
         .form-input, .form-textarea {{\n\
           background:{}; border:1px solid {};\n\
           border-radius:{}; color:{};\n\
           padding:{} {}; font-size:{}; width:100%;\n\
           font-family:{}; transition:border-color {};\n\
         }}\n\
         .form-input:focus, .form-textarea:focus {{ outline:none; border-color:{}; }}\n\
         .form-input.sm {{ width:90px; }}\n\
         \n\
         /* Loading */\n\
         .spinner-lg {{ width:40px; height:40px; border:3px solid {}; border-top-color:{}; border-radius:50%; animation:spin .8s linear infinite; }}\n\
         .spinner-sm {{ width:14px; height:14px; border:2px solid rgb(255 255 255 / 30%); border-top-color:#fff; border-radius:50%; animation:spin .7s linear infinite; display:inline-block; }}\n\
         .loading-state {{ display:flex; flex-direction:column; align-items:center; justify-content:center; gap:{}; padding:{}; color:{}; flex:1; }}\n\
         .loading-hint  {{ font-size:{}; color:{}; }}\n\
         \n\
         /* Pagination / empty */\n\
         .pagination-bar {{ display:flex; align-items:center; justify-content:space-between; gap:{}; padding:8px 0; }}\n\
         .page-info {{ font-size:{}; color:{}; }}\n\
         .empty-state {{ display:flex; flex-direction:column; align-items:center; gap:{}; padding:{} {}; color:{}; }}",
        SURFACE,
        BORDER,
        RADIUS_SM,
        TEXT,
        FORM_INPUT_PADDING_V,
        FORM_INPUT_PADDING_H,
        FS_UI,
        FONT_SANS,
        TRANSITION_TIMING,
        ACCENT,
        BORDER,
        ACCENT,
        GAP_LG,
        LOADING_STATE_PADDING,
        TEXT2,
        FS_0,
        TEXT3,
        GAP_MD,
        FS_0,
        TEXT2,
        GAP_MD,
        EMPTY_STATE_PADDING_V,
        EMPTY_STATE_PADDING_H,
        TEXT2,
    )
}

fn reduced_motion_and_perf() -> String {
    "@supports not ((backdrop-filter: blur(2px)) or (-webkit-backdrop-filter: blur(2px))) {\n\
     .sidebar,\n\
     .main-content,\n\
     .page-header {\n\
       background: var(--bg2);\n\
     }\n\
   }\n\
   \n\
   @media (prefers-reduced-motion: reduce), (update: slow) {\n\
     .data-row:hover,\n\
     .id-badge:hover {\n\
       transform: none;\n\
     }\n\
     \n\
     /* Always show copy button at full opacity — no hover-fade when motion is reduced */\n\
     .query-copy-btn { opacity: 1; }\n\
     \n\
     .data-row,\n\
     .id-badge,\n\
     .page-header-meta,\n\
     .query-panel,\n\
     .ketcher-panel,\n\
     .table-scroll,\n\
     .notice {\n\
       transition: none;\n\
     }\n\
   }\n\
   \n\
   @media (prefers-reduced-data: reduce) {\n\
     body {\n\
       background: var(--bg);\n\
     }\n\
     \n\
     .sidebar,\n\
     .main-content,\n\
     .page-header,\n\
     .results-toolbar,\n\
     .stat-badge,\n\
     .query-panel,\n\
     .table-scroll,\n\
     .ketcher-panel,\n\
     .notice {\n\
       box-shadow: none;\n\
       backdrop-filter: none;\n\
       background-image: none;\n\
     }\n\
   }"
    .to_string()
}

fn large_screen() -> String {
    "/* Large-screen refinements (≥ 1440 px) */\n\
     \n\
     /* Give the main panel uniform, more generous horizontal spacing so every\n\
        section — header, notices, meta bar, share bar, results — shares the same gutter. */\n\
     @media (width >= 1440px) {\n\
       .page-header { padding-left: 32px; padding-right: 32px; }\n\
       .page-header-meta { margin-left: 32px; margin-right: 32px; }\n\
       \n\
       /* share-bar mirrors page-header-meta margin */\n\
       .share-bar { margin-left: 32px; margin-right: 32px; }\n\
       \n\
       /* flex-container children keep their own zero margin */\n\
       .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }\n\
       \n\
       .main-content > .notice {\n\
         padding-left: 32px;\n\
         padding-right: 32px;\n\
       }\n\
       .results-wrap { padding-left: 32px; padding-right: 32px; }\n\
       .curation-wrap { padding-left: 32px; padding-right: 32px; }\n\
       .draw-wrap     { padding-left: 32px; padding-right: 32px; }\n\
     }\n\
     \n\
     /* Removed max-width constraint to allow stats and results to expand freely\n\
        on wide monitors, matching the behavior of share and hashes panels. */"
        .to_string()
}

pub fn css() -> String {
    [
        reset_and_tokens(),
        data_theme_light(),
        data_theme_dark(),
        global_base(),
        controls_and_forms(),
        reduced_motion_and_perf(),
        large_screen(),
    ]
    .join("\n\n")
}
