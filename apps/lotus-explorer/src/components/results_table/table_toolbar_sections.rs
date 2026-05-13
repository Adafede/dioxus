// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Sub-components for ResultsTable toolbar sections.

use crate::download::{DownloadFormat, execute_download, trigger_download};
use crate::export;
use crate::i18n::{CountNoun, TextKey, count_label, t};
use crate::models::*;
use crate::perf;
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
struct DownloadQuerySpec {
    format: DownloadFormat,
    status_key: TextKey,
    title_key: TextKey,
    label_key: TextKey,
}

#[derive(Clone, Copy)]
struct DownloadMetadataSpec {
    title_key: TextKey,
    label_key: TextKey,
}

const DOWNLOAD_QUERY_CSV_SPEC: DownloadQuerySpec = DownloadQuerySpec {
    format: DownloadFormat::Csv,
    status_key: TextKey::StartingCsvDownload,
    title_key: TextKey::DownloadCsvTitle,
    label_key: TextKey::DownloadCsvLabel,
};

const DOWNLOAD_QUERY_JSON_SPEC: DownloadQuerySpec = DownloadQuerySpec {
    format: DownloadFormat::Json,
    status_key: TextKey::PreparingJsonDownload,
    title_key: TextKey::DownloadJsonTitle,
    label_key: TextKey::DownloadJsonLabel,
};

const DOWNLOAD_QUERY_RDF_SPEC: DownloadQuerySpec = DownloadQuerySpec {
    format: DownloadFormat::Rdf,
    status_key: TextKey::PreparingRdfDownload,
    title_key: TextKey::DownloadRdfTitle,
    label_key: TextKey::DownloadRdfLabel,
};

const DOWNLOAD_METADATA_SPEC: DownloadMetadataSpec = DownloadMetadataSpec {
    title_key: TextKey::DownloadMetadataTitle,
    label_key: TextKey::DownloadMetadataLabel,
};

const QLEVER_UI: &str = "https://qlever.dev/wikidata";
const DOWNLOAD_METADATA_MIME: &str = "application/ld+json";

fn log_download_evt(phase: &str, state: &str, details: Option<&str>) {
    let msg = match details {
        Some(d) if !d.is_empty() => format!("event=download phase={phase} state={state} {d}"),
        _ => format!("event=download phase={phase} state={state}"),
    };
    perf::log_info(&msg);
}

fn log_download_timing(
    phase: &str,
    state: &str,
    elapsed: std::time::Duration,
    details: Option<&str>,
) {
    let ms = elapsed.as_secs_f64() * 1000.0;
    let msg = match details {
        Some(d) if !d.is_empty() => {
            format!("event=download phase={phase} state={state} elapsed_ms={ms:.1} {d}")
        }
        _ => format!("event=download phase={phase} state={state} elapsed_ms={ms:.1}"),
    };
    perf::log_info(&msg);
}

