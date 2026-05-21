// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Download actions toolbar group — buttons to trigger query/metadata downloads
//! and links to open the query in the QLever UI.

use super::super::download_model::{
    DOWNLOAD_METADATA_SPEC, DOWNLOAD_QUERY_CSV_SPEC, DOWNLOAD_QUERY_JSON_SPEC,
    DOWNLOAD_QUERY_RDF_SPEC, DownloadQuerySpec, build_download_toolbar_model,
};
use crate::download::{DownloadFormat, execute_download, trigger_download};
use crate::i18n::{TextKey, t};
use crate::models::SearchCriteria;
use crate::perf;
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

const DOWNLOAD_METADATA_MIME: &str = "application/ld+json";

// ── private helpers ───────────────────────────────────────────────────────────

fn spawn_query_download(
    format: DownloadFormat,
    status_message: String,
    criteria_snapshot: Arc<SearchCriteria>,
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
            criteria_snapshot,
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
        #[cfg(not(target_arch = "wasm32"))]
        let _ = &criteria_snapshot;
        *download_busy.write() = false;
        *download_status.write() = None;
    });
}

fn dispatch_query_download_spec(
    spec: DownloadQuerySpec,
    locale: crate::i18n::Locale,
    criteria_snapshot: Arc<SearchCriteria>,
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
    log::info!("event=download phase=table_dispatch state=started format=metadata");
    let trigger_timer = perf::start_timer("LOTUS:table_download_meta_trigger");
    trigger_download(filename, DOWNLOAD_METADATA_MIME, body);
    let elapsed_ms =
        perf::end_timer("LOTUS:table_download_meta_trigger", trigger_timer).as_secs_f64() * 1000.0;
    log::info!(
        "event=download phase=table_trigger state=success format=metadata elapsed_ms={elapsed_ms:.1}"
    );
}

// ── component ─────────────────────────────────────────────────────────────────

#[component]
pub fn DownloadActionsGroup() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;

    // Each selector subscribes to exactly one field; the component only
    // re-renders when any of these specific fields change.
    let criteria = crate::features::explore::selectors::use_ui_selector(explore, |ui| {
        ui.executed_criteria.clone()
    });
    let sparql_query =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.sparql_query.clone()
        });
    let metadata_json =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.metadata_json.clone()
        });
    let query_hash = crate::features::explore::selectors::use_result_selector(explore, |result| {
        result.query_hash.clone()
    });
    let result_hash = crate::features::explore::selectors::use_result_selector(explore, |result| {
        result.result_hash.clone()
    });

    let toolbar_model = build_download_toolbar_model(
        &criteria.read(),
        sparql_query.read().as_deref(),
        metadata_json.read().as_deref(),
        query_hash.read().as_deref(),
        result_hash.read().as_deref(),
    );

    let download_results_label = t(locale, TextKey::DownloadResults);
    let csv_title = t(locale, DOWNLOAD_QUERY_CSV_SPEC.title_key);
    let csv_label = t(locale, DOWNLOAD_QUERY_CSV_SPEC.label_key);
    let json_title = t(locale, DOWNLOAD_QUERY_JSON_SPEC.title_key);
    let json_label = t(locale, DOWNLOAD_QUERY_JSON_SPEC.label_key);
    let rdf_title = t(locale, DOWNLOAD_QUERY_RDF_SPEC.title_key);
    let rdf_label = t(locale, DOWNLOAD_QUERY_RDF_SPEC.label_key);
    let metadata_title = t(locale, DOWNLOAD_METADATA_SPEC.title_key);
    let metadata_label = t(locale, DOWNLOAD_METADATA_SPEC.label_key);
    let qlever_title = t(locale, TextKey::OpenInQleverTitle);
    let qlever_label = t(locale, TextKey::OpenInQlever);

    // Local download state — busy flag and status text.
    let download_busy = use_signal(|| false);
    let download_status: Signal<Option<String>> = use_signal(|| None);
    let download_status_text = download_status
        .read()
        .clone()
        .unwrap_or_else(|| t(locale, TextKey::PreparingDownload).to_string());

    let criteria_value = Arc::new(criteria.read().clone());
    let sparql_query_value = sparql_query.read().clone();
    let metadata_json_value = metadata_json.read().clone();

    rsx! {
        div { class: "toolbar-actions",
            if *download_busy.read() {
                span {
                    class: "btn btn-sm",
                    role: "status",
                    aria_live: "polite",
                    span { class: "spinner-sm", "aria-hidden": "true" }
                    {download_status_text}
                }
            }
            if toolbar_model.export_available {
                div {
                    class: "dl-group",
                    role: "group",
                    aria_label: "{download_results_label}",
                    if let Some(query) = sparql_query_value.as_ref() {
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria_value.clone();
                                let filename = toolbar_model.csv_filename.clone();
                                move |_| {
                                    dispatch_query_download_spec(
                                        DOWNLOAD_QUERY_CSV_SPEC,
                                        locale,
                                        criteria_snapshot.clone(),
                                        filename.clone(),
                                        q.clone(),
                                        download_busy,
                                        download_status,
                                    );
                                }
                            },
                            aria_label: "{csv_title}",
                            title: "{csv_title}",
                            "{csv_label}"
                        }
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria_value.clone();
                                let filename = toolbar_model.json_filename.clone();
                                move |_| {
                                    dispatch_query_download_spec(
                                        DOWNLOAD_QUERY_JSON_SPEC,
                                        locale,
                                        criteria_snapshot.clone(),
                                        filename.clone(),
                                        q.clone(),
                                        download_busy,
                                        download_status,
                                    );
                                }
                            },
                            aria_label: "{json_title}",
                            title: "{json_title}",
                            "{json_label}"
                        }
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria_value.clone();
                                let filename = toolbar_model.rdf_filename.clone();
                                move |_| {
                                    dispatch_query_download_spec(
                                        DOWNLOAD_QUERY_RDF_SPEC,
                                        locale,
                                        criteria_snapshot.clone(),
                                        filename.clone(),
                                        q.clone(),
                                        download_busy,
                                        download_status,
                                    );
                                }
                            },
                            aria_label: "{rdf_title}",
                            title: "{rdf_title}",
                            "{rdf_label}"
                        }
                    }
                    if let Some(body) = metadata_json_value.as_ref() {
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let body = body.clone();
                                let filename = toolbar_model.metadata_filename.clone();
                                move |_| {
                                    dispatch_metadata_download_blob(&filename, body.as_ref());
                                }
                            },
                            title: "{metadata_title}",
                            aria_label: "{metadata_title}",
                            "{metadata_label}"
                        }
                    }
                }
            }
            if let Some(url) = toolbar_model.qlever_ui_url.as_deref() {
                a {
                    class: "btn btn-sm",
                    href: "{url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{qlever_title}",
                    aria_label: "{qlever_title}",
                    "{qlever_label}"
                }
            }
        }
    }
}
