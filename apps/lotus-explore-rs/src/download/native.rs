// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::download::DownloadFormat;
use crate::perf;
use crate::sparql;
use lotus::transport::ResponseFormat;
use std::sync::Arc;

pub(super) async fn execute_download_native(
    format: DownloadFormat,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    execute_download_direct(format, query, filename, dl_timer).await
}

async fn execute_download_direct(
    format: DownloadFormat,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    match fetch_direct(format, query.as_ref()).await {
        Ok(body) => {
            let fetch_elapsed = perf::end_timer(format.timer_label(), dl_timer);
            perf::log_timing(
                "download",
                &format!(
                    "event=download format={} phase=fetch state=success source=direct body_bytes={}",
                    format.log_name(),
                    body.len()
                ),
                Some(fetch_elapsed),
            );

            let trigger_timer = perf::start_timer(&format.trigger_timer_label());
            trigger_download(&filename, format.content_type(), &body);
            let trigger_elapsed = perf::end_timer(&format.trigger_timer_label(), trigger_timer);
            perf::log_timing(
                "download",
                &format!(
                    "event=download format={} phase=trigger state=success source=direct",
                    format.log_name()
                ),
                Some(trigger_elapsed),
            );
            Ok(())
        }
        Err(e) => {
            let elapsed = perf::end_timer(format.timer_label(), dl_timer);
            perf::log_timing(
                "download",
                &format!(
                    "event=download format={} phase=fetch state=error source=direct reason={e}",
                    format.log_name()
                ),
                Some(elapsed),
            );
            log::warn!(
                "event=download format={} phase=fetch state=error source=direct reason={e}",
                format.log_name()
            );
            Err(e)
        }
    }
}

async fn fetch_direct(format: DownloadFormat, query: &str) -> Result<String, String> {
    let prepared_query = format.prepared_query(query);
    match format {
        DownloadFormat::Csv => sparql::execute_query(&prepared_query)
            .await
            .map_err(|e| e.to_string()),
        DownloadFormat::Json => {
            sparql::execute_sparql_format(&prepared_query, ResponseFormat::SparqlJson)
                .await
                .map_err(|e| e.to_string())
        }
        DownloadFormat::Rdf => {
            sparql::execute_sparql_format(&prepared_query, ResponseFormat::Turtle)
                .await
                .map_err(|e| e.to_string())
        }
    }
}

pub(super) fn trigger_download(filename: &str, mime: &str, content: &str) {
    let _ = upload::download_text(content, filename);
    let _ = mime;
}
