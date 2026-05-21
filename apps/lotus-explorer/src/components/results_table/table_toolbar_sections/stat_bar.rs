// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{CountNoun, TextKey, count_label, format_count, t};
use crate::models::DatasetStats;
use crate::state::use_results_context;
use dioxus::prelude::*;

#[component]
fn StatBadge(
    value: usize,
    secondary_value: Option<usize>,
    secondary_label: Option<&'static str>,
    noun: CountNoun,
    plus: bool,
) -> Element {
    let locale = crate::hooks::use_locale();
    let display_value = if plus {
        format!("{}+", format_count(locale, value))
    } else {
        format_count(locale, value)
    };
    let label = count_label(locale, noun, value);
    rsx! {
        div { class: "stat-badge",
            div { class: "stat-value-row",
                span { class: "stat-value", "{display_value}" }
                if let Some(secondary) = secondary_value {
                    div { class: "stat-secondary-row",
                        span { class: "stat-value-secondary mono", "{format_count(locale, secondary)}" }
                        if let Some(label) = secondary_label {
                            span { class: "stat-secondary-label", "{label}" }
                        }
                    }
                }
            }
            span { class: "stat-label", "{label}" }
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
    let entries: Memo<crate::models::Rows> = use_memo(move || entries_arc.read().0.clone());
    let total_stats = crate::features::explore::selectors::use_result_selector(explore, |result| {
        result.total_stats.clone()
    });
    let total_matches =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.total_matches
        });

    let fallback_stats: Memo<DatasetStats> =
        use_memo(move || DatasetStats::from_entries(entries.read().as_ref()));
    let stats = total_stats
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_else(|| fallback_stats.read().clone());
    let entries_value = total_matches.read().unwrap_or(stats.n_entries);
    let entries_unique_value = stats.n_entries_unique;

    rsx! {
        div {
            class: "stat-bar",
            role: "group",
            aria_label: "{t(locale, TextKey::DatasetStatistics)}",
            StatBadge {
                value: stats.n_compounds,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Compound,
                plus: false,
            }
            StatBadge {
                value: stats.n_taxa,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Taxon,
                plus: false,
            }
            StatBadge {
                value: stats.n_references,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Reference,
                plus: false,
            }
            StatBadge {
                value: entries_value,
                secondary_value: (entries_unique_value != entries_value).then_some(entries_unique_value),
                secondary_label: Some(t(locale, TextKey::Unique)),
                noun: CountNoun::Entry,
                plus: false,
            }
        }
    }
}

#[component]
pub fn CappedRowsNotice() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let display_capped_rows =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.display_capped_rows
        });

    rsx! {
        if *display_capped_rows.read() {
            div { class: "notice notice-warn", role: "status",
                span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
                span { class: "notice-value", "{t(locale, TextKey::DisplayCappedHint)}" }
            }
        }
    }
}
