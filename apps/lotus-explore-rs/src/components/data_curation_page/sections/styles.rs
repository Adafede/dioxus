// SPDX-License-Identifier: AGPL-3.0-only
//! Curation section Tailwind class fragments.

pub(super) const CARD: &str =
    "flex flex-col gap-4 rounded-xl border border-panel-border bg-panel-soft p-4 shadow-xs";

pub(super) const FORM_GRID: &str = "grid grid-cols-1 gap-3";

pub(super) fn actions(space_between: bool) -> &'static str {
    if space_between {
        "flex flex-wrap items-center justify-between gap-2.5"
    } else {
        "flex flex-wrap items-center gap-2.5"
    }
}

pub(super) const HINT: &str = "text-ui text-subtle leading-snug";

pub(super) const TEXTAREA_130: &str = "form-textarea mono w-full min-h-[130px] rounded-lg border border-border bg-surface p-2.5 font-mono text-body text-text shadow-xs focus:outline-none focus-visible:border-accent focus-visible:ring-2 focus-visible:ring-accent/40";

pub(super) const TEXTAREA_220: &str = "form-textarea mono w-full min-h-[220px] rounded-lg border border-border bg-surface p-2.5 font-mono text-body text-text shadow-xs focus:outline-none focus-visible:border-accent focus-visible:ring-2 focus-visible:ring-accent/40";

pub(super) const FILE_INPUT: &str = "curation-file-input max-w-full text-ui text-muted";

pub(super) const NOTICE_VALUE: &str = "break-words leading-snug text-inherit";

pub(super) const TABLE_SCROLL: &str = "w-full overflow-x-auto rounded-lg border border-panel-border focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40";

pub(super) const QUEUE_TABLE: &str = "w-full table-fixed border-collapse text-ui";

pub(super) const TH: &str = "border-b border-panel-border bg-panel-soft px-3 py-2.5 text-left text-xs font-semibold uppercase tracking-wide text-muted";

pub(super) const TD: &str = "border-b border-panel-border px-3 py-2.5 align-top text-ui";

pub(super) const QUEUE_ACTION_COL: &str = "w-[110px] min-w-[110px]";

pub(super) const QUEUE_INDEX_COL: &str = "min-w-[3ch]";

pub(super) const QUEUE_SMILES_COL: &str = "min-w-[220px] max-w-[320px] break-all";
