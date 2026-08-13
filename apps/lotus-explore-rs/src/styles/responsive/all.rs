// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: responsive.

use super::super::tokens::*;

// ─────────────────────────────────────────────────────────────────────────────
// RESPONSIVE BREAKPOINT STYLES - Media queries for different screen sizes
// ─────────────────────────────────────────────────────────────────────────────

fn tablet_768_and_below() -> String {
    "/* Responsive breakpoint pack extracted from style.css for maintainability. */\n\
     \n\
     /* Import enhanced responsive typography tokens */\n\
     :root {\n\
       /* Enhanced responsive font sizing for better mobile UX */\n\
       \n\
       /* Tighter scaling for tablets (768px and below) */\n\
       --fs-title-tablet: clamp(1.2rem, 0.85rem + 1.05vw, 1.5rem);\n\
       --fs-heading-tablet: clamp(1rem, 0.8rem + 0.6vw, 1.25rem);\n\
       --fs-body-tablet: clamp(0.85rem, 0.8rem + 0.15vw, 0.9rem);\n\
       --fs-small-tablet: clamp(0.7rem, 0.68rem + 0.1vw, 0.8rem);\n\
     }\n\
     \n\
     @media (max-width: 768px) {\n\
       :root {\n\
         /* Tablet-specific responsive sizes */\n\
         --fs-0:      clamp(0.7rem, 0.68rem + 0.12vw, 0.8rem);\n\
         --fs-1:      clamp(0.8rem, 0.78rem + 0.12vw, 0.85rem);\n\
         --fs-2:      clamp(0.85rem, 0.83rem + 0.18vw, 0.95rem);\n\
         --fs-3:      clamp(1rem, 0.92rem + 0.4vw, 1.25rem);\n\
         --fs-4:      clamp(1.2rem, 0.95rem + 0.65vw, 1.5rem);\n\
         --fs-body:   clamp(0.8rem, 0.78rem + 0.12vw, 0.85rem);\n\
         --fs-label:  clamp(0.65rem, 0.63rem + 0.1vw, 0.7rem);\n\
         --fs-micro:  clamp(0.7rem, 0.68rem + 0.08vw, 0.75rem);\n\
         --fs-ui:     clamp(0.75rem, 0.73rem + 0.1vw, 0.8rem);\n\
         --fs-stat:   clamp(1rem, 0.92rem + 0.4vw, 1.2rem);\n\
       }\n\
       \n\
       .app-layout   { flex-direction:column; height:auto; min-height:100dvh; overflow:visible; padding:0; gap:0; }\n\
       .sidebar      { width:100%; height:auto; max-height:none; overflow-y:visible; border-radius:0; border-left:0; border-right:0; }\n\
       .main-content { height:auto; min-height:0; overflow-y:visible; border-radius:0; border-left:0; border-right:0; }\n\
       \n\
       .page-header, .welcome, .results-wrap, .app-footer {\n\
         padding-left:max(18px, env(safe-area-inset-left));\n\
         padding-right:max(18px, env(safe-area-inset-right));\n\
       }\n\
       .page-header-meta { margin-left:18px; margin-right:18px; }\n\
       .page-header-meta .meta-item {\n\
         flex-wrap: wrap;\n\
         gap: 6px 4px;\n\
       }\n\
       .notice       { margin-left:18px; margin-right:18px; }\n\
       .draw-wrap { padding-left:18px; padding-right:18px; }\n\
       .ketcher-panel { margin-left:0; margin-right:0; }\n\
       .ketcher-iframe { height:min(70vh, 560px); min-height:420px; }\n\
       .app-footer { gap:0; }\n\
       .app-footer { padding-bottom: max(16px, env(safe-area-inset-bottom)); }\n\
       .footer-row { row-gap:4px; }\n\
       \n\
       .page-brand {\n\
         flex-wrap: wrap;\n\
         align-items: flex-start;\n\
         gap: 8px 10px;\n\
       }\n\
       \n\
       .page-title {\n\
         min-width: 0;\n\
         flex: 1 1 260px;\n\
         font-size: var(--fs-4);\n\
       }\n\
       \n\
       .page-home-link {\n\
         max-width: 100%;\n\
         gap: 8px;\n\
       }\n\
       \n\
       .lang-switch {\n\
         margin-left: 0;\n\
         width: 100%;\n\
         justify-content: flex-start;\n\
         flex-wrap: wrap;\n\
         font-size: var(--fs-0);\n\
       }\n\
       .stat-bar { grid-template-columns: repeat(2, 1fr) !important; }\n\
       .view-switch { flex-wrap:wrap; }\n\
       .view-switch .btn { flex:1 1 180px; justify-content:center; font-size: var(--fs-0); }\n\
       \n\
       /* share-bar: reduce margin to match page-header-meta at this breakpoint */\n\
       .share-bar { margin-left: 18px; margin-right: 18px; }\n\
       .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }\n\
       \n\
       /* Typography scaling for tablet */\n\
       .page-sub { font-size: var(--fs-1); }\n\
       .meta-key { font-size: var(--fs-label); }\n\
       .form-label { font-size: var(--fs-0); }\n\
       .form-hint { font-size: var(--fs-micro); }\n\
       .radio-label { font-size: var(--fs-0); }\n\
       .search-btn { font-size: var(--fs-ui); }\n\
       .btn, .btn-sm { font-size: var(--fs-0); }\n\
       .stat-badge { font-size: var(--fs-stat); }\n\
     }".to_string()
}

