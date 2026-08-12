// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus Knowledge Explorer CSS bundled as Rust constants.

pub const LOTUS_BASE_CSS: &str = r###"/* ─────────────────────────────────────────────────────────────────────────────
   LOTUS Knowledge Explorer — design tokens + base + app layout
   Previously injected at runtime via `dangerous_inner_html`. Now shipped as
   a static asset so the browser caches it and the wasm bundle is smaller.
   ───────────────────────────────────────────────────────────────────────── */

/* Accessibility and responsive packs are linked from index.html so they can
   be loaded after this base stylesheet and override it predictably. */

/* Keep feature-specific styling in small packs to avoid a monolithic file. */
@import url("./styles/query_panel_pack.css");
@import url("./styles/curation_pack.css");
@import url("./styles/results_pack.css");
@import url("./styles/layout_shell_pack.css");
@import url("./styles/form_controls_pack.css");
@import url("./styles/welcome_pack.css");
@import url("./styles/table_cells_pack.css");
@import url("./styles/footer_pack.css");

/* ── Reset & base ────────────────────────────────────────────────────────── */
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html, body { height: 100%; }

html, body, #main {
  width: 100%;
  max-width: 100%;
  overflow-x: hidden;
}

img, svg, canvas, video {
  max-width: 100%;
  height: auto;
}

