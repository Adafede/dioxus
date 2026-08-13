// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Download actions toolbar group — buttons to trigger query/metadata downloads
//! and links to open the query in the QLever UI.

use super::super::download_model::{
    DOWNLOAD_METADATA_SPEC, DOWNLOAD_QUERY_CSV_SPEC, DOWNLOAD_QUERY_JSON_SPEC,
    DOWNLOAD_QUERY_RDF_SPEC, DownloadQuerySpec, build_download_toolbar_model,
};
use crate::download::{DownloadFormat, execute_download, trigger_download};
use crate::features::explore::use_toolbar_result_snapshot;
use crate::i18n::{TextKey, t};
use crate::models::SearchCriteria;
use crate::perf;
use crate::state::use_results_context;
use crate::ui::style_constants::{buttons, downloads};
use dioxus::prelude::*;
use std::sync::Arc;
use ui::prelude::*;

const DOWNLOAD_METADATA_MIME: &str = "application/ld+json";

// ── private helpers ───────────────────────────────────────────────────────────

fn spawn_query_download(
    format: DownloadFormat,
    status_message: String,
    _criteria_snapshot: Option<Arc<SearchCriteria>>,
    filename: String,
    query: Arc<str>,
    mut download_busy: Signal<bool>,
    mut download_status: Signal<Option<String>>,
) {
    *download_busy.write() = true;
    *download_status.write() = Some(status_message);
    spawn(async move {
        log::info!(
            "event=download phase=table_dispatch state=started format={}",
            format.log_name()
        );
        log::info!(
            "event=download phase=table_query state=check format={} has_SERVICE={} query_bytes={}",
            format.log_name(),
            query.contains("SERVICE"),
            query.len()
        );
        if let Err(err) = execute_download(
            format,
            #[cfg(target_arch = "wasm32")]
            _criteria_snapshot.expect("wasm download requires criteria snapshot"),
            query,
            filename,
        )
        .await
        {
            log::warn!(
                "event=download phase=table_fetch state=error format={} reason={err}",
                format.log_name()
            );
        }
        *download_busy.write() = false;
        *download_status.write() = None;
    });
}

fn dispatch_query_download_spec(
    spec: DownloadQuerySpec,
    locale: crate::i18n::Locale,
    criteria_snapshot: Option<Arc<SearchCriteria>>,
    filename: String,
    query: Arc<str>,
    download_busy: Signal<bool>,
    download_status: Signal<Option<String>>,
) {
    spawn_query_download(
        spec.format,
        t(locale, spec.status_key).to_string(),
        criteria_snapshot,
        filename,
        query,
        download_busy,
        download_status,
    );
}

fn dispatch_metadata_download_blob(filename: &str, body: &str) {
    log::info!(
        "event=download phase=table_dispatch state=started format=metadata filename={} size={}",
        filename,
        body.len()
    );
    let trigger_timer = perf::start_timer("LOTUS:table_download_meta_trigger");
    if body.is_empty() {
        log::error!(
            "event=download phase=table_dispatch state=error format=metadata reason=empty_body"
        );
        return;
    }
    trigger_download(filename, DOWNLOAD_METADATA_MIME, body);
    let elapsed_ms =
        perf::end_timer("LOTUS:table_download_meta_trigger", trigger_timer).as_secs_f64() * 1000.0;
    log::info!(
        "event=download phase=table_trigger state=success format=metadata elapsed_ms={elapsed_ms:.1}"
    );
}

// ── components ───────────────────────────────────────────────────────────────

/// Displays download status with spinning indicator.
#[component]
fn DownloadStatusSpinner(
    download_status: ReadSignal<Option<String>>,
    locale: crate::i18n::Locale,
) -> Element {
    let status_msg = download_status.read().clone();
    let text = status_msg
        .as_deref()
        .unwrap_or_else(|| t(locale, TextKey::PreparingDownload));

    rsx! {
        span {
            role: "status",
            aria_live: "polite",
            style: crate::ui::style_constants::buttons::button_transparent_style(),
            span { style: crate::ui::style_constants::downloads::spinner_sm_style(), "aria-hidden": "true" }
            {text}
        }
    }
}

/// Download button for query results (CSV, JSON, RDF formats).
#[component]
fn DownloadQueryButton(
    spec: DownloadQuerySpec,
    toolbar_model: ReadSignal<
        crate::components::results_table::download_model::DownloadToolbarModel,
    >,
    sparql_query: Arc<str>,
    locale: crate::i18n::Locale,
    disabled: bool,
    download_busy: Signal<bool>,
    download_status: Signal<Option<String>>,
    criteria: ReadSignal<SearchCriteria>,
    filename: String,
) -> Element {
    let title = t(locale, spec.title_key);
    let label = t(locale, spec.label_key);

    rsx! {
        button {
            r#type: "button",
            disabled,
            style: crate::ui::style_constants::downloads::button_small_style(),
            onclick: {
                let q = sparql_query.clone();
                let fname = filename.clone();
                #[cfg(target_arch = "wasm32")]
                let criteria_snapshot = Some(Arc::new(criteria.read().clone()));
                #[cfg(not(target_arch = "wasm32"))]
                let criteria_snapshot = None;
                move |_| {
                    dispatch_query_download_spec(
                        spec,
                        locale,
                        criteria_snapshot.clone(),
                        fname.clone(),
                        q.clone(),
                        download_busy,
                        download_status,
                    );
                }
            },
            aria_label: "{title}",
            title: "{title}",
            "{label}"
        }
    }
}

