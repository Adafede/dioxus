// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared Tailwind class fragments mapped to Lotus design tokens.
//!
//! Prefer these over raw `slate-*` / `sky-*` / `dark:` pairs so colors track
//! `data-theme` via CSS variables.
//!
//! All tokens map to CSS variables defined in `tailwind/styles.css` and
//! exposed via `@theme` in Tailwind v4.

// ============================================================================
// RADIUS SCALE
// --radius-xs: 4px, --radius-sm: 6px, --radius-md: 8px, --radius-lg: 12px,
// --radius-xl: 16px, --radius-2xl: 20px
// ============================================================================

pub const RADIUS_PILL: &str = "rounded-full"; // 9999px - pills, badges, tags (fully circular)
pub const RADIUS_SM_CTRL: &str = "rounded-md"; // 8px - small controls: checkboxes, radios, slider
pub const RADIUS_INPUT: &str = "rounded-xl"; // 16px - inputs, buttons (rounded rectangles)
pub const RADIUS_CARD: &str = "rounded-xl"; // 16px - cards, textareas, containers
pub const RADIUS_PANEL: &str = "rounded-2xl"; // 20px - major panels: main content, toolbars
pub const RADIUS_FULL: &str = "rounded-full"; // 9999px - circular: avatars, toggle icons, spinners

// ============================================================================
// COLOR PALETTE (maps to CSS variables via @theme)
// ============================================================================

// Borders
pub const BORDER_PANEL: &str = "border-panel-border";

// Wikidata organism colors (text)
pub const WD_COMPOUND: &str = "text-wd-compound";
pub const WD_TAXON: &str = "text-wd-taxon";
pub const WD_REFERENCE: &str = "text-wd-reference";
pub const WD_ENTRIES: &str = "text-wd-entries";
pub const WD_STRUCTURE: &str = "text-wd-structure";

// Wikidata organism colors (borders)
pub const WD_COMPOUND_BORDER: &str = "border-wd-compound";
pub const WD_REFERENCE_BORDER: &str = "border-wd-reference";

// ============================================================================
// SHADOWS
// --shadow-xs: 0 1px 2px (4%/30%), --shadow-md: 0 4px 8px (8%/50%)
// ============================================================================

pub const SHADOW_XS: &str = "shadow-xs";
pub const SHADOW_MD: &str = "shadow-md";

// ============================================================================
// FOCUS RING
// --focus-ring: 0 0 0 3px rgb(accent / 28%), offset 2px
// ============================================================================

/// Standard focus-visible ring using design token variables.
pub const FOCUS_RING: &str = "focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2";

/// Focus ring for buttons
pub const FOCUS_RING_BTN: &str = "focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2";

// ============================================================================
// TRANSITIONS
// --default-transition-duration: 150ms, --default-transition-timing-function: cubic-bezier(.4, 0, .2, 1)
// ============================================================================

pub const TRANSITION_TRANSFORM: &str =
    "transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)]";

// ============================================================================
// COMPONENT COMPOSITES (common patterns) - fully resolved, no interpolation
// ============================================================================

/// Field / control label.
pub const LABEL: &str = "text-body font-semibold text-text";

/// Secondary / hint text under controls.
pub const HINT: &str = "text-micro text-subtle";

/// Uppercase micro label (range min/max, etc.).
pub const MICRO_LABEL: &str = "text-micro font-semibold uppercase tracking-wide text-subtle";

/// Standard text/number input.
pub const INPUT: &str = "w-full rounded-xl border border-border bg-surface px-3 py-2 text-body text-text placeholder:text-subtle shadow-xs focus-visible:outline-none focus-visible:border-accent focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2";

/// Compact number input inside range pairs.
pub const INPUT_SM: &str = "w-full rounded-md border border-border bg-surface px-2 py-1.5 text-body text-text shadow-xs focus-visible:outline-none focus-visible:border-accent focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2";

/// Search / form section card.
pub const SECTION: &str =
    "flex flex-col gap-1.5 rounded-xl border border-border bg-panel p-1.5 shadow-xs";

/// Generic surface card (curation sections).
pub const CARD: &str =
    "flex flex-col gap-4 rounded-xl border border-panel-border bg-panel-soft p-4 shadow-xs";

/// Form grid layout.
pub const FORM_GRID: &str = "grid grid-cols-1 gap-3";

/// Actions row.
pub const ACTIONS: &str = "flex flex-wrap items-center gap-2.5";

/// Curation textarea (130px min-height).
pub const TEXTAREA_130: &str = "form-textarea mono w-full min-h-[130px] rounded-xl border border-border bg-surface p-2.5 font-mono text-body text-text shadow-xs focus:outline-none focus-visible:border-accent focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2";