/* ── Design tokens ───────────────────────────────────────────────────────── */
:root {
  color-scheme: light dark;

  --bg:        #f6f8fb;
  --bg2:       #fff;
  --surface:   #fbfcfe;
  --surface2:  #e7edf5;
  --border:    #c3cfdd;
  --text:      #111827;
  --text2:     #233548;
  --text3:     #516274;
  --accent:    #0b5cab;
  --accent2:   #084b8a;
  --btn-primary-bg: #0b5cab;
  --btn-primary-hover-bg: #084b8a;
  --green:     #1f7a4d;
  --red:       #b42318;
  --yellow:    #8a4b0f;
  --purple:    #6941c6;
  --radius:    10px;
  --radius-sm: 4px;
  --shadow-xs: 0 1px 2px rgb(15 23 42 / 6%);
  --shadow-sm: 0 4px 14px rgb(15 23 42 / 6%);
  --shadow-md: 0 10px 30px rgb(15 23 42 / 9%);
  --mono:      'Fira Code', ui-monospace, sfmono-regular, 'JetBrains Mono', consolas, monospace;
  --sans:      'Inter', -apple-system, blinkmacsystemfont, 'Segoe UI', roboto, 'Helvetica Neue', arial, sans-serif;
  --fs-0:      clamp(0.75rem, 0.725rem + 0.17vw, 0.875rem);
  --fs-1:      clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem);
  --fs-2:      clamp(0.9375rem, 0.9rem + 0.28vw, 1.0625rem);
  --fs-3:      clamp(1.125rem, 1.02rem + 0.6vw, 1.5rem);
  --fs-4:      clamp(1.375rem, 1.1rem + 0.85vw, 1.85rem);
  --fs-body:   clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem);
  --fs-label:  clamp(0.6875rem, 0.66rem + 0.14vw, 0.75rem);
  --fs-micro:  clamp(0.75rem, 0.73rem + 0.12vw, 0.8125rem);
  --fs-ui:     clamp(0.8125rem, 0.785rem + 0.16vw, 0.875rem);
  --fs-stat:   clamp(1.125rem, 1.02rem + 0.52vw, 1.375rem);
  --tap-target-min: 40px;
  --space-1:   6px;
  --space-2:   10px;
  --space-3:   14px;
  --space-4:   20px;
  --space-5:   28px;
  --glass:     rgb(255 255 255 / 82%);
  --ring:      0 0 0 3px rgb(11 92 171 / 22%);
  --critical-text: #172535;
  --critical-muted: #33475c;
  --panel-bg: color-mix(in srgb, var(--surface) 92%, var(--bg2));
  --panel-bg-soft: color-mix(in srgb, var(--surface) 88%, var(--bg2));
  --panel-border: color-mix(in srgb, var(--border) 82%, transparent);
  --results-border: var(--panel-border);
  --panel-shadow: var(--shadow-xs);

  /* Wikidata colour palette */
  --wd-compound:  #900;
  --wd-taxon:     #396;
  --wd-reference: #069;
  --wd-entries:   #484848;
  --wd-compound-stripe: color-mix(in srgb, var(--wd-compound) 78%, #fff);
  --wd-taxon-stripe: color-mix(in srgb, var(--wd-taxon) 78%, #fff);
  --wd-reference-stripe: color-mix(in srgb, var(--wd-reference) 78%, #fff);
  --wd-entries-stripe: color-mix(in srgb, var(--wd-entries) 74%, #fff);
  --wd-compound-soft-bg: color-mix(in srgb, var(--wd-compound) 12%, var(--surface));
  --wd-compound-soft-border: color-mix(in srgb, var(--wd-compound) 34%, var(--results-border));
  --wd-compound-soft-border-weak: color-mix(in srgb, var(--wd-compound) 30%, var(--results-border));
  --wd-taxon-soft-bg: color-mix(in srgb, var(--wd-taxon) 12%, var(--surface));
  --wd-taxon-soft-border: color-mix(in srgb, var(--wd-taxon) 34%, var(--results-border));
  --wd-reference-soft-bg: color-mix(in srgb, var(--wd-reference) 14%, var(--surface));
  --wd-reference-soft-border: color-mix(in srgb, var(--wd-reference) 34%, var(--results-border));
  --wd-reference-soft-border-weak: color-mix(in srgb, var(--wd-reference) 30%, var(--results-border));

  /* Stats palette tuned for readable contrast in light mode. */
  --stat-compound-bg: color-mix(in srgb, var(--wd-compound) 10%, var(--surface));
  --stat-compound-border: color-mix(in srgb, var(--wd-compound) 30%, var(--border));
  --stat-compound-stripe: color-mix(in srgb, var(--wd-compound) 78%, #fff);
  --stat-taxon-bg: color-mix(in srgb, var(--wd-taxon) 11%, var(--surface));
  --stat-taxon-border: color-mix(in srgb, var(--wd-taxon) 30%, var(--border));
  --stat-taxon-stripe: color-mix(in srgb, var(--wd-taxon) 78%, #fff);
  --stat-reference-bg: color-mix(in srgb, var(--wd-reference) 10%, var(--surface));
  --stat-reference-border: color-mix(in srgb, var(--wd-reference) 30%, var(--border));
  --stat-reference-stripe: color-mix(in srgb, var(--wd-reference) 78%, #fff);
  --stat-total-bg: color-mix(in srgb, var(--wd-entries) 8%, var(--surface));
  --stat-total-border: color-mix(in srgb, var(--wd-entries) 28%, var(--border));
  --stat-total-stripe: color-mix(in srgb, var(--wd-entries) 74%, #fff);
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg:        #10141b;
    --bg2:       #171d26;
    --surface:   #1f2733;
    --surface2:  #2a3443;
    --border:    #38475a;
    --text:      #eef4fb;
    --text2:     #d5deea;
    --text3:     #a7b4c7;
    --accent:    #8cbcff;
    --accent2:   #5e98f3;
    --btn-primary-bg: #2f6fed;
    --btn-primary-hover-bg: #285fcc;
    --green:     #4cc38a;
    --red:       #ff8a80;
    --yellow:    #f0b35e;
    --purple:    #c3a0ff;
    --shadow-xs: 0 1px 2px rgb(0 0 0 / 45%);
    --shadow-sm: 0 4px 14px rgb(0 0 0 / 35%);
    --shadow-md: 0 10px 30px rgb(0 0 0 / 35%);
    --glass:     rgb(22 27 34 / 78%);
    --ring:      0 0 0 3px rgb(140 188 255 / 28%);
    --critical-text: #e8edf5;
    --critical-muted: #d0d9e5;

    /* Slightly stronger fills/stripes in dark mode to preserve distinction. */
    --stat-compound-bg: color-mix(in srgb, var(--wd-compound) 24%, var(--surface));
    --stat-compound-border: color-mix(in srgb, var(--wd-compound) 42%, var(--border));
    --stat-compound-stripe: color-mix(in srgb, var(--wd-compound) 64%, #fff);
    --stat-taxon-bg: color-mix(in srgb, var(--wd-taxon) 24%, var(--surface));
    --stat-taxon-border: color-mix(in srgb, var(--wd-taxon) 42%, var(--border));
    --stat-taxon-stripe: color-mix(in srgb, var(--wd-taxon) 64%, #fff);
    --stat-reference-bg: color-mix(in srgb, var(--wd-reference) 24%, var(--surface));
    --stat-reference-border: color-mix(in srgb, var(--wd-reference) 42%, var(--border));
    --stat-reference-stripe: color-mix(in srgb, var(--wd-reference) 64%, #fff);
    --stat-total-bg: color-mix(in srgb, var(--wd-entries) 20%, var(--surface));
    --stat-total-border: color-mix(in srgb, var(--wd-entries) 40%, var(--border));
    --stat-total-stripe: color-mix(in srgb, var(--wd-entries) 62%, #fff);
  }
}

body {
  background: var(--bg);
  color: var(--text);
  font-family: var(--sans);
  font-size: var(--fs-body);
  line-height: 1.52;
  min-height: 100vh;
  text-size-adjust: 100%;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  font-feature-settings: 'cv02', 'cv03', 'cv04', 'cv11';
  font-optical-sizing: auto;
}

.skip-link {
  position: absolute;
  left: 12px;
  top: -40px;
  background: var(--text);
  color: var(--bg2);
  border: 1px solid var(--border);
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  z-index: 50;
}

.skip-link:focus {
  top: 10px;
}
a { color: var(--accent); text-decoration: none; transition: color .15s ease; }
a:hover { text-decoration: underline; }

.page-archive-link,
.notice a:not(.copy-btn),
.curation-hint a,
.footer-link,
.welcome-inline-link {
  text-decoration: underline;
  text-decoration-thickness: 0.08em;
  text-underline-offset: 0.14em;
}

.page-archive-link:hover,
.notice a:not(.copy-btn):hover,
.curation-hint a:hover,
.footer-link:hover,
.welcome-inline-link:hover {
  text-decoration-thickness: 0.11em;
}
::selection { background: color-mix(in srgb, var(--accent) 22%, transparent); color: var(--text); }

:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}

.sr-only {
  position: absolute !important;
  width: 1px; height: 1px;
  padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0,0,0,0);
  white-space: nowrap; border: 0;
}

@keyframes spin    { to { transform: rotate(360deg); } }

@keyframes fadeIn  { from { opacity:0; transform:translateY(4px) } to { opacity:1; transform:none } }

::-webkit-scrollbar { width:6px; height:6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border); border-radius:3px; }
::-webkit-scrollbar-thumb:hover { background: var(--text3); }

/* ── Buttons ─────────────────────────────────────────────────────────────── */
.btn {
  display: inline-flex; align-items: center; gap: 6px;
  border: 1px solid var(--border); border-radius: var(--radius-sm);
  min-height: var(--tap-target-min);
  padding: 8px 14px; font-size: var(--fs-0); font-weight: 600;
  cursor: pointer; background: var(--surface); color: var(--text);
  box-shadow: var(--shadow-xs);
  transition: background .15s, border-color .15s, box-shadow .15s, transform .12s ease;
}
.btn:disabled { opacity: .45; cursor: not-allowed; }

.btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--surface2) 82%, var(--bg2));
  box-shadow: var(--shadow-sm);
}
.btn:active { transform: translateY(1px); }
.btn-primary { background: var(--btn-primary-bg); border-color: var(--btn-primary-bg); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--btn-primary-hover-bg); border-color: var(--btn-primary-hover-bg); }

.btn-soft-accent {
  color: var(--text);
  border-color: color-mix(in srgb, var(--accent2) 52%, var(--border));
  background: color-mix(in srgb, var(--accent) 20%, var(--surface));
}

.btn-soft-accent:hover:not(:disabled) {
  color: var(--text);
  border-color: color-mix(in srgb, var(--accent2) 66%, var(--border));
  background: color-mix(in srgb, var(--accent) 28%, var(--surface));
}
.btn-sm { min-height: 34px; padding: 5px 10px; font-size: var(--fs-0); }
.btn-xs { min-height: 30px; padding: 2px 8px; font-size: var(--fs-label); line-height: 1.2; border-radius: 4px; }
.btn-block { width: 100%; justify-content: center; text-align: center; }

/* ── Copy button (used next to QIDs, hashes, share URL, SPARQL queries) ─── */
.copy-btn {
  margin-left: 6px;
  font-family: var(--sans), system-ui, sans-serif;
  font-weight: 500;
  letter-spacing: .02em;
  color: var(--text2);
  background: var(--surface);
  border: 1px solid var(--border);
  cursor: pointer;
  transition: color .15s, background .15s, border-color .15s;
  vertical-align: baseline;
}
.copy-btn:hover { color: var(--text); background: var(--surface2); border-color: var(--text3); }
.copy-btn:active { transform: translateY(1px); }


/* ── Forms ───────────────────────────────────────────────────────────────── */
.form-input, .form-textarea {
  background:var(--surface); border:1px solid var(--border);
  border-radius:var(--radius-sm); color:var(--text);
  padding:9px 11px; font-size:var(--fs-ui); width:100%;
  font-family:var(--sans); transition:border-color .15s;
}
.form-input:focus, .form-textarea:focus { outline:none; border-color:var(--accent); }
.form-input.sm { width:90px; }

/* ── Loading ─────────────────────────────────────────────────────────────── */
.spinner-lg { width:40px; height:40px; border:3px solid var(--border); border-top-color:var(--accent); border-radius:50%; animation:spin .8s linear infinite; }
.spinner-sm { width:14px; height:14px; border:2px solid rgb(255 255 255 / 30%); border-top-color:#fff; border-radius:50%; animation:spin .7s linear infinite; display:inline-block; }
.loading-state { display:flex; flex-direction:column; align-items:center; justify-content:center; gap:14px; padding:48px; color:var(--text2); flex:1; }
.loading-hint  { font-size:var(--fs-0); color:var(--text3); }

/* ── Pagination / empty ──────────────────────────────────────────────────── */
.pagination-bar { display:flex; align-items:center; justify-content:space-between; gap:12px; padding:8px 0; }
.page-info { font-size:var(--fs-0); color:var(--text2); }
.empty-state { display:flex; flex-direction:column; align-items:center; gap:12px; padding:64px 24px; color:var(--text2); }

@supports not ((backdrop-filter: blur(2px)) or (-webkit-backdrop-filter: blur(2px))) {
  .sidebar,
  .main-content,
  .page-header {
    background: var(--bg2);
  }
}

@media (prefers-reduced-motion: reduce), (update: slow) {
  .data-row:hover,
  .id-badge:hover,
  .btn:active,
  .search-btn:active {
    transform: none;
  }

  /* Always show copy button at full opacity — no hover-fade when motion is reduced */
  .query-copy-btn { opacity: 1; }

  .btn,
  .search-btn,
  .copy-btn,
  .query-copy-btn,
  .data-row,
  .id-badge,
  .page-header-meta,
  .query-panel,
  .ketcher-panel,
  .table-scroll,
  .notice {
    transition: none;
  }
}

@media (prefers-reduced-data: reduce) {
  body {
    background: var(--bg);
  }

  .sidebar,
  .main-content,
  .page-header,
  .results-toolbar,
  .stat-badge,
  .query-panel,
  .table-scroll,
  .ketcher-panel,
  .notice {
    box-shadow: none;
    backdrop-filter: none;
    background-image: none;
  }
}

/* ── Large-screen refinements (≥ 1440 px) ───────────────────────────────── */

/* Give the main panel uniform, more generous horizontal spacing so every
   section — header, notices, meta bar, share bar, results — shares the same gutter. */
@media (width >= 1440px) {
  .page-header { padding-left: 32px; padding-right: 32px; }
  .page-header-meta { margin-left: 32px; margin-right: 32px; }

  /* share-bar mirrors page-header-meta margin */
  .share-bar { margin-left: 32px; margin-right: 32px; }

  /* flex-container children keep their own zero margin */
  .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }

  .main-content > .notice {
    padding-left: 32px;
    padding-right: 32px;
  }
  .results-wrap { padding-left: 32px; padding-right: 32px; }
  .curation-wrap { padding-left: 32px; padding-right: 32px; }
  .draw-wrap     { padding-left: 32px; padding-right: 32px; }
}

/* Removed max-width constraint to allow stats and results to expand freely
   on wide monitors, matching the behavior of share and hashes panels. */
"###;
pub const LOTUS_ACCESSIBILITY_CSS: &str = r###"/* Accessibility-specific styles and media queries.
   Extracted from style.css for maintainability and cacheable separately. */

/* ── Skip link (WCAG 2.4.1 — Bypass Blocks) ─────────────────────────────── */
.skip-link {
  position: absolute;
  top: -100%;
  left: 0.5rem;
  z-index: 9999;
  padding: 0.5rem 1rem;
  background: transparent;
  color: var(--text);
  font-size: 0.875rem;
  font-weight: 600;
  border-radius: 0 0 4px 4px;
  text-decoration: none;
  transition: top 0.1s;
}
.skip-link:focus {
  top: 0;
  outline: 2px solid var(--text);
  outline-offset: 2px;
}
.skip-link:hover {
  text-decoration: underline;
}

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

@media (prefers-reduced-transparency: reduce) {
  .sidebar,
  .main-content,
  .page-header {
    backdrop-filter: none;
    background: var(--bg2);
  }
}

@media (prefers-contrast: more) {
  :root {
    --border: #7a879a;
    --text3: #334155;
    --critical-muted: #223247;
  }

  .notice,
  .query-panel,
  .table-scroll,
  .ketcher-panel,
  .curation-card {
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
pub const LOTUS_CURATION_CSS: &str = r###"/* Curation and structure-editor layout pack extracted from style.css. */

.curation-wrap {
  padding: 12px 22px 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* Notices inside curation-wrap must not add extra horizontal margin -
   the wrapper already provides horizontal padding. */
.curation-wrap > .notice {
  margin-left: 0;
  margin-right: 0;
  margin-top: 0;
}

.curation-title { font-size: var(--fs-3); color: var(--text); }
.curation-subtitle { color: var(--text); font-size: var(--fs-ui); }
.curation-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 280px), 1fr)); gap: 12px; }

.curation-card {
  border: 1px solid var(--panel-border);
  border-radius: var(--radius);
  background: var(--panel-bg-soft);
  box-shadow: var(--panel-shadow);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.curation-form-grid { display: grid; grid-template-columns: 1fr; gap: 8px; }
.curation-actions { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
.curation-space-between { justify-content: space-between; }
.curation-hint { font-size: var(--fs-0); color: var(--text); }

.draw-wrap {
  padding: 12px 22px 18px;
}

.curation-tsv { min-height: 130px; font-family: var(--mono); border-radius: 8px; }
.curation-qs { min-height: 220px; font-family: var(--mono); border-radius: 8px; }

.curation-file-input {
  color: var(--text2);
  max-width: 100%;
  font-size: var(--fs-0);
}

.curation-file-input::file-selector-button {
  margin-right: 10px;
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-sm);
  padding: 6px 10px;
  min-height: 34px;
  background: color-mix(in srgb, var(--surface2) 44%, var(--surface));
  color: var(--text);
  font-weight: 600;
  cursor: pointer;
  transition: background .15s ease, border-color .15s ease;
}

.curation-file-input::file-selector-button:hover {
  background: color-mix(in srgb, var(--surface2) 68%, var(--surface));
  border-color: color-mix(in srgb, var(--border) 84%, var(--accent));
}

.curation-table { width: 100%; border-collapse: collapse; font-size: var(--fs-ui); table-layout: auto; word-break: break-word; }

.curation-table th,
.curation-table td {
  border-bottom: 1px solid var(--border);
  text-align: left;
  vertical-align: top;
  padding: 8px 10px;
}

.curation-table thead { position: sticky; top: 0; z-index: 2; background: var(--bg2); }

.curation-table tbody tr { transition: background .14s ease; }
.curation-table tbody tr:hover td { background: color-mix(in srgb, var(--surface2) 84%, var(--bg2)); }
.curation-table tbody tr:nth-child(odd) td { background: color-mix(in srgb, var(--surface) 94%, transparent); }
.curation-table tbody tr:nth-child(even) td { background: color-mix(in srgb, var(--surface) 88%, transparent); }

.curation-table-scroll {
  width: 100%;
  min-width: 0;
  overflow-x: auto;
  overflow-y: visible;
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  background: var(--panel-bg-soft);
  box-shadow: var(--panel-shadow);
  transition: background .15s ease, border-color .15s ease, box-shadow .15s ease;
}

.curation-table-scroll:focus-visible {
  outline: none;
  border-color: color-mix(in srgb, var(--accent) 44%, var(--panel-border));
  box-shadow: var(--ring);
}

.curation-scroll-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text3);
  font-size: var(--fs-0);
  line-height: 1.4;
}

.curation-scroll-hint::before {
  /* Use unicode escape to keep this stylesheet ASCII-only. */
  content: "\2194";
  color: var(--accent);
  font-weight: 700;
  font-size: 1.05em;
}

.curation-results-table {
  min-width: max-content;
}

/* col 1 – Status + badges + note */
.curation-results-table th:nth-child(1),
.curation-results-table td:nth-child(1) {
  min-width: 220px;
}

/* col 2 – Wikidata QID */
.curation-results-table th:nth-child(2),
.curation-results-table td:nth-child(2) {
  min-width: 12ch;
}

/* col 3 – Name */
.curation-results-table th:nth-child(3),
.curation-results-table td:nth-child(3) {
  min-width: 18ch;
}

/* col 4 – Original SMILES  /  col 5 – Canonical SMILES */
.curation-results-table th:nth-child(4),
.curation-results-table th:nth-child(5),
.curation-results-table td:nth-child(4),
.curation-results-table td:nth-child(5) {
  min-width: 220px;
  max-width: 320px;
}

/* col 6 – InChIKey (always 27 chars in mono) */
.curation-results-table th:nth-child(6),
.curation-results-table td:nth-child(6) {
  min-width: 28ch;
}

/* col 7 – InChI */
.curation-results-table th:nth-child(7),
.curation-results-table td:nth-child(7) {
  min-width: 220px;
  max-width: 320px;
}

/* col 8 – Formula */
.curation-results-table th:nth-child(8),
.curation-results-table td:nth-child(8) {
  min-width: 12ch;
}

/* col 9 – Exact Mass */
.curation-results-table th:nth-child(9),
.curation-results-table td:nth-child(9) {
  min-width: 12ch;
}

/* Queue rows table column widths */
.curation-queue-table {
  min-width: max-content;
}

.curation-queue-table th:nth-child(1),
.curation-queue-table td:nth-child(1) {
  width: 110px;
  min-width: 110px;
}

.curation-queue-table th:nth-child(4),
.curation-queue-table td:nth-child(4) {
  min-width: 220px;
  max-width: 320px;
}

.curation-note { font-size: var(--fs-label); color: var(--text); margin-top: 3px; white-space: pre-line; }
.curation-badges { display: flex; flex-wrap: wrap; gap: 6px; }
.curation-row-badges { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px; }

.curation-cell-wrap {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.curation-status {
  font-weight: 700;
  text-transform: uppercase;
  font-size: var(--fs-micro);
  letter-spacing: 0.04em;
  color: var(--text);
}

.curation-status-badge,
.curation-table .curation-status {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 4px;
  border-left: 3px solid transparent;
  background: color-mix(in srgb, var(--surface) 90%, transparent);
}

/* ok → wd-taxon green (complete/existing) */
.curation-status.is-ok      { border-left-color: var(--wd-taxon); }
/* warn → wd-entries neutral (needs updates, not critical) */
.curation-status.is-warn    { border-left-color: var(--wd-entries); }
/* new → wd-reference blue (informational: new item to create) */
.curation-status.is-new     { border-left-color: var(--wd-reference); }
/* pending → wd-reference blue (waiting on dependencies) */
.curation-status.is-pending { border-left-color: var(--wd-reference); }
/* error → wd-compound red (failure) */
.curation-status.is-error   { border-left-color: var(--wd-compound); }


"###;
pub const LOTUS_FOOTER_CSS: &str = r###"/* Footer and download-group pack extracted from style.css. */

.app-footer { margin-top:auto; padding:16px 28px 20px; border-top:1px solid var(--panel-border); background:var(--panel-bg-soft); color:var(--text2); display:flex; flex-direction:column; gap:0; font-size:var(--fs-0); box-shadow:var(--panel-shadow); }
.footer-line { display:grid; grid-template-columns:repeat(auto-fit, minmax(300px, 1fr)); gap:0 24px; align-items:start; padding:10px 0; border-bottom:1px solid var(--panel-border); }
.footer-line:last-child { border-bottom:none; padding-bottom:0; }
.footer-line:first-child { padding-top:0; }
.footer-row { display:grid; grid-template-columns:clamp(7.5rem, 7vw, 9rem) minmax(0, 1fr); gap:4px 6px; align-items:start; padding:2px 0; }
.footer-label { color:var(--text2); font-weight:700; text-transform:uppercase; font-size:var(--fs-0); letter-spacing:1px; min-width:0; white-space:nowrap; padding-top:4px; text-align:left; }
.footer-aside { color:var(--text2); font-size:var(--fs-0); }
.footer-links { list-style:none; display:flex; flex-wrap:wrap; gap:3px 5px; margin:0; padding:0; min-width:0; justify-content:flex-start; align-items:flex-start; }
.footer-links li { display:inline-flex; align-items:center; gap:4px; min-width:0; padding:1px 7px; border-radius:999px; background:var(--panel-bg); border:1px solid var(--panel-border); transition: border-color .12s, box-shadow .12s; }
.footer-links li:hover { border-color:color-mix(in srgb, var(--panel-border) 60%, var(--accent)); box-shadow:var(--shadow-xs); }
.footer-link { color:var(--text); text-decoration:none; }
.footer-link:hover { text-decoration:underline; }
.footer-link.red { color:var(--wd-compound); font-weight:700; }
.footer-link.green { color:var(--wd-taxon); font-weight:700; }
.footer-link.blue { color:var(--wd-reference); font-weight:700; }
.footer-link.muted { color:var(--text2); font-weight:700; }

/* Download buttons — .dl-group base styles live in results_pack.css */

"###;
pub const LOTUS_FORM_CONTROLS_CSS: &str = r###"/* Form controls pack: shared form primitives, structure editor, and search button. */

.form-section { display:flex; flex-direction:column; gap:5px; padding:10px 12px; border:1px solid var(--panel-border); border-radius:12px; background:var(--panel-bg-soft); }
.form-section.nested { padding-left:10px; border-left:1px solid var(--border); margin-top:4px; }
.form-label { font-size:var(--fs-0); font-weight:700; color:var(--critical-text); text-transform:uppercase; letter-spacing:0.08em; }
.form-label.sm { font-size:var(--fs-0); font-weight:700; color:var(--text); text-transform:none; letter-spacing:0; }
.form-hint { font-size:var(--fs-0); color:var(--text2); }
.radio-group { display:flex; gap:14px; }
.radio-label { display:flex; align-items:center; gap:6px; font-size:var(--fs-0); cursor:pointer; color:var(--text2); }
.radio-label input { accent-color:var(--accent); }
.range-input { width:100%; accent-color:var(--accent); margin-top:4px; }
.range-inputs { display:flex; align-items:flex-end; gap:8px; }
.range-pair { display:flex; flex-direction:column; gap:3px; }
.range-sep { color:var(--text3); padding-bottom:8px; }

.formula-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }

.formula-exact-row,
.formula-exact-input { width: 100%; max-width: none; }

.formula-num-pair {
  border: 1px solid var(--panel-border);
  border-radius: 10px;
  background: var(--panel-bg-soft);
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.formula-num-label { color: var(--text2); }
.formula-minmax-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }

.formula-minmax-grid .range-pair,
.formula-grid .range-pair { min-width: 0; }

.formula-minmax-grid .form-input.sm,
.formula-grid .form-input.sm {
  width: 100%;
  min-width: 6ch;
  padding-left: 6px;
  padding-right: 6px;
  font-variant-numeric: tabular-nums;
}

/* Make two-ended range filters responsive without affecting formula rows. */
.range-inputs--pair {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: end;
  gap: 8px;
}

.range-inputs--pair .range-pair { min-width: 0; }
.range-inputs--pair .form-input { width: 100%; }

/* Normalize number input chrome for consistent borders on Safari/Firefox. */
input[type="number"].form-input { appearance: textfield; }

input[type="number"].form-input::-webkit-outer-spin-button,
input[type="number"].form-input::-webkit-inner-spin-button { appearance: none; margin: 0; }

/* Structure section */
.form-textarea.mono, .mono { font-family: var(--mono); }

.kind-pill {
  display:inline-block;
  padding:1px 7px;
  border-radius:999px;
  font-size:var(--fs-micro);
  font-weight:700;
  letter-spacing:1px;
  text-transform:uppercase;
  margin-right:6px;
  color:#fff;
  background:var(--text3);
}

.kind-pill[data-kind="smiles"] { background:var(--accent2, #5b6cff); }
.kind-pill[data-kind="mol2000"] { background:#c97a2b; }
.kind-pill[data-kind="mol3000"] { background:#2b8f57; }
.kind-note { color:var(--text2); }

.ketcher-panel { margin:0; border:1px solid var(--panel-border); border-radius:var(--radius); background:var(--panel-bg-soft); box-shadow:var(--panel-shadow); transition: background .15s ease, border-color .15s ease, box-shadow .15s ease; }
.ketcher-wrap { padding:0 14px 14px; display:flex; flex-direction:column; gap:10px; }
.ketcher-iframe { width:100%; height:min(78vh, 820px); min-height:600px; border:1px solid var(--border); border-radius:var(--radius-sm); background:#fff; }
.ketcher-hint { margin-top:2px; font-size:var(--fs-0); color:var(--text2); }

.search-btn { display:flex; align-items:center; justify-content:center; gap:8px; background:var(--btn-primary-bg); color:#fff; border:0; border-radius:var(--radius-sm); padding:11px 16px; font-size:var(--fs-ui); font-weight:700; cursor:pointer; box-shadow:var(--shadow-xs); transition:background .15s, box-shadow .15s, transform .12s ease; text-align:center; line-height:1.2; white-space:normal; }
.search-btn:hover:not(:disabled) { background:var(--btn-primary-hover-bg); box-shadow:var(--shadow-sm); }
.search-btn:active { transform: translateY(1px); }
.search-btn:disabled { opacity:.5; cursor:not-allowed; }

.btn:focus-visible,
.search-btn:focus-visible,
.sort-btn:focus-visible,
.notice-dismiss:focus-visible,
.primary-link:focus-visible,
.id-badge:focus-visible,
.filters-toggle:focus-visible,
.copy-btn:focus-visible {
  outline: 3px solid var(--accent2);
  outline-offset: 2px;
}

@media (width <= 760px) {
  .formula-grid,
  .formula-minmax-grid {
    grid-template-columns: 1fr;
  }
}

"###;
pub const LOTUS_LAYOUT_SHELL_CSS: &str = r###"/* Layout shell pack: app frame, header/meta, notices, share bar, and sidebar shell. */

.app-layout { display:flex; min-height:100dvh; height:100dvh; overflow:hidden; gap:10px; padding:10px; }
.app-layout.no-sidebar { display:block; }

.sidebar {
  width:332px;
  min-width:288px;
  height:100%;
  overflow-y:auto;
  background:var(--panel-bg);
  border:1px solid var(--panel-border);
  border-radius:16px;
  flex-shrink:0;
  box-shadow:var(--shadow-sm);
  display:flex;
  flex-direction:column;
  position: relative;
  isolation: isolate;
}

.main-content {
  flex:1;
  min-width:0;
  height:100%;
  overflow-y:auto;
  display:flex;
  flex-direction:column;
  border:1px solid var(--panel-border);
  border-radius:16px;
  background:var(--panel-bg);
  box-shadow:var(--shadow-sm);
}

.main-content.single-pane { width:100%; }

/* Perceived perf: skip off-screen paint work */
.welcome, .results-wrap, .query-panel, .ketcher-panel, .table-scroll {
  content-visibility: auto;
  contain-intrinsic-size: 900px;
}

.page-header {
  padding:14px 24px 10px;
  border-bottom:1px solid var(--panel-border);
  background:color-mix(in srgb, var(--panel-bg-soft) 92%, var(--surface));
  box-shadow:var(--shadow-xs);
  position: sticky;
  top: 0;
  z-index: 3;
  overflow: clip;
}

.page-header-meta {
  margin: 10px 24px 0;
  padding: 7px 12px;
  display: flex;
  flex-flow: row wrap;
  align-items: center;
  gap: 4px 20px;
  border: 1px solid var(--panel-border);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel-bg-soft) 92%, var(--surface));
  box-shadow: var(--panel-shadow);
  transition: background .15s ease, border-color .15s ease, box-shadow .15s ease;
}

.page-brand { display:flex; align-items:center; gap:12px; }
.sidebar-logo-link { display: inline-flex; align-items: center; justify-content: center; border-radius: 999px; text-decoration: none; }
.page-home-link { display: inline-flex; align-items: center; gap: 0; min-width: 0; }
.page-title-text { min-width: 0; overflow-wrap: anywhere; }
.sidebar-logo-link { border-radius: 14px; }
.page-title { font-size:var(--fs-4); font-weight:800; letter-spacing:-.028em; line-height:1.06; color:var(--text); }

.page-title-link,
.page-title-link:visited { color: inherit; text-decoration: none; }
.page-title-link:hover { text-decoration: none; }

.lang-switch { margin-left:auto; display:flex; gap:4px; align-items:center; }
.lang-btn { min-width:40px; padding:3px 8px; }
.lang-btn.active { background:var(--btn-primary-bg); color:#fff; border-color:var(--btn-primary-bg); }

.view-switch .lang-btn,
.lang-switch .lang-btn {
  color: var(--text2);
  background: color-mix(in srgb, var(--panel-bg-soft) 84%, var(--surface));
  border-color: var(--panel-border);
  box-shadow: none;
}

.view-switch .lang-btn:hover:not(:disabled),
.lang-switch .lang-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--surface2) 52%, var(--surface));
  border-color: color-mix(in srgb, var(--border) 86%, var(--accent));
  box-shadow: none;
}

.view-switch .lang-btn.active,
.lang-switch .lang-btn.active {
  color: #fff;
  background: var(--btn-primary-bg);
  border-color: var(--btn-primary-bg);
  box-shadow: var(--shadow-xs);
}

.view-switch .lang-btn.active:hover:not(:disabled),
.lang-switch .lang-btn.active:hover:not(:disabled) {
  background: var(--btn-primary-hover-bg);
  border-color: var(--btn-primary-hover-bg);
}

.page-sub { font-size:var(--fs-1); color:var(--critical-muted); margin-top:4px; }
.page-archive-note { display: inline; margin-left: 0.4em; }

.page-archive-label {
  font-size: inherit;
  text-transform: none;
  letter-spacing: normal;
  color: var(--text2);
  font-weight: 500;
  margin-right: 0.25em;
}
.page-archive-link { color: var(--accent); font-weight: 500; }

.page-meta { display: contents; }
.meta-item { display:inline-flex; align-items:center; gap:4px; white-space: normal; overflow-wrap: anywhere; }
.meta-key { text-transform:uppercase; letter-spacing:0.08em; font-weight:700; font-size: var(--fs-0); color: var(--text2); }
.meta-val.mono { font-family:var(--mono); color:var(--critical-text); font-variant-numeric: tabular-nums; }
.meta-sep { color:var(--text3); }

/* Notices */
.notice {
  margin:10px 24px 0;
  padding:9px 12px;
  display:flex;
  align-items:center;
  gap:12px;
  border-radius:var(--radius);
  font-size:var(--fs-0);
  border:1px solid var(--panel-border);
  background:var(--panel-bg-soft);
  box-shadow:var(--panel-shadow);
  transition: background .15s ease, border-color .15s ease, box-shadow .15s ease;
}

/* Notices that are direct children of the results pane span full width. */
.main-content > .notice {
  margin-left: 0;
  margin-right: 0;
  padding-left: 24px;
  padding-right: 24px;
  border-radius: 0;
  border-left: 0;
  border-right: 0;
}

.results-wrap > .notice { margin: 0; }
.notice:hover { box-shadow: var(--shadow-sm); }

.notice-label {
  display:inline-flex;
  align-items:center;
  text-transform:uppercase;
  letter-spacing:1px;
  font-size:var(--fs-label);
  font-weight:700;
  line-height:1.4;
  padding:2px 6px;
  border-radius:3px;
  flex-shrink:0;
}

.notice-value { flex:1; color:var(--text); word-break:break-word; line-height:1.4; }

.notice-copy-field {
  min-width: min(220px, 100%);
  max-width: 100%;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text2);
  padding: 4px 8px;
}

.notice-info { border-color:color-mix(in srgb, var(--accent) 34%, var(--results-border)); background:color-mix(in srgb, var(--accent) 9%, var(--panel-bg-soft)); }
.notice-info .notice-label { background:color-mix(in srgb, var(--accent) 16%, var(--surface)); color:color-mix(in srgb, var(--accent2) 86%, var(--text)); }
.notice-warn { border-color:color-mix(in srgb, var(--yellow) 34%, var(--results-border)); background:color-mix(in srgb, var(--yellow) 8%, var(--panel-bg-soft)); }
.notice-warn .notice-label { background:color-mix(in srgb, var(--yellow) 16%, var(--surface)); color:color-mix(in srgb, var(--yellow) 88%, var(--text)); }
.notice-error { border-color:color-mix(in srgb, var(--red) 34%, var(--results-border)); background:color-mix(in srgb, var(--red) 8%, var(--panel-bg-soft)); }
.notice-error .notice-label { background:color-mix(in srgb, var(--red) 16%, var(--surface)); color:color-mix(in srgb, var(--red) 88%, var(--text)); }
.notice-dismiss { margin-left:auto; background:none; border:0; color:inherit; cursor:pointer; font-size:18px; line-height:1; padding:0 4px; opacity:.7; }
.notice-dismiss:hover { opacity:1; }

/* Share bar */
.share-bar {
  display: flex;
  flex-flow: row wrap;
  align-items: center;
  gap: 6px 10px;
  margin: 10px 24px 0;
  padding: 7px 12px;
  border: 1px solid var(--panel-border);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel-bg-soft) 92%, var(--surface));
  box-shadow: var(--panel-shadow);
  font-size: var(--fs-0);
  transition: background .15s ease, border-color .15s ease, box-shadow .15s ease;
}

