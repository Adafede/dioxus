// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Virtualization hook encapsulating scroll handling and visible row calculation.

/// Configuration for virtual scrolling.
#[derive(Clone, Copy, Debug)]
pub struct VirtualizationConfig {
    /// Height of each row in pixels.
    pub row_height_px: usize,
    /// Number of extra rows to render outside viewport for smooth scrolling.
    pub overscan_rows: usize,
    /// Fallback viewport height if not yet measured.
    pub viewport_fallback_px: usize,
    /// DOM ID of the scroll container.
    pub scroll_id: &'static str,
}

/// Output state from virtualization hook.
#[derive(Clone, Copy, Debug)]
pub struct VirtualizationState {
    /// Index of first visible row.
    pub start_row: usize,
    /// Index of last visible row (exclusive).
    pub end_row: usize,
    /// Top padding in pixels.
    pub top_spacer_px: usize,
    /// Bottom padding in pixels.
    pub bottom_spacer_px: usize,
}

/// Hook that manages virtual scrolling state and calculations.
///
/// Always returns consistent state for SSR compatibility.
#[must_use]
pub fn use_virtualization(
    config: VirtualizationConfig,
    total_rows: usize,
    first_visible_row: usize,
    viewport_height_px: usize,
) -> VirtualizationState {
    let viewport_height_px = viewport_height_px.max(config.viewport_fallback_px);
    let window_rows = ((viewport_height_px.saturating_add(config.row_height_px - 1))
        / config.row_height_px)
        .max(1)
        .saturating_add(config.overscan_rows * 2);
    let first_row = first_visible_row.min(total_rows);
    let start_row = first_row.saturating_sub(config.overscan_rows);
    let end_row = start_row.saturating_add(window_rows).min(total_rows);
    let top_spacer_px = start_row.saturating_mul(config.row_height_px);
    let bottom_spacer_px = total_rows
        .saturating_sub(end_row)
        .saturating_mul(config.row_height_px);
    VirtualizationState {
        start_row,
        end_row,
        top_spacer_px,
        bottom_spacer_px,
    }
}
