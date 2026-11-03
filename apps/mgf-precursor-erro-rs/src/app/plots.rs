// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use dioxus::prelude::*;
use ui::theme::StyleBuilder;

use crate::metrics::PlotPoint;
use crate::plotting::{
    make_svg_responsive, render_absolute_mass_bias_svg, render_ecdf_svg, tolerance_step_color,
};

#[cfg(target_arch = "wasm32")]
use super::browser::download_svg;

pub(super) fn format_value(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "n/a".to_string()
    }
}

pub(super) fn format_count_with_percentage(count: usize, total: usize) -> String {
    if total == 0 {
        format!("{count} (0.0%)")
    } else {
        let count = u32::try_from(count).unwrap_or(u32::MAX);
        let total = u32::try_from(total).unwrap_or(u32::MAX);
        let pct = (f64::from(count) / f64::from(total)) * 100.0;
        format!("{count} ({pct:.1}%)")
    }
}

pub(super) fn format_cumulative_bucket_count(
    metrics: &crate::metrics::PrecursorStats,
    bucket: &str,
    total: usize,
) -> String {
    let count = match bucket {
        "0.1_da" => metrics.within_0_1_da,
        "0.5_da" => metrics.within_0_1_da + metrics.within_0_5_da,
        "1.0_da" => metrics.within_0_1_da + metrics.within_0_5_da + metrics.within_1_da,
        "5.0_da" => {
            metrics.within_0_1_da
                + metrics.within_0_5_da
                + metrics.within_1_da
                + metrics.within_5_da
        }
        ">5.0_da" => metrics.above_5_da,
        "0.5_ppm" => metrics.within_0_5_ppm,
        "1.0_ppm" => metrics.within_0_5_ppm + metrics.within_1_ppm,
        "5.0_ppm" => metrics.within_0_5_ppm + metrics.within_1_ppm + metrics.within_5_ppm,
        "10.0_ppm" => {
            metrics.within_0_5_ppm
                + metrics.within_1_ppm
                + metrics.within_5_ppm
                + metrics.within_10_ppm
        }
        ">10.0_ppm" => metrics.above_10_ppm,
        _ => 0,
    };
    format_count_with_percentage(count, total)
}

pub(super) fn tolerance_card_style(index: usize) -> String {
    let color = tolerance_step_color(index, 5);
    StyleBuilder::new()
        .padding("0.6rem 0.7rem")
        .border_radius("12px")
        .border(&format!("1px solid {color}"))
        .property("background", "#f8fafc")
        .color(&color)
        .build()
}

fn format_threshold_compliance(errors: &[f64], threshold: f64, above: bool) -> String {
    let count = errors
        .iter()
        .filter(|e| {
            let abs = e.abs();
            if above {
                abs > threshold
            } else {
                abs <= threshold
            }
        })
        .count();
    format_count_with_percentage(count, errors.len())
}

pub(super) fn format_recalibrated_compliance_mda(
    errors: &[f64],
    threshold_mda: f64,
    above: bool,
) -> String {
    format_threshold_compliance(errors, threshold_mda / 1000.0, above)
}

pub(super) fn format_recalibrated_compliance_ppm(
    errors: &[f64],
    threshold_ppm: f64,
    above: bool,
) -> String {
    format_threshold_compliance(errors, threshold_ppm, above)
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
#[allow(clippy::needless_pass_by_value)] // rsx! macro captures by move, clippy cannot see through macro expansion
fn plot_shell(title: &'static str, subtitle: &'static str, svg_markup: Memo<String>) -> Element {
    rsx! {
        div {
            style: StyleBuilder::new().padding("0.95rem").border("1px solid #e2e8f0").border_radius("18px").property("background", "linear-gradient(180deg, #ffffff 0%, #f8fafc 100%)").box_shadow("0 12px 24px rgba(15, 23, 42, 0.04)").build(),
            div { style: StyleBuilder::new().display("flex").align_items("center").justify_content("space-between").gap("0.6rem").property("margin-bottom", "0.65rem").build(),
                div { style: StyleBuilder::new().flex("1").build(),
                    h4 { style: StyleBuilder::new().margin("0 0 0.2rem").font_size("0.95rem").color("#0f172a").build(), "{title}" }
                    p { style: StyleBuilder::new().margin("0").color("#64748b").font_size("0.84rem").build(), "{subtitle}" }
                }
                button {
                    r#type: "button",
                    style: StyleBuilder::new().border("1px solid #cbd5e1").border_radius("999px").property("background", "white").color("#334155").font_size("0.76rem").font_weight("700").padding("0.35rem 0.65rem").cursor("pointer").build(),
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        download_svg(&svg_markup.read(), title);
                    },
                    "Download"
                }
            }
            div { style: StyleBuilder::new().border_radius("16px").property("overflow", "visible").border("1px solid #e2e8f0").property("background", "#fcfdff").build(),
                dangerous_inner_html: svg_markup.read().clone()
            }
        }
    }
}

#[component]
pub fn ecdf_plot(
    title: &'static str,
    subtitle: &'static str,
    values: Vec<f64>,
    thresholds: &'static [f64],
    unit: &'static str,
) -> Element {
    let svg_markup = use_memo(move || {
        make_svg_responsive(render_ecdf_svg(title, &values, thresholds, unit).unwrap_or_default())
    });
    plot_shell(title, subtitle, svg_markup)
}

#[component]
pub fn absolute_mass_bias_plot(
    title: &'static str,
    subtitle: &'static str,
    points: Vec<PlotPoint>,
    unit: &'static str,
    ticks: &'static [f64],
) -> Element {
    let svg_markup = use_memo(move || {
        make_svg_responsive(
            render_absolute_mass_bias_svg(title, &points, unit, ticks).unwrap_or_default(),
        )
    });
    plot_shell(title, subtitle, svg_markup)
}
