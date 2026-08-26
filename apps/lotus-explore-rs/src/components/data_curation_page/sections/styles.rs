// SPDX-License-Identifier: AGPL-3.0-only
//! Curation section Tailwind class fragments.

pub(super) const CARD: &str =
    "flex flex-col gap-2.5 rounded-lotus border border-panel-border bg-panel-soft p-3 shadow-xs";

pub(super) const FORM_GRID: &str = "grid grid-cols-1 gap-2";

pub(super) fn actions(space_between: bool) -> &'static str {
    if space_between {
        "flex flex-wrap items-center justify-between gap-2"
    } else {
        "flex flex-wrap items-center gap-2"
    }
}

pub(super) const HINT: &str = "text-ui text-text";

pub(super) const TEXTAREA_130: &str =
    "form-textarea mono w-full min-h-[130px] rounded-lg border border-border bg-surface p-2.5 font-mono text-ui text-text shadow-xs focus:outline-none focus-visible:border-accent focus-visible:ring-2 focus-visible:ring-accent/25";

pub(super) const TEXTAREA_220: &str =
    "form-textarea mono w-full min-h-[220px] rounded-lg border border-border bg-surface p-2.5 font-mono text-ui text-text shadow-xs focus:outline-none focus-visible:border-accent focus-visible:ring-2 focus-visible:ring-accent/25";

pub(super) const FILE_INPUT: &str = "curation-file-input max-w-full text-ui text-muted";

pub(super) const NOTICE_VALUE: &str = "break-words leading-snug text-inherit";

pub(super) const TABLE_SCROLL: &str =
    "curation-table-scroll w-full min-w-0 overflow-x-auto overflow-y-visible rounded-xl border border-panel-border bg-panel-soft shadow-xs";

pub(super) const QUEUE_TABLE: &str =
    "w-full table-auto border-collapse text-ui [word-break:break-word]";

pub(super) const QUEUE_ACTION_COL: &str = "w-[110px] min-w-[110px] px-2 py-2";

pub(super) const QUEUE_INDEX_COL: &str = "min-w-[3ch] px-2 py-2";

pub(super) const QUEUE_SMILES_COL: &str = "min-w-[220px] max-w-[320px] px-2 py-2";

pub(super) fn row_stripe(idx: usize) -> &'static str {
    if idx.is_multiple_of(2) {
        "bg-surface/90 transition-colors hover:bg-bg"
    } else {
        "bg-surface/80 transition-colors hover:bg-bg"
    }
}
