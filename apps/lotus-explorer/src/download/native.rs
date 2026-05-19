// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::coordinator::query_export_plan;
use crate::download::DownloadFormat;
use crate::perf;
use crate::sparql;
use shared::sparql::SparqlResponseFormat;
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
            trigger_download(&filename, content_type(format), &body);
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
    let plan = query_export_plan(format, query);
    match format {
        DownloadFormat::Csv => sparql::execute_sparql(plan.query.as_ref())
            .await
            .map_err(|e| e.to_string()),
        DownloadFormat::Json => {
            sparql::execute_sparql_format(plan.query.as_ref(), SparqlResponseFormat::SparqlJson)
                .await
                .map_err(|e| e.to_string())
        }
        DownloadFormat::Rdf => {
            sparql::execute_sparql_format(plan.query.as_ref(), SparqlResponseFormat::Turtle)
                .await
                .map_err(|e| e.to_string())
        }
    }
}

fn content_type(format: DownloadFormat) -> &'static str {
    match format {
        DownloadFormat::Csv => "text/csv;charset=utf-8",
        DownloadFormat::Json => "application/sparql-results+json;charset=utf-8",
        DownloadFormat::Rdf => "text/turtle;charset=utf-8",
    }
}

pub(super) fn trigger_download(_: &str, _: &str, _: &str) {}