fn phone_480() -> String {
    "@media (max-width: 480px) {\n\
     :root {\n\
       /* Phone-specific responsive sizing */\n\
       --fs-0:      clamp(0.65rem, 0.63rem + 0.08vw, 0.75rem);\n\
       --fs-1:      clamp(0.75rem, 0.73rem + 0.08vw, 0.8rem);\n\
       --fs-2:      clamp(0.8rem, 0.78rem + 0.12vw, 0.9rem);\n\
       --fs-3:      clamp(0.95rem, 0.87rem + 0.3vw, 1.15rem);\n\
       --fs-4:      clamp(1.1rem, 0.85rem + 0.6vw, 1.35rem);\n\
       --fs-body:   clamp(0.75rem, 0.73rem + 0.08vw, 0.8rem);\n\
       --fs-label:  clamp(0.6rem, 0.58rem + 0.08vw, 0.65rem);\n\
       --fs-micro:  clamp(0.65rem, 0.63rem + 0.06vw, 0.7rem);\n\
       --fs-ui:     clamp(0.7rem, 0.68rem + 0.08vw, 0.75rem);\n\
       --fs-stat:   clamp(0.95rem, 0.87rem + 0.3vw, 1.1rem);\n\
     }\n\
     \n\
     .sidebar { padding:0; }\n\
     .search-panel { padding:14px 12px; gap:12px; font-size: var(--fs-0); }\n\
     .form-section { padding:8px 10px; border-radius:10px; font-size: var(--fs-body); }\n\
     \n\
     .page-header, .welcome, .results-wrap, .app-footer {\n\
       padding-left:12px;\n\
       padding-right:12px;\n\
       font-size: var(--fs-body);\n\
     }\n\
     \n\
     .page-header-meta {\n\
       margin-left:12px;\n\
       margin-right:12px;\n\
       font-size: var(--fs-label);\n\
       flex-direction: column;\n\
       gap: 8px;\n\
     }\n\
     .page-header-meta .meta-item {\n\
       display: flex;\n\
       flex-direction: column;\n\
       align-items: flex-start;\n\
       gap: 6px;\n\
       width: 100%;\n\
     }\n\
     .page-header-meta .meta-item > span:last-child {\n\
       align-self: flex-end;\n\
     }\n\
     .draw-wrap { padding-left:12px; padding-right:12px; }\n\
     .notice, .ketcher-panel { margin-left:12px; margin-right:12px; }\n\
     .main-content > .notice { padding-left:12px; padding-right:12px; }\n\
     .notice { padding:8px 10px; gap:8px; flex-direction:column; align-items:flex-start; font-size: var(--fs-label); }\n\
     .notice-copy-field { width:100%; min-width:0; }\n\
     .notice-dismiss { align-self:flex-end; margin-left:0; font-size: var(--fs-0); }\n\
     \n\
     /* share-bar: match notice margin and stack input */\n\
     .share-bar { margin-left: 12px; margin-right: 12px; }\n\
     .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }\n\
     .share-bar-input { width: 100%; min-width: 0; font-size: 16px; }\n\
     \n\
     .sidebar-logo-wrap { border-top: none; margin-top: 0; }\n\
     \n\
     .page-title {\n\
       font-size: var(--fs-4);\n\
       line-height: 1.1;\n\
     }\n\
     .page-title-text { line-height:1.1; }\n\
     .page-sub { font-size: var(--fs-1); }\n\
     .sidebar-logo-wrap { padding-top:10px; padding-bottom:12px; }\n\
     .sidebar-logo { width:120px; height:120px; }\n\
     .radio-group, .range-inputs, .toolbar-actions { flex-wrap:wrap; }\n\
     \n\
     .toolbar-actions > .btn,\n\
     .toolbar-actions > .dl-group {\n\
       width:100%;\n\
       font-size: var(--fs-0);\n\
     }\n\
     .footer-line { grid-template-columns:1fr; }\n\
     \n\
     .footer-row {\n\
       grid-template-columns:max-content minmax(0,1fr);\n\
       align-items:flex-start;\n\
       font-size: var(--fs-micro);\n\
     }\n\
     .footer-label { min-width:0; }\n\
     .footer-links { gap:4px 6px; }\n\
     .footer-links li { width:auto; }\n\
     \n\
     .footer-link, .footer-aside, .footer-sep {\n\
       line-height:1.35;\n\
       font-size: var(--fs-micro);\n\
     }\n\
     .range-pair { min-width:120px; }\n\
     \n\
     .range-inputs--pair {\n\
       grid-template-columns: 1fr;\n\
       gap: 8px;\n\
     }\n\
     \n\
     .range-sep--pair {\n\
       display: none;\n\
     }\n\
     \n\
     .form-input, .form-textarea, .search-btn, select, input, textarea { font-size:16px; }\n\
     \n\
     .btn, .search-btn {\n\
       min-height: 44px;\n\
       font-size: var(--fs-0);\n\
     }\n\
     .search-btn { justify-content:center; text-align:center; }\n\
     \n\
     .results-wrap {\n\
       gap: 8px;\n\
       padding: 10px 12px;\n\
     }\n\
     .curation-table-scroll { border-radius: 8px; }\n\
     .curation-results-table { min-width: 900px; }\n\
     \n\
     .curation-results-table th,\n\
     .curation-results-table td {\n\
       padding: 4px 5px;\n\
       font-size: var(--fs-micro);\n\
     }\n\
     \n\
     .stat-bar {\n\
       grid-template-columns: repeat(2, 1fr) !important;\n\
       gap: 6px;\n\
     }\n\
     \n\
     .stat-badge {\n\
       padding: 6px 8px;\n\
       gap: 2px;\n\
       font-size: var(--fs-stat);\n\
     }\n\
     \n\
     .results-table {\n\
       font-size: var(--fs-label);\n\
       table-layout: auto;\n\
       word-break: break-word;\n\
     }\n\
     \n\
     .sort-th, .th-static {\n\
       padding: 5px 4px;\n\
       font-size: var(--fs-micro);\n\
       letter-spacing: 0.05em;\n\
       white-space: normal;\n\
     }\n\
     \n\
     .data-row td {\n\
       padding: 4px 5px;\n\
       font-size: var(--fs-0);\n\
       vertical-align: middle;\n\
       word-break: break-word;\n\
     }\n\
     \n\
     /* Preserve full names on phones - use auto layout */\n\
     .td-depict {\n\
       width: auto;\n\
       padding: 3px 4px !important;\n\
       min-width: 0;\n\
       flex-shrink: 0;\n\
     }\n\
     \n\
     .depict-img {\n\
       width: min(100%, 65px);\n\
       max-width: 65px;\n\
       height: auto;\n\
     }\n\
     \n\
     /* Allow compound/taxon/ref cells to expand for full names */\n\
     .td-compound, .td-taxon, .td-ref {\n\
       min-width: 120px;\n\
       width: auto;\n\
       border-radius: 6px;\n\
       padding: 4px 5px;\n\
     }\n\
     \n\
     .cell-primary {\n\
       font-weight: 500;\n\
       line-height: 1.4;\n\
     }\n\
     \n\
     .id-badge {\n\
       font-size: var(--fs-micro);\n\
       padding: 1px 3px;\n\
       border-radius: 2px;\n\
     }\n\
     \n\
     .badge-row {\n\
       gap: 2px;\n\
       margin-top: 2px;\n\
     }\n\
     \n\
     /* Reduce table scroll max-height on phones */\n\
     .table-scroll {\n\
       max-height: min(60vh, 500px);\n\
       overflow-x: auto;\n\
     }\n\
     \n\
     /* Optimize stat value display */\n\
     .stat-value-row {\n\
       gap: 3px;\n\
     }\n\
     \n\
     .stat-value {\n\
       font-size: var(--fs-stat);\n\
       line-height: 1.1;\n\
     }\n\
     \n\
     .stat-label {\n\
       font-size: var(--fs-micro);\n\
       margin-top: 1px;\n\
     }\n\
     \n\
     .stat-secondary-label {\n\
       font-size: var(--fs-micro);\n\
       padding: 0 3px;\n\
     }\n\
     \n\
      .td-depict {\n\
        width: auto;\n\
        padding:4px 6px !important;\n\
        min-width: 0;\n\
      }\n\
     \n\
      .depict-img {\n\
        width:88px;\n\
        height:56px;\n\
      }\n\
      .td-compound, .td-taxon, .td-ref { min-width:0; width:auto; }\n\
     \n\
      .id-badge {\n\
        font-size: var(--fs-micro);\n\
        padding:1px 5px;\n\
      }\n\
     \n\
      .ketcher-iframe { height:min(62vh, 420px); min-height:300px; }\n\
   }".to_string()
}

