// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::api::{api_base_url, build_pubchem_trees, fetch_pubchem_dataset, resolve_api_url};
use crate::app_state::AppState;
use crate::components::stat_grid::StatsGrid;
use crate::components::tree_viewer::{InteractiveTreeViewer, TreeViewerConfig};
use crate::models::{BuildResult, BusyState, PreviewTab};
use crate::services::present_api_error;
use dioxus::prelude::*;

#[component]
pub fn AppRoot() -> Element {
    let app_state = use_signal(AppState::default);
    let mut active_tab = use_signal(|| PreviewTab::Biological);
    let api_base = api_base_url();

    let fetch_disabled = app_state.read().busy.is_busy() || api_base.is_none();
    let build_disabled = app_state.read().busy.is_busy()
        || api_base.is_none()
        || app_state.read().session_id.is_none();

    let on_fetch = {
        let app_state_fetch = app_state;
        move |_| {
            let mut app_state = app_state_fetch;
            spawn(async move {
                app_state.write().busy = BusyState::Fetching;
                app_state.write().error = None;
                app_state.write().status_message = Some(BusyState::Fetching.label().to_string());
                match fetch_pubchem_dataset().await {
                    Ok(response) => {
                        let mut state = app_state.write();
                        state.session_id = Some(response.session_id);
                        state.stats = Some(response.stats.into());
                        state.build = None;
                        state.busy = BusyState::Idle;
                        state.status_message = Some(
                            "Data fetched and processed. You can now build the tree previews."
                                .to_string(),
                        );
                    }
                    Err(error) => {
                        let mut state = app_state.write();
                        state.busy = BusyState::Idle;
                        state.error = Some(present_api_error(&error));
                        state.status_message = None;
                    }
                }
            });
        }
    };

    let on_build = {
        let app_state_build = app_state;
        let active_tab_build = active_tab;
        let api_base = api_base.clone().unwrap_or_default();
        move |_| {
            let Some(session_id) = app_state_build.read().session_id.clone() else {
                return;
            };
            let mut app_state = app_state_build;
            let mut active_tab = active_tab_build;
            let api_base = api_base.clone();
            spawn(async move {
                app_state.write().busy = BusyState::Building;
                app_state.write().error = None;
                app_state.write().status_message = Some(BusyState::Building.label().to_string());
                match build_pubchem_trees(&session_id).await {
                    Ok(response) => {
                        let mut build: BuildResult = response.into();
                        for download in &mut build.downloads {
                            download.url = resolve_api_url(&api_base, &download.url);
                        }
                        let np_available = build.npclassifier_summary.total_nodes > 0;
                        *active_tab.write() = if np_available {
                            PreviewTab::Npclassifier
                        } else {
                            PreviewTab::Biological
                        };
                        let mut state = app_state.write();
                        state.build = Some(build);
                        state.busy = BusyState::Idle;
                        state.status_message = Some(
                            "Trees built. Preview the truncated hierarchy below or download the full JSON artifacts."
                                .to_string(),
                        );
                    }
                    Err(error) => {
                        let mut state = app_state.write();
                        state.busy = BusyState::Idle;
                        state.error = Some(present_api_error(&error));
                        state.status_message = None;
                    }
                }
            });
        }
    };

    let state_snapshot = app_state.read().clone();
    let build = state_snapshot.build.clone();

    rsx! {
        div { class: "app-layout no-sidebar",
            main { class: "main-content single-pane",
                header { class: "page-header",
                    div { class: "page-brand",
                        div { class: "page-title-text",
                            h1 { class: "page-title", "LOTUS PubChem Tree Generator" }
                            p { class: "page-sub",
                                "Fetch the LOTUS-linked Wikidata dataset, build biological and chemical trees, preview the hierarchy, and download PubChem-ready JSON artifacts."
                            }
                            p { class: "page-sub",
                                "The heavy lifting runs through "
                                a { href: "https://github.com/adafede/dioxus/tree/main/apps/lotus-api", "lotus-api" }
                                ", matching the explorer app’s backend-driven workflow."
                            }
                        }
                    }

                    div { class: "actions",
                        button {
                            class: "btn btn-primary",
                            disabled: fetch_disabled,
                            onclick: on_fetch,
                            "Fetch Data from Wikidata"
                        }
                        button {
                            class: "btn",
                            disabled: build_disabled,
                            onclick: on_build,
                            "Build Trees"
                        }
                    }

                    p { class: "small", "Current state: {state_snapshot.busy.label()}" }
                }

                if api_base.is_none() {
                    div { class: "notice notice-warn",
                        "This app requires the native lotus-api backend. Start "
                        code { "cargo run -p lotus-api" }
                        " and open the app with "
                        code { "?api_base=http://127.0.0.1:8787" }
                        " if it is not auto-detected."
                    }
                }

                if let Some(message) = state_snapshot.status_message.as_ref() {
                    div { class: "notice notice-info", "{message}" }
                }
                if let Some(error) = state_snapshot.error.as_ref() {
                    div { class: "notice notice-error", "{error}" }
                }

                if let Some(stats) = state_snapshot.stats.clone() {
                    section { class: "card",
                        h2 { "Data Statistics" }
                        StatsGrid { stats }
                    }
                }

                if let Some(build) = build.clone() {
                    section { class: "grid two",
                        SummaryCard {
                            title: "Biological Tree",
                            summary: format!(
                                "{} root nodes · {} total nodes",
                                build.biological_summary.root_nodes,
                                build.biological_summary.total_nodes
                            )
                        }
                        SummaryCard {
                            title: "Chemical Tree (Wikidata)",
                            summary: format!(
                                "{} root nodes · {} total nodes",
                                build.chemical_summary.root_nodes,
                                build.chemical_summary.total_nodes
                            )
                        }
                        SummaryCard {
                            title: "Chemical Tree (NPClassifier)",
                            summary: format!(
                                "{} root nodes · {} total nodes",
                                build.npclassifier_summary.root_nodes,
                                build.npclassifier_summary.total_nodes
                            )
                        }
                    }

                    if let Some(warning) = build.npclassifier_warning.as_ref() {
                        div { class: "notice notice-warn", "{warning}" }
                    }

                    section { class: "card",
                        h2 { "Preview" }
                        p { class: "small",
                            "Preview output is truncated for performance. Download the generated JSON files for the complete trees."
                        }
                        div { class: "tabs",
                            TabButton {
                                label: format!("Biological Tree ({})", build.biological_preview.total_nodes),
                                active: *active_tab.read() == PreviewTab::Biological,
                                onclick: move |_| *active_tab.write() = PreviewTab::Biological,
                            }
                            TabButton {
                                label: format!("Chemical Tree — Wikidata ({})", build.chemical_preview.total_nodes),
                                active: *active_tab.read() == PreviewTab::Chemical,
                                onclick: move |_| *active_tab.write() = PreviewTab::Chemical,
                            }
                            if build.npclassifier_summary.total_nodes > 0 {
                                TabButton {
                                    label: format!(
                                        "Chemical Tree — NPClassifier ({})",
                                        build.npclassifier_preview.total_nodes
                                    ),
                                    active: *active_tab.read() == PreviewTab::Npclassifier,
                                    onclick: move |_| *active_tab.write() = PreviewTab::Npclassifier,
                                }
                            }
                        }
                        match *active_tab.read() {
                            PreviewTab::Biological => rsx! { PreviewPanel { preview: build.biological_preview.clone() } },
                            PreviewTab::Chemical => rsx! { PreviewPanel { preview: build.chemical_preview.clone() } },
                            PreviewTab::Npclassifier => rsx! { PreviewPanel { preview: build.npclassifier_preview.clone() } },
                        }
                    }

                    section { class: "card",
                        h2 { "Download Trees" }
                        p { class: "small", "Generated at {build.generated_at}" }
                        div { class: "download-list",
                            for artifact in build.downloads {
                                a {
                                    class: "link-btn secondary",
                                    href: artifact.url,
                                    title: "Download {artifact.filename}",
                                    "{artifact.label}"
                                }
                            }
                        }
                    }
                }

                footer { class: "footer",
                    p {
                        strong { "Data:" }
                        " "
                        a { href: "https://www.wikidata.org/wiki/Q104225190", "LOTUS Initiative" }
                        " & "
                        a { href: "https://www.wikidata.org/", "Wikidata" }
                        " · "
                        strong { "Target:" }
                        " "
                        a {
                            href: "https://pubchem.ncbi.nlm.nih.gov/classification/#hid=115",
                            "PubChem classification"
                        }
                    }
                    p {
                        strong { "Code:" }
                        " Dioxus frontend + lotus-api backend · "
                        strong { "License:" }
                        " CC0 1.0 for data and AGPL-3.0 for code"
                    }
                }
            }
        }
    }
}

#[component]
fn SummaryCard(title: &'static str, summary: String) -> Element {
    rsx! {
        div { class: "card",
            h3 { "{title}" }
            p { "{summary}" }
        }
    }
}

#[component]
fn TabButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let class = if active { "tab-btn active" } else { "tab-btn" };
    rsx! {
        button { class: "{class}", onclick: move |event| onclick.call(event), "{label}" }
    }
}

#[component]
fn PreviewPanel(preview: crate::models::PreviewTree) -> Element {
    rsx! {
        div {
            p { class: "small", "Showing ~{preview.shown_nodes} nodes out of {preview.total_nodes}." }
            InteractiveTreeViewer {
                nodes: preview.nodes,
                config: TreeViewerConfig {
                    search_enabled: true,
                    max_height: Some(600),
                    show_counts: false,
                    initial_expansion_depth: 0,
                }
            }
        }
    }
}
