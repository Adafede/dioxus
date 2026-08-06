pub const CSS: &str = r"
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

  --wd-compound:  #900;
  --wd-taxon:     #396;
  --wd-reference: #069;
  --wd-entries:   #484848;
}

/* ── Box model reset ───────────────────────────────────────────────── */
*, *::before, *::after {
  box-sizing: border-box;
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

.shell {
  min-height: 100vh;
  padding: 24px;
  background: var(--bg);
  color: var(--text);
  font-family: var(--sans);
}

.card, .panel {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  box-shadow: var(--shadow-xs);
}

.panel { padding: 20px; margin-bottom: 20px; }

.hero {
  display: grid;
  gap: 8px;
  margin-bottom: 20px;
}

.hero h1 {
  margin: 0;
  font-size: 2rem;
  color: var(--text);
}

.hero p {
  margin: 0;
  color: var(--text2);
  line-height: 1.5;
}

.hero .small.muted {
  font-size: 0.84rem;
  color: var(--text3);
}

/* ── Input split: CSV dropzone + SMILES paste, side by side ─────────── */
.input-split {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  width: 100%;
  max-width: 56rem;
  margin-inline: auto;
  align-items: stretch;
}

.input-card {
  flex: 1 1 280px;
  display: grid;
  gap: 10px;
  min-height: 160px;
  width: 100%;
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  padding: 16px;
  background: var(--panel-bg-soft);
  box-shadow: var(--shadow-xs);
  transition: border-color .15s, box-shadow .15s, background-color .15s;
}

.input-card-body {
  display: grid;
  gap: 8px;
}

.dropzone {
  place-items: center;
  text-align: center;
  cursor: pointer;
  position: relative;
  border-style: dashed;
}
.dropzone.dragging {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, var(--panel-bg-soft));
}
.dropzone input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}

