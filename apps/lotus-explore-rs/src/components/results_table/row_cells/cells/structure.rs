// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Structure depiction cell for results-table rows.
//!
//! Renders a depiction image (lazy-loaded) when available, otherwise a dash.

use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_chemical_structure};
use dioxus::prelude::*;

pub(in crate::components::results_table::row_cells) fn structure_cell(
    locale: Locale,
    text: RowText,
    depict_url: Option<std::sync::Arc<str>>,
    name: &str,
) -> Element {
    rsx! {
        td { class: "px-3 py-2 align-middle",
            if let Some(url) = depict_url {
                a {
                    href: "{url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_full_size_depiction}",
                    img {
                        src: "{url}",
                        alt: "{aria_chemical_structure(locale, name)}",
                        loading: "lazy",
                        width: "120",
                        height: "72",
                        class: "block h-auto w-full max-w-[108px] rounded-md bg-transparent object-contain",
                    }
                }
            } else {
                span { class: "text-subtle", "-" }
            }
        }
    }
}
