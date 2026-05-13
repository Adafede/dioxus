// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
#[must_use]
pub(super) fn resolve_row_height_px(
    measured_row_height_px: usize,
    fallback_row_height_px: usize,
) -> usize {
    if measured_row_height_px > 0 {
        measured_row_height_px
    } else {
        fallback_row_height_px.max(1)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[must_use]
pub(super) fn next_first_visible_row(
    scroll_top_px: usize,
    row_height_px: usize,
    total_rows: usize,
) -> usize {
    if row_height_px == 0 {
        return 0;
    }
    (scroll_top_px / row_height_px).min(total_rows)
}

#[cfg(target_arch = "wasm32")]
fn measured_row_height_px(
    scroll_host: &web_sys::HtmlElement,
    fallback_row_height_px: usize,
) -> usize {
    let measured = scroll_host
        .get_elements_by_class_name("data-row")
        .item(0)
        .and_then(|row| row.dyn_into::<web_sys::HtmlElement>().ok())
        .map(|row| row.offset_height().max(0) as usize)
        .unwrap_or(0);
    resolve_row_height_px(measured, fallback_row_height_px)
}

#[cfg(target_arch = "wasm32")]
pub(super) fn schedule_virtual_scroll_frame(
    mut scroll_host: Signal<Option<web_sys::HtmlElement>>,
    mut scroll_raf_scheduled: Signal<bool>,
    mut scroll_raf_cb: Signal<Option<wasm_bindgen::closure::Closure<dyn FnMut(f64)>>>,
    mut scroll_raf_id: Signal<Option<i32>>,
    scroll_id: &'static str,
    row_height_px: Signal<usize>,
    row_height_fallback_px: usize,
    total_rows: usize,
    first_visible_row: Signal<usize>,
    viewport_height_px: Signal<usize>,
) {
    let div = if let Some(existing) = scroll_host.peek().as_ref() {
        existing.clone()
    } else {
        let Some(win) = web_sys::window() else {
            return;
        };
        let Some(document) = win.document() else {
            return;
        };
        let Some(node) = document.get_element_by_id(scroll_id) else {
            return;
        };
        let Ok(found) = node.dyn_into::<web_sys::HtmlElement>() else {
            return;
        };
        *scroll_host.write() = Some(found.clone());
        found
    };

    if *scroll_raf_scheduled.peek() {
        return;
    }
    *scroll_raf_scheduled.write() = true;

    let mut first_visible_row_sig = first_visible_row;
    let mut viewport_height_px_sig = viewport_height_px;
    let mut row_height_px_sig = row_height_px;
    let mut scroll_raf_scheduled_sig = scroll_raf_scheduled;
    let mut scroll_raf_cb_sig = scroll_raf_cb;
    let mut scroll_raf_id_sig = scroll_raf_id;
    let div_for_raf = div.clone();
    let raf_cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |_ts: f64| {
        let top = div_for_raf.scroll_top().max(0) as usize;
        let height = div_for_raf.client_height().max(0) as usize;
        let measured_row_height = measured_row_height_px(&div_for_raf, row_height_fallback_px);
        if measured_row_height != *row_height_px_sig.peek() {
            *row_height_px_sig.write() = measured_row_height;
        }
        let next_first = next_first_visible_row(top, measured_row_height, total_rows);
        if next_first != *first_visible_row_sig.peek() {
            *first_visible_row_sig.write() = next_first;
        }
        if height > 0 && height != *viewport_height_px_sig.peek() {
            *viewport_height_px_sig.write() = height;
        }
        *scroll_raf_id_sig.write() = None;
        *scroll_raf_scheduled_sig.write() = false;
        *scroll_raf_cb_sig.write() = None;
    }) as Box<dyn FnMut(f64)>);

    *scroll_raf_cb.write() = Some(raf_cb);
    let scheduled_id = if let Some(win) = web_sys::window() {
        if let Some(cb) = scroll_raf_cb.peek().as_ref() {
            win.request_animation_frame(cb.as_ref().unchecked_ref())
                .ok()
        } else {
            None
        }
    } else {
        None
    };
    if let Some(id) = scheduled_id {
        *scroll_raf_id.write() = Some(id);
    } else {
        *scroll_raf_id.write() = None;
        *scroll_raf_scheduled.write() = false;
        *scroll_raf_cb.write() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_row_height_uses_measurement_when_available() {
        assert_eq!(resolve_row_height_px(92, 114), 92);
    }

    #[test]
    fn resolve_row_height_falls_back_when_measurement_is_missing() {
        assert_eq!(resolve_row_height_px(0, 114), 114);
        assert_eq!(resolve_row_height_px(0, 0), 1);
    }

    #[test]
    fn scroll_top_maps_to_first_visible_row() {
        assert_eq!(next_first_visible_row(0, 100, 50), 0);
        assert_eq!(next_first_visible_row(250, 100, 50), 2);
        assert_eq!(next_first_visible_row(990, 100, 50), 9);
    }

    #[test]
    fn next_first_visible_row_clamps_to_total_rows() {
        assert_eq!(next_first_visible_row(50_000, 100, 7), 7);
    }

    #[test]
    fn zero_row_height_is_safe() {
        assert_eq!(next_first_visible_row(50, 0, 7), 0);
    }
}
