// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Mass/precursor bound inputs and adduct selector row for the summary panel.

use dioxus::prelude::*;
use ui::StyleBuilder;

use super::types::SummaryFilters;

pub(super) fn parse_filter_bound(raw: &str, empty_default: f64) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(empty_default);
    }
    trimmed.parse::<f64>().ok().filter(|v| v.is_finite())
}

fn bound_input(
    mut target: Signal<f64>,
    placeholder: &'static str,
    empty_default: f64,
    width: &'static str,
) -> Element {
    rsx! {
        input {
            r#type: "number",
            placeholder,
            value: "{target.read()}",
            oninput: move |evt| {
                if let Some(val) = parse_filter_bound(&evt.value(), empty_default) {
                    target.set(val);
                }
            },
            style: StyleBuilder::new().width(width).padding("0.3rem 0.5rem").border("1px solid #cbd5e1").border_radius("6px").font_size("0.8rem").build(),
        }
    }
}

pub(super) fn filter_controls_row(filters: SummaryFilters, adduct_options: &[String]) -> Element {
    let mut adduct_filter = filters.adduct_filter;
    let label_style = StyleBuilder::new()
        .font_size("0.7rem")
        .color("#475569")
        .font_weight("600")
        .build();
    rsx! {
        div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.75rem").align_items("flex-end").property("margin", "0.75rem 0").build(),
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.25rem").build(),
                span { style: "{label_style}", "Exact mass (Da)" }
                div { style: StyleBuilder::new().display("flex").gap("0.3rem").align_items("center").build(),
                    { bound_input(filters.mz_min, "min", 0.0, "75px") }
                    span { style: StyleBuilder::new().color("#94a3b8").font_size("0.9rem").build(), "–" }
                    { bound_input(filters.mz_max, "max", 5000.0, "75px") }
                }
            }
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.25rem").build(),
                span { style: "{label_style}", "Precursor m/z" }
                div { style: StyleBuilder::new().display("flex").gap("0.3rem").align_items("center").build(),
                    { bound_input(filters.precursor_min, "min", 0.0, "80px") }
                    span { style: StyleBuilder::new().color("#94a3b8").font_size("0.9rem").build(), "–" }
                    { bound_input(filters.precursor_max, "max", 5000.0, "80px") }
                }
            }
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.25rem").build(),
                span { style: "{label_style}", "adduct" }
                select {
                    value: "{adduct_filter.read()}",
                    onchange: move |evt| adduct_filter.set(evt.value()),
                    style: StyleBuilder::new().width("110px").padding("0.3rem 0.5rem").border("1px solid #cbd5e1").border_radius("6px").font_size("0.8rem").build(),
                    option { value: "", "All" }
                    for opt in adduct_options {
                        option { key: "{opt}", value: "{opt}", "{opt}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_filter_bound;

    #[test]
    fn parse_filter_bound_handles_empty_numbers_and_non_finite_text() {
        assert_eq!(parse_filter_bound("", 0.0), Some(0.0));
        assert_eq!(parse_filter_bound("   ", 5000.0), Some(5000.0));
        assert_eq!(parse_filter_bound(" 123.5 ", 0.0), Some(123.5));
        assert_eq!(parse_filter_bound("NaN", 0.0), None);
        assert_eq!(parse_filter_bound("inf", 5000.0), None);
        assert_eq!(parse_filter_bound("abc", 0.0), None);
    }
}
