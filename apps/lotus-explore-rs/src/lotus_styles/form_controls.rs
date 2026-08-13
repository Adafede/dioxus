// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: form_controls.

const FORM_SECTIONS: &str = r"/* Form controls pack: shared form primitives, structure editor, and search button. */

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
";

const FORM_RANGES: &str = r"
.formula-grid { display: grid; grid-template-columns: 1fr; gap: 10px; }
.formula-minmax-grid { display: grid; grid-template-columns: 1fr; gap: 8px; }

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
";

const FORM_STRUCTURE: &str = r#"
/* Normalize number input chrome for consistent borders on Safari/Firefox. */
input[type="number"].form-input { appearance: textfield; }

input[type="number"].form-input::-webkit-outer-spin-button,
input[type="number"].form-input::-webkit-inner-spin-button { appearance: none; margin: 0; }

/* Structure section */
.form-textarea.mono, .mono { font-family: var(--mono); }
"#;

const FORM_ACTIONS_AND_RESPONSIVE: &str = r"
/* Focus-visible styles for interactive elements */
button:focus-visible,
.sort-btn:focus-visible,
.notice-dismiss:focus-visible,
.primary-link:focus-visible,
.id-badge:focus-visible,
.filters-toggle:focus-visible {
  outline: 3px solid var(--accent2);
  outline-offset: 2px;
}
";

pub fn css() -> String {
    [
        FORM_SECTIONS,
        FORM_RANGES,
        FORM_STRUCTURE,
        FORM_ACTIONS_AND_RESPONSIVE,
    ]
    .join("\n\n")
}