fn spawn_query_download(
    format: DownloadFormat,
    status_message: String,
    criteria_snapshot: SearchCriteria,
    filename: String,
    query: Arc<str>,
    mut download_busy: Signal<bool>,
    mut download_status: Signal<Option<String>>,
) {
    *download_busy.write() = true;
    *download_status.write() = Some(status_message);
    spawn(async move {
        log_download_evt(
            "table_dispatch",
            "started",
            Some(&format!("format={}", format.log_name())),
        );
        log_download_evt(
            "table_query",
            "check",
            Some(&format!(
                "format={} has_SERVICE={} query_bytes={}",
                format.log_name(),
                query.contains("SERVICE"),
                query.len()
            )),
        );
        if let Err(err) = execute_download(
            format,
            #[cfg(target_arch = "wasm32")]
            Arc::new(criteria_snapshot),
            query.to_string(),
            filename,
        )
        .await
        {
            log_download_evt(
                "table_fetch",
                "error",
                Some(&format!("format={} reason={err}", format.log_name())),
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
    criteria_snapshot: SearchCriteria,
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
    log_download_evt("table_dispatch", "started", Some("format=metadata"));
    let trigger_timer = perf::start_timer("LOTUS:table_download_meta_trigger");
    trigger_download(filename, DOWNLOAD_METADATA_MIME, body);
    let trigger_elapsed = perf::end_timer("LOTUS:table_download_meta_trigger", trigger_timer);
    log_download_timing(
        "table_trigger",
        "success",
        trigger_elapsed,
        Some("format=metadata"),
    );
}

#[component]
pub fn QueryPanel(sparql_query: Option<Arc<str>>) -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        if let Some(q) = sparql_query.as_ref() {
            details { class: "query-panel",
                summary { "{t(locale, TextKey::SparqlQuery)}" }
                div { class: "query-panel-actions",
                    crate::components::copy_button::CopyButton {
                        text: q.clone(),
                        title: t(locale, TextKey::CopySparqlQuery),
                        locale,
                    }
                }
                pre { class: "query-text", "{q.as_ref()}" }
            }
        }
    }
}

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
        format!("{value}+")
    } else {
        value.to_string()
    };
    let label = count_label(locale, noun, value);
    rsx! {
        div { class: "stat-badge",
            div { class: "stat-value-row",
                span { class: "stat-value", "{display_value}" }
                if let Some(secondary) = secondary_value {
                    div { class: "stat-secondary-row",
                        span { class: "stat-value-secondary mono", "{secondary}" }
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
pub fn StatBar(stats: DatasetStats, total_matches: Option<usize>, stats_partial: bool) -> Element {
    let locale = crate::hooks::use_locale();
    let entries_value = total_matches.unwrap_or(stats.n_entries);
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
                plus: stats_partial,
            }
            StatBadge {
                value: stats.n_taxa,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Taxon,
                plus: stats_partial,
            }
            StatBadge {
                value: stats.n_references,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Reference,
                plus: stats_partial,
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
    rsx! {
        div { class: "notice notice-warn", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
            span { class: "notice-value", "{t(locale, TextKey::DisplayCappedHint)}" }
        }
    }
}

#[component]
pub fn DownloadActionsGroup(
    criteria: SearchCriteria,
    sparql_query: Option<Arc<str>>,
    metadata_json: Option<Arc<str>>,
    query_hash: Option<String>,
    result_hash: Option<String>,
) -> Element {
    let locale = crate::hooks::use_locale();
    let export_available = sparql_query.is_some() || metadata_json.is_some();
    let csv_filename = export::generate_filename(&criteria, "csv");
    let json_filename = export::generate_filename(&criteria, "json");
    let rdf_filename = export::generate_filename(&criteria, "rdf");
    let metadata_filename = match (query_hash.as_deref(), result_hash.as_deref()) {
        (Some(q), Some(r)) => format!("{q}_{r}_metadata.json"),
        _ => export::generate_filename(&criteria, "metadata.json"),
    };
    let qlever_ui_url = sparql_query
        .as_ref()
        .map(|q| format!("{QLEVER_UI}?query={}", urlencoding::encode(q.as_ref())));
    let download_busy = use_signal(|| false);
    let download_status: Signal<Option<String>> = use_signal(|| None);
    let download_status_text = download_status
        .read()
        .clone()
        .unwrap_or_else(|| t(locale, TextKey::PreparingDownload).to_string());

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
            if export_available {
                div {
                    class: "dl-group",
                    role: "group",
                    aria_label: "{t(locale, TextKey::DownloadResults)}",
                    if let Some(query) = sparql_query.as_ref() {
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria.clone();
                                let filename = csv_filename.clone();
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
                            title: "{t(locale, DOWNLOAD_QUERY_CSV_SPEC.title_key)}",
                            aria_label: "{t(locale, DOWNLOAD_QUERY_CSV_SPEC.title_key)}",
                            "{t(locale, DOWNLOAD_QUERY_CSV_SPEC.label_key)}"
                        }
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria.clone();
                                let filename = json_filename.clone();
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
                            title: "{t(locale, DOWNLOAD_QUERY_JSON_SPEC.title_key)}",
                            aria_label: "{t(locale, DOWNLOAD_QUERY_JSON_SPEC.title_key)}",
                            "{t(locale, DOWNLOAD_QUERY_JSON_SPEC.label_key)}"
                        }
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria.clone();
                                let filename = rdf_filename.clone();
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
                            title: "{t(locale, DOWNLOAD_QUERY_RDF_SPEC.title_key)}",
                            aria_label: "{t(locale, DOWNLOAD_QUERY_RDF_SPEC.title_key)}",
                            "{t(locale, DOWNLOAD_QUERY_RDF_SPEC.label_key)}"
                        }
                    }
                    if let Some(body) = metadata_json.as_ref() {
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let body = body.clone();
                                let filename = metadata_filename.clone();
                                move |_| {
                                    dispatch_metadata_download_blob(&filename, body.as_ref());
                                }
                            },
                            title: "{t(locale, DOWNLOAD_METADATA_SPEC.title_key)}",
                            aria_label: "{t(locale, DOWNLOAD_METADATA_SPEC.title_key)}",
                            "{t(locale, DOWNLOAD_METADATA_SPEC.label_key)}"
                        }
                    }
                }
            }
            if let Some(url) = qlever_ui_url.as_deref() {
                a {
                    class: "btn btn-sm",
                    href: "{url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{t(locale, TextKey::OpenInQleverTitle)}",
                    aria_label: "{t(locale, TextKey::OpenInQleverTitle)}",
                    "{t(locale, TextKey::OpenInQlever)}"
                }
            }
        }
    }
}
