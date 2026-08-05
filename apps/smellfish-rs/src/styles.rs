pub const CSS: &str = r#"
.shell {
  min-height: 100vh;
  padding: 24px;
  background: #f5f7fb;
  color: #162033;
  font-family: Inter, system-ui, sans-serif;
}
.card, .panel {
  background: #fff;
  border: 1px solid #d7dee9;
  border-radius: 16px;
  box-shadow: 0 10px 35px rgba(25, 39, 62, 0.06);
}
.panel { padding: 20px; margin-bottom: 20px; }
.hero {
  display: grid;
  gap: 10px;
  margin-bottom: 20px;
}
.hero h1 {
  margin: 0;
  font-size: 2rem;
}
.hero p {
  margin: 0;
  color: #4d5b74;
  line-height: 1.5;
}
.dropzone {
  position: relative;
  display: grid;
  gap: 8px;
  place-items: center;
  min-height: 150px;
  border: 2px dashed #9eb0cc;
  border-radius: 16px;
  padding: 18px;
  cursor: pointer;
  background: #fbfcfe;
  text-align: center;
}
.dropzone.dragging { border-color: #0d6efd; background: #eef5ff; }
.dropzone input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}
.status {
  margin: 12px 0 0;
  font-weight: 600;
}
.alert {
  margin-top: 12px;
  padding: 12px 14px;
  border-radius: 12px;
  background: #fff4e5;
  border: 1px solid #ffd29c;
  color: #8a4b00;
  font-weight: 600;
}
.summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px;
}
.summary-item {
  padding: 14px;
  border: 1px solid #e1e7f0;
  border-radius: 14px;
  background: #fcfdff;
}
.summary-item h3, .summary-item h4 {
  margin: 0 0 8px;
  font-size: 0.95rem;
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
  background: #eef4ff;
  color: #21467a;
  font-size: 0.82rem;
}
.chip.alt { background: #f3f0ff; color: #5e44ad; }
.chip.good { background: #e9fbef; color: #167345; }
.chip.warn { background: #fff4e5; color: #8a4b00; }
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
  border: 1px solid #eef2f7;
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
  color: #384861;
  overflow-wrap: anywhere;
}
.result-box {
  display: grid;
  gap: 4px;
  padding: 12px;
  border: 1px solid #e3e9f2;
  border-radius: 12px;
  background: #f9fbff;
}
.result-grid {
  display: grid;
  gap: 4px;
  font-size: 0.88rem;
}
.result-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 12px;
}
.result-row span:first-child {
  overflow-wrap: anywhere;
}
.result-badge {
  font-weight: 700;
  white-space: nowrap;
}
.verdict {
  margin-top: 4px;
  font-weight: 700;
}
.muted { color: #63738d; }
.error { color: #b42318; font-weight: 600; }
.small { font-size: 0.82rem; }
.literature-list {
  display: grid;
  gap: 10px;
}
.literature-item {
  padding: 10px 12px;
  border: 1px solid #e1e7f0;
  border-radius: 12px;
  background: #fcfdff;
}
.literature-item strong {
  display: block;
  margin-bottom: 4px;
}
"#;
