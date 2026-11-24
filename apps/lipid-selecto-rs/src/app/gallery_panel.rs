// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Filtered lipid gallery card grid.

use dioxus::prelude::*;
use lipid_core::{Analysis, FilterCriteria, GalleryItem, passes_item_filters};
use ui::{ColorScheme, StyleBuilder};

use super::types::SummaryFilters;

fn gallery_class_badge(item: &GalleryItem) -> Element {
    let bg_color = &item.primary_class_color;
    let badge_label = item
        .primary_class_name
        .as_deref()
        .or_else(|| item.classification.as_ref().map(|c| c.family.category()));
    let badge_title = item.classification.as_ref().map_or_else(String::new, |c| {
        let src = if c.derived_from_smiles {
            "SMILES"
        } else {
            "formula"
        };
        format!("{} ({src})", c.family.category())
    });
    badge_label.map_or_else(
        || rsx! {},
        |class_name| {
            rsx! {
                div {
                    title: "{badge_title}",
                    style: StyleBuilder::new().display("inline-flex").align_items("center").gap("0.25rem").padding("0.1rem 0.45rem").property("margin-bottom", "0.2rem").border_radius("999px").property("background", &format!("{bg_color}22")).border(&format!("1px solid {bg_color}55")).color(ColorScheme::LIGHT.text).font_size("0.68rem").font_weight("600").build(),
                    span { style: StyleBuilder::new().width("6px").height("6px").border_radius("50%").property("background", bg_color).build(), }
                    "{class_name}"
                }
            }
        },
    )
}

fn gallery_card(item: &GalleryItem) -> Element {
    let precursor_text = item
        .precursor_mz
        .map_or_else(|| "—".to_string(), |mz| format!("{mz:.3}"));
    let adduct_text = item.adduct.as_deref().unwrap_or("-");
    let bg_color = &item.primary_class_color;
    let card_style = StyleBuilder::new()
        .property(
            "background",
            &format!("linear-gradient(180deg, {bg_color}15 0%, {bg_color}08 100%)"),
        )
        .padding("0.6rem 0.7rem")
        .border_radius("14px")
        .border(&format!("1px solid {bg_color}40"))
        .box_shadow("0 6px 16px rgba(15, 23, 42, 0.05)")
        .property("overflow", "hidden")
        .build();
    let svg_box_style = StyleBuilder::new()
        .flex("0 0 auto")
        .width("160px")
        .height("120px")
        .display("grid")
        .property("place-items", "center")
        .property("background", &format!("{bg_color}10"))
        .border(&format!("1px solid {bg_color}30"))
        .border_radius("10px")
        .property("overflow", "hidden")
        .build();

    rsx! {
        div { style: card_style,
            div { style: StyleBuilder::new().display("flex").gap("0.55rem").align_items("flex-start").build(),
                div { style: svg_box_style,
                    div { style: StyleBuilder::new().width("100%").height("100%").display("flex").align_items("center").justify_content("center").build(),
                        div { dangerous_inner_html: item.svg.as_str() }
                    }
                }
                div { style: StyleBuilder::new().flex("1 1 auto").property("min-width", "0").build(),
                    if let Some(title) = &item.title {
                        div { style: StyleBuilder::new().property("margin-bottom", "0.2rem").color(ColorScheme::LIGHT.text).font_size("0.82rem").font_weight("600").build(), "{title}" }
                    }
                    { gallery_class_badge(item) }
                    if let Some(smiles) = &item.smiles {
                        div { style: StyleBuilder::new().property("margin-bottom", "0.15rem").color(ColorScheme::LIGHT.text3).font_size("0.72rem").property("font-family", "ui-monospace, monospace").property("max-width", "100%").property("overflow", "hidden").property("text-overflow", "ellipsis").property("white-space", "nowrap").build(), "{smiles}" }
                    }
                    div { style: StyleBuilder::new().color(ColorScheme::LIGHT.text3).font_size("0.75rem").build(),
                        "exact mass "
                        strong { style: StyleBuilder::new().color(ColorScheme::LIGHT.text).build(), "{item.exact_mass:.3} Da" }
                        " · precursor "
                        strong { style: StyleBuilder::new().color(ColorScheme::LIGHT.text).build(), "{precursor_text}" }
                        " · adduct "
                        strong { style: StyleBuilder::new().color(ColorScheme::LIGHT.text).build(), "{adduct_text}" }
                    }
                }
            }
        }
    }
}

/// Displays filtered gallery cards matching selected classes, mass/m/z ranges, and adduct.
#[component]
pub(super) fn GalleryPanel(analysis: Signal<Option<Analysis>>, filters: SummaryFilters) -> Element {
    let guard = analysis.read();
    let Some(data) = guard.as_ref() else {
        return rsx! {};
    };
    let adduct = filters.read_adduct();
    let criteria = FilterCriteria {
        selected_classes: *filters.selected_classes.read(),
        mz_min: *filters.mz_min.read(),
        mz_max: *filters.mz_max.read(),
        precursor_min: *filters.precursor_min.read(),
        precursor_max: *filters.precursor_max.read(),
        adduct: &adduct,
    };
    let filtered: Vec<&GalleryItem> = data
        .gallery
        .iter()
        .filter(|item| {
            passes_item_filters(
                item.class_matches,
                item.exact_mass,
                item.precursor_mz,
                item.adduct.as_deref(),
                &criteria,
            )
        })
        .collect();
    let count = filtered.len();

    rsx! {
        div { style: StyleBuilder::new().property("margin-top", "1.25rem").build(),
            h2 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("1.05rem").color(ColorScheme::LIGHT.text).build(),
                "Structures matching selected classes ({count} shown)"
            }
            if count == 0 {
                p { style: StyleBuilder::new().color(ColorScheme::LIGHT.text3).build(),
                    "Select one or more chemical families to see matching structures."
                }
            } else {
                div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fill, minmax(320px, 1fr))").gap("0.75rem").build(),
                    for item in filtered {
                        { gallery_card(item) }
                    }
                }
            }
        }
    }
}