fn phone_430_and_360() -> String {
    "@media (max-width: 430px) {\n\
     :root {\n\
       /* Extra small screen (< 430px) optimized sizing */\n\
       --fs-0:      clamp(0.62rem, 0.60rem + 0.06vw, 0.7rem);\n\
       --fs-1:      clamp(0.7rem, 0.68rem + 0.06vw, 0.75rem);\n\
       --fs-2:      clamp(0.75rem, 0.73rem + 0.1vw, 0.85rem);\n\
       --fs-3:      clamp(0.9rem, 0.82rem + 0.25vw, 1.05rem);\n\
       --fs-4:      clamp(1rem, 0.78rem + 0.55vw, 1.2rem);\n\
       --fs-body:   clamp(0.7rem, 0.68rem + 0.06vw, 0.75rem);\n\
       --fs-label:  clamp(0.55rem, 0.53rem + 0.06vw, 0.6rem);\n\
       --fs-micro:  clamp(0.6rem, 0.58rem + 0.04vw, 0.65rem);\n\
       --fs-ui:     clamp(0.65rem, 0.63rem + 0.06vw, 0.7rem);\n\
       --fs-stat:   clamp(0.9rem, 0.82rem + 0.25vw, 1rem);\n\
     }\n\
     \n\
     .page-header,\n\
     .welcome,\n\
     .results-wrap,\n\
     .app-footer,\n\
     .draw-wrap {\n\
       padding-left:10px;\n\
       padding-right:10px;\n\
     }\n\
     \n\
     .notice,\n\
     .ketcher-panel,\n\
     .page-header-meta {\n\
       margin-left:10px;\n\
       margin-right:10px;\n\
     }\n\
     .share-bar { margin-left: 10px; margin-right: 10px; }\n\
     .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }\n\
     \n\
     /* Stack meta items vertically on very narrow screens */\n\
     .page-header-meta {\n\
       flex-direction: column;\n\
       gap: 4px;\n\
       align-items: flex-start;\n\
     }\n\
     \n\
     .page-header-meta .meta-item {\n\
       display: flex;\n\
       flex-direction: row;\n\
       flex-wrap: nowrap;\n\
       align-items: baseline;\n\
       gap: 2px;\n\
       min-width: 0;\n\
     }\n\
     \n\
     .page-header-meta .meta-key {\n\
       font-size: var(--fs-micro);\n\
       white-space: nowrap;\n\
       flex-shrink: 0;\n\
     }\n\
     \n\
     .page-header-meta .meta-sep {\n\
       white-space: nowrap;\n\
       flex-shrink: 0;\n\
     }\n\
     \n\
     .page-header-meta .meta-val {\n\
       white-space: nowrap;\n\
       overflow: hidden;\n\
       text-overflow: ellipsis;\n\
       min-width: 0;\n\
       max-width: 100%;\n\
       font-size: var(--fs-label);\n\
     }\n\
     \n\
     .copy-btn {\n\
       margin-left:0;\n\
       font-size: var(--fs-micro);\n\
     }\n\
     \n\
     .results-toolbar {\n\
       gap:8px;\n\
     }\n\
     \n\
     .toolbar-actions {\n\
       width:100%;\n\
       gap:6px;\n\
     }\n\
     \n\
     .toolbar-actions .btn,\n\
     .toolbar-actions .dl-group {\n\
       width:100%;\n\
       font-size: var(--fs-0);\n\
     }\n\
     \n\
     .dl-group {\n\
       display:flex;\n\
       flex-wrap:wrap;\n\
       gap:6px;\n\
     }\n\
     \n\
     .dl-group .btn {\n\
       flex:1 1 160px;\n\
       min-width:0;\n\
       border-right-width:1px;\n\
       border-radius:var(--radius-sm);\n\
       font-size: var(--fs-0);\n\
     }\n\
     \n\
     .results-table {\n\
       font-size: var(--fs-label);\n\
     }\n\
     \n\
     .curation-results-table {\n\
       min-width: 900px;\n\
       font-size: var(--fs-label);\n\
     }\n\
     \n\
     .th-static,\n\
     .sort-th,\n\
     .sort-btn {\n\
       font-size: var(--fs-micro);\n\
       letter-spacing:0.06em;\n\
     }\n\
     \n\
     .view-switch .btn {\n\
       width:100%;\n\
       flex:1 1 100%;\n\
       font-size: var(--fs-0);\n\
     }\n\
   }\n\
   \n\
   @media (max-width: 360px) {\n\
     :root {\n\
       /* Ultra-small screens (< 360px) - extreme minimum sizing */\n\
       --fs-0:      clamp(0.6rem, 0.58rem + 0.04vw, 0.68rem);\n\
       --fs-1:      clamp(0.68rem, 0.66rem + 0.04vw, 0.72rem);\n\
       --fs-2:      clamp(0.72rem, 0.70rem + 0.08vw, 0.8rem);\n\
       --fs-3:      clamp(0.85rem, 0.77rem + 0.2vw, 1rem);\n\
       --fs-4:      clamp(0.95rem, 0.73rem + 0.5vw, 1.1rem);\n\
       --fs-body:   clamp(0.68rem, 0.66rem + 0.04vw, 0.72rem);\n\
       --fs-label:  clamp(0.52rem, 0.50rem + 0.04vw, 0.58rem);\n\
       --fs-micro:  clamp(0.58rem, 0.56rem + 0.02vw, 0.62rem);\n\
       --fs-ui:     clamp(0.62rem, 0.60rem + 0.04vw, 0.68rem);\n\
       --fs-stat:   clamp(0.85rem, 0.77rem + 0.2vw, 0.95rem);\n\
     }\n\
     \n\
     .page-header,\n\
     .welcome,\n\
     .results-wrap,\n\
     .app-footer,\n\
     .draw-wrap {\n\
       padding-left:8px;\n\
       padding-right:8px;\n\
     }\n\
     \n\
     .page-header-meta,\n\
     .notice,\n\
     .ketcher-panel {\n\
       margin-left:8px;\n\
       margin-right:8px;\n\
     }\n\
     .share-bar { margin-left: 8px; margin-right: 8px; }\n\
     .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }\n\
     \n\
     .main-content > .notice {\n\
       padding-left:8px;\n\
       padding-right:8px;\n\
     }\n\
     \n\
     .filters-toggle {\n\
       min-height: 44px;\n\
     }\n\
     \n\
     .lang-switch .btn,\n\
     .view-switch .btn {\n\
       flex:1 1 100%;\n\
       font-size: var(--fs-0);\n\
     }\n\
     \n\
      .page-title {\n\
        font-size: var(--fs-4);\n\
      }\n\
     \n\
      /* Optimize typography further for ultra-small screens */\n\
      .page-sub { font-size: var(--fs-1); }\n\
      .meta-key { font-size: var(--fs-micro); }\n\
      .form-label { font-size: var(--fs-0); }\n\
      .form-hint { font-size: var(--fs-micro); }\n\
      .radio-label { font-size: var(--fs-0); }\n\
      .search-btn { font-size: var(--fs-0); }\n\
      .btn { font-size: var(--fs-0); }\n\
      .stat-badge { font-size: var(--fs-stat); }\n\
      .footer-link, .footer-aside { font-size: var(--fs-micro); }\n\
     \n\
       /* Table optimization for ultra-small screens - PRESERVE FULL NAMES */\n\
       .results-table {\n\
         font-size: var(--fs-label);\n\
         table-layout: auto;\n\
         word-break: break-word;\n\
       }\n\
     \n\
       .sort-th, .th-static {\n\
         padding: 4px 3px;\n\
         font-size: var(--fs-micro);\n\
         letter-spacing: 0.04em;\n\
         white-space: normal;\n\
       }\n\
     \n\
       .data-row td {\n\
         padding: 3px 4px;\n\
         font-size: var(--fs-0);\n\
         vertical-align: middle;\n\
         word-break: break-word;\n\
       }\n\
     \n\
       .td-depict {\n\
         width: auto;\n\
         padding: 2px 3px !important;\n\
         min-width: 0;\n\
         flex-shrink: 0;\n\
       }\n\
     \n\
       .depict-img {\n\
         width: min(100%, 55px);\n\
         max-width: 55px;\n\
         height: auto;\n\
       }\n\
     \n\
       /* Allow compound/taxon/ref cells space for full names */\n\
       .td-compound, .td-taxon, .td-ref {\n\
         min-width: 100px;\n\
         width: auto;\n\
         border-radius: 4px;\n\
         padding: 3px 4px;\n\
       }\n\
     \n\
       .cell-primary {\n\
         font-weight: 500;\n\
         line-height: 1.3;\n\
         white-space: normal;\n\
         overflow-wrap: break-word;\n\
       }\n\
     \n\
       .id-badge {\n\
         font-size: var(--fs-micro);\n\
         padding: 0 2px;\n\
         border-radius: 2px;\n\
         line-height: 1.2;\n\
       }\n\
     \n\
     \n\
      .badge-row {\n\
        gap: 1px;\n\
        margin-top: 0;\n\
      }\n\
     \n\
      .table-scroll {\n\
        max-height: min(55vh, 400px);\n\
      }\n\
     \n\
      .stat-value {\n\
        font-size: var(--fs-stat);\n\
        line-height: 1.1;\n\
      }\n\
     \n\
      .stat-label {\n\
        font-size: var(--fs-micro);\n\
        margin-top: 0;\n\
      }\n\
     \n\
      /* Hide non-essential columns on ultra-small screens (optional) */\n\
      .results-table tbody tr:nth-child(odd) td {\n\
        background: color-mix(in srgb, var(--surface) 92%, transparent);\n\
      }\n\
     \n\
      .results-table tbody tr:nth-child(even) td {\n\
        background: color-mix(in srgb, var(--surface) 86%, transparent);\n\
      }\n\
     \n\
      /* Stat bar: 2 columns on very small screens */\n\
      .stat-bar {\n\
        grid-template-columns: repeat(2, minmax(0, 1fr)) !important;\n\
      }\n\
   }"
    .to_string()
}