.curation-wrap .share-bar { margin: 0; }

.share-bar-label {
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-weight: 700;
  font-size: var(--fs-0);
  color: var(--text2);
  flex-shrink: 0;
  white-space: nowrap;
}

.share-bar-input {
  flex: 1;
  min-width: min(200px, 100%);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  padding: 4px 8px;
  font-size: var(--fs-0);
}

.share-bar-input:focus {
  outline: none;
  border-color: var(--accent);
}

/* Search panel shell */
.search-panel { padding:18px 16px; display:flex; flex-direction:column; gap:14px; background:var(--panel-bg); flex:1; }
.search-panel-body { display:flex; flex-direction:column; gap:12px; }
.filters-toggle { display:none; }
.sidebar-logo-wrap { margin-top:auto; padding:6px 8px 8px; display:flex; justify-content:center; border-top:1px solid var(--border); }
.view-switch { margin-top: 10px; display: flex; gap: 8px; }
.view-switch .btn { font-weight: 700; }
.sidebar-logo { display:block; width:128px; height:128px; }

"###;
pub const LOTUS_QUERY_PANEL_CSS: &str = r###"/* Query panel pack: extracted from style.css for maintainability. */

.query-panel {
  background: var(--panel-bg-soft);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius);
  box-shadow: var(--panel-shadow);
  transition: background .15s ease, border-color .15s ease, box-shadow .15s ease;
}

