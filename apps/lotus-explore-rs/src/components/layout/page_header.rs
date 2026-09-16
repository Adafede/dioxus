// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Page header: brand title, language switcher, view switcher, subtitle, archive note.
//!
//! Zero props -- all data comes from context (use_locale, AppStateContext).

use crate::components::layout::dark_mode_toggle::DarkModeToggle;
use crate::components::layout::lang_switch::LangSwitch;
use crate::components::layout::view_switch::ViewSwitch;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::ui::a11y_contract::PAGE_TITLE_ID;
use crate::ui::classes;
use dioxus::prelude::*;

/// Full page header section.
///
/// Composes `LangSwitch` (EN/FR/DE/IT), `DarkModeToggle` (light/dark), and
/// `ViewSwitch` (Search / Curation / Structure editor) as context-aware
/// children. Zero props -- only re-renders when locale or view changes.
#[component]
pub fn PageHeader() -> Element {
    let locale = use_locale();

    rsx! {
        header {
            class: "sticky top-0 z-3 min-h-[46px] bg-panel/92 backdrop-blur-sm border-b border-panel-border {classes::SHADOW_XS} px-4 sm:px-8",
            div {
                class: "flex flex-wrap items-start justify-between gap-3 sm:gap-4",
                h1 { id: PAGE_TITLE_ID,
                    class: "text-display font-bold truncate",
                    a {
                        href: "/dioxus/lotus-explore-rs/",
                        class: "text-inherit no-underline hover:no-underline",
                        aria_label: "{t(locale, TextKey::GoToHomepage)}",
                        "{t(locale, TextKey::PageTitle)}"
                    }
                }
                div {
                    class: "flex flex-wrap items-center gap-2 shrink-0",
                    ViewSwitch {}
                    LangSwitch {}
                    DarkModeToggle {}
                }
            }
            p {
                class: "max-w-[72ch] text-title text-critical-muted mt-3 pb-2",
                "{t(locale, TextKey::PageSubtitle)}"
            }
        }
    }
}