fn tablet_769_1023() -> String {
    "/* Medium screens (768px - 1023px) - tablet optimization */\n\
     @media (min-width: 769px) and (max-width: 1023px) {\n\
       :root {\n\
         /* Tablet-optimized responsive typography */\n\
         --fs-0:      clamp(0.72rem, 0.68rem + 0.15vw, 0.8rem);\n\
         --fs-1:      clamp(0.87rem, 0.82rem + 0.18vw, 0.92rem);\n\
         --fs-2:      clamp(0.94rem, 0.88rem + 0.24vw, 1.02rem);\n\
         --fs-3:      clamp(1.125rem, 1.02rem + 0.46vw, 1.35rem);\n\
         --fs-4:      clamp(1.375rem, 1.1rem + 0.7vw, 1.7rem);\n\
         --fs-body:   clamp(0.87rem, 0.82rem + 0.18vw, 0.92rem);\n\
         --fs-label:  clamp(0.68rem, 0.64rem + 0.12vw, 0.75rem);\n\
         --fs-micro:  clamp(0.75rem, 0.71rem + 0.1vw, 0.8rem);\n\
         --fs-ui:     clamp(0.8125rem, 0.76rem + 0.14vw, 0.87rem);\n\
         --fs-stat:   clamp(1.125rem, 1.02rem + 0.4vw, 1.3rem);\n\
       }\n\
       \n\
       /* Table optimization for tablets - PRESERVE FULL NAMES */\n\
       .results-table {\n\
         font-size: var(--fs-ui);\n\
         table-layout: auto;\n\
         word-break: break-word;\n\
       }\n\
       \n\
       .sort-th, .th-static {\n\
         padding: 8px;\n\
         font-size: var(--fs-label);\n\
         white-space: normal;\n\
       }\n\
       \n\
       .data-row td {\n\
         padding: 6px 8px;\n\
         vertical-align: middle;\n\
         word-break: break-word;\n\
       }\n\
       \n\
       .td-depict {\n\
         padding: 4px 5px !important;\n\
         width: auto !important;\n\
         flex-shrink: 0;\n\
       }\n\
       \n\
       .depict-img {\n\
         width: min(100%, 95px);\n\
         max-width: 95px;\n\
       }\n\
       \n\
       /* Allow full names in cells */\n\
       .td-compound, .td-taxon, .td-ref {\n\
         width: auto;\n\
         min-width: 150px;\n\
       }\n\
       \n\
       .cell-primary {\n\
         font-weight: 500;\n\
         line-height: 1.4;\n\
         white-space: normal;\n\
       }\n\
       \n\
       .stat-badge {\n\
         padding: 10px 12px;\n\
         gap: 5px;\n\
       }\n\
       \n\
       .table-scroll {\n\
         max-height: min(72vh, 900px);\n\
       }\n\
       \n\
       .sidebar-logo-wrap { border-top: none; margin-top: 0; }\n\
     }"
    .to_string()
}