.query-panel > summary {
  cursor: pointer;
  padding: 8px 14px;
  font-size: var(--fs-0);
  color: var(--text2);
  user-select: none;
  letter-spacing: 0.04em;
  font-weight: 600;
  list-style: none;
}

.query-panel > summary::-webkit-details-marker { display: none; }
.query-panel > summary::before { content: "▸ "; color: var(--text3); }
.query-panel[open] > summary::before { content: "▾ "; }
.query-panel > summary:hover { color: var(--text); }

/* Wrapper so the copy button can be aligned next to the header. */
.query-body {
  position: relative;
  border-radius: 0 0 var(--radius) var(--radius);
  overflow: hidden;
}

/* Header with label and copy button aligned horizontally. */
.query-header {
  display: flex;
  align-items: center;
  position: relative;
  padding: 8px 14px;
  background: var(--panel-bg-soft);
  border-bottom: 1px solid var(--panel-border);
}

.query-label {
  font-size: var(--fs-0);
  color: var(--text2);
  user-select: none;
  letter-spacing: 0.04em;
  font-weight: 600;
}

/* Copy button in the header aligned to the right. */
.query-copy-btn {
  position: absolute;
  right: 14px;
  opacity: 1;
}

/* Distinct code surface with high contrast in light and dark mode. */
.query-text {
  padding: 12px 16px;
  margin: 0;
  font-family: var(--mono);
  font-size: var(--fs-0);
  color: var(--text);
  background: var(--bg2);
  border-left: 3px solid var(--wd-entries);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 320px;
  overflow: auto;
}

