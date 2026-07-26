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
use shared::progress::ProgressThrottler;

// ---------------------------------------------------------------------------
// Performance notes (see accompanying explanation):
//
// The scanner keeps exactly one chunk of the file buffered in memory and
// parses it with plain synchronous loops. We only `.await` when the buffer
// is exhausted and we need to pull the next chunk from the Blob. This means
// a 10 GB file with 16 MiB chunks needs on the order of ~650 async
// suspension points total, instead of one per byte (10+ billion). All
// string/number/literal scanning that would otherwise straddle a chunk
// boundary carries its parse state (escaped-flag, in-string, in-token,
// depth) across the refill in local variables, so correctness is preserved.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
const CHUNK_SIZE: usize = 16 * 1024 * 1024; // 16 MiB per Blob read
#[cfg(target_arch = "wasm32")]
const PROGRESS_BYTE_INTERVAL: u64 = 4 * 1024 * 1024; // report at least every 4 MiB
#[cfg(target_arch = "wasm32")]
const PROGRESS_TIME_INTERVAL_MS: f64 = 120.0; // ...or at least every 120ms

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

/// Extract a Blob from the file picker or drag-drop event and start scanning.
/// Returns `true` if extraction succeeded, `false` otherwise.
#[cfg(target_arch = "wasm32")]
fn extract_blob_from_file(file: &dioxus::html::geometry::screen_space::FileEngine) -> Option<Blob> {
    file.inner()
        .downcast_ref::<web_sys::File>()
        .and_then(|web_file| web_file.clone().dyn_into::<Blob>().ok())
}

#[cfg(target_arch = "wasm32")]
fn begin_scan_from_blob(
    blob: Blob,
    file_name: String,
    file_name_signal: Signal<String>,
    status: Signal<String>,
    results: Signal<Vec<ColumnResult>>,
    busy: Signal<bool>,
    drag_active: Signal<bool>,
) {
    let mut file_name_for_state = file_name_signal;
    let mut status_for_state = status;
    let mut results_for_state = results;
    let mut busy_for_state = busy;
    let mut drag_active_for_state = drag_active;

    file_name_for_state.set(file_name);
    busy_for_state.set(true);
    drag_active_for_state.set(false);
    status_for_state.set("Reading file...".to_string());
    results_for_state.set(vec![]);
    spawn_scan(blob, status_for_state, results_for_state, busy_for_state);
}

#[cfg(target_arch = "wasm32")]
fn spawn_scan(
    blob: Blob,
    mut status: Signal<String>,
    mut results: Signal<Vec<ColumnResult>>,
    mut busy: Signal<bool>,
) {
    spawn(async move {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let total_bytes = blob.size() as u64;
        status.set(format!("Scanning {total_bytes} bytes..."));

        let cols = match scan_blob_with_progress(&blob, move |processed, total| {
            let safe_total = total.max(1);
            let displayed = processed.min(safe_total);
            let percent = (displayed * 100 / safe_total).min(100);
            status.set(format!(
                "Scanning {displayed}/{safe_total} bytes ({percent}%)..."
            ));
        })
        .await
        {
            Ok(cols) => cols,
            Err(error) => {
                status.set(format!("Error reading file: {error:?}"));
                Vec::new()
            }
        };

        let total: u64 = cols.iter().map(|col| col.count).sum();
        status.set(format!(
            "Done — {} columns, {} total non-null values",
            cols.len(),
            total
        ));
        results.set(cols);
        busy.set(false);
    });
}