.paste-card {
  border-style: solid;
}
.paste-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.smiles-textarea {
  width: 100%;
  min-height: 100px;
  padding: 12px 14px;
  border-radius: 12px;
  border: 1px solid var(--panel-border);
  background: var(--panel-bg-soft);
  color: var(--text);
  font: 0.85rem/1.5 var(--mono);
  resize: vertical;
  transition: border-color .15s, box-shadow .15s, background-color .15s;
}
.smiles-textarea:focus {
  outline: none;
  box-shadow: var(--ring);
  border-color: var(--accent);
  background: var(--bg2);
}
.paste-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}
.status {
  margin: 12px 0 0;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid color-mix(in srgb, var(--border) 90%, transparent);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
.alert {
  margin-top: 12px;
  padding: 12px 14px;
  border-radius: 12px;
  background: color-mix(in srgb, var(--yellow) 10%, var(--panel-bg-soft));
  border: 1px solid color-mix(in srgb, var(--yellow) 32%, var(--border));
  color: color-mix(in srgb, var(--yellow) 88%, var(--text));
  font-weight: 600;
}
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: var(--tap-target-min);
  padding: 8px 16px;
  border: none;
  border-radius: 999px;
  font-weight: 700;
  cursor: pointer;
  transition: background-color .15s, box-shadow .15s, transform .05s;
}
.btn-primary {
  background: var(--accent);
  color: #fff;
}
.btn-primary:hover {
  background: var(--accent2);
  box-shadow: var(--shadow-sm);
}
.btn-primary:active {
  transform: scale(0.97);
}
.btn:disabled,
.btn:disabled:hover {
  opacity: .5;
  cursor: not-allowed;
  transform: none;
}
.chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-width: 100%;
}
.chip {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  padding: 5px 9px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 10%, var(--surface));
  color: color-mix(in srgb, var(--accent) 88%, var(--text));
  font-size: 0.82rem;
  border: 1px solid var(--panel-border);
}
.chip.alt { background: var(--surface); color: var(--text2); }
.chip.good { background: color-mix(in srgb, var(--green) 10%, var(--surface)); color: var(--green); border: 1px solid color-mix(in srgb, var(--green) 28%, var(--border)); }
.chip.warn { background: color-mix(in srgb, var(--yellow) 10%, var(--surface)); color: color-mix(in srgb, var(--yellow) 88%, var(--text)); border: 1px solid color-mix(in srgb, var(--yellow) 28%, var(--border)); }
.chip.fail { background: color-mix(in srgb, var(--red) 10%, var(--surface)); color: var(--red); border: 1px solid color-mix(in srgb, var(--red) 28%, var(--border)); }
.chip-np,
.chip.chip-np { background: color-mix(in srgb, var(--green) 10%, var(--surface)); color: var(--green); border: 1px solid color-mix(in srgb, var(--green) 28%, var(--border)); }
.chip-scaffold,
.chip.chip-scaffold { background: color-mix(in srgb, var(--accent) 10%, var(--surface)); color: var(--accent); border: 1px solid color-mix(in srgb, var(--accent) 28%, var(--border)); }
.motif-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 4px 0 10px;
}
.motif-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid var(--panel-border);
  font-size: 0.78rem;
  font-weight: 700;
}
.motif-legend-green {
  background: color-mix(in srgb, var(--green) 10%, var(--surface));
  color: var(--green);
  border-color: color-mix(in srgb, var(--green) 28%, var(--border));
}
.motif-legend-blue {
  background: color-mix(in srgb, var(--accent) 10%, var(--surface));
  color: var(--accent);
  border-color: color-mix(in srgb, var(--accent) 28%, var(--border));
}
.motif-legend-neutral {
  background: var(--surface);
  color: var(--text2);
}
.motif-groups {
  display: grid;
  gap: 10px;
}
.motif-group {
  display: grid;
  gap: 6px;
}
.motif-group h3,
.motif-group h4 {
  margin: 0;
  font-size: 0.82rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text3);
}
.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}
.card {
  overflow: visible;
}
.card-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 14px 0;
}
.card-body {
  display: grid;
  gap: 10px;
  padding: 14px;
}
.svg-wrap {
  display: grid;
  place-items: center;
  min-height: 210px;
  background: var(--bg2);
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  overflow: visible;
}
.svg-wrap svg {
  display: block;
  max-width: 100%;
  max-height: 260px;
  width: 100%;
  height: auto;
}
.svg-wrap > div {
  width: 100%;
}
.meta {
  display: grid;
  gap: 6px;
  font-size: 0.9rem;
  color: var(--text2);
  overflow-wrap: anywhere;
}
.meta strong {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--text2);
}
.checklist {
  display: grid;
  gap: 4px;
}
.check-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  font-size: 0.88rem;
}
.check-status {
  padding: 2px 8px;
  border-radius: 999px;
  font-weight: 600;
  font-size: 0.82rem;
}
.check-status.pass { background: color-mix(in srgb, var(--green) 10%, var(--surface)); color: var(--green); }
.check-status.warn { background: color-mix(in srgb, var(--yellow) 10%, var(--surface)); color: color-mix(in srgb, var(--yellow) 88%, var(--text)); }
.check-status.fail { background: color-mix(in srgb, var(--red) 10%, var(--surface)); color: var(--red); }
.cid-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--wd-reference) 8%, var(--surface));
  color: var(--wd-reference);
  text-decoration: none;
  font-size: 0.82rem;
  font-weight: 600;
  border: 1px solid color-mix(in srgb, var(--wd-reference) 30%, var(--border));
}
.cid-link:hover {
  background: color-mix(in srgb, var(--wd-reference) 15%, var(--surface));
  text-decoration: underline;
}
.cid-link.green {
  background: color-mix(in srgb, var(--wd-taxon) 8%, var(--surface));
  color: var(--wd-taxon);
  border-color: color-mix(in srgb, var(--wd-taxon) 30%, var(--border));
}
.cid-link.green:hover {
  background: color-mix(in srgb, var(--wd-taxon) 15%, var(--surface));
}
.cid-link.red {
  background: color-mix(in srgb, var(--wd-compound) 8%, var(--surface));
  color: var(--wd-compound);
  border-color: color-mix(in srgb, var(--wd-compound) 30%, var(--border));
}
.cid-link.red:hover {
  background: color-mix(in srgb, var(--wd-compound) 15%, var(--surface));
}
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
.footer-link {
  color: var(--text);
  text-decoration: none;
  transition: color .15s;
}
.footer-link:hover { text-decoration: underline; }
.footer-link.red { color: var(--wd-compound); font-weight: 700; }
.footer-link.green { color: var(--wd-taxon); font-weight: 700; }
.footer-link.blue { color: var(--wd-reference); font-weight: 700; }
.footer-link.purple { color: var(--wd-reference); font-weight: 700; }
.footer-link.muted { color: var(--text2); font-weight: 700; }
.meta.small { font-size: 0.82rem; }
.meta.small.muted { color: var(--text3); }
.verdict {
  margin: 8px;
  font-weight: 700;
  padding: 8px 14px;
  border-radius: 10px;
  text-align: center;
  font-size: 0.9rem;
}
/* Verdict badges use ACTUAL Wikidata colors: #990000 #339966 #006699 #484848 */
.verdict-likely { background: color-mix(in srgb, var(--green) 12%, var(--surface)); color: var(--green); border: 1px solid color-mix(in srgb, var(--green) 40%, var(--border)); }
.verdict-neutral { background: color-mix(in srgb, var(--wd-reference) 12%, var(--surface)); color: var(--wd-reference); border: 1px solid color-mix(in srgb, var(--wd-reference) 40%, var(--border)); }
.verdict-caution { background: color-mix(in srgb, var(--red) 12%, var(--surface)); color: var(--red); border: 1px solid color-mix(in srgb, var(--red) 40%, var(--border)); }
.verdict-skeptical {
  background: color-mix(in srgb, var(--yellow) 12%, var(--surface));
  color: color-mix(in srgb, var(--yellow) 88%, var(--text));
  border: 1px solid color-mix(in srgb, var(--yellow) 40%, var(--border));
}
.verdict-fishy {
  background: color-mix(in srgb, var(--red) 20%, var(--surface));
  color: var(--red);
  border: 1px solid color-mix(in srgb, var(--red) 50%, var(--border));
}
.muted { color: var(--text3); }
.error { color: var(--red); font-weight: 600; }
.small { font-size: 0.82rem; }
.evidence {
  margin-top: 6px;
  margin-left: auto;
  width: fit-content;
}
.evidence details summary {
  cursor: pointer;
  font-weight: 600;
  color: var(--text2);
  list-style: none;
}
.evidence details[open] summary {
  color: var(--accent);
}
.evidence details p {
  margin: 4px 0;
  padding-left: 4px;
  border-left: 2px solid var(--border);
  color: var(--text2);
}