"###;
pub const LOTUS_RESULTS_CSS: &str = r###"/* Results viewport pack extracted from style.css. */

.results-wrap {
  padding: 12px 22px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 100%;
  width: 100%;
}

.results-toolbar {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
  border: 1px solid var(--results-border);
  border-radius: 12px;
  padding: 10px 12px;
  background: var(--panel-bg-soft);
  box-shadow: var(--panel-shadow);
  width: 100%;
  min-width: 0;
}

.results-toolbar > .stat-bar,
.results-toolbar > .toolbar-actions {
  width: 100%;
  min-width: 0;
}

/* Stats */
.stat-bar {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
  align-items: stretch;
  width: 100%;
  min-width: 0;
}

.toolbar-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
  min-width: 0;
}

.toolbar-actions > .dl-group {
  flex: 1 1 520px;
  min-width: 0;
  max-width: 100%;
}

.toolbar-actions > .btn {
  flex: 0 1 auto;
  min-width: 0;
  max-width: 100%;
}

.toolbar-actions .btn {
  white-space: normal;
}

.dl-group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  min-width: 0;
  max-width: 100%;
  align-items: center;
}

.dl-group .btn {
  border-radius: var(--radius-sm);
  border-right-width: 1px;
  min-width: 0;
}

/* Within the results toolbar the dl-group buttons should stretch to fill available space. */
.toolbar-actions .dl-group .btn {
  flex: 1 1 160px;
}

.table-scroll {
  overflow: auto;
  max-height: min(72vh, 980px);
  border: 1px solid var(--results-border);
  border-radius: 14px;
  background: var(--panel-bg-soft);
  box-shadow: var(--panel-shadow);
  transition: background .15s ease, border-color .15s ease, box-shadow .15s ease;
}

.table-scroll:focus-visible {
  outline: none;
  border-color: color-mix(in srgb, var(--accent) 44%, var(--results-border));
  box-shadow: var(--ring);
}

.results-table {
  width: 100%;
  min-width: max-content;
  border-collapse: collapse;
  font-size: var(--fs-ui);
  table-layout: auto;
  word-break: break-word;
}

.col-structure { width: 124px; }
.col-compound { width: 38ch; }
.col-mass { width: 10ch; }
.col-formula { width: 14ch; }
.col-taxon { width: 28ch; }
.col-reference { width: 38ch; }
.col-year { width: 8ch; }
.results-table thead { position: sticky; top: 0; z-index: 2; background: var(--bg2); }
.virtual-spacer-row { border: 0 !important; }
.virtual-spacer-cell { padding: 0 !important; border: 0 !important; }

.sort-th,
.th-static {
  padding: 9px 10px;
  text-align: left;
  font-size: var(--fs-label);
  font-weight: 700;
  color: var(--critical-muted);
  border-bottom: 1px solid var(--results-border);
  white-space: nowrap;
  user-select: none;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  width: auto;
  min-width: max-content;
}

.th-static { color: var(--critical-muted); }

.th-label {
  font: inherit;
  text-transform: inherit;
  letter-spacing: inherit;
  display: block;
  min-width: max-content;
  white-space: nowrap;
  overflow: visible;
  text-overflow: clip;
  line-height: 1.2;
}

.th-static .th-label {
  text-transform: none;
  letter-spacing: normal;
}

.sort-th { cursor: pointer; }
.sort-th:hover { color: var(--text); }

.sort-btn {
  appearance: none;
  background: none;
  border: 0;
  color: inherit;
  font: inherit;
  padding: 0;
  margin: 0;
  cursor: pointer;
  display: grid;
  align-items: start;
  grid-template-columns: auto auto;
  column-gap: 6px;
  width: 100%;
  min-width: max-content;
}

.sort-icon { color: var(--text3); font-size: var(--fs-0); font-weight: 700; line-height: 1; margin-left: 0; margin-top: 0; }
.data-row { border-bottom: 1px solid var(--results-border); transition: background .14s ease; contain: layout paint; }
.data-row:hover { background: color-mix(in srgb, var(--surface2) 84%, var(--bg2)); }
.results-table tbody tr:nth-child(odd) td { background: color-mix(in srgb, var(--surface) 94%, transparent); }
.results-table tbody tr:nth-child(even) td { background: color-mix(in srgb, var(--surface) 88%, transparent); }

.data-row td {
  padding: 8px 10px;
  vertical-align: top;
  contain: layout paint;
  word-break: break-word;
}

.results-table td {
  min-width: 0;
}

.stat-badge {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  padding: 10px 12px;
  border-radius: 12px;
  border: 1px solid var(--results-border);
  background: var(--surface);
  box-shadow: var(--shadow-xs);
  position: relative;
  overflow: hidden;
  flex: 1 1 0;
}

.stat-badge::before {
  content: "";
  position: absolute;
  left: 0;
  top: 0;
  width: 3px;
  height: 100%;
  background: var(--wd-reference);
}

.stat-value-row { display: flex; flex-wrap: wrap; align-items: baseline; gap: 8px; min-width: 0; width: 100%; justify-content: center; }
.stat-value { font-size: var(--fs-stat); font-weight: 800; color: var(--text); font-variant-numeric: tabular-nums; letter-spacing: -0.02em; min-width: 0; flex: 0 1 auto; line-height: 1.2; }

.stat-secondary-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px;
  min-width: 0;
  max-width: 100%;
  width: 100%;
  justify-content: center;
}

.stat-secondary-label {
  font-size: var(--fs-micro);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text);
  background: color-mix(in srgb, var(--surface2) 62%, var(--surface));
  border: 1px solid var(--results-border);
  border-radius: 999px;
  padding: 1px 6px;
  white-space: nowrap;
  overflow-wrap: anywhere;
  flex: 0 0 auto;
}

.stat-value-secondary {
  font-size: var(--fs-0);
  font-weight: 700;
  color: var(--text);
  font-variant-numeric: tabular-nums;
  min-width: 0;
  max-width: 100%;
  overflow-wrap: anywhere;
  flex: 0 0 auto;
}

.stat-label {
  font-size: var(--fs-0);
  color: var(--text2);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  min-width: 0;
  overflow-wrap: anywhere;
  font-weight: 700;
  width: 100%;
  text-align: center;
}

.stat-badge:nth-child(1),
.stat-badge:nth-child(2),
.stat-badge:nth-child(3),
.stat-badge:nth-child(4) {
  background: var(--surface);
  border-color: var(--results-border);
}

.stat-badge:nth-child(1)::before { background: var(--wd-compound-stripe); }
.stat-badge:nth-child(2)::before { background: var(--wd-taxon-stripe); }
.stat-badge:nth-child(3)::before { background: var(--wd-reference-stripe); }
.stat-badge:nth-child(4)::before { background: var(--wd-entries-stripe); }

.stat-badge:nth-child(1) .stat-value,
.stat-badge:nth-child(2) .stat-value,
.stat-badge:nth-child(3) .stat-value,
.stat-badge:nth-child(4) .stat-value {
  color: var(--text);
}

"###;
pub const LOTUS_RESPONSIVE_CSS: &str = r###"/* Responsive breakpoint pack extracted from style.css for maintainability. */

/* Import enhanced responsive typography tokens */
:root {
  /* Enhanced responsive font sizing for better mobile UX */

  /* Tighter scaling for tablets (768px and below) */
  --fs-title-tablet: clamp(1.2rem, 0.85rem + 1.05vw, 1.5rem);
  --fs-heading-tablet: clamp(1rem, 0.8rem + 0.6vw, 1.25rem);
  --fs-body-tablet: clamp(0.85rem, 0.8rem + 0.15vw, 0.9rem);
  --fs-small-tablet: clamp(0.7rem, 0.68rem + 0.1vw, 0.8rem);
}

@media (width <=768px) {
  :root {
    /* Tablet-specific responsive sizes */
    --fs-0:      clamp(0.7rem, 0.68rem + 0.12vw, 0.8rem);
    --fs-1:      clamp(0.8rem, 0.78rem + 0.12vw, 0.85rem);
    --fs-2:      clamp(0.85rem, 0.83rem + 0.18vw, 0.95rem);
    --fs-3:      clamp(1rem, 0.92rem + 0.4vw, 1.25rem);
    --fs-4:      clamp(1.2rem, 0.95rem + 0.65vw, 1.5rem);
    --fs-body:   clamp(0.8rem, 0.78rem + 0.12vw, 0.85rem);
    --fs-label:  clamp(0.65rem, 0.63rem + 0.1vw, 0.7rem);
    --fs-micro:  clamp(0.7rem, 0.68rem + 0.08vw, 0.75rem);
    --fs-ui:     clamp(0.75rem, 0.73rem + 0.1vw, 0.8rem);
    --fs-stat:   clamp(1rem, 0.92rem + 0.4vw, 1.2rem);
  }

  .app-layout   { flex-direction:column; height:auto; min-height:100dvh; overflow:visible; padding:0; gap:0; }
  .sidebar      { width:100%; height:auto; max-height:none; overflow-y:visible; border-radius:0; border-left:0; border-right:0; }
  .main-content { height:auto; min-height:0; overflow-y:visible; border-radius:0; border-left:0; border-right:0; }

  .page-header, .welcome, .results-wrap, .app-footer {
    padding-left:max(18px, env(safe-area-inset-left));
    padding-right:max(18px, env(safe-area-inset-right));
  }
  .page-header-meta { margin-left:18px; margin-right:18px; }
  .notice       { margin-left:18px; margin-right:18px; }
  .draw-wrap { padding-left:18px; padding-right:18px; }
  .ketcher-panel { margin-left:0; margin-right:0; }
  .ketcher-iframe { height:min(70vh, 560px); min-height:420px; }
  .app-footer { gap:0; }
  .app-footer { padding-bottom: max(16px, env(safe-area-inset-bottom)); }
  .footer-row { row-gap:4px; }

  .page-brand {
    flex-wrap: wrap;
    align-items: flex-start;
    gap: 8px 10px;
  }

  .page-title {
    min-width: 0;
    flex: 1 1 260px;
    font-size: var(--fs-4);
  }

  .page-home-link {
    max-width: 100%;
    gap: 8px;
  }

  .lang-switch {
    margin-left: 0;
    width: 100%;
    justify-content: flex-start;
    flex-wrap: wrap;
    font-size: var(--fs-0);
  }
  .stat-bar { grid-template-columns:repeat(2, minmax(130px, 1fr)); }
  .view-switch { flex-wrap:wrap; }
  .view-switch .btn { flex:1 1 180px; justify-content:center; font-size: var(--fs-0); }

  /* share-bar: reduce margin to match page-header-meta at this breakpoint */
  .share-bar { margin-left: 18px; margin-right: 18px; }
  .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }

  /* Typography scaling for tablet */
  .page-sub { font-size: var(--fs-1); }
  .meta-key { font-size: var(--fs-label); }
  .form-label { font-size: var(--fs-0); }
  .form-hint { font-size: var(--fs-micro); }
  .radio-label { font-size: var(--fs-0); }
  .search-btn { font-size: var(--fs-ui); }
  .btn, .btn-sm { font-size: var(--fs-0); }
  .stat-badge { font-size: var(--fs-stat); }
}

