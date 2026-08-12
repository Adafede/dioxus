// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::use_toolbar_result_snapshot;
use crate::i18n::{CountNoun, TextKey, count_label, format_count, t};
use crate::models::DatasetStats;
use crate::state::use_results_context;
use crate::ui::style_constants::{StatStripe, spacing, text, typography};
use dioxus::prelude::*;
use ui::prelude::*;

#[component]
fn StatBadge(
    value: usize,
    secondary_value: Option<usize>,
    secondary_label: Option<&'static str>,
    noun: CountNoun,
    plus: bool,
    stripe: StatStripe,
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
        div { style: stat_badge_style(stripe),
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
                stripe: StatStripe::Compound,
            }
            StatBadge {
                value: stats.n_taxa,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Taxon,
                plus: false,
                stripe: StatStripe::Taxon,
            }
            StatBadge {
                value: stats.n_references,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Reference,
                plus: false,
                stripe: StatStripe::Reference,
            }
            StatBadge {
                value: entries_value,
                secondary_value: (entries_unique_value != entries_value).then_some(entries_unique_value),
                secondary_label: Some(t(locale, TextKey::Unique)),
                noun: CountNoun::Entry,
                plus: false,
                stripe: StatStripe::Entries,
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
            NoticeBar {
                label: t(locale, TextKey::Notice).to_string(),
                tone: NoticeTone::Warning,
                role: "status",
                aria_live: "polite",
                dark: true,
                margin: "10px 0 0",
                span { style: notice_value_style(), "{t(locale, TextKey::DisplayCappedHint)}" }
            }
        }
    }
}

fn stat_badge_style(stripe: StatStripe) -> String {
    let stripe_color = stripe.as_color();
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap(spacing::STAT_BADGE_GAP)
        .property("min-width", "0")
        .padding(spacing::STAT_BADGE_PAD)
        .border_radius("12px")
        .property(
            "border",
            &format!(
                "1px solid {}",
                crate::ui::style_constants::borders::RESULTS_BORDER
            ),
        )
        .background_color(crate::ui::style_constants::backgrounds::SURFACE)
        .box_shadow(crate::ui::style_constants::shadows::SHADOW_XS)
        .property("position", "relative")
        .property("overflow", "hidden")
        .property("flex", "1 1 0")
        .property("border-left", &format!("3px solid {stripe_color}"))
        .build()
}

fn stat_value_style() -> String {
    StyleBuilder::new()
        .font_size(typography::FONT_SIZE_STAT)
        .font_weight(typography::FONT_WEIGHT_BOLD)
        .color(text::PRIMARY)
        .property("font-variant-numeric", "tabular-nums")
        .property("letter-spacing", typography::LETTER_SPACING_STAT)
        .property("min-width", "0")
        .property("flex", "0 1 auto")
        .property("line-height", typography::LINE_HEIGHT_STAT)
        .build()
}

fn stat_secondary_style() -> String {
    StyleBuilder::new()
        .font_size(typography::FONT_SIZE_0)
        .font_weight(typography::FONT_WEIGHT_SEMIBOLD)
        .color(text::PRIMARY)
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
        .property(
            "grid-template-columns",
            "repeat(auto-fit, minmax(120px, 1fr))",
        )
        .gap(spacing::STAT_BAR_GAP)
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
        .gap(spacing::STAT_VALUE_GAP)
        .property("min-width", "0")
        .property("width", "100%")
        .justify_content("center")
        .build()
}

fn stat_label_style() -> String {
    StyleBuilder::new()
        .font_size(typography::FONT_SIZE_0)
        .color(text::SECONDARY)
        .property("text-transform", "uppercase")
        .property("letter-spacing", typography::LETTER_SPACING_TITLE)
        .font_weight(typography::FONT_WEIGHT_SEMIBOLD)
        .property("width", "100%")
        .text_align("center")
        .build()
}

fn notice_value_style() -> String {
    crate::ui::style_constants::shared::notice_value_style()
}
