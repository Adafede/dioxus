pub const CSS: &str = r"
.shell {
  min-height: 100vh;
  padding: 24px;
  background: #f8fafc;
  color: #0f172a;
  font-family: Inter, system-ui, sans-serif;
}
.card, .panel {
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 16px;
  box-shadow: 0 10px 35px rgba(15, 23, 42, 0.06);
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
}
.hero p {
  margin: 0;
  color: #475569;
  line-height: 1.5;
}
.hero .small.muted {
  font-size: 0.84rem;
}
.dropzone {
  position: relative;
  display: grid;
  gap: 8px;
  place-items: center;
  min-height: 150px;
  border: 2px dashed #94a3b8;
  border-radius: 16px;
  padding: 18px;
  cursor: pointer;
  background: #f8fafc;
  text-align: center;
}
.dropzone.dragging { border-color: #3b82f6; background: #eff6ff; }
.dropzone input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
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
  border: 2px solid #cbd5e1;
  border-top-color: #3b82f6;
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
  background: #fffbeb;
  border: 1px solid #fde68a;
  color: #92400e;
  font-weight: 600;
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
  background: #eff6ff;
  color: #2563eb;
  font-size: 0.82rem;
}
.chip.alt { background: #f1f5f9; color: #475569; }
.chip.good { background: #dcfce7; color: #15803d; }
.chip.warn { background: #fffbeb; color: #92400e; }
.chip.fail { background: #fee2e2; color: #991b1b; }
.chip-np,
.chip.chip-np { background: #dcfce7; color: #15803d; border: 1px solid #4ade80; }
.chip-scaffold,
.chip.chip-scaffold { background: #dbeafe; color: #2563eb; border: 1px solid #60a5fa; }
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
  background: #fff;
  border: 1px solid #e2e8f0;
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
  color: #334155;
  overflow-wrap: anywhere;
}
.meta strong {
  font-size: 0.82rem;
  font-weight: 600;
  color: #475569;
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
.check-status.pass { background: #dcfce7; color: #15803d; }
.check-status.warn { background: #fffbeb; color: #92400e; }
.check-status.fail { background: #fee2e2; color: #991b1b; }
.cid-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 999px;
  background: rgba(0, 102, 153, 0.08);
  color: #006699;
  text-decoration: none;
  font-size: 0.82rem;
  font-weight: 600;
  border: 1px solid rgba(0, 102, 153, 0.3);
}
.cid-link:hover {
  background: rgba(0, 102, 153, 0.15);
  text-decoration: underline;
}
.cid-link.green {
  background: rgba(51, 153, 102, 0.08);
  color: #339966;
  border-color: rgba(51, 153, 102, 0.3);
}
.cid-link.green:hover {
  background: rgba(51, 153, 102, 0.15);
}
.cid-link.red {
  background: rgba(153, 0, 0, 0.08);
  color: #990000;
  border-color: rgba(153, 0, 0, 0.3);
}
.cid-link.red:hover {
  background: rgba(153, 0, 0, 0.15);
}
.ertl-work-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 0;
  border: none;
  background: transparent;
  color: #006699;
  font-size: 0.84rem;
  font-weight: 600;
  cursor: pointer;
  text-decoration: none;
}
.ertl-work-btn:hover {
  text-decoration: underline;
}
a { color: #006699; text-decoration: none; }
a:hover { text-decoration: underline; }
.footer-link.blue { color: #006699; }
.meta.small { font-size: 0.82rem; }
.meta.small.muted { color: #94a3b8; }
.verdict {
  margin: 8px;
  font-weight: 700;
  padding: 8px 14px;
  border-radius: 10px;
  text-align: center;
  font-size: 0.9rem;
}
/* Verdict badges use ACTUAL Wikidata colors: #990000 #339966 #006699 #484848 */
.verdict-likely {
  background: rgba(51, 153, 102, 0.12);
  color: #339966;
  border: 1px solid rgba(51, 153, 102, 0.4);
}
.verdict-neutral {
  background: rgba(0, 102, 153, 0.12);
  color: #006699;
  border: 1px solid rgba(0, 102, 153, 0.4);
}
.verdict-caution {
  background: rgba(153, 0, 0, 0.12);
  color: #990000;
  border: 1px solid rgba(153, 0, 0, 0.4);
}
.verdict-skeptical {
  background: rgba(180, 155, 0, 0.12);
  color: #b39b00;
  border: 1px solid rgba(180, 155, 0, 0.4);
}
.verdict-fishy {
  background: rgba(153, 0, 0, 0.2);
  color: #990000;
  border: 1px solid rgba(153, 0, 0, 0.5);
}
.muted { color: #64748b; }
.error { color: #b91c1c; font-weight: 600; }
.small { font-size: 0.82rem; }
.evidence {
  margin-top: 6px;
  margin-left: auto;
  width: fit-content;
}
.evidence details summary {
  cursor: pointer;
  font-weight: 600;
  color: #475569;
  list-style: none;
}
.evidence details[open] summary {
  color: #2563eb;
}
.evidence details p {
  margin: 4px 0;
  padding-left: 4px;
  border-left: 2px solid #cbd5e1;
  color: #334155;
}

/* ── Footer (same style as lotus-explore-rs) ────────────────────────── */
.app-footer {
  margin-top: 24px;
  padding: 16px 24px 20px;
  border-top: 1px solid #e2e8f0;
  background: #f8fafc;
  color: #475569;
  font-size: 0.84rem;
}
.footer-line {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 0 16px;
  align-items: start;
  padding: 8px 0;
  border-bottom: 1px solid #e2e8f0;
}
.footer-line:last-child { border-bottom: none; }
.footer-row {
  display: grid;
  grid-template-columns: 7rem minmax(0, 1fr);
  gap: 4px 6px;
  align-items: start;
  padding: 2px 0;
}
.footer-label {
  font-weight: 700;
  text-transform: uppercase;
  font-size: 0.75rem;
  letter-spacing: 0.5px;
  color: #475569;
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
  padding: 1px 7px;
  border-radius: 999px;
  background: transparent;
  border: 1px solid #e2e8f0;
  min-width: 0;
  transition: border-color .12s, box-shadow .12s;
}
.footer-links li:hover {
  border-color: #cbd5e1;
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.05);
}
.footer-link {
  color: #0f172a;
  text-decoration: none;
}
.footer-link:hover { text-decoration: underline; }
.footer-link.red { color: #990000; }
.footer-link.green { color: #339966; }
.footer-link.blue { color: #006699; }
";
