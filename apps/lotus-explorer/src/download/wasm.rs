// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::coordinator::query_export_plan;
use crate::api;
use crate::download::DownloadFormat;
use crate::models::SearchCriteria;
use crate::perf;
use shared::sparql::QLEVER_WIKIDATA;
use std::sync::Arc;
use wasm_bindgen::JsCast;

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
    let plan = query_export_plan(format, query.as_ref());

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
    trigger_download_post(QLEVER_WIKIDATA, plan.query.as_ref(), plan.action, &filename)?;
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

fn trigger_download_post(
    endpoint: &str,
    query: &str,
    action: &str,
    filename: &str,
) -> Result<(), String> {
    let Some((window, document)) = window_and_document() else {
        return Err("window/document unavailable".to_string());
    };

    let form = document
        .create_element("form")
        .map_err(|_| "failed to create form element".to_string())?
        .dyn_into::<web_sys::HtmlFormElement>()
        .map_err(|_| "failed to cast form element".to_string())?;
    form.set_method("POST");
    form.set_action(endpoint);
    form.set_target("_blank");
    let _ = form.set_attribute("accept-charset", "UTF-8");
    let _ = form.set_attribute("enctype", "application/x-www-form-urlencoded");

    append_hidden_input(&document, &form, "query", query)?;
    append_hidden_input(&document, &form, "action", action)?;
    append_hidden_input(&document, &form, "filename", filename)?;

    let body = document
        .body()
        .ok_or_else(|| "document body unavailable".to_string())?;
    body.append_child(&form)
        .map_err(|_| "failed to append form to body".to_string())?;
    form.submit()
        .map_err(|_| "failed to submit download form".to_string())?;
    let _ = body.remove_child(&form);
    let _ = window;
    Ok(())
}

fn append_hidden_input(
    document: &web_sys::Document,
    form: &web_sys::HtmlFormElement,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let input = document
        .create_element("input")
        .map_err(|_| format!("failed to create input {name}"))?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| format!("failed to cast input {name}"))?;
    input.set_type("hidden");
    input.set_name(name);
    input.set_value(value);
    form.append_child(&input)
        .map_err(|_| format!("failed to append input {name}"))?;
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

pub fn trigger_download_url(filename: &str, url: &str) {
    let Some((window, document)) = window_and_document() else {
        return;
    };

    if !click_download_anchor(&document, url, filename, true) {
        let _ = window.open_with_url(url);
    }
}

fn window_and_document() -> Option<(web_sys::Window, web_sys::Document)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    Some((window, document))
}

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
