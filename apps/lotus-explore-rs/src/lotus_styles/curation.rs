// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus CSS pack: curation.

const CURATION_NOTICE_AND_INPUT: &str = r"/* Curation surface: only selectors that still need CSS. */

.curation-wrap > .notice {
  margin-left: 0;
  margin-right: 0;
  margin-top: 0;
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
";

const CURATION_TABLE: &str = r"
.curation-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--fs-ui);
  table-layout: auto;
  word-break: break-word;
}

.curation-table th,
.curation-table td {
  border-bottom: 1px solid var(--border);
  text-align: left;
  vertical-align: top;
  padding: 8px 10px;
}

.curation-table thead {
  position: sticky;
  top: 0;
  z-index: 2;
  background: var(--bg2);
}

.curation-table tbody tr {
  transition: background .14s ease;
  background: var(--row-bg, transparent);
}

.curation-table tbody tr:hover {
  background: color-mix(in srgb, var(--surface2) 84%, var(--bg2));
}
";

const CURATION_SCROLL_AND_TABLES: &str = r"
.curation-table-scroll {
  border-radius: 14px;
}

.curation-table-scroll:focus-visible {
  outline: none;
  border-color: color-mix(in srgb, var(--accent) 44%, var(--panel-border)) !important;
  box-shadow: var(--ring) !important;
}

.curation-results-table {
  min-width: max-content;
}

.curation-queue-table {
  min-width: max-content;
}
";

pub fn css() -> String {
    [
        CURATION_NOTICE_AND_INPUT,
        CURATION_TABLE,
        CURATION_SCROLL_AND_TABLES,
    ]
    .join("\n\n")
}
