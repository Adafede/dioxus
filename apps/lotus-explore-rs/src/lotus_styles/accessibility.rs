// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: accessibility.

const SKIP_LINK: &str = r###"/* Accessibility-specific styles and media queries.
   Extracted from style.css for maintainability and cacheable separately. */

.skip-link:focus {
  top: 0;
  outline: 2px solid var(--text);
  outline-offset: 2px;
}
.skip-link:hover {
  text-decoration: underline;
}
"###;

const REDUCED_MOTION: &str = r###"
@media (prefers-reduced-motion: reduce) {
  .sidebar::before,
  .page-header::before {
    animation: none;
  }

  .data-row:hover,
  .id-badge:hover {
    transform: none;
  }

  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }

  .sidebar,
  .main-content,
  .page-header {
    backdrop-filter: none;
  }
}
"###;

const REDUCED_TRANSPARENCY: &str = r###"
@media (prefers-reduced-transparency: reduce) {
  .sidebar,
  .main-content,
  .page-header {
    backdrop-filter: none;
    background: var(--bg2);
  }
}
"###;

const HIGH_CONTRAST: &str = r###"
@media (prefers-contrast: more) {
  :root {
    --border: #7a879a;
    --text3: #334155;
    --critical-muted: #223247;
  }

  .notice,
  .query-panel,
  .table-scroll,
  .ketcher-panel {
    border-width: 2px;
    box-shadow: none;
  }

  .sort-th,
  .th-static,
  .form-label,
  .footer-label,
  .notice-label,
  .stat-label,
  .meta-key {
    color: var(--text);
  }

  .meta-val,
  .page-sub,
  .welcome-lead,
  .form-hint {
    color: var(--text);
  }
}
"###;

const FORCED_COLORS: &str = r###"
@media (forced-colors: active) {
  .btn,
  .search-btn,
  .notice,
  .results-toolbar,
  .table-scroll,
  .query-panel,
  .stat-badge,
  .id-badge {
    border: 1px solid CanvasText;
    box-shadow: none;
    forced-color-adjust: auto;
  }

  .btn:focus-visible,
  .search-btn:focus-visible,
  .sort-btn:focus-visible,
  .notice-dismiss:focus-visible,
  .id-badge:focus-visible,
  .copy-btn:focus-visible {
    outline: 2px solid Highlight;
    outline-offset: 2px;
  }
}
"###;

pub fn css() -> String {
    [
        SKIP_LINK,
        REDUCED_MOTION,
        REDUCED_TRANSPARENCY,
        HIGH_CONTRAST,
        FORCED_COLORS,
    ]
    .join("\n\n")
}