fn desktop_1024() -> String {
    "/* Large screens (1024px and above) - desktop optimization */\n\
     @media (min-width: 1024px) {\n\
       :root {\n\
         /* Desktop-optimized responsive typography */\n\
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
       }\n\
       \n\
       /* Table optimization for desktop - PRESERVE FULL NAMES */\n\
       .results-table {\n\
         font-size: var(--fs-ui);\n\
         table-layout: auto;\n\
         word-break: break-word;\n\
       }\n\
       \n\
       .sort-th, .th-static {\n\
         padding: 10px 12px;\n\
         font-size: var(--fs-label);\n\
         white-space: normal;\n\
       }\n\
       \n\
       .data-row td {\n\
         padding: 8px 12px;\n\
         vertical-align: middle;\n\
         word-break: break-word;\n\
       }\n\
       \n\
       .td-depict {\n\
         padding: 6px 10px !important;\n\
         width: auto !important;\n\
         flex-shrink: 0;\n\
       }\n\
       \n\
       .depict-img {\n\
         width: min(100%, 110px);\n\
         max-width: 110px;\n\
       }\n\
       \n\
       /* Allow full names and references */\n\
       .td-compound, .td-taxon, .td-ref {\n\
         width: auto;\n\
         min-width: 180px;\n\
       }\n\
       \n\
       .cell-primary {\n\
         font-weight: 500;\n\
         line-height: 1.4;\n\
         white-space: normal;\n\
       }\n\
       \n\
       .stat-badge {\n\
         padding: 12px 14px;\n\
         gap: 6px;\n\
       }\n\
       \n\
       .table-scroll {\n\
         max-height: min(72vh, 980px);\n\
       }\n\
       \n\
       /* Wider badges on desktop */\n\
       .id-badge {\n\
         padding: 2px 6px;\n\
         border-radius: 3px;\n\
       }\n\
       \n\
       .sidebar-logo-wrap { border-top: none; margin-top: 0; }\n\
       .filters-toggle { display:none; }\n\
     }"
    .to_string()
}