/* ── Accessibility ──────────────────────────────────────────────────── */
.visually-hidden {
  position: absolute !important;
  width: 1px !important;
  height: 1px !important;
  padding: 0 !important;
  margin: -1px !important;
  overflow: hidden !important;
  clip: rect(0, 0, 0, 0) !important;
  white-space: nowrap !important;
  border: 0 !important;
}

/* ── Endpoint status chips ──────────────────────────────────────────── */
.endpoint-status {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 6px;
}
.endpoint-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 0.78rem;
  font-weight: 600;
  border: 1px solid var(--panel-border);
}
.endpoint-chip.ok {
  background: color-mix(in srgb, var(--green) 10%, var(--surface));
  color: var(--green);
  border-color: color-mix(in srgb, var(--green) 28%, var(--border));
}
.endpoint-chip.down {
  background: color-mix(in srgb, var(--red) 10%, var(--surface));
  color: var(--red);
  border-color: color-mix(in srgb, var(--red) 28%, var(--border));
}

/* ── Footer ─────────────────────────────────────────────────────────── */
.app-footer {
  margin-top:auto;
  padding:16px 28px 20px;
  border-top:1px solid var(--panel-border);
  background:var(--panel-bg-soft);
  color:var(--text2);
  display:flex;
  flex-direction:column;
  gap:0;
  font-size:var(--fs-1);
  box-shadow:var(--panel-shadow);
  border-radius: 16px;
  overflow: hidden;
}
.footer-line {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 0 24px;
  align-items: start;
  padding: 10px 0;
  border-bottom: 1px solid var(--panel-border);
}
.footer-line:last-child { border-bottom:none; padding-bottom:0; }
.footer-line:first-child { padding-top:0; }
.footer-row {
  display: grid;
  grid-template-columns: clamp(7.5rem, 7vw, 9rem) minmax(0, 1fr);
  gap: 4px 6px;
  align-items: start;
  padding: 2px 0;
}
.footer-label {
  color: var(--text2);
  font-weight: 700;
  text-transform: uppercase;
  font-size: var(--fs-0);
  letter-spacing: 1px;
  min-width: 0;
  white-space: nowrap;
}
.footer-links {
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: 3px 5px;
  margin: 0;
  padding: 0;
  min-width: 0;
  justify-content: flex-start;
}
.footer-links li {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  padding: 1px 7px;
  border-radius: 999px;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  transition: border-color .12s, box-shadow .12s;
}
.footer-links li:hover {
  border-color: color-mix(in srgb, var(--panel-border) 60%, var(--accent));
  box-shadow: var(--shadow-xs);
}
";