@media (width <=480px) {
  :root {
    /* Phone-specific responsive sizing */
    --fs-0:      clamp(0.65rem, 0.63rem + 0.08vw, 0.75rem);
    --fs-1:      clamp(0.75rem, 0.73rem + 0.08vw, 0.8rem);
    --fs-2:      clamp(0.8rem, 0.78rem + 0.12vw, 0.9rem);
    --fs-3:      clamp(0.95rem, 0.87rem + 0.3vw, 1.15rem);
    --fs-4:      clamp(1.1rem, 0.85rem + 0.6vw, 1.35rem);
    --fs-body:   clamp(0.75rem, 0.73rem + 0.08vw, 0.8rem);
    --fs-label:  clamp(0.6rem, 0.58rem + 0.08vw, 0.65rem);
    --fs-micro:  clamp(0.65rem, 0.63rem + 0.06vw, 0.7rem);
    --fs-ui:     clamp(0.7rem, 0.68rem + 0.08vw, 0.75rem);
    --fs-stat:   clamp(0.95rem, 0.87rem + 0.3vw, 1.1rem);
  }

  .sidebar { padding:0; }
  .search-panel { padding:14px 12px; gap:12px; font-size: var(--fs-0); }
  .form-section { padding:8px 10px; border-radius:10px; font-size: var(--fs-body); }

  .page-header, .welcome, .results-wrap, .app-footer {
    padding-left:12px;
    padding-right:12px;
    font-size: var(--fs-body);
  }

  .page-header-meta {
    margin-left:12px;
    margin-right:12px;
    font-size: var(--fs-label);
  }
  .draw-wrap { padding-left:12px; padding-right:12px; }
  .notice, .ketcher-panel { margin-left:12px; margin-right:12px; }
  .main-content > .notice { padding-left:12px; padding-right:12px; }
  .notice { padding:8px 10px; gap:8px; flex-direction:column; align-items:flex-start; font-size: var(--fs-label); }
  .notice-copy-field { width:100%; min-width:0; }
  .notice-dismiss { align-self:flex-end; margin-left:0; font-size: var(--fs-0); }

  /* share-bar: match notice margin and stack input */
  .share-bar { margin-left: 12px; margin-right: 12px; }
  .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }
  .share-bar-input { width: 100%; min-width: 0; font-size: 16px; }

  .filters-toggle {
    display:flex;
    width:calc(100% - 24px);
    margin:12px;
    padding:10px 12px;
    justify-content:center;
    align-items:center;
    border:1px solid var(--border);
    border-radius:var(--radius-sm);
    background:var(--bg2);
    color:var(--text);
    font-size:var(--fs-ui);
    font-weight:600;
    min-height: 44px;
    cursor:pointer;
  }
  .sidebar.mobile-closed .search-panel .search-panel-body { display:none; }
  .sidebar.mobile-open .search-panel .search-panel-body { display:block; }

  .page-title {
    font-size: var(--fs-4);
    line-height: 1.1;
  }
  .page-title-text { line-height:1.1; }
  .page-sub { font-size: var(--fs-1); }
  .sidebar-logo-wrap { padding-top:10px; padding-bottom:12px; }
  .sidebar-logo { width:120px; height:120px; }
  .radio-group, .range-inputs, .toolbar-actions { flex-wrap:wrap; }

  .toolbar-actions > .btn,
  .toolbar-actions > .dl-group {
    width:100%;
    font-size: var(--fs-0);
  }
  .footer-line { grid-template-columns:1fr; }

  .footer-row {
    grid-template-columns:max-content minmax(0,1fr);
    align-items:flex-start;
    font-size: var(--fs-micro);
  }
  .footer-label { min-width:0; }
  .footer-links { gap:4px 6px; }
  .footer-links li { width:auto; }

  .footer-link, .footer-aside, .footer-sep {
    line-height:1.35;
    font-size: var(--fs-micro);
  }
  .range-pair { min-width:120px; }

  .range-inputs--pair {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .range-sep--pair {
    display: none;
  }

  .form-input, .form-textarea, .search-btn, select, input, textarea { font-size:16px; }

  .btn, .search-btn {
    min-height: 44px;
    font-size: var(--fs-0);
  }
  .search-btn { justify-content:center; text-align:center; }

  .results-wrap {
    gap: 8px;
    padding: 10px 12px;
  }
  .curation-scroll-hint { display: inline-flex; }
  .curation-table-scroll { border-radius: 8px; }
  .curation-results-table { min-width: 900px; }

  .curation-results-table th,
  .curation-results-table td {
    padding: 4px 5px;
    font-size: var(--fs-micro);
  }

  .stat-bar {
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
  }

  .stat-badge {
    padding: 6px 8px;
    gap: 2px;
    font-size: var(--fs-stat);
  }

  .results-table {
    font-size: var(--fs-label);
    table-layout: auto;
    word-break: break-word;
  }

  .sort-th, .th-static {
    padding: 5px 4px;
    font-size: var(--fs-micro);
    letter-spacing: 0.05em;
    white-space: normal;
  }

  .data-row td {
    padding: 4px 5px;
    font-size: var(--fs-0);
    vertical-align: middle;
    word-break: break-word;
  }

  /* Preserve full names on phones - use auto layout */
  .td-depict {
    width: auto;
    padding: 3px 4px !important;
    min-width: 0;
    flex-shrink: 0;
  }

  .depict-img {
    width: min(100%, 65px);
    max-width: 65px;
    height: auto;
  }

  /* Allow compound/taxon/ref cells to expand for full names */
  .td-compound, .td-taxon, .td-ref {
    min-width: 120px;
    width: auto;
    border-radius: 6px;
    padding: 4px 5px;
  }

  .cell-primary {
    font-weight: 500;
    line-height: 1.4;
  }

  .id-badge {
    font-size: var(--fs-micro);
    padding: 1px 3px;
    border-radius: 2px;
  }

  .badge-row {
    gap: 2px;
    margin-top: 2px;
  }

  /* Reduce table scroll max-height on phones */
  .table-scroll {
    max-height: min(60vh, 500px);
    overflow-x: auto;
  }

  /* Optimize stat value display */
  .stat-value-row {
    gap: 3px;
  }

  .stat-value {
    font-size: var(--fs-stat);
    line-height: 1.1;
  }

  .stat-label {
    font-size: var(--fs-micro);
    margin-top: 1px;
  }

  .stat-secondary-label {
    font-size: var(--fs-micro);
    padding: 0 3px;
  }

   .td-depict {
     width: auto;
     padding:4px 6px !important;
     min-width: 0;
   }

   .depict-img {
     width:88px;
     height:56px;
   }
   .td-compound, .td-taxon, .td-ref { min-width:0; width:auto; }

   .id-badge {
     font-size: var(--fs-micro);
     padding:1px 5px;
   }

   .ketcher-iframe { height:min(62vh, 420px); min-height:300px; }
}

@media (width <=430px) {
  :root {
    /* Extra small screen (< 430px) optimized sizing */
    --fs-0:      clamp(0.62rem, 0.60rem + 0.06vw, 0.7rem);
    --fs-1:      clamp(0.7rem, 0.68rem + 0.06vw, 0.75rem);
    --fs-2:      clamp(0.75rem, 0.73rem + 0.1vw, 0.85rem);
    --fs-3:      clamp(0.9rem, 0.82rem + 0.25vw, 1.05rem);
    --fs-4:      clamp(1rem, 0.78rem + 0.55vw, 1.2rem);
    --fs-body:   clamp(0.7rem, 0.68rem + 0.06vw, 0.75rem);
    --fs-label:  clamp(0.55rem, 0.53rem + 0.06vw, 0.6rem);
    --fs-micro:  clamp(0.6rem, 0.58rem + 0.04vw, 0.65rem);
    --fs-ui:     clamp(0.65rem, 0.63rem + 0.06vw, 0.7rem);
    --fs-stat:   clamp(0.9rem, 0.82rem + 0.25vw, 1rem);
  }

  .page-header,
  .welcome,
  .results-wrap,
  .app-footer,
  .draw-wrap {
    padding-left:10px;
    padding-right:10px;
  }

  .notice,
  .ketcher-panel,
  .page-header-meta {
    margin-left:10px;
    margin-right:10px;
  }
  .share-bar { margin-left: 10px; margin-right: 10px; }
  .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }

  /* Stack meta items vertically on very narrow screens */
  .page-header-meta {
    flex-direction: column;
    gap: 4px;
    align-items: flex-start;
  }

  .page-header-meta .meta-item {
    white-space: normal;
    flex-wrap: wrap;
    min-width: 0;
    font-size: var(--fs-label);
  }

  .page-header-meta .meta-key {
    font-size: var(--fs-micro);
  }

  .page-header-meta .meta-sep {
    display: none;
  }

  .page-header-meta .meta-val {
    min-width: 0;
    max-width: 100%;
    overflow-wrap: anywhere;
    font-size: var(--fs-label);
  }

  .copy-btn {
    margin-left:0;
    font-size: var(--fs-micro);
  }

  .results-toolbar {
    gap:8px;
  }

  .toolbar-actions {
    width:100%;
    gap:6px;
  }

  .toolbar-actions .btn,
  .toolbar-actions .dl-group {
    width:100%;
    font-size: var(--fs-0);
  }

  .dl-group {
    display:flex;
    flex-wrap:wrap;
    gap:6px;
  }

  .dl-group .btn {
    flex:1 1 160px;
    min-width:0;
    border-right-width:1px;
    border-radius:var(--radius-sm);
    font-size: var(--fs-0);
  }

  .results-table {
    font-size: var(--fs-label);
  }

  .curation-results-table {
    min-width: 900px;
    font-size: var(--fs-label);
  }

  .th-static,
  .sort-th,
  .sort-btn {
    font-size: var(--fs-micro);
    letter-spacing:0.06em;
  }

  .view-switch .btn {
    width:100%;
    flex:1 1 100%;
    font-size: var(--fs-0);
  }
}

