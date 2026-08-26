// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared Tailwind class fragments mapped to Lotus design tokens.
//!
//! Prefer these over raw `slate-*` / `sky-*` / `dark:` pairs so colors track
//! `data-theme` via CSS variables.

/// Field / control label.
pub const LABEL: &str = "text-ui font-semibold text-text";

/// Secondary / hint text under controls.
pub const HINT: &str = "text-micro text-subtle";

/// Uppercase micro label (range min/max, etc.).
pub const MICRO_LABEL: &str = "text-[10px] font-semibold uppercase tracking-wide text-subtle";

/// Standard text/number input.
pub const INPUT: &str = "w-full rounded-md border border-border bg-surface px-3 py-2 text-ui text-text placeholder:text-subtle shadow-xs transition-colors focus:outline-none focus-visible:border-accent focus-visible:ring-2 focus-visible:ring-accent/40";

/// Compact number input inside range pairs.
pub const INPUT_SM: &str = "w-full rounded-sm border border-border bg-surface px-2 py-1.5 text-ui text-text shadow-xs transition-colors focus:outline-none focus-visible:border-accent focus-visible:ring-2 focus-visible:ring-accent/40";

/// Search / form section card.
pub const SECTION: &str =
    "flex flex-col gap-1.5 rounded-lg border border-border bg-panel p-1.5 shadow-xs";

/// Generic surface card.
pub const CARD: &str = "rounded-lg border border-panel-border bg-surface shadow-xs";

/// Inline text link using accent.
pub const LINK: &str = "font-medium text-accent hover:underline";

/// Muted supporting paragraph.
pub const SUPPORT: &str = "text-ui text-subtle";

/// Toolbar / panel shell.
pub const PANEL_SHELL: &str = "rounded-xl border border-panel-border bg-panel-soft shadow-xs";
