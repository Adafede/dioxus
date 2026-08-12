// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::use_toolbar_result_snapshot;
use crate::i18n::{CountNoun, TextKey, count_label, format_count, t};
use crate::models::DatasetStats;
use crate::state::use_results_context;
use dioxus::prelude::*;
use ui::prelude::*;

#[component]
fn StatBadge(
    value: usize,
    secondary_value: Option<usize>,
    secondary_label: Option<&'static str>,
    noun: CountNoun,
    plus: bool,
    stripe_color: &'static str,
) -> Element {
    let locale = crate::hooks::use_locale();
    let mut display_value = format_count(locale, value);
    if plus {
        display_value.push('+');
    }
    let label = count_label(locale, noun, value);
    let secondary_inline = secondary_value.map(|secondary| {
        secondary_label.map_or_else(
            || format!("({})", format_count(locale, secondary)),
            |label| {
                let inline_label = label.to_lowercase();
                format!("({} {inline_label})", format_count(locale, secondary))
            },
        )
    });
    rsx! {
        div { style: stat_badge_style(stripe_color),
            div { style: stat_value_row_style(),
                span { style: stat_value_style(), "{display_value}" }
                if let Some(secondary_text) = secondary_inline.as_ref() {
                    span { style: stat_secondary_style(),
                        "{secondary_text}"
                    }
                }
            }
            span { style: stat_label_style(), "{label}" }
        }
    }
}

#[component]
pub fn StatBar() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let entries_arc =
        crate::features::explore::selectors::use_result_arc_selector(explore, |result| {
            result.entries.clone()
        });
    let toolbar_snapshot = use_toolbar_result_snapshot(explore);
    let fallback_stats: Memo<DatasetStats> =
        use_memo(move || DatasetStats::from_entries(entries_arc.read().0.as_ref()));
    let snapshot_ref = toolbar_snapshot.read();
    let fallback_stats_ref = fallback_stats.read();
    let stats = snapshot_ref
        .total_stats
        .as_ref()
        .unwrap_or(&fallback_stats_ref);
    let entries_value = snapshot_ref.total_matches.unwrap_or(stats.n_entries);
    let entries_unique_value = stats.n_entries_unique;

    rsx! {
        div {
            role: "group",
            aria_label: "{t(locale, TextKey::DatasetStatistics)}",
            style: stat_bar_style(),
            StatBadge {
                value: stats.n_compounds,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Compound,
                plus: false,
                stripe_color: "var(--wd-compound-stripe)",
            }
            StatBadge {
                value: stats.n_taxa,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Taxon,
                plus: false,
                stripe_color: "var(--wd-taxon-stripe)",
            }
            StatBadge {
                value: stats.n_references,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Reference,
                plus: false,
                stripe_color: "var(--wd-reference-stripe)",
            }
            StatBadge {
                value: entries_value,
                secondary_value: (entries_unique_value != entries_value).then_some(entries_unique_value),
                secondary_label: Some(t(locale, TextKey::Unique)),
                noun: CountNoun::Entry,
                plus: false,
                stripe_color: "var(--wd-entries-stripe)",
            }
        }
    }
}

#[component]
pub fn CappedRowsNotice() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let toolbar_snapshot = use_toolbar_result_snapshot(explore);

    rsx! {
        if toolbar_snapshot.read().display_capped_rows {
            div { role: "status", style: notice_base_style(),
                span { style: notice_label_style(), "{t(locale, TextKey::Notice)}" }
                span { style: notice_value_style(), "{t(locale, TextKey::DisplayCappedHint)}" }
            }
        }
    }
}

fn stat_badge_style(stripe_color: &str) -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("4px")
        .property("min-width", "0")
        .padding("10px 12px")
        .border_radius("12px")
        .border("1px solid var(--results-border)")
        .background_color("var(--surface)")
        .box_shadow("var(--shadow-xs)")
        .property("position", "relative")
        .property("overflow", "hidden")
        .property("flex", "1 1 0")
        .property("border-left", &format!("3px solid {stripe_color}"))
        .build()
}

fn stat_value_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-stat)")
        .font_weight("800")
        .color("var(--text)")
        .property("font-variant-numeric", "tabular-nums")
        .property("letter-spacing", "-0.02em")
        .property("min-width", "0")
        .property("flex", "0 1 auto")
        .property("line-height", "1.2")
        .build()
}

fn stat_secondary_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .font_weight("700")
        .color("var(--text)")
        .property("font-variant-numeric", "tabular-nums")
        .property("min-width", "0")
        .property("max-width", "100%")
        .property("overflow-wrap", "anywhere")
        .property("flex", "0 0 auto")
        .build()
}

fn stat_bar_style() -> String {
    StyleBuilder::new()
        .display("grid")
        .property("grid-template-columns", "repeat(4, minmax(0, 1fr))")
        .gap("10px")
        .align_items("stretch")
        .property("width", "100%")
        .property("min-width", "0")
        .build()
}

fn stat_value_row_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .align_items("baseline")
        .gap("8px")
        .property("min-width", "0")
        .property("width", "100%")
        .justify_content("center")
        .build()
}

fn stat_label_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .color("var(--text2)")
        .property("text-transform", "uppercase")
        .property("letter-spacing", "0.08em")
        .font_weight("700")
        .property("width", "100%")
        .text_align("center")
        .build()
}

fn notice_base_style() -> String {
    StyleBuilder::new()
        .margin("10px 24px 0")
        .padding("9px 12px")
        .display("flex")
        .align_items("center")
        .gap("12px")
        .border_radius("var(--radius)")
        .font_size("var(--fs-0)")
        .border("1px solid var(--panel-border)")
        .background_color("var(--panel-bg-soft)")
        .box_shadow("var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn notice_label_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .property("text-transform", "uppercase")
        .property("letter-spacing", "1px")
        .font_size("var(--fs-label)")
        .font_weight("700")
        .property("line-height", "1.4")
        .padding("2px 6px")
        .border_radius("3px")
        .property("flex-shrink", "0")
        .build()
}

fn notice_value_style() -> String {
    StyleBuilder::new()
        .property("flex", "1")
        .color("var(--text)")
        .property("word-break", "break-word")
        .property("line-height", "1.4")
        .build()
}