@media (width <=360px) {
  :root {
    /* Ultra-small screens (< 360px) - extreme minimum sizing */
    --fs-0:      clamp(0.6rem, 0.58rem + 0.04vw, 0.68rem);
    --fs-1:      clamp(0.68rem, 0.66rem + 0.04vw, 0.72rem);
    --fs-2:      clamp(0.72rem, 0.70rem + 0.08vw, 0.8rem);
    --fs-3:      clamp(0.85rem, 0.77rem + 0.2vw, 1rem);
    --fs-4:      clamp(0.95rem, 0.73rem + 0.5vw, 1.1rem);
    --fs-body:   clamp(0.68rem, 0.66rem + 0.04vw, 0.72rem);
    --fs-label:  clamp(0.52rem, 0.50rem + 0.04vw, 0.58rem);
    --fs-micro:  clamp(0.58rem, 0.56rem + 0.02vw, 0.62rem);
    --fs-ui:     clamp(0.62rem, 0.60rem + 0.04vw, 0.68rem);
    --fs-stat:   clamp(0.85rem, 0.77rem + 0.2vw, 0.95rem);
  }

  .page-header,
  .welcome,
  .results-wrap,
  .app-footer,
  .draw-wrap {
    padding-left:8px;
    padding-right:8px;
  }

  .page-header-meta,
  .notice,
  .ketcher-panel {
    margin-left:8px;
    margin-right:8px;
  }
  .share-bar { margin-left: 8px; margin-right: 8px; }
  .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }

  .main-content > .notice {
    padding-left:8px;
    padding-right:8px;
  }

  .filters-toggle {
    width:calc(100% - 16px);
    margin:8px;
  }

  .lang-switch .btn,
  .view-switch .btn {
    flex:1 1 100%;
    font-size: var(--fs-0);
  }

   .page-title {
     font-size: var(--fs-4);
   }

   /* Optimize typography further for ultra-small screens */
   .page-sub { font-size: var(--fs-1); }
   .meta-key { font-size: var(--fs-micro); }
   .form-label { font-size: var(--fs-0); }
   .form-hint { font-size: var(--fs-micro); }
   .radio-label { font-size: var(--fs-0); }
   .search-btn { font-size: var(--fs-0); }
   .btn { font-size: var(--fs-0); }
   .stat-badge { font-size: var(--fs-stat); }
   .footer-link, .footer-aside { font-size: var(--fs-micro); }

    /* Table optimization for ultra-small screens - PRESERVE FULL NAMES */
    .results-table {
      font-size: var(--fs-label);
      table-layout: auto;
      word-break: break-word;
    }

    .sort-th, .th-static {
      padding: 4px 3px;
      font-size: var(--fs-micro);
      letter-spacing: 0.04em;
      white-space: normal;
    }

    .data-row td {
      padding: 3px 4px;
      font-size: var(--fs-0);
      vertical-align: middle;
      word-break: break-word;
    }

    .td-depict {
      width: auto;
      padding: 2px 3px !important;
      min-width: 0;
      flex-shrink: 0;
    }

    .depict-img {
      width: min(100%, 55px);
      max-width: 55px;
      height: auto;
    }

    /* Allow compound/taxon/ref cells space for full names */
    .td-compound, .td-taxon, .td-ref {
      min-width: 100px;
      width: auto;
      border-radius: 4px;
      padding: 3px 4px;
    }

    .cell-primary {
      font-weight: 500;
      line-height: 1.3;
      white-space: normal;
      overflow-wrap: break-word;
    }

    .id-badge {
      font-size: var(--fs-micro);
      padding: 0 2px;
      border-radius: 2px;
      line-height: 1.2;
    }


   .badge-row {
     gap: 1px;
     margin-top: 0;
   }

   .table-scroll {
     max-height: min(55vh, 400px);
   }

   .stat-value {
     font-size: var(--fs-stat);
     line-height: 1.1;
   }

   .stat-label {
     font-size: var(--fs-micro);
     margin-top: 0;
   }

   /* Hide non-essential columns on ultra-small screens (optional) */
   .results-table tbody tr:nth-child(odd) td {
     background: color-mix(in srgb, var(--surface) 92%, transparent);
   }

   .results-table tbody tr:nth-child(even) td {
     background: color-mix(in srgb, var(--surface) 86%, transparent);
   }
}

/* Medium screens (768px - 1023px) - tablet optimization */
@media (width >= 769px) and (width <= 1023px) {
  :root {
    /* Tablet-optimized responsive typography */
    --fs-0:      clamp(0.72rem, 0.68rem + 0.15vw, 0.8rem);
    --fs-1:      clamp(0.87rem, 0.82rem + 0.18vw, 0.92rem);
    --fs-2:      clamp(0.94rem, 0.88rem + 0.24vw, 1.02rem);
    --fs-3:      clamp(1.125rem, 1.02rem + 0.46vw, 1.35rem);
    --fs-4:      clamp(1.375rem, 1.1rem + 0.7vw, 1.7rem);
    --fs-body:   clamp(0.87rem, 0.82rem + 0.18vw, 0.92rem);
    --fs-label:  clamp(0.68rem, 0.64rem + 0.12vw, 0.75rem);
    --fs-micro:  clamp(0.75rem, 0.71rem + 0.1vw, 0.8rem);
    --fs-ui:     clamp(0.8125rem, 0.76rem + 0.14vw, 0.87rem);
    --fs-stat:   clamp(1.125rem, 1.02rem + 0.4vw, 1.3rem);
  }

  /* Table optimization for tablets - PRESERVE FULL NAMES */
  .results-table {
    font-size: var(--fs-ui);
    table-layout: auto;
    word-break: break-word;
  }

  .sort-th, .th-static {
    padding: 8px;
    font-size: var(--fs-label);
    white-space: normal;
  }

  .data-row td {
    padding: 6px 8px;
    vertical-align: middle;
    word-break: break-word;
  }

  .td-depict {
    padding: 4px 5px !important;
    width: auto !important;
    flex-shrink: 0;
  }

  .depict-img {
    width: min(100%, 95px);
    max-width: 95px;
  }

  /* Allow full names in cells */
  .td-compound, .td-taxon, .td-ref {
    width: auto;
    min-width: 150px;
  }

  .cell-primary {
    font-weight: 500;
    line-height: 1.4;
    white-space: normal;
  }

  .stat-badge {
    padding: 10px 12px;
    gap: 5px;
  }

  .table-scroll {
    max-height: min(72vh, 900px);
  }
}

/* Large screens (1024px and above) - desktop optimization */
@media (width >= 1024px) {
  :root {
    /* Desktop-optimized responsive typography */
    --fs-0:      clamp(0.75rem, 0.725rem + 0.17vw, 0.875rem);
    --fs-1:      clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem);
    --fs-2:      clamp(0.9375rem, 0.9rem + 0.28vw, 1.0625rem);
    --fs-3:      clamp(1.125rem, 1.02rem + 0.6vw, 1.5rem);
    --fs-4:      clamp(1.375rem, 1.1rem + 0.85vw, 1.85rem);
    --fs-body:   clamp(0.875rem, 0.845rem + 0.2vw, 0.9375rem);
    --fs-label:  clamp(0.6875rem, 0.66rem + 0.14vw, 0.75rem);
    --fs-micro:  clamp(0.75rem, 0.73rem + 0.12vw, 0.8125rem);
    --fs-ui:     clamp(0.8125rem, 0.785rem + 0.16vw, 0.875rem);
    --fs-stat:   clamp(1.125rem, 1.02rem + 0.52vw, 1.375rem);
  }

  /* Table optimization for desktop - PRESERVE FULL NAMES */
  .results-table {
    font-size: var(--fs-ui);
    table-layout: auto;
    word-break: break-word;
  }

  .sort-th, .th-static {
    padding: 10px 12px;
    font-size: var(--fs-label);
    white-space: normal;
  }

  .data-row td {
    padding: 8px 12px;
    vertical-align: middle;
    word-break: break-word;
  }

  .td-depict {
    padding: 6px 10px !important;
    width: auto !important;
    flex-shrink: 0;
  }

  .depict-img {
    width: min(100%, 110px);
    max-width: 110px;
  }

  /* Allow full names and references */
  .td-compound, .td-taxon, .td-ref {
    width: auto;
    min-width: 180px;
  }

  .cell-primary {
    font-weight: 500;
    line-height: 1.4;
    white-space: normal;
  }

  .stat-badge {
    padding: 12px 14px;
    gap: 6px;
  }

  .table-scroll {
    max-height: min(72vh, 980px);
  }

  /* Wider badges on desktop */
  .id-badge {
    padding: 2px 6px;
    border-radius: 3px;
  }
}

/* Extra large screens (1440px+) - ensure optimal readability */
@media (width >= 1440px) {
  .page-header { padding-left: 32px; padding-right: 32px; }
  .page-header-meta { margin-left: 32px; margin-right: 32px; }

  /* share-bar mirrors page-header-meta margin */
  .share-bar { margin-left: 32px; margin-right: 32px; }

  /* flex-container children keep their own zero margin */
  .curation-wrap .share-bar { margin-left: 0; margin-right: 0; }

  .main-content > .notice {
    padding-left: 32px;
    padding-right: 32px;
  }
  .results-wrap { padding-left: 32px; padding-right: 32px; }
  .curation-wrap { padding-left: 32px; padding-right: 32px; }
  .draw-wrap     { padding-left: 32px; padding-right: 32px; }
}

