use dioxus::prelude::*;

use crate::metrics::PlotPoint;
use crate::plotting::{
    make_svg_responsive, render_absolute_mass_bias_svg, render_ecdf_svg, render_mass_bias_svg,
    tolerance_step_color,
};

#[cfg(target_arch = "wasm32")]
use super::browser::download_svg;

fn format_value(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "n/a".to_string()
    }
}

pub fn format_count_with_percentage(count: usize, total: usize) -> String {
    if total == 0 {
        format!("{count} (0.0%)")
    } else {
        let count = u32::try_from(count).unwrap_or(u32::MAX);
        let total = u32::try_from(total).unwrap_or(u32::MAX);
        let pct = (f64::from(count) / f64::from(total)) * 100.0;
        format!("{count} ({pct:.1}%)")
    }
}

pub fn format_cumulative_bucket_count(
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

pub fn tolerance_card_style(index: usize) -> String {
    let color = tolerance_step_color(index, 5);
    format!(
        "padding: 0.6rem 0.7rem; border-radius: 12px; border: 1px solid {color}; background: #f8fafc; color: {color};"
    )
}

pub fn estimate_compliance_mda(errors: &[f64], threshold_mda: f64) -> f64 {
    if errors.is_empty() {
        return 0.0;
    }
    let dalton_threshold = threshold_mda / 1000.0;
    let count = errors
        .iter()
        .filter(|e| e.abs() <= dalton_threshold)
        .count();
    #[allow(clippy::cast_precision_loss)]
    {
        (count as f64 / errors.len() as f64) * 100.0
    }
}

pub fn estimate_compliance_ppm(errors: &[f64], threshold_ppm: f64) -> f64 {
    if errors.is_empty() {
        return 0.0;
    }
    let count = errors.iter().filter(|e| e.abs() <= threshold_ppm).count();
    #[allow(clippy::cast_precision_loss)]
    {
        (count as f64 / errors.len() as f64) * 100.0
    }
}

fn plot_shell(
    title: String,
    subtitle: String,
    svg_markup: String,
    _download_markup: Option<String>,
) -> Element {
    let title_for_display = title;
    let _title_for_download = title_for_display.clone();
    let subtitle_for_display = subtitle;
    rsx! {
        div {
            style: "padding: 0.95rem; border: 1px solid #e2e8f0; border-radius: 18px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%); box-shadow: 0 12px 24px rgba(15, 23, 42, 0.04);",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.6rem; margin-bottom: 0.65rem;",
                div { style: "flex: 1;",
                    h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title_for_display}" }
                    p { style: "margin: 0; color: #64748b; font-size: 0.84rem;", "{subtitle_for_display}" }
                }
                button {
                    r#type: "button",
                    style: "border: 1px solid #cbd5e1; border-radius: 999px; background: white; color: #334155; font-size: 0.76rem; font-weight: 700; padding: 0.35rem 0.65rem; cursor: pointer;",
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        if let Some(download_markup) = _download_markup.as_ref() {
                            download_svg(download_markup, &_title_for_download);
                        }
                    },
                    "Download"
                }
            }
            div { style: "border-radius: 16px; overflow: visible; border: 1px solid #e2e8f0; background: #fcfdff;",
                dangerous_inner_html: svg_markup
            }
        }
    }
}

#[component]
pub fn ecdf_plot(
    title: String,
    subtitle: String,
    values: Vec<f64>,
    thresholds: Vec<f64>,
    unit: String,
) -> Element {
    let title_for_svg = title.clone();
    let values_for_svg = values;
    let thresholds_for_svg = thresholds;
    let unit_for_svg = unit;
    let svg_markup = use_memo(move || {
        make_svg_responsive(render_ecdf_svg(
            &title_for_svg,
            &values_for_svg,
            &thresholds_for_svg,
            &unit_for_svg,
        ))
    });
    let svg_markup = svg_markup.read().clone();
    let download_markup = Some(svg_markup.clone());
    plot_shell(title, subtitle, svg_markup, download_markup)
}

#[component]
pub fn mass_bias_plot(
    title: String,
    subtitle: String,
    points: Vec<PlotPoint>,
    other_label: Option<String>,
) -> Element {
    let title_for_svg = title.clone();
    let points_for_svg = points;
    let svg_markup = use_memo(move || {
        make_svg_responsive(render_mass_bias_svg(&title_for_svg, &points_for_svg))
    });
    let svg_markup = svg_markup.read().clone();
    let _ = other_label;
    let download_markup = Some(svg_markup.clone());
    plot_shell(title, subtitle, svg_markup, download_markup)
}

#[component]
pub fn absolute_mass_bias_plot(
    title: String,
    subtitle: String,
    points: Vec<PlotPoint>,
    unit: String,
    ticks: Vec<f64>,
) -> Element {
    let title_for_svg = title.clone();
    let points_for_svg = points;
    let unit_for_svg = unit;
    let ticks_for_svg = ticks;
    let svg_markup = use_memo(move || {
        make_svg_responsive(render_absolute_mass_bias_svg(
            &title_for_svg,
            &points_for_svg,
            &unit_for_svg,
            &ticks_for_svg,
        ))
    });
    let svg_markup = svg_markup.read().clone();
    let download_markup = Some(svg_markup.clone());
    plot_shell(title, subtitle, svg_markup, download_markup)
}

pub fn format_value_text(value: f64) -> String {
    format_value(value)
}

pub fn format_bucket_text(
    metrics: &crate::metrics::PrecursorStats,
    bucket: &str,
    total: usize,
) -> String {
    format_cumulative_bucket_count(metrics, bucket, total)
}

pub fn tolerance_style(index: usize) -> String {
    tolerance_card_style(index)
}
