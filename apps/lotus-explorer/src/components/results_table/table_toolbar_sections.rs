// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Sub-components for ResultsTable toolbar sections.

use super::download_model::{
    DOWNLOAD_METADATA_SPEC, DOWNLOAD_QUERY_CSV_SPEC, DOWNLOAD_QUERY_JSON_SPEC,
    DOWNLOAD_QUERY_RDF_SPEC, DownloadQuerySpec, build_download_toolbar_model,
};
use crate::download::{DownloadFormat, execute_download, trigger_download};
use crate::i18n::{CountNoun, TextKey, count_label, format_count, t};
use crate::models::*;
use crate::perf;
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;
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
pub fn QueryPanel() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let sparql_query =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.sparql_query.clone()
        });
    rsx! {
        if let Some(q) = sparql_query.read().as_ref() {
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
        format!("{}+", format_count(locale, value))
    } else {
        format_count(locale, value)
    };
    let label = count_label(locale, noun, value);
    rsx! {
        div { class: "stat-badge",
            div { class: "stat-value-row",
                span { class: "stat-value", "{display_value}" }
                if let Some(secondary) = secondary_value {
                    div { class: "stat-secondary-row",
                        span { class: "stat-value-secondary mono", "{format_count(locale, secondary)}" }
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
pub fn StatBar() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let entries = crate::features::explore::selectors::use_result_selector(explore, |result| {
        result.entries.clone()
    });
    let total_stats = crate::features::explore::selectors::use_result_selector(explore, |result| {
        result.total_stats.clone()
    });
    let total_matches =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.total_matches
        });

    let fallback_stats: Memo<DatasetStats> =
        use_memo(move || DatasetStats::from_entries(entries.read().as_ref()));
    let stats = total_stats
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_else(|| fallback_stats.read().clone());
    let entries_value = total_matches.read().unwrap_or(stats.n_entries);
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
                plus: false,
            }
            StatBadge {
                value: stats.n_taxa,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Taxon,
                plus: false,
            }
            StatBadge {
                value: stats.n_references,
                secondary_value: None,
                secondary_label: None,
                noun: CountNoun::Reference,
                plus: false,
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
    let explore = use_results_context().explore;
    let display_capped_rows =
        crate::features::explore::selectors::use_result_selector(explore, |result| {
            result.display_capped_rows
        });

    rsx! {
        if *display_capped_rows.read() {
            div { class: "notice notice-warn", role: "status",
                span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
                span { class: "notice-value", "{t(locale, TextKey::DisplayCappedHint)}" }
            }
        }
    }
}

#[component]
pub fn DownloadActionsGroup() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
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

    let download_busy = use_signal(|| false);
    let download_status: Signal<Option<String>> = use_signal(|| None);
    let download_status_text = download_status
        .read()
        .clone()
        .unwrap_or_else(|| t(locale, TextKey::PreparingDownload).to_string());

    let criteria_value = criteria.read().clone();
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