#[component]
fn app() -> Element {
    let mut file_name = use_signal(String::new);
    let results = use_signal(Vec::<ColumnResult>::new);
    let mut status = use_signal(|| "Choose a JSON file to begin.".to_string());
    let busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);

    let on_file_change = move |evt: Event<FormData>| {
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        {
            let Some(blob) = extract_blob_from_file(&file) else {
                status.set("Unable to read the selected file as a blob.".to_string());
                return;
            };

            begin_scan_from_blob(
                blob,
                file.name(),
                file_name,
                status,
                results,
                busy,
                drag_active,
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            status.set("This app needs to run in a browser.".to_string());
        }
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
        {
            let Some(blob) = extract_blob_from_file(&file) else {
                status.set("Unable to read the selected file as a blob.".to_string());
                return;
            };

            begin_scan_from_blob(
                blob,
                file.name(),
                file_name,
                status,
                results,
                busy,
                drag_active,
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            status.set("This app needs to run in a browser.".to_string());
        }
    };

    rsx! {
        div {
            style: "min-height: 100vh; padding: 2rem 1rem 3rem; background: linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%); color: #0f172a; font-family: sans-serif;",
            div {
                style: "max-width: 760px; margin: 0 auto; background: rgba(255,255,255,0.92); border: 1px solid rgba(148,163,184,0.22); border-radius: 20px; box-shadow: 0 12px 40px rgba(15, 23, 42, 0.08); padding: 1.4rem; backdrop-filter: blur(12px);",
                h2 { style: "margin: 0 0 0.35rem; font-size: 1.6rem; letter-spacing: -0.02em;", "JSON Non-Null Field Counter" }
                p { style: "margin: 0 0 1rem; color: #475569;", "Drop a JSON file into the upload area below or browse for it on disk. The scanner streams multi-gigabyte files in the browser while keeping memory bounded." }

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

// ---------------------------------------------------------------------------
// Streaming scanner
// ---------------------------------------------------------------------------

/// Buffered, chunked reader over a `Blob`. Holds a single in-flight chunk
/// (`buf[pos..]`) and only performs an async Blob read when that chunk is
/// exhausted. All parsing helpers below scan `buf` synchronously and only
/// call `fill()` at the buffer boundary, which is the key difference from
/// a naive byte-at-a-time async reader.
#[cfg(target_arch = "wasm32")]
struct Cursor<F> {
    blob: Blob,
    total_bytes: u64,
    blob_read: u64,
    buf: Vec<u8>,
    pos: usize,
    processed_before_buf: u64,
    eof: bool,
    progress: ProgressThrottler<F, fn() -> f64>,
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)]
impl<F> Cursor<F>
where
    F: FnMut(u64, u64),
{
    fn new(blob: &Blob, total_bytes: u64, on_progress: F) -> Self {
        Self {
            blob: blob.clone(),
            total_bytes,
            blob_read: 0,
            buf: Vec::with_capacity(CHUNK_SIZE),
            pos: 0,
            processed_before_buf: 0,
            eof: false,
            progress: ProgressThrottler::new(
                on_progress,
                js_sys::Date::now,
                PROGRESS_BYTE_INTERVAL,
                PROGRESS_TIME_INTERVAL_MS,
            ),
        }
    }

    fn processed(&self) -> u64 {
        self.processed_before_buf + self.pos as u64
    }

    /// Drops already-consumed bytes and pulls in the next chunk from the
    /// blob. Returns `Ok(true)` if data is available to read, `Ok(false)`
    /// once the stream is fully exhausted.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    async fn fill(&mut self) -> Result<bool, ScanError> {
        if self.pos > 0 {
            self.buf.drain(0..self.pos);
            self.processed_before_buf += self.pos as u64;
            self.pos = 0;
        }

        if self.eof {
            return Ok(!self.buf.is_empty());
        }

        let start = self.blob_read;
        let end = (self.blob_read + CHUNK_SIZE as u64).min(self.total_bytes);
        if start >= end {
            self.eof = true;
            return Ok(!self.buf.is_empty());
        }

        let slice = self
            .blob
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(JsValue::from)?;
        let array_buffer = JsFuture::from(slice.array_buffer()).await?;
        let array = Uint8Array::new(&array_buffer);

        let old_len = self.buf.len();
        let add_len = (end - start) as usize;
        self.buf.resize(old_len + add_len, 0);
        array.copy_to(&mut self.buf[old_len..old_len + add_len]);

        self.blob_read = end;
        if self.blob_read >= self.total_bytes {
            self.eof = true;
        }

        if self
            .progress
            .maybe_report(self.processed(), self.total_bytes)
        {
            // Yield to the event loop only when we actually reported, so we
            // don't stall the UI thread on huge files but also don't yield
            // needlessly on every single chunk.
            TimeoutFuture::new(0).await;
        }

        Ok(true)
    }

    async fn ensure_any(&mut self) -> Result<bool, ScanError> {
        while self.pos >= self.buf.len() {
            if !self.fill().await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn peek(&mut self) -> Result<Option<u8>, ScanError> {
        if self.ensure_any().await? {
            Ok(Some(self.buf[self.pos]))
        } else {
            Ok(None)
        }
    }

    async fn next_byte(&mut self) -> Result<Option<u8>, ScanError> {
        if self.ensure_any().await? {
            let b = self.buf[self.pos];
            self.pos += 1;
            Ok(Some(b))
        } else {
            Ok(None)
        }
    }

    async fn skip_ws(&mut self) -> Result<(), ScanError> {
        loop {
            while self.pos < self.buf.len()
                && matches!(self.buf[self.pos], b' ' | b'\t' | b'\n' | b'\r')
            {
                self.pos += 1;
            }
            if self.pos < self.buf.len() {
                return Ok(());
            }
            if !self.fill().await? {
                return Ok(());
            }
        }
    }

    /// Reads a JSON string (consuming the opening and closing quotes) and
    /// returns its unescaped contents. Used only for object keys, since
    /// those need to be preserved as text.
    async fn read_key(&mut self) -> Result<String, ScanError> {
        if self.next_byte().await? != Some(b'"') {
            return Err(scan_error("Expected opening quote for string"));
        }

        let mut raw = Vec::new();
        let mut escaped = false;
        loop {
            let start = self.pos;
            let len = self.buf.len();
            let mut i = start;
            let mut closed = false;
            while i < len {
                let b = self.buf[i];
                if escaped {
                    escaped = false;
                } else if b == b'\\' {
                    escaped = true;
                } else if b == b'"' {
                    closed = true;
                    break;
                }
                i += 1;
            }
            raw.extend_from_slice(&self.buf[start..i]);
            self.pos = i;
            if closed {
                self.pos += 1;
                break;
            }
            if !self.fill().await? {
                return Err(scan_error("Unexpected EOF while reading string"));
            }
        }

        Ok(unescape_json_string(&raw))
    }

    /// Skips over a JSON string (consuming opening/closing quotes) and
    /// reports only whether it had at least one character. No allocation.
    async fn skip_string_nonempty(&mut self) -> Result<bool, ScanError> {
        if self.next_byte().await? != Some(b'"') {
            return Err(scan_error("Expected opening quote for string"));
        }

        let mut escaped = false;
        let mut any = false;
        loop {
            let start = self.pos;
            let len = self.buf.len();
            let mut i = start;
            let mut closed = false;
            while i < len {
                let b = self.buf[i];
                if escaped {
                    escaped = false;
                } else if b == b'\\' {
                    escaped = true;
                } else if b == b'"' {
                    closed = true;
                    break;
                }
                i += 1;
            }
            if i > start {
                any = true;
            }
            self.pos = i;
            if closed {
                self.pos += 1;
                break;
            }
            if !self.fill().await? {
                return Err(scan_error("Unexpected EOF while reading string"));
            }
        }

        Ok(any)
    }

    /// Counts the number of non-null "leaf" values inside a JSON value.
    /// Nested objects/arrays are flattened and counted recursively in a
    /// single synchronous pass; strings count as 1 if non-empty; numbers
    /// and booleans count as 1; `null` counts as 0.
    async fn count_value(&mut self) -> Result<u64, ScanError> {
        if !self.ensure_any().await? {
            return Ok(0);
        }
        let first = self.buf[self.pos];

        if first == b'"' {
            return Ok(u64::from(self.skip_string_nonempty().await?));
        }

        if first == b'{' || first == b'[' {
            self.pos += 1;
            let mut depth: i32 = 1;
            let mut count: u64 = 0;
            let mut in_string = false;
            let mut escaped = false;
            let mut in_token = false;
            let mut token_first_byte = 0u8;

            loop {
                while self.pos < self.buf.len() {
                    let b = self.buf[self.pos];

                    if in_string {
                        if escaped {
                            escaped = false;
                        } else if b == b'\\' {
                            escaped = true;
                        } else if b == b'"' {
                            in_string = false;
                        }
                        self.pos += 1;
                        continue;
                    }

                    if in_token {
                        if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                            if token_first_byte != b'n' {
                                count += 1;
                            }
                            in_token = false;
                            // Fall through: this byte still needs normal handling below.
                        } else {
                            self.pos += 1;
                            continue;
                        }
                    }

                    match b {
                        b'"' => {
                            in_string = true;
                            count += 1;
                            self.pos += 1;
                        }
                        b'{' | b'[' => {
                            depth += 1;
                            self.pos += 1;
                        }
                        b'}' | b']' => {
                            depth -= 1;
                            self.pos += 1;
                            if depth == 0 {
                                return Ok(count);
                            }
                        }
                        b':' | b',' | b' ' | b'\t' | b'\n' | b'\r' => {
                            self.pos += 1;
                        }
                        _ => {
                            // Start of a number, `true`, `false`, or `null`.
                            // We don't need to validate the exact literal —
                            // just its first byte, to distinguish `null`
                            // (uncounted) from everything else (counted).
                            in_token = true;
                            token_first_byte = b;
                            self.pos += 1;
                        }
                    }
                }
                if !self.fill().await? {
                    return Err(scan_error(
                        "Unexpected EOF while scanning nested JSON value",
                    ));
                }
            }
        }

        // Bare scalar at this position: number, true, false, or null.
        self.pos += 1;
        loop {
            while self.pos < self.buf.len() {
                let b = self.buf[self.pos];
                if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                    return Ok(u64::from(first != b'n'));
                }
                self.pos += 1;
            }
            if !self.fill().await? {
                return Ok(u64::from(first != b'n'));
            }
        }
    }
}

/// Unescapes a raw (still-escaped) JSON string body, e.g. turning `a\"b`
/// into `a"b`. Handles the standard JSON escapes plus `\uXXXX` (BMP only;
/// surrogate pairs are decoded per-unit rather than combined, which is a
/// reasonable simplification for typical field-name content).
#[cfg(target_arch = "wasm32")]
fn unescape_json_string(raw: &[u8]) -> String {
    if !raw.contains(&b'\\') {
        return String::from_utf8_lossy(raw).into_owned();
    }

    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\\' && i + 1 < raw.len() {
            match raw[i + 1] {
                b'"' => {
                    out.push('"');
                    i += 2;
                }
                b'\\' => {
                    out.push('\\');
                    i += 2;
                }
                b'/' => {
                    out.push('/');
                    i += 2;
                }
                b'b' => {
                    out.push('\u{8}');
                    i += 2;
                }
                b'f' => {
                    out.push('\u{c}');
                    i += 2;
                }
                b'n' => {
                    out.push('\n');
                    i += 2;
                }
                b'r' => {
                    out.push('\r');
                    i += 2;
                }
                b't' => {
                    out.push('\t');
                    i += 2;
                }
                b'u' if i + 6 <= raw.len() => {
                    if let Ok(hex) = std::str::from_utf8(&raw[i + 2..i + 6]) {
                        if let Ok(code) = u32::from_str_radix(hex, 16) {
                            if let Some(c) = char::from_u32(code) {
                                out.push(c);
                            }
                        }
                    }
                    i += 6;
                }
                other => {
                    out.push(other as char);
                    i += 2;
                }
            }
        } else {
            let next = raw[i..]
                .iter()
                .position(|&c| c == b'\\')
                .map_or(raw.len(), |p| i + p);
            out.push_str(&String::from_utf8_lossy(&raw[i..next]));
            i = next;
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)]
async fn scan_blob_with_progress<'a>(
    blob: &'a Blob,
    on_progress: impl FnMut(u64, u64) + 'a,
) -> Result<Vec<ColumnResult>, ScanError> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let total_bytes = blob.size() as u64;
    let mut cur = Cursor::new(blob, total_bytes, on_progress);

    cur.skip_ws().await?;
    let Some(open) = cur.next_byte().await? else {
        return Ok(Vec::new());
    };
    if open != b'{' {
        return Err(scan_error("Expected a top-level JSON object"));
    }

    let mut fields = Vec::new();
    loop {
        cur.skip_ws().await?;
        if cur.peek().await? == Some(b'}') {
            cur.next_byte().await?;
            break;
        }

        let key = cur.read_key().await?;
        cur.skip_ws().await?;
        if cur.next_byte().await? != Some(b':') {
            return Err(scan_error("Expected ':' after object key"));
        }
        cur.skip_ws().await?;
        let count = cur.count_value().await?;
        fields.push(ColumnResult { key, count });

        cur.skip_ws().await?;
        match cur.peek().await? {
            Some(b',') => {
                cur.next_byte().await?;
            }
            Some(b'}') => {
                cur.next_byte().await?;
                break;
            }
            _ => break,
        }
    }

    Ok(fields)
}
