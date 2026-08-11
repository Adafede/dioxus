// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::api;
use crate::download::DownloadFormat;
use crate::models::SearchCriteria;
use crate::perf;
use lotus::transport::QLEVER_WIKIDATA;
use std::sync::Arc;

pub(super) async fn execute_download_wasm(
    format: DownloadFormat,
    criteria: Arc<SearchCriteria>,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
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
            execute_download_wasm_browser_post(format, query, filename, dl_timer).await
        }
    }
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

    log::debug!("download_text_as_blob filename={} mime={} content_len={}", filename, mime, content_or_url.len());
    if let Err(e) = upload::download_text_as_blob(content_or_url, filename, "", mime) {
        log::error!("download failed: filename={} mime={} error={}", filename, mime, e);
    }
}
