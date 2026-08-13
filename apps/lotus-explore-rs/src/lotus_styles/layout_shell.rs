// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: layout_shell.

const APP_FRAME: &str = r"/* Layout shell pack: app frame, header/meta, notices, share bar, and sidebar shell. */

.app-layout { display:flex; min-height:100dvh; height:100dvh; overflow:hidden; gap:10px; padding:10px; }
.app-layout.no-sidebar { display:block; }

.sidebar {
  width:300px;
  min-width:250px;
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
";

const PAGE_HEADER: &str = r"
.page-header {
  padding:14px 22px 10px;
  border-bottom:1px solid var(--panel-border);
  background:color-mix(in srgb, var(--panel-bg-soft) 92%, var(--surface));
  box-shadow:var(--shadow-xs);
  position: sticky;
  top: 0;
  z-index: 3;
  overflow: clip;
}


.page-brand { display:flex; align-items:center; gap:12px; }
.sidebar-logo-link { display: inline-flex; align-items: center; justify-content: center; border-radius: 14px; text-decoration: none; }
.page-home-link { display: inline-flex; align-items: center; gap: 0; min-width: 0; }
.page-title-text { min-width: 0; overflow-wrap: anywhere; }
.page-title { font-size:var(--fs-4); font-weight:800; letter-spacing:-.028em; line-height:1.06; color:var(--text); }

.page-title-link,
.page-title-link:visited { color: inherit; text-decoration: none; }
.page-title-link:hover { text-decoration: none; }

.lang-switch { margin-left:auto; display:flex; gap:4px; align-items:center; }
.lang-btn { min-width:40px; padding:3px 8px; }
.lang-btn.active { background:var(--btn-primary-bg); color:var(--text); border-color:var(--btn-primary-bg); }

.view-switch .lang-btn,
.lang-switch .lang-btn {
  color: var(--text2);
  background: transparent;
  border-color: var(--border);
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
  color: var(--text);
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
.meta-item { display:inline-flex; align-items:center; gap:4px; white-space: normal; overflow-wrap: anywhere; line-height: 1.4; }
.meta-key { text-transform:uppercase; letter-spacing:0.08em; font-weight:700; font-size: var(--fs-0); color: var(--text2); }
.meta-val.mono { font-family:var(--mono); color:var(--critical-text); font-variant-numeric: tabular-nums; font-size: var(--fs-0); }
.meta-sep { color:var(--text3); }
";

const NOTICES: &str = r"
/* Notices */
.notice {
  margin:10px 22px 0;
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
  padding-left: 22px;
  padding-right: 22px;
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

.notice-value { flex:1; color:inherit; word-break:break-word; line-height:1.4; }

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
";

const SHARE_BAR: &str = r"
/* Share bar */
.share-bar {
  display: flex;
  flex-flow: row wrap;
  align-items: center;
  gap: 6px 10px;
  margin: 10px 22px 0;
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
";

const SEARCH_PANEL: &str = r#"
/* Search panel shell */
.search-panel { align-self:stretch; padding:18px 16px; display:flex; flex-direction:column; gap:14px; background:var(--panel-bg); flex:0 0 auto; box-sizing:border-box; min-width:240px; overflow-y:auto; max-height:calc(100vh - 200px); margin-top:auto; }
.search-panel-body { display:flex; flex-direction:column; gap:12px; }
.filters-toggle { display:none; }
.sidebar-logo-wrap { padding:6px 8px 8px; display:flex; justify-content:center; border-top:1px solid var(--border); margin-top:auto; }
.view-switch { margin-top: 10px; display: flex; gap: 8px; }
.view-switch .btn { font-weight: 700; }
.sidebar-logo { display:block; width:128px; height:128px; }
.view-switch [role="group"] { background: transparent !important; border-color: var(--border) !important; }
.lang-switch [role="group"] { background: transparent !important; border-color: var(--border) !important; }
.search-btn { white-space: normal; word-break: break-word; }
@media (max-width: 768px) {
  .filters-toggle { display:flex; flex-wrap:wrap; align-items:center; justify-content:center; background:var(--btn-primary-bg); color:#fff; border:0; border-radius:var(--radius-sm); padding:11px 16px; font-size:var(--fs-ui); font-weight:700; cursor:pointer; box-shadow:var(--shadow-xs); transition:background .15s, box-shadow .15s, transform .12s ease; text-align:center; line-height:1.2; white-space:normal; width:calc(100% - 32px); margin:0 16px; box-sizing:border-box; }
  .filters-toggle:active { transform: translateY(1px); }
  .filters-toggle:disabled { opacity:.5; cursor:not-allowed; }
  .search-panel-body { display:none !important; }
  .sidebar.mobile-open .search-panel-body { display:flex !important; }
}
"#;

const FOOTER: &str = r"
/* Footer responsive grid sizing */
@media (min-width: 640px) {
  footer > div {
    grid-template-columns: 1.2fr 1fr !important;
  }
}
";

pub fn css() -> String {
    [
        APP_FRAME,
        PAGE_HEADER,
        NOTICES,
        SHARE_BAR,
        SEARCH_PANEL,
        FOOTER,
    ]
    .join("\n\n")
}