/// Download button for metadata JSON file.
#[component]
fn DownloadMetadataButton(
    metadata_json: Arc<str>,
    toolbar_model: ReadSignal<
        crate::components::results_table::download_model::DownloadToolbarModel,
    >,
    locale: crate::i18n::Locale,
    disabled: bool,
) -> Element {
    let title = t(locale, DOWNLOAD_METADATA_SPEC.title_key);
    let label = t(locale, DOWNLOAD_METADATA_SPEC.label_key);

    rsx! {
        button {
            r#type: "button",
            disabled,
            style: crate::ui::style_constants::downloads::button_small_style(),
            onclick: {
                let body = metadata_json.clone();
                let filename = toolbar_model.read().metadata_filename.clone();
                move |_| {
                    dispatch_metadata_download_blob(&filename, body.as_ref());
                }
            },
            title: "{title}",
            aria_label: "{title}",
            "{label}"
        }
    }
}

#[component]
pub fn DownloadActionsGroup() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;

    // Each selector subscribes to exactly one field; the component only
    // re-renders when any of these specific fields change.
    let criteria = crate::features::explore::selectors::use_ui_selector(explore, |ui| {
        ui.executed_criteria.clone()
    });
    let toolbar_snapshot = use_toolbar_result_snapshot(explore);

    let snapshot = toolbar_snapshot.read();
    let toolbar_model = use_signal(|| {
        build_download_toolbar_model(
            &criteria.read(),
            snapshot.sparql_query.as_deref(),
            snapshot.metadata_json.as_deref(),
            snapshot.query_hash.as_deref(),
            snapshot.result_hash.as_deref(),
        )
    });

    let download_results_label = t(locale, TextKey::DownloadResults);
    let qlever_title = t(locale, TextKey::OpenInQleverTitle);
    let qlever_label = t(locale, TextKey::OpenInQlever);

    // Local download state — busy flag and status text.
    let download_busy = use_signal(|| false);
    let download_status: Signal<Option<String>> = use_signal(|| None);

    let sparql_query_value = snapshot.sparql_query.clone();
    let metadata_json_value = snapshot.metadata_json.clone();
    let export_available = toolbar_model.read().export_available;
    let qlever_ui_url = toolbar_model.read().qlever_ui_url.clone();
    drop(snapshot);

    rsx! {
        div { style: crate::ui::style_constants::downloads::toolbar_actions_style(),
            if *download_busy.read() {
                DownloadStatusSpinner {
                    download_status,
                    locale,
                }
            }
            if export_available {
                div {
                    role: "group",
                    aria_label: "{download_results_label}",
                    style: crate::ui::style_constants::downloads::dl_group_style(),
                    if let Some(query) = sparql_query_value.as_ref() {
                        DownloadQueryButton {
                            spec: DOWNLOAD_QUERY_CSV_SPEC,
                            toolbar_model,
                            sparql_query: query.clone(),
                            locale,
                            disabled: *download_busy.read(),
                            download_busy,
                            download_status,
                            criteria,
                            filename: toolbar_model.read().csv_filename.clone(),
                        }
                        DownloadQueryButton {
                            spec: DOWNLOAD_QUERY_JSON_SPEC,
                            toolbar_model,
                            sparql_query: query.clone(),
                            locale,
                            disabled: *download_busy.read(),
                            download_busy,
                            download_status,
                            criteria,
                            filename: toolbar_model.read().json_filename.clone(),
                        }
                        DownloadQueryButton {
                            spec: DOWNLOAD_QUERY_RDF_SPEC,
                            toolbar_model,
                            sparql_query: query.clone(),
                            locale,
                            disabled: *download_busy.read(),
                            download_busy,
                            download_status,
                            criteria,
                            filename: toolbar_model.read().rdf_filename.clone(),
                        }
                    }
                    if let Some(body) = metadata_json_value.as_ref() {
                        DownloadMetadataButton {
                            metadata_json: body.clone(),
                            toolbar_model,
                            locale,
                            disabled: *download_busy.read(),
                        }
                    }
                    if let Some(url) = qlever_ui_url.as_deref() {
                        a {
                            href: "{url}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            style: crate::ui::style_constants::downloads::button_small_style(),
                            title: "{qlever_title}",
                            aria_label: "{qlever_title}",
                            "{qlever_label}"
                        }
                    }
                }
            }
        }
    }
}