/// Curation textarea (220px min-height).
pub const TEXTAREA_220: &str = "form-textarea mono w-full min-h-[220px] rounded-xl border border-border bg-surface p-2.5 font-mono text-body text-text shadow-xs focus:outline-none focus-visible:border-accent focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2";

/// Share bar (custom styling).
pub const SHARE_BAR: &str =
    "flex flex-col gap-2 p-3 rounded-xl border border-panel-border bg-panel-soft shadow-xs";
pub const SHARE_BAR_LABEL: &str = "text-ui font-semibold text-text2";

/// Inline text link using accent.
pub const LINK: &str = "font-medium text-accent hover:underline";

/// Muted supporting paragraph.
pub const SUPPORT: &str = "text-body text-subtle";

/// Primary button variant
pub const BTN_PRIMARY: &str = "inline-flex items-center justify-center font-sans select-none transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 rounded-xl bg-accent text-bg font-semibold shadow-xs hover:bg-accent-2 active:bg-accent-2";

/// Secondary button variant
pub const BTN_SECONDARY: &str = "inline-flex items-center justify-center font-sans select-none transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 rounded-xl border border-border bg-surface text-text font-semibold shadow-xs hover:bg-bg active:bg-bg";

/// Danger button variant
pub const BTN_DANGER: &str = "inline-flex items-center justify-center font-sans select-none transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 rounded-xl border border-danger/35 bg-danger/10 text-danger font-semibold hover:bg-danger/15 active:bg-danger/20";

/// Accent button variant (emphasized primary)
pub const BTN_ACCENT: &str = "inline-flex items-center justify-center font-sans select-none transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 rounded-xl border border-border bg-accent text-bg font-semibold shadow-xs ring-2 ring-accent/40 hover:bg-accent-2 active:bg-accent-2";

// ============================================================================
// INTERACTIVE STATES
// ============================================================================

/// Disabled state (consistent across buttons, inputs, etc.)
pub const DISABLED: &str = "opacity-50 cursor-not-allowed pointer-events-none";

/// Active/pressed state (scale down slightly)
pub const ACTIVE_SCALE: &str = "active:scale-[0.98] transition-transform duration-50";

// ============================================================================
// BUTTON SIZE COMPOSITES
// ============================================================================

pub const BTN_SIZE_SM: &str = "min-h-[34px] gap-1.5 px-3 py-1.5 text-ui";
pub const BTN_SIZE_MD: &str = "min-h-[40px] gap-2 px-3.5 py-2 text-ui";

// ============================================================================
// TABLE CELL COMPOSITES
// ============================================================================

/// Base table cell (td) with stripe color injected via parameter
pub const TABLE_CELL_BASE: &str = "min-w-0 rounded-xl px-3 py-2.5 align-middle text-ui";

/// Table header cell
pub const TABLE_TH: &str = "border-b border-panel-border bg-panel-soft px-3 py-2 text-left text-micro font-semibold uppercase tracking-wide text-muted";

/// Table data cell
pub const TD: &str = "border-b border-panel-border px-3 py-2.5 align-top text-ui";

/// PILL badge base (used for status badges in cells)
pub const PILL: &str = "inline-flex items-center rounded-full border border-panel-border bg-surface px-2 py-0.5 text-micro font-semibold uppercase tracking-wide";

// ============================================================================
// NOTICE BAR (Tailwind-only, replaces inline-style NoticeBar)
// ============================================================================

/// Notice bar outer container
pub const NOTICE_BAR: &str = "flex flex-wrap items-center gap-2 rounded-xl border p-2.5 shadow-xs";

/// Notice bar label pill
pub const NOTICE_LABEL: &str = "inline-flex items-center px-2 py-0.5 rounded-full font-semibold uppercase tracking-[0.08em] text-micro flex-shrink-0";

/// Notice bar body
pub const NOTICE_BODY: &str = "flex-1 min-w-0 text-ui";

// Notice tone variants (compose with NOTICE_BAR)
pub const NOTICE_WARNING: &str = "border-warning/35 bg-warning/10";

// Notice label tone variants (compose with NOTICE_LABEL)
pub const NOTICE_LABEL_WARNING: &str = "bg-warning/12 text-warning";

// ============================================================================
// QUEUE TABLE COLUMNS
// ============================================================================

pub const QUEUE_ACTION_COL: &str = "w-[110px] min-w-[110px]";
pub const QUEUE_INDEX_COL: &str = "min-w-[3ch]";