/* Mobile-first heading and typography scaling */
@media (width <= 768px) {
  /* Ensure all headings are readable and don't overflow */
  h1, .page-title { word-break: break-word; overflow-wrap: break-word; }
  h2, h3, h4, h5, h6 { word-break: break-word; overflow-wrap: break-word; }

  /* Button and form element minimum touch target on mobile */
  button, .btn, input[type="button"], input[type="submit"] {
    min-height: 44px;
    min-width: 44px;
  }

  /* Improve link and interactive element sizing on touch devices */
  a, .copy-btn, .id-badge { padding: 4px 8px; }

  /* Optimize table cell padding for mobile readability */
  table td, table th {
    padding: 6px 4px;
    word-break: break-word;
    overflow-wrap: break-word;
  }

  /* Improve textarea usability on mobile */
  textarea {
    min-height: 120px;
    font-size: 16px;
  }

  /* Stack form groups vertically on mobile */
  .form-group, .form-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* Improve list readability on mobile */
  ul, ol, li {
    word-break: break-word;
    overflow-wrap: break-word;
  }
}

/* Tablet-specific optimizations (768px - 1023px) */
@media (width >= 769px) and (width <= 1023px) {
  /* Improve text readability on tablets */
  body { line-height: 1.6; }

  /* Optimize list item spacing */
  li { margin-bottom: 8px; }

  /* Improve form field spacing */
  .form-input, .form-textarea, input, textarea, select {
    min-height: 40px;
  }
}

/* Accessibility: Improve font sizing for readability */
@media (prefers-contrast: more) {
  body { font-size: 18px; }

  :root {
    /* Increase all font sizes by ~10% for accessibility */
    --fs-0:      clamp(0.82rem, 0.80rem + 0.18vw, 0.96rem);
    --fs-1:      clamp(0.96rem, 0.93rem + 0.22vw, 1.03rem);
    --fs-2:      clamp(1.03rem, 0.99rem + 0.31vw, 1.17rem);
    --fs-3:      clamp(1.24rem, 1.12rem + 0.66vw, 1.65rem);
    --fs-4:      clamp(1.51rem, 1.21rem + 0.94vw, 2.04rem);
    --fs-body:   clamp(0.96rem, 0.93rem + 0.22vw, 1.03rem);
    --fs-label:  clamp(0.76rem, 0.73rem + 0.15vw, 0.83rem);
    --fs-micro:  clamp(0.82rem, 0.80rem + 0.13vw, 0.89rem);
    --fs-ui:     clamp(0.89rem, 0.86rem + 0.18vw, 0.96rem);
    --fs-stat:   clamp(1.24rem, 1.12rem + 0.57vw, 1.51rem);
  }
}

/* Dark mode: Slightly larger text for better readability */
@media (prefers-color-scheme: dark) {
  /* Text is perceived as smaller in dark mode, so we can increase it slightly */
  body { letter-spacing: 0.3px; }
}

"###;
pub const LOTUS_TABLE_CELLS_CSS: &str = r###"/* Results table cell pack extracted from style.css. */

.td-depict { width:auto; min-width:0; padding:6px 10px !important; }
.depict-img { display:block; background:var(--bg2); border:1px solid var(--border); border-radius:6px; width:min(100%, 108px); max-width:108px; height:auto; object-fit:contain; box-shadow:var(--shadow-xs); }
.td-compound { min-width:0; }
.td-taxon { min-width:0; }
.td-ref { min-width:0; }
.cell-primary { font-weight:500; }
.primary-link { color:var(--text); }
.primary-link:hover { color:var(--accent); text-decoration:none; }


.cell-primary .primary-link {
  display: block;
  line-height: 1.4;
  overflow-wrap: break-word;
  word-break: break-word;
  white-space: normal;
}

.td-compound,
.td-taxon,
.td-ref {
  border-radius: 10px;
  background: color-mix(in srgb, var(--surface) 90%, transparent);
  box-shadow: inset 0 0 0 1px var(--results-border);
}

.td-compound { box-shadow: inset 3px 0 0 rgb(153 0 0 / 38%), inset 0 0 0 1px var(--results-border); }
.td-taxon { box-shadow: inset 3px 0 0 rgb(51 153 102 / 42%), inset 0 0 0 1px var(--results-border); }
.td-ref { box-shadow: inset 3px 0 0 rgb(0 102 153 / 44%), inset 0 0 0 1px var(--results-border); }

.td-compound a:not(.primary-link) { color:var(--wd-compound); }
.td-compound a:not(.primary-link):hover { color:color-mix(in srgb, var(--wd-compound) 88%, var(--text)); text-decoration:none; }
.td-taxon a { color:var(--wd-taxon); }
.td-taxon a:hover { color:color-mix(in srgb, var(--wd-taxon) 88%, var(--text)); text-decoration:none; }
.primary-link.taxon { color:var(--text); font-style:italic; }
.primary-link.taxon:hover { color:var(--text); text-decoration:none; }
.td-ref a:not(.primary-link) { color:var(--wd-reference); }
.td-ref a:not(.primary-link):hover { color:color-mix(in srgb, var(--wd-reference) 88%, var(--text)); text-decoration:none; }

.td-compound .primary-link,
.td-ref .primary-link { color:var(--text); }

.td-compound .primary-link:hover,
.td-ref .primary-link:hover { color:var(--text); text-decoration:none; }

.badge-row { display:flex; flex-wrap:wrap; gap:4px; margin-top:4px; overflow:visible; min-width:0; }
.id-badge { display:inline-block; font-size:var(--fs-micro); padding:1px 5px; border-radius:3px; font-weight:600; text-decoration:none !important; line-height:1.5; border:1px solid transparent; font-family:var(--mono); max-width:100%; white-space:normal; overflow-wrap:anywhere; }
.id-badge:hover { filter:brightness(1.15); }

.id-badge {
  backdrop-filter: blur(6px);
  transition: transform .12s ease, box-shadow .12s ease, filter .12s ease;
}

.id-badge:hover { box-shadow: var(--shadow-xs); }

.td-compound .id-badge.wd { background:var(--wd-compound-soft-bg); color:var(--wd-compound); border-color:var(--wd-compound-soft-border); }
.td-taxon .id-badge.wd { background:var(--wd-taxon-soft-bg); color:var(--wd-taxon); border-color:var(--wd-taxon-soft-border); }
.td-ref .id-badge.wd { background:var(--wd-reference-soft-bg); color:var(--wd-reference); border-color:var(--wd-reference-soft-border); }
.id-badge.sc { background:var(--wd-compound-soft-bg); color:var(--wd-compound); border-color:var(--wd-compound-soft-border-weak); }
.id-badge.doi { background:var(--wd-reference-soft-bg); color:var(--wd-reference); border-color:var(--wd-reference-soft-border-weak); }
.id-badge.mono { background:var(--surface); color:var(--text2); border-color:var(--border); }

/* `.stmt` must out-specify `.mono` when both classes are present. */
.id-badge.stmt,
.id-badge.stmt.mono { background:var(--wd-reference-soft-bg); color:var(--wd-reference); border-color:var(--wd-reference-soft-border-weak); }
.id-badge.mono.inchikey { background:var(--wd-compound-soft-bg); color:var(--wd-compound); border-color:var(--wd-compound-soft-border-weak); }

.td-mono { font-family:var(--mono); font-size:var(--fs-label); white-space:nowrap; }
.td-num { text-align:right; white-space:nowrap; font-variant-numeric:tabular-nums; }
.td-formula .formula { font-family:var(--mono); font-size:var(--fs-0); color:var(--text); }
.td-year { text-align:center; color:var(--text); white-space:nowrap; font-variant-numeric:tabular-nums; }
.na { color:var(--text3); }

"###;
pub const LOTUS_WELCOME_CSS: &str = r###"/* Welcome screen pack extracted from style.css. */

.welcome { padding:16px 22px; width:100%; max-width:none; display:flex; flex-direction:column; gap:12px; }

.welcome-hero,
.welcome-examples { width:100%; min-width:0; }
.welcome-hero h2 { font-size:clamp(1.6rem, 1.28rem + 1vw, 2.3rem); font-weight:800; letter-spacing:-.02em; line-height:1.08; }
.welcome-lead { font-size:var(--fs-1); color:var(--text2); margin-top:6px; line-height:1.60; max-width:none; overflow-wrap:anywhere; }

.welcome-support-text {
  font-size: var(--fs-1);
  line-height: 1.55;
  color: var(--text2);
}

.welcome-language-note { margin-top: 10px; max-width: 72ch; }
.welcome-inline-link { text-decoration: underline; text-underline-offset: 2px; font-weight: 600; }
.welcome-examples h3 { font-size:var(--fs-0); font-weight:700; color:var(--text3); text-transform:uppercase; letter-spacing:1px; margin-bottom:6px; }
.example-list { list-style:none; display:flex; flex-direction:column; gap:6px; }

.example-list .notice,
.welcome-cli-list .notice { margin: 0; }
.welcome-cli-hint { margin-top:16px; max-width:72ch; }
.welcome-cli-list { margin-top:3px; display:flex; flex-direction:column; gap:8px; }

"###;

pub fn bundled_lotus_styles() -> String {
    let base_css = LOTUS_BASE_CSS
        .lines()
        .filter(|line| !line.trim_start().starts_with("@import url("))
        .collect::<Vec<_>>()
        .join("\n");

    [
        base_css,
        LOTUS_ACCESSIBILITY_CSS.to_string(),
        LOTUS_CURATION_CSS.to_string(),
        LOTUS_FOOTER_CSS.to_string(),
        LOTUS_FORM_CONTROLS_CSS.to_string(),
        LOTUS_LAYOUT_SHELL_CSS.to_string(),
        LOTUS_QUERY_PANEL_CSS.to_string(),
        LOTUS_RESULTS_CSS.to_string(),
        LOTUS_RESPONSIVE_CSS.to_string(),
        LOTUS_TABLE_CELLS_CSS.to_string(),
        LOTUS_WELCOME_CSS.to_string(),
    ]
    .join("\n\n")
}
