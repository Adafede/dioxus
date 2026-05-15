// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared download helpers for browser/native targets, including format handling & deduplication.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(not(target_arch = "wasm32"))]
use crate::sparql;
#[cfg(target_arch = "wasm32")]
use crate::{api, models::SearchCriteria};
use crate::{perf, queries};
#[cfg(not(target_arch = "wasm32"))]
use shared::sparql::SparqlResponseFormat;
#[cfg(target_arch = "wasm32")]
use shared::sparql::{QLEVER_WIKIDATA, SparqlResponseFormat};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadFormat {
    Csv,
    Json,
    Rdf,
}

impl DownloadFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "csv" => Some(Self::Csv),
            "json" | "ndjson" => Some(Self::Json),
            "rdf" => Some(Self::Rdf),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    pub fn log_name(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Csv => "text/csv;charset=utf-8",
            Self::Json => "application/sparql-results+json;charset=utf-8",
            Self::Rdf => "text/turtle;charset=utf-8",
        }
    }

    pub fn timer_label(&self) -> &'static str {
        match self {
            Self::Csv => "LOTUS:download_csv",
            Self::Json => "LOTUS:download_json",
            Self::Rdf => "LOTUS:download_rdf",
        }
    }

    pub fn trigger_timer_label(&self) -> String {
        format!("{}_trigger", self.timer_label())
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(dead_code)]
    pub fn export_url_from_query(&self, query: &str) -> String {
        let q = match self {
            Self::Rdf => queries::query_construct_from_select(query),
            _ => query.to_string(),
        };
        let action = match self {
            Self::Csv => "csv_export",
            Self::Json => "qlever_json_export",
            Self::Rdf => "turtle_export",
        };
        format!(
            "{}?query={}&action={action}",
            QLEVER_WIKIDATA,
            urlencoding::encode(&q)
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn fetch_direct(&self, query: &str) -> Result<String, String> {
        match self {
            Self::Csv => sparql::execute_sparql(query)
                .await
                .map_err(|e| e.to_string()),
            Self::Json => sparql::execute_sparql_format(query, SparqlResponseFormat::SparqlJson)
                .await
                .map_err(|e| e.to_string()),
            Self::Rdf => {
                let construct_query = queries::query_construct_from_select(query);
                sparql::execute_sparql_format(&construct_query, SparqlResponseFormat::Turtle)
                    .await
                    .map_err(|e| e.to_string())
            }
        }
    }
}

/// Execute a download in the given format using direct query export.
pub async fn execute_download(
    format: DownloadFormat,
    #[cfg(target_arch = "wasm32")] criteria: std::sync::Arc<SearchCriteria>,
    query: String,
    filename: String,
) -> Result<(), String> {
    let dl_timer = perf::start_timer(format.timer_label());
    log::info!("event=download format={} state=started", format.log_name());

    #[cfg(target_arch = "wasm32")]
    {
        return execute_download_wasm(format, criteria, query, filename, dl_timer).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        execute_download_native(format, query, filename, dl_timer).await
    }
}

#[cfg(target_arch = "wasm32")]
async fn execute_download_wasm(
    format: DownloadFormat,
    criteria: std::sync::Arc<SearchCriteria>,
    query: String,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    // Prefer same-origin API file URLs when available so browsers honor the
    // requested LOTUS filename while still streaming outside wasm memory.
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
            trigger_download_url(&filename, &url);
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
            execute_download_wasm_direct_post(format, query, filename, dl_timer).await
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn execute_download_wasm_direct_post(
    format: DownloadFormat,
    query: String,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    let (query_for_fetch, response_format, mime) = match format {
        DownloadFormat::Csv => (query, SparqlResponseFormat::Csv, "text/csv;charset=utf-8"),
        DownloadFormat::Json => (
            query,
            SparqlResponseFormat::SparqlJson,
            "application/sparql-results+json;charset=utf-8",
        ),
        DownloadFormat::Rdf => (
            queries::query_construct_from_select(&query),
            SparqlResponseFormat::Turtle,
            "text/turtle;charset=utf-8",
        ),
    };

    let body = shared::sparql::execute_sparql_with_format(
        &query_for_fetch,
        QLEVER_WIKIDATA,
        response_format,
    )
    .await
    .map_err(|e| e.to_string())?;

    let fetch_elapsed = perf::end_timer(format.timer_label(), dl_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=fetch state=success source=direct_post body_bytes={}",
            format.log_name(),
            body.len()
        ),
        Some(fetch_elapsed),
    );

    let trigger_timer = perf::start_timer(&format.trigger_timer_label());
    trigger_download(&filename, mime, &body);
    let trigger_elapsed = perf::end_timer(&format.trigger_timer_label(), trigger_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=trigger state=success source=direct_post",
            format.log_name()
        ),
        Some(trigger_elapsed),
    );
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn select_export_url(format: DownloadFormat, urls: &api::ExportUrlResponse) -> &str {
    match format {
        DownloadFormat::Csv => urls.csv_gz_url.as_deref().unwrap_or(&urls.csv_url),
        DownloadFormat::Json => urls.json_gz_url.as_deref().unwrap_or(&urls.json_url),
        DownloadFormat::Rdf => urls.rdf_gz_url.as_deref().unwrap_or(&urls.rdf_url),
    }
}

#[cfg(target_arch = "wasm32")]
fn append_filename_query(url: &str, filename: &str) -> String {
    let sep = if url.contains('?') { '&' } else { '?' };
    format!("{url}{sep}filename={}", urlencoding::encode(filename))
}

#[cfg(not(target_arch = "wasm32"))]
async fn execute_download_native(
    format: DownloadFormat,
    query: String,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    execute_download_direct(format, query, filename, dl_timer).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn execute_download_direct(
    format: DownloadFormat,
    query: String,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    match format.fetch_direct(&query).await {
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

pub fn trigger_download(filename: &str, mime: &str, content_or_url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if content_or_url.starts_with("http://") || content_or_url.starts_with("https://") {
            trigger_download_url(filename, content_or_url);
            let _ = mime;
            return;
        }

        let Some((window, document)) = window_and_document() else {
            return;
        };

        let parts = js_sys::Array::new();
        parts.push(&wasm_bindgen::JsValue::from_str(content_or_url));

        let blob = {
            let options = web_sys::BlobPropertyBag::new();
            options.set_type(mime);
            web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)
                .or_else(|_| web_sys::Blob::new_with_str_sequence(&parts))
        };
        let Ok(blob) = blob else {
            return;
        };

        let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
            return;
        };

        if !click_download_anchor(&document, &url, filename, false) {
            let _ = window.open_with_url(&url);
        }

        let _ = web_sys::Url::revoke_object_url(&url);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (filename, mime, content_or_url);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_download_url(filename: &str, url: &str) {
    let Some((window, document)) = window_and_document() else {
        return;
    };

    if !click_download_anchor(&document, url, filename, true) {
        let _ = window.open_with_url(url);
    }
}

#[cfg(target_arch = "wasm32")]
fn window_and_document() -> Option<(web_sys::Window, web_sys::Document)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    Some((window, document))
}

#[cfg(target_arch = "wasm32")]
fn click_download_anchor(
    document: &web_sys::Document,
    href: &str,
    filename: &str,
    new_tab: bool,
) -> bool {
    let Some(anchor) = document
        .create_element("a")
        .ok()
        .and_then(|el| el.dyn_into::<web_sys::HtmlAnchorElement>().ok())
    else {
        return false;
    };
    let Some(body_el) = document.body() else {
        return false;
    };

    anchor.set_href(href);
    anchor.set_download(filename);
    anchor.set_rel("noopener noreferrer");
    if new_tab {
        anchor.set_target("_blank");
    }

    let _ = body_el.append_child(&anchor);
    anchor.click();
    let _ = body_el.remove_child(&anchor);
    true
}

#[cfg(test)]
mod tests {
    use super::DownloadFormat;

    #[test]
    fn parse_download_format_supports_documented_aliases() {
        assert_eq!(DownloadFormat::from_str("csv"), Some(DownloadFormat::Csv));
        assert_eq!(DownloadFormat::from_str("json"), Some(DownloadFormat::Json));
        assert_eq!(
            DownloadFormat::from_str("ndjson"),
            Some(DownloadFormat::Json)
        );
        assert_eq!(DownloadFormat::from_str("rdf"), Some(DownloadFormat::Rdf));
        assert_eq!(DownloadFormat::from_str("ttl"), None);
    }
}