fn wide_1440() -> String {
    "/* Extra large screens (1440px+) - ensure optimal readability */\n\
     @media (min-width: 1440px) {\n\
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
     }"
    .to_string()
}

fn tablet_fields() -> String {
    "/* Tablet-specific optimizations (768px - 1023px) */\n\
     @media (min-width: 769px) and (max-width: 1023px) {\n\
       /* Improve text readability on tablets */\n\
       body { line-height: 1.6; }\n\
       \n\
       /* Optimize list item spacing */\n\
       li { margin-bottom: 8px; }\n\
       \n\
       /* Improve form field spacing */\n\
       .form-input, .form-textarea, input, textarea, select {\n\
         min-height: 40px;\n\
       }\n\
     }"
    .to_string()
}

fn high_contrast() -> String {
    "/* Accessibility: Improve font sizing for readability */\n\
     @media (prefers-contrast: more) {\n\
       body { font-size: 18px; }\n\
       \n\
       :root {\n\
         /* Increase all font sizes by ~10% for accessibility */\n\
         --fs-0:      clamp(0.82rem, 0.80rem + 0.18vw, 0.96rem);\n\
         --fs-1:      clamp(0.96rem, 0.93rem + 0.22vw, 1.03rem);\n\
         --fs-2:      clamp(1.03rem, 0.99rem + 0.31vw, 1.17rem);\n\
         --fs-3:      clamp(1.24rem, 1.12rem + 0.66vw, 1.65rem);\n\
         --fs-4:      clamp(1.51rem, 1.21rem + 0.94vw, 2.04rem);\n\
         --fs-body:   clamp(0.96rem, 0.93rem + 0.22vw, 1.03rem);\n\
         --fs-label:  clamp(0.76rem, 0.73rem + 0.15vw, 0.83rem);\n\
         --fs-micro:  clamp(0.82rem, 0.80rem + 0.13vw, 0.89rem);\n\
         --fs-ui:     clamp(0.89rem, 0.86rem + 0.18vw, 0.96rem);\n\
         --fs-stat:   clamp(1.24rem, 1.12rem + 0.57vw, 1.51rem);\n\
       }\n\
     }"
    .to_string()
}

