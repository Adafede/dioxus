//! Design tokens and base CSS.  Inject once per app:
//!
//! ```rust
//! rsx! { style { dangerous_inner_html: shared::theme::BASE_CSS } … }
//! ```

pub const BASE_CSS: &str = r#"
/* ── Typography (Fira Code for code, system sans for everything else) ────── */
@import url('https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600&display=swap');

/* ── Reset & base ────────────────────────────────────────────────────────── */
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html, body { height: 100%; }

/* ── Design tokens ───────────────────────────────────────────────────────── */
:root {
  --bg:        #0f1117;
  --bg2:       #161b22;
  --surface:   #21262d;
  --surface2:  #2d333b;
  --border:    #30363d;
  --text:      #e6edf3;
  --text2:     #8b949e;
  --text3:     #6e7681;
  --accent:    #58a6ff;
  --accent2:   #388bfd;
  --green:     #3fb950;
  --red:       #f85149;
  --yellow:    #d29922;
  --purple:    #bc8cff;
  --radius:    8px;
  --radius-sm: 4px;
  --mono:      'Fira Code', ui-monospace, SFMono-Regular, 'JetBrains Mono', Consolas, monospace;
  --sans:      -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

body {
  background: var(--bg);
  color: var(--text);
  font-family: var(--sans);
  font-size: 14px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }

/* ── Accessibility: visible keyboard focus everywhere ────────────────────── */
:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}
/* Screen-reader-only helper class — use for icon-only button labels. */
.sr-only {
  position: absolute !important;
  width: 1px; height: 1px;
  padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0,0,0,0);
  white-space: nowrap; border: 0;
}

/* ── Animations ──────────────────────────────────────────────────────────── */
@keyframes spin    { to { transform: rotate(360deg); } }
@keyframes fadeIn  { from { opacity:0; transform:translateY(4px) } to { opacity:1; transform:none } }

/* ── Scrollbar ───────────────────────────────────────────────────────────── */
::-webkit-scrollbar { width:6px; height:6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border); border-radius:3px; }
::-webkit-scrollbar-thumb:hover { background: var(--text3); }

/* ── Buttons ─────────────────────────────────────────────────────────────── */
.btn {
  display: inline-flex; align-items: center; gap: 6px;
  border: 1px solid var(--border); border-radius: var(--radius-sm);
  padding: 7px 14px; font-size: 13px; font-weight: 500;
  cursor: pointer; background: var(--surface); color: var(--text);
  transition: background .15s, border-color .15s;
}
.btn:disabled { opacity: .45; cursor: not-allowed; }
.btn:hover:not(:disabled) { background: var(--surface2); }
.btn-primary { background: var(--accent2); border-color: var(--accent2); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--accent); border-color: var(--accent); }
.btn-sm { padding: 4px 10px; font-size: 12px; }

/* ── Badges ──────────────────────────────────────────────────────────────── */
.badge { display:inline-flex; align-items:center; gap:3px; padding:2px 7px; border-radius:10px; font-size:11px; font-weight:600; }
.badge-blue   { background:rgba(88,166,255,.15); color:var(--accent); border:1px solid rgba(88,166,255,.3); }
.badge-green  { background:rgba(63,185,80,.15);  color:var(--green);  border:1px solid rgba(63,185,80,.3);  }
.badge-yellow { background:rgba(210,153,34,.15); color:var(--yellow); border:1px solid rgba(210,153,34,.3); }
.badge-purple { background:rgba(188,140,255,.15);color:var(--purple); border:1px solid rgba(188,140,255,.3); }
.badge-neutral{ background:var(--surface); color:var(--text2); border:1px solid var(--border); }

/* ── Stat bar ────────────────────────────────────────────────────────────── */
.stat-bar { display:flex; flex-wrap:wrap; gap:10px; }
.stat-badge { display:flex; align-items:center; gap:10px; background:var(--surface); border:1px solid var(--border); border-radius:var(--radius); padding:10px 16px; }
.stat-icon  { font-size:20px; }
.stat-value { display:block; font-size:20px; font-weight:700; line-height:1.2; }
.stat-label { display:block; font-size:11px; color:var(--text2); text-transform:uppercase; letter-spacing:.5px; }

/* ── Error banner ────────────────────────────────────────────────────────── */
.error-banner { display:flex; align-items:center; gap:10px; padding:12px 16px; border-radius:var(--radius); background:rgba(248,81,73,.1); border:1px solid rgba(248,81,73,.4); color:var(--red); animation:fadeIn .2s; }
.error-banner .dismiss { margin-left:auto; background:none; border:none; color:var(--red); cursor:pointer; font-size:16px; }

/* ── Loading ─────────────────────────────────────────────────────────────── */
.spinner-lg { width:40px; height:40px; border:3px solid var(--border); border-top-color:var(--accent); border-radius:50%; animation:spin .8s linear infinite; }
.spinner-sm { width:14px; height:14px; border:2px solid rgba(255,255,255,.3); border-top-color:#fff; border-radius:50%; animation:spin .7s linear infinite; display:inline-block; }
.loading-state { display:flex; flex-direction:column; align-items:center; justify-content:center; gap:14px; padding:48px; color:var(--text2); flex:1; }
.loading-hint  { font-size:12px; color:var(--text3); }

/* ── Forms ───────────────────────────────────────────────────────────────── */
.form-input, .form-textarea {
  background:var(--surface); border:1px solid var(--border);
  border-radius:var(--radius-sm); color:var(--text);
  padding:8px 10px; font-size:13px; width:100%;
  font-family:var(--sans); transition:border-color .15s;
}
.form-input:focus, .form-textarea:focus { outline:none; border-color:var(--accent); }
.form-input.sm { width:90px; }
.form-label { font-size:12px; font-weight:600; color:var(--text); display:flex; align-items:center; gap:6px; }
.form-label.sm { font-size:11px; font-weight:500; color:var(--text2); }
.form-hint  { font-size:11px; color:var(--text3); margin-top:2px; }

/* ── Pagination ──────────────────────────────────────────────────────────── */
.pagination-bar { display:flex; align-items:center; justify-content:space-between; gap:12px; padding:8px 0; }
.page-info { font-size:12px; color:var(--text2); }

/* ── Empty state ─────────────────────────────────────────────────────────── */
.empty-state { display:flex; flex-direction:column; align-items:center; gap:12px; padding:64px 24px; color:var(--text2); }
.empty-icon  { font-size:40px; }

/* ── Card ────────────────────────────────────────────────────────────────── */
.card { background:var(--surface); border:1px solid var(--border); border-radius:var(--radius); padding:16px; }
.card:hover { border-color:var(--text3); }

/* ── Responsive ──────────────────────────────────────────────────────────── */
@media (max-width:768px) {
  .stat-bar { gap:6px; }
  .stat-badge { padding:8px 12px; }
  .stat-value { font-size:16px; }
}
"#;
