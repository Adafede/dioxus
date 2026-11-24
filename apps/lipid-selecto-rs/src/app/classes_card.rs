// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Collapsible "Available Lipid Classes" card grouped by LIPID MAPS family.

use dioxus::prelude::*;
use lipid_core::{ChemicalClass, LipidFamily, lmsd_classes, lmsd_classes_by_family};
use ui::prelude::*;

/// Displays all 86 compiled LMSD main classes organized into expandable families.
#[component]
pub(super) fn LipidClassesCard() -> Element {
    let families = lmsd_classes_by_family();
    let family_expanded = use_signal(Vec::<usize>::new);
    let total_classes = lmsd_classes().len();
    let fam_count = families.len();
    let light = ColorScheme::LIGHT;
    let card_style = StyleBuilder::new()
        .property("background", "rgba(255,255,255,0.9)")
        .border("1px solid rgba(148,163,184,0.22)")
        .border_radius("20px")
        .box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)")
        .padding("1.25rem")
        .property("margin-bottom", "1.25rem")
        .build();

    rsx! {
        div { style: card_style,
            h2 { style: StyleBuilder::new().margin("0 0 0.5rem").font_size("1.1rem").build(), "Available Lipid Classes" }
            p { style: StyleBuilder::new().margin("0 0 0.75rem").color(light.text3).font_size("0.8rem").build(),
                "LMSD classification scheme — {total_classes} classes across {fam_count} families"
            }
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.3rem").property("max-height", "320px").property("overflow-y", "auto").build(),
                for (idx, (family, classes)) in families.iter().enumerate() {
                    { family_group_row(idx, *family, classes, family_expanded) }
                }
            }
        }
    }
}

fn family_group_row(
    idx: usize,
    family: LipidFamily,
    classes: &[&'static ChemicalClass],
    mut family_expanded: Signal<Vec<usize>>,
) -> Element {
    let family_color = classes.first().map_or("#cbd5e1", |c| c.color);
    let is_expanded = family_expanded.read().contains(&idx);
    let row_style = StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .justify_content("space-between")
        .padding("0.35rem 0.5rem")
        .cursor("pointer")
        .border_bottom("1px solid #f1f5f9")
        .build();
    let pill_style = StyleBuilder::new()
        .padding("0.3rem 0.5rem")
        .property("background", "#f1f5f9")
        .border("1px solid #e2e8f0")
        .border_radius("6px")
        .font_size("0.75rem")
        .build();

    rsx! {
        div {
            div {
                style: row_style,
                onclick: move |_| {
                    let mut expanded = family_expanded.read().clone();
                    if expanded.contains(&idx) {
                        expanded.retain(|&i| i != idx);
                    } else {
                        expanded.push(idx);
                    }
                    family_expanded.set(expanded);
                },
                div { style: StyleBuilder::new().display("flex").align_items("center").gap("0.3rem").build(),
                    span { style: StyleBuilder::new().width("10px").height("10px").border_radius("50%").property("background", family_color).build(), }
                    strong { style: StyleBuilder::new().font_size("0.82rem").font_weight("700").color(ColorScheme::LIGHT.text).build(), "{family.name()}" }
                }
                span { style: StyleBuilder::new().font_size("0.72rem").color("#94a3b8").build(), "{classes.len()} classes" }
            }
            if is_expanded {
                div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.3rem").property("padding", "0.4rem 0 0.2rem 0.8rem").build(),
                    for class in classes {
                        div {
                            title: "{class.smarts}",
                            style: "{pill_style}",
                            strong { style: StyleBuilder::new().property("word-break", "break-word").display("inline-flex").align_items("center").gap("0.25rem").build(),
                                span { style: StyleBuilder::new().width("6px").height("6px").border_radius("50%").property("background", class.color).build(), }
                                "{class.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