fn dark_mode_text() -> String {
    "/* Dark mode: Slightly larger text for better readability */\n\
     @media (prefers-color-scheme: dark) {\n\
       /* Text is perceived as smaller in dark mode, so we can increase it slightly */\n\
       body { letter-spacing: 0.3px; }\n\
     }"
    .to_string()
}

fn mobile_heading_and_fields() -> String {
    "/* Mobile-first heading and typography scaling */\n\
     @media (max-width: 768px) {\n\
       /* Ensure all headings are readable and don't overflow */\n\
       h1, .page-title { word-break: break-word; overflow-wrap: break-word; }\n\
       h2, h3, h4, h5, h6 { word-break: break-word; overflow-wrap: break-word; }\n\
       \n\
       /* Button and form element minimum touch target on mobile */\n\
       button, .btn, input[type=\"button\"], input[type=\"submit\"] {\n\
         min-height: 44px;\n\
         min-width: 44px;\n\
       }\n\
       \n\
       /* Remove search panel width constraints on mobile */\n\
       .search-panel { max-width: none; width: 100%; }\n\
       \n\
       /* Improve link and interactive element sizing on touch devices */\n\
       a, .copy-btn, .id-badge { padding: 4px 8px; }\n\
       \n\
       /* Optimize table cell padding for mobile readability */\n\
       table td, table th {\n\
         padding: 6px 4px;\n\
         word-break: break-word;\n\
         overflow-wrap: break-word;\n\
       }\n\
       \n\
       /* Improve textarea usability on mobile */\n\
       textarea {\n\
         min-height: 120px;\n\
         font-size: 16px;\n\
       }\n\
       \n\
       /* Stack form groups vertically on mobile */\n\
       .form-group, .form-row {\n\
         display: flex;\n\
         flex-direction: column;\n\
         gap: 8px;\n\
       }\n\
       \n\
       /* Improve list readability on mobile */\n\
       ul, ol, li {\n\
         word-break: break-word;\n\
         overflow-wrap: break-word;\n\
       }\n\
     }"
    .to_string()
}

pub fn css() -> String {
    [
        tablet_768_and_below(),
        phone_480(),
        phone_430_and_360(),
        tablet_769_1023(),
        desktop_1024(),
        wide_1440(),
        mobile_heading_and_fields(),
        tablet_fields(),
        high_contrast(),
        dark_mode_text(),
    ]
    .join("\n\n")
}
