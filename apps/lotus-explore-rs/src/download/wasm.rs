// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::api;
use crate::download::DownloadFormat;
use crate::models::SearchCriteria;
use crate::perf;
use crate::repositories::is_wdqs_fallback_used;
use lotus::queries::transform_query_for_wdqs;
use lotus::transport::{QLEVER_WIKIDATA, WDQS_SCHOLARLY, WDQS_WIKIDATA};
use std::sync::Arc;

pub(super) async fn execute_download_wasm(
    format: DownloadFormat,
    criteria: Arc<SearchCriteria>,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    // If WDQS fallback was used for the interactive query, download from WDQS directly
    if is_wdqs_fallback_used() {
        log::warn!(
            "event=download format={} phase=fetch state=wdqs_fallback fallback=true",
            format.log_name()
        );
        return execute_download_wasm_wdqs(format, query, filename, dl_timer).await;
    }

    match api::export_urls(&criteria).await {
        Ok(urls) => {
            let url = append_filename_query(select_export_url(format, &urls), &filename);
            let fetch_elapsed = perf::end_timer(format.timer_label(), dl_timer);
            perf::log_timing(
                "download",
                &format!(
                    "event=download format={} phase=fetch state=success source=api_url",
                    format.log_name()
                ),
                Some(fetch_elapsed),
            );

            let trigger_timer = perf::start_timer(&format.trigger_timer_label());
            let _ = upload::download_url(&url, &filename);
            let trigger_elapsed = perf::end_timer(&format.trigger_timer_label(), trigger_timer);
            perf::log_timing(
                "download",
                &format!(
                    "event=download format={} phase=trigger state=success source=api_url",
                    format.log_name()
                ),
                Some(trigger_elapsed),
            );
            Ok(())
        }
        Err(err) => {
            log::warn!(
                "event=download format={} phase=fetch state=fallback reason=api_export_urls_failed detail={err}",
                format.log_name()
            );
            // Check if WDQS fallback was used for interactive query
            if is_wdqs_fallback_used() {
                log::warn!(
                    "event=download format={} phase=fetch state=wdqs_fallback_from_api_error",
                    format.log_name()
                );
                execute_download_wasm_wdqs(format, query, filename, dl_timer).await
            } else {
                execute_download_wasm_browser_post(format, query, filename, dl_timer).await
            }
        }
    }
}

async fn execute_download_wasm_wdqs(
    format: DownloadFormat,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    // For simple reference queries, use scholarly endpoint directly without transformation
    let (wdqs_query, endpoint) =
        if query.contains("SELECT ?ref WHERE {") && query.contains("wdt:P356") {
            let query_without_prefix = query.replace("{CURATION_SPARQL_PREFIXES}\n", "");
            (query_without_prefix, WDQS_SCHOLARLY)
        } else {
            (transform_query_for_wdqs(&query), WDQS_WIKIDATA)
        };

    // Build WDQS export URL with proper format
    let encoded_query = urlencoding::encode(&wdqs_query);
    let accept_header = match format {
        DownloadFormat::Csv => "text/csv",
        DownloadFormat::Json => "application/sparql-results+json",
        DownloadFormat::Rdf => "text/turtle",
    };

    let wdqs_url = format!(
        "{}?query={}&Accept={}",
        endpoint, encoded_query, accept_header
    );

    let fetch_elapsed = perf::end_timer(format.timer_label(), dl_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=fetch state=success source=wdqs_url",
            format.log_name(),
        ),
        Some(fetch_elapsed),
    );

    let trigger_timer = perf::start_timer(&format.trigger_timer_label());
    let _ = upload::download_url(&wdqs_url, &filename);
    let trigger_elapsed = perf::end_timer(&format.trigger_timer_label(), trigger_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=trigger state=success source=wdqs_url",
            format.log_name()
        ),
        Some(trigger_elapsed),
    );
    Ok(())
}

async fn execute_download_wasm_browser_post(
    format: DownloadFormat,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    let prepared_query = format.prepared_query(query.as_ref());
    let action = format.qlever_action();

    let fetch_elapsed = perf::end_timer(format.timer_label(), dl_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=fetch state=delegated source=browser_post",
            format.log_name(),
        ),
        Some(fetch_elapsed),
    );

    let trigger_timer = perf::start_timer(&format.trigger_timer_label());
    if let Err(e) = upload::submit_download_form(
        QLEVER_WIKIDATA,
        &[
            ("query", &prepared_query),
            ("action", action),
            ("filename", &filename),
        ],
    )
    .await
    {
        return Err(e);
    }
    let trigger_elapsed = perf::end_timer(&format.trigger_timer_label(), trigger_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=trigger state=success source=browser_post",
            format.log_name()
        ),
        Some(trigger_elapsed),
    );
    Ok(())
}

fn select_export_url(format: DownloadFormat, urls: &api::ExportUrlResponse) -> &str {
    match format {
        DownloadFormat::Csv => urls.csv_gz_url.as_deref().unwrap_or(&urls.csv_url),
        DownloadFormat::Json => urls.json_gz_url.as_deref().unwrap_or(&urls.json_url),
        DownloadFormat::Rdf => urls.rdf_gz_url.as_deref().unwrap_or(&urls.rdf_url),
    }
}

fn append_filename_query(url: &str, filename: &str) -> String {
    let sep = if url.contains('?') { '&' } else { '?' };
    format!("{url}{sep}filename={}", urlencoding::encode(filename))
}

pub(super) fn trigger_download(filename: &str, mime: &str, content_or_url: &str) {
    if content_or_url.starts_with("http://") || content_or_url.starts_with("https://") {
        let _ = upload::download_url(content_or_url, filename);
        let _ = mime;
        return;
    }

    log::debug!(
        "download_text_as_blob filename={} mime={} content_len={}",
        filename,
        mime,
        content_or_url.len()
    );
    if let Err(e) = upload::download_text_as_blob(content_or_url, filename, "", mime) {
        log::error!(
            "download failed: filename={} mime={} error={}",
            filename,
            mime,
            e
        );
    }
}
