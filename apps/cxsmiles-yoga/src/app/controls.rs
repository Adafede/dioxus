// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Input controls and example picker items for `cxsmiles-yoga`.

use dioxus::prelude::*;
use ui::{ColorScheme, Radius, SegmentedControlItem, Shadow, Spacing, StyleBuilder, Typography};

use crate::examples::{EXAMPLE_SETS, ExampleSet};

const fn to_item(set: &ExampleSet) -> SegmentedControlItem {
    SegmentedControlItem {
        label: set.label,
        value: set.id,
    }
}

const EXAMPLE_ITEMS: [SegmentedControlItem; 6] = {
    let [ref a, ref b, ref c, ref d, ref e, ref f] = EXAMPLE_SETS;
    [
        to_item(a),
        to_item(b),
        to_item(c),
        to_item(d),
        to_item(e),
        to_item(f),
    ]
};

/// `SegmentedControl` items backing the example selector bar.
#[must_use]
pub(super) const fn example_items() -> &'static [SegmentedControlItem] {
    &EXAMPLE_ITEMS
}

/// SMILES textarea input card with live non-empty line count and aromatic option.
#[component]
pub(super) fn InputCard(input: Signal<String>, aromatic: Signal<bool>) -> Element {
    let light = ColorScheme::LIGHT;
    let line_count = input
        .read()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    let card_style = StyleBuilder::new()
        .background_color(light.surface)
        .border(&format!("1px solid {}", light.border))
        .border_radius(Radius::LG)
        .box_shadow(Shadow::MD)
        .padding(Spacing::LG)
        .property("margin-bottom", Spacing::LG)
        .build();
    let header_style = StyleBuilder::new()
        .display("flex")
        .justify_content("space-between")
        .align_items("baseline")
        .property("margin-bottom", "0.5rem")
        .build();
    let area_style = StyleBuilder::new()
        .width("100%")
        .property("min-height", "140px")
        .font_family("ui-monospace, monospace")
        .font_size(Typography::UI)
        .padding("0.6rem 0.7rem")
        .border(&format!("1px solid {}", light.border))
        .border_radius(Radius::MD)
        .property("resize", "vertical")
        .color(light.text)
        .build();
    let check_row_style = StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .gap("0.5rem")
        .property("margin-top", "0.6rem")
        .font_size(Typography::UI)
        .color(light.text3)
        .build();

    rsx! {
        div { style: card_style,
            div { style: header_style,
                label { r#for: "smiles-input", style: StyleBuilder::new().font_size(Typography::UI).font_weight("600").color(light.text).build(), "SMILES list" }
                span { style: StyleBuilder::new().font_size(Typography::LABEL).color(light.text3).build(), "{line_count} line(s)" }
            }
            textarea {
                id: "smiles-input",
                style: area_style,
                value: "{input.read()}",
                placeholder: "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1",
                oninput: move |e| input.set(e.value()),
            }
            div { style: check_row_style,
                input {
                    r#type: "checkbox",
                    id: "aromatic-equiv",
                    checked: *aromatic.read(),
                    onchange: move |_| {
                        let cur = *aromatic.read();
                        aromatic.set(!cur);
                    },
                }
                label { r#for: "aromatic-equiv", "Enumerate equivalent aromatic positions" }
            }
        }
    }
}
