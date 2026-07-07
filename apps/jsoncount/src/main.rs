use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;

#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::Blob;

#[cfg(target_arch = "wasm32")]
const CHUNK_SIZE: usize = 1 << 20;
#[cfg(target_arch = "wasm32")]
const PROGRESS_INTERVAL: usize = 1 << 20;

#[cfg(target_arch = "wasm32")]
type ScanError = JsValue;

#[cfg(target_arch = "wasm32")]
fn scan_error(message: &str) -> ScanError {
    JsValue::from_str(message)
}

fn main() {
    dioxus::launch(app);
}

#[derive(Clone, PartialEq, Eq)]
struct ColumnResult {
    key: String,
    count: u64,
}

#[component]
fn app() -> Element {
    let mut file_name = use_signal(String::new);
    let mut results = use_signal(Vec::<ColumnResult>::new);
    let mut status = use_signal(|| "Choose a JSON file to begin.".to_string());
    let mut busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);

    let on_file_change = move |evt: Event<FormData>| {
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        file_name.set(file.name());
        busy.set(true);
        drag_active.set(false);
        status.set("Reading file...".to_string());
        results.set(vec![]);

        let mut status_for_progress = status;

        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let total_bytes = blob.size() as u64;
                status_for_progress.set(format!("Counting {total_bytes} bytes..."));
                let cols = match scan_blob_with_progress(&blob, move |processed, total| {
                    let safe_total = total.max(1);
                    let displayed_processed = processed.min(safe_total);
                    let percent = (displayed_processed * 100 / safe_total).min(100);
                    status_for_progress.set(format!(
                        "Counting {displayed_processed}/{safe_total} bytes ({percent}%)..."
                    ));
                })
                .await
                {
                    Ok(cols) => cols,
                    Err(error) => {
                        status_for_progress.set(format!("Error reading file: {error:?}"));
                        vec![]
                    }
                };
                let total: u64 = cols.iter().map(|col| col.count).sum();
                status_for_progress.set(format!(
                    "Done — {} columns, {} total non-null values",
                    cols.len(),
                    total
                ));
                results.set(cols);
                busy.set(false);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                status_for_progress.set("This app needs to run in a browser.".to_string());
                busy.set(false);
            }
        });
    };

    let on_drag_enter = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    let on_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    let on_drag_leave = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
    };

    let on_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        file_name.set(file.name());
        busy.set(true);
        status.set("Reading file...".to_string());
        results.set(vec![]);

        let mut status_for_progress = status;

        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let total_bytes = blob.size() as u64;
                status_for_progress.set(format!("Counting {total_bytes} bytes..."));
                let cols = match scan_blob_with_progress(&blob, move |processed, total| {
                    let safe_total = total.max(1);
                    let displayed_processed = processed.min(safe_total);
                    let percent = (displayed_processed * 100 / safe_total).min(100);
                    status_for_progress.set(format!(
                        "Counting {displayed_processed}/{safe_total} bytes ({percent}%)..."
                    ));
                })
                .await
                {
                    Ok(cols) => cols,
                    Err(error) => {
                        status_for_progress.set(format!("Error reading file: {error:?}"));
                        vec![]
                    }
                };
                let total: u64 = cols.iter().map(|col| col.count).sum();
                status_for_progress.set(format!(
                    "Done — {} columns, {} total non-null values",
                    cols.len(),
                    total
                ));
                results.set(cols);
                busy.set(false);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                status_for_progress.set("This app needs to run in a browser.".to_string());
                busy.set(false);
            }
        });
    };

    rsx! {
        div {
            style: "min-height: 100vh; padding: 2rem 1rem 3rem; background: linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%); color: #0f172a; font-family: sans-serif;",
            div {
                style: "max-width: 760px; margin: 0 auto; background: rgba(255,255,255,0.92); border: 1px solid rgba(148,163,184,0.22); border-radius: 20px; box-shadow: 0 12px 40px rgba(15, 23, 42, 0.08); padding: 1.4rem; backdrop-filter: blur(12px);",
                h2 { style: "margin: 0 0 0.35rem; font-size: 1.6rem; letter-spacing: -0.02em;", "JSON Non-Null Field Counter" }
                p { style: "margin: 0 0 1rem; color: #475569;", "Drop a JSON file into the upload area below or browse for it on disk." }

                label {
                    r#for: "json-upload",
                    style: format!(
                        "display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.6rem; min-height: 140px; width: 100%; box-sizing: border-box; position: relative; isolation: isolate; border: 2px dashed {}; border-radius: 18px; padding: 1rem; cursor: pointer; background: {}; color: #334155; font-weight: 600; text-align: center; transition: border-color 160ms ease, background 160ms ease;",
                        if *drag_active.read() { "#2563eb" } else { "#94a3b8" },
                        if *drag_active.read() { "linear-gradient(135deg, rgba(219,234,254,0.96), rgba(239,246,255,0.94))" } else { "linear-gradient(135deg, rgba(248,250,252,0.95), rgba(239,246,255,0.95))" }
                    ),
                    ondragenter: on_drag_enter,
                    ondragover: on_drag_over,
                    ondragleave: on_drag_leave,
                    ondrop: on_drop,
                    span { style: "font-size: 1rem;", "Drop a JSON file here or click to browse" }
                    span { style: "font-size: 0.85rem; font-weight: 500; color: #64748b;", ".json files only" }
                    input {
                        id: "json-upload",
                        r#type: "file",
                        accept: ".json",
                        disabled: *busy.read(),
                        onchange: on_file_change,
                        style: "position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer;",
                    }
                }

                p {
                    style: "margin: 0.8rem 0 0; color: #475569; font-size: 0.9rem;",
                    if !file_name.read().is_empty() {
                        "Selected file: {file_name}"
                    }
                }

                p { style: "margin: 0.7rem 0 0; font-weight: 600; color: #334155;", "{status}" }

                if !results.read().is_empty() {
                    table {
                        style: "width: 100%; border-collapse: collapse; margin-top: 1rem;",
                        thead {
                            tr {
                                th { style: "text-align: left; border-bottom: 2px solid #333; padding: 4px;", "Column" }
                                th { style: "text-align: right; border-bottom: 2px solid #333; padding: 4px;", "Non-null count" }
                            }
                        }
                        tbody {
                            for col in results.read().iter() {
                                tr {
                                    td { style: "padding: 4px; border-bottom: 1px solid #ddd;", "{col.key}" }
                                    td { style: "padding: 4px; border-bottom: 1px solid #ddd; text-align: right;", "{col.count}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct ProgressReporter<'a> {
    last_reported: u64,
    callback: Box<dyn FnMut(u64, u64) + 'a>,
}

#[cfg(target_arch = "wasm32")]
impl<'a> ProgressReporter<'a> {
    fn new<F>(callback: F) -> Self
    where
        F: FnMut(u64, u64) + 'a,
    {
        Self {
            last_reported: 0,
            callback: Box::new(callback),
        }
    }

    fn report_now(&mut self, processed: u64, total: u64) {
        (self.callback)(processed, total);
        self.last_reported = processed;
    }

    fn maybe_report(&mut self, processed: u64, total: u64) -> bool {
        if processed.saturating_sub(self.last_reported) >= PROGRESS_INTERVAL as u64 {
            self.report_now(processed, total);
            true
        } else {
            false
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct ChunkReader<'a> {
    blob: Blob,
    offset: u64,
    buffer: Vec<u8>,
    position: usize,
    pending: Option<u8>,
    processed: u64,
    progress: ProgressReporter<'a>,
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)]
impl<'a> ChunkReader<'a> {
    fn new<F>(blob: &Blob, on_progress: F) -> Self
    where
        F: FnMut(u64, u64) + 'a,
    {
        Self {
            blob: blob.clone(),
            offset: 0,
            buffer: Vec::new(),
            position: 0,
            pending: None,
            processed: 0,
            progress: ProgressReporter::new(on_progress),
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn total_bytes(&self) -> u64 {
        self.blob.size() as u64
    }

    fn report_progress(&mut self) {
        self.progress.report_now(self.processed, self.total_bytes());
    }

    async fn maybe_report_progress(&mut self) {
        if self
            .progress
            .maybe_report(self.processed, self.total_bytes())
        {
            TimeoutFuture::new(0).await;
        }
    }

    async fn read_byte(&mut self) -> Result<Option<u8>, ScanError> {
        loop {
            if let Some(byte) = self.pending.take() {
                self.processed = self.processed.saturating_add(1);
                self.maybe_report_progress().await;
                return Ok(Some(byte));
            }

            if self.position < self.buffer.len() {
                let byte = self.buffer[self.position];
                self.position += 1;
                self.processed = self.processed.saturating_add(1);
                self.maybe_report_progress().await;
                return Ok(Some(byte));
            }

            if self.offset >= self.total_bytes() {
                return Ok(None);
            }

            self.load_next_chunk().await?;
        }
    }

    async fn peek_byte(&mut self) -> Result<Option<u8>, ScanError> {
        if let Some(byte) = self.pending {
            return Ok(Some(byte));
        }

        let byte = self.read_byte().await?;
        if let Some(b) = byte {
            self.pending = Some(b);
        }
        Ok(byte)
    }

    async fn skip_ws(&mut self) -> Result<(), ScanError> {
        while let Some(byte) = self.peek_byte().await? {
            if matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
                self.read_byte().await?;
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn read_string(&mut self) -> Result<String, ScanError> {
        let Some(open) = self.read_byte().await? else {
            return Err(scan_error("Unexpected EOF while reading string"));
        };
        if open != b'"' {
            return Err(scan_error("Expected opening quote for string"));
        }

        let mut bytes = Vec::new();
        let mut escaped = false;
        loop {
            let Some(byte) = self.read_byte().await? else {
                return Err(scan_error("Unexpected EOF while reading string"));
            };
            match (escaped, byte) {
                (true, _) => {
                    bytes.push(byte);
                    escaped = false;
                }
                (false, b'\\') => escaped = true,
                (false, b'"') => break,
                (false, _) => bytes.push(byte),
            }
        }

        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::redundant_closure,
        clippy::useless_conversion
    )]
    async fn load_next_chunk(&mut self) -> Result<(), ScanError> {
        let start = self.offset as f64;
        let end = self
            .offset
            .saturating_add(CHUNK_SIZE as u64)
            .min(self.total_bytes());
        let end_f64 = end as f64;
        let chunk = self
            .blob
            .slice_with_f64_and_f64(start, end_f64)
            .map_err(JsValue::from)?;
        let promise = chunk.array_buffer();
        let buffer = JsFuture::from(promise).await?;
        let bytes = Uint8Array::new(&buffer);
        self.buffer = bytes.to_vec();
        self.position = 0;
        self.offset = end;
        Ok(())
    }
}

#[allow(clippy::future_not_send)]
#[cfg(target_arch = "wasm32")]
async fn scan_blob_with_progress<'a>(
    blob: &'a Blob,
    mut on_progress: impl FnMut(u64, u64) + 'a,
) -> Result<Vec<ColumnResult>, ScanError> {
    let mut reader = ChunkReader::new(blob, move |processed, total| {
        on_progress(processed, total);
    });
    reader.skip_ws().await?;
    let Some(opening_brace) = reader.read_byte().await? else {
        return Ok(Vec::new());
    };
    if opening_brace != b'{' {
        return Err(scan_error("Expected a top-level JSON object"));
    }

    reader.skip_ws().await?;
    let mut fields = Vec::new();
    loop {
        reader.skip_ws().await?;
        if reader.peek_byte().await? == Some(b'}') {
            reader.read_byte().await?;
            break;
        }

        let key = reader.read_string().await?;
        reader.skip_ws().await?;
        if reader.read_byte().await? != Some(b':') {
            return Err(scan_error("Expected ':' after object key"));
        }
        reader.skip_ws().await?;
        let count = count_json_value(&mut reader).await?;
        fields.push(ColumnResult { key, count });
        reader.skip_ws().await?;
        if reader.peek_byte().await? == Some(b',') {
            reader.read_byte().await?;
        } else if reader.peek_byte().await? == Some(b'}') {
            reader.read_byte().await?;
            break;
        } else {
            break;
        }
        reader.report_progress();
        TimeoutFuture::new(0).await;
    }

    Ok(fields)
}

#[allow(clippy::future_not_send, clippy::too_many_lines)]
#[cfg(target_arch = "wasm32")]
async fn count_json_value(reader: &mut ChunkReader<'_>) -> Result<u64, ScanError> {
    let Some(first) = reader.peek_byte().await? else {
        return Ok(0);
    };

    if first == b'"' {
        let value = reader.read_string().await?;
        return Ok(u64::from(!value.is_empty()));
    }

    if first == b'{' || first == b'[' {
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escaped = false;
        let mut count = 0u64;
        reader.read_byte().await?.unwrap();
        depth += 1;
        loop {
            let Some(byte) = reader.read_byte().await? else {
                return Err(JsValue::from_str(
                    "Unexpected EOF while scanning nested JSON value",
                ));
            };
            if in_string {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    in_string = false;
                }
                continue;
            }

            match byte {
                b'"' => {
                    in_string = true;
                    count += 1;
                }
                b'{' | b'[' => {
                    depth += 1;
                }
                b'}' | b']' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                b':' | b',' | b' ' | b'\t' | b'\n' | b'\r' => {}
                b'n' => {
                    let literal = [b'u', b'l', b'l'];
                    if reader.read_literal(&literal).await? {
                        // null literal consumed entirely
                    } else {
                        count += 1;
                    }
                }
                b't' => {
                    if reader.read_literal(b"rue").await? {
                        count += 1;
                    }
                }
                b'f' => {
                    let false_suffix = [b'a', b'l', b's', b'e'];
                    if reader.read_literal(&false_suffix).await? {
                        count += 1;
                    }
                }
                _ => {
                    while let Some(next) = reader.peek_byte().await? {
                        if matches!(
                            next,
                            b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']'
                        ) {
                            break;
                        }
                        reader.read_byte().await?;
                    }
                    count += 1;
                }
            }
        }
        return Ok(count);
    }

    if matches!(first, b't' | b'f' | b'n') {
        if first == b't' && reader.read_literal(b"true").await? {
            return Ok(1);
        }
        if first == b'f' && reader.read_literal(b"false").await? {
            return Ok(1);
        }
        if first == b'n' && reader.read_literal(b"null").await? {
            return Ok(0);
        }
        return Ok(1);
    }

    if first == b'-' || first.is_ascii_digit() {
        while let Some(next) = reader.peek_byte().await? {
            if matches!(
                next,
                b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']'
            ) {
                break;
            }
            reader.read_byte().await?;
        }
        return Ok(1);
    }

    Ok(0)
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)]
impl ChunkReader<'_> {
    async fn read_literal(&mut self, literal: &[u8]) -> Result<bool, JsValue> {
        for expected in literal {
            let Some(byte) = self.read_byte().await? else {
                return Ok(false);
            };
            if byte != *expected {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
