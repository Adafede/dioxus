use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use ui::prelude::*;

#[cfg(target_arch = "wasm32")]
use file_upload::{BlobCursor, ScanError};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::Blob;

// ---------------------------------------------------------------------------
// Performance notes:
//
// The scanner uses BlobCursor from the file-upload crate to keep exactly
// one chunk of the file buffered in memory. It parses with plain synchronous
// loops, only `.await`ing when the buffer is exhausted. This means a 10 GB
// file with 16 MiB chunks needs ~650 async suspension points total,
// instead of one per byte (10+ billion). Parse state carries across
// chunk boundaries in local variables, preserving correctness.
// ---------------------------------------------------------------------------

fn main() {
    launch(app);
}

#[derive(Clone, PartialEq, Eq)]
struct ColumnResult {
    key: String,
    count: u64,
}

#[cfg(target_arch = "wasm32")]
fn begin_scan_from_blob(
    blob: Blob,
    file_name: String,
    mut file_name_signal: Signal<String>,
    mut status: Signal<String>,
    mut results: Signal<Vec<ColumnResult>>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
) {
    file_name_signal.set(file_name);
    busy.set(true);
    drag_active.set(false);
    status.set("Reading file...".to_string());
    results.set(vec![]);
    spawn_scan(blob, status, results, busy);
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
    #[cfg(target_arch = "wasm32")]
    let file_name = use_signal(String::new);
    #[cfg(not(target_arch = "wasm32"))]
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
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        begin_scan_from_blob(
            blob,
            file.name(),
            file_name,
            status,
            results,
            busy,
            drag_active,
        );

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
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        begin_scan_from_blob(
            blob,
            file.name(),
            file_name,
            status,
            results,
            busy,
            drag_active,
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            status.set("This app needs to run in a browser.".to_string());
        }
    };

    rsx! {
        div {
            style: StyleBuilder::new()
                .property("min-height", "100vh")
                .padding(&format!("{} {} 3rem", Spacing::XL, Spacing::LG))
                .background_color(ColorScheme::LIGHT.bg)
                .color(ColorScheme::LIGHT.text)
                .font_family(Typography::SANS)
                .build(),

            style { ".skip-link:focus {{ top: 0 !important; outline: 3px solid #0b5cab; outline-offset: 2px; }}" }
            skip_link {}

            main {
                id: "main-content",
                style: StyleBuilder::new()
                    .property("max-width", "760px")
                    .margin("0 auto")
                    .background_color(ColorScheme::LIGHT.surface)
                    .border(&format!("1px solid {}", ColorScheme::LIGHT.border))
                    .border_radius(Radius::LG)
                    .box_shadow(Shadow::MD)
                    .padding(Spacing::LG)
                    .build(),

                h1 {
                    style: StyleBuilder::new()
                        .margin("0 0 0.35rem 0")
                        .font_size(Typography::H1)
                        .font_weight("600")
                        .color(ColorScheme::LIGHT.text)
                        .build(),
                    "JSON Non-Null Field Counter"
                }

                p {
                    style: StyleBuilder::new()
                        .margin("0 0 1rem 0")
                        .color(ColorScheme::LIGHT.text2)
                        .font_size(Typography::BODY)
                        .line_height(Typography::LINE_HEIGHT)
                        .build(),
                    "Drop a JSON file into the upload area below or browse for it on disk. The scanner streams multi-gigabyte files in the browser while keeping memory bounded."
                }

                p {
                    id: "json-upload-help",
                    style: StyleBuilder::new()
                        .margin("0 0 1rem 0")
                        .color(ColorScheme::LIGHT.text3)
                        .font_size(Typography::LABEL)
                        .line_height(Typography::LINE_HEIGHT)
                        .build(),
                    "Accepts .json files. The upload area is keyboard focusable and supports drag and drop."
                }

                label {
                    r#for: "json-upload",
                    style: format!(
                        "{}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}",
                        StyleBuilder::new().display("flex").build(),
                        StyleBuilder::new().flex_direction("column").build(),
                        StyleBuilder::new().align_items("center").build(),
                        StyleBuilder::new().justify_content("center").build(),
                        StyleBuilder::new().gap(Spacing::MD).build(),
                        StyleBuilder::new().property("min-height", "140px").build(),
                        StyleBuilder::new().width("100%").property("box-sizing", "border-box").build(),
                        StyleBuilder::new().property("position", "relative").build(),
                        StyleBuilder::new().border(&format!("2px dashed {}", if *drag_active.read() { ColorScheme::LIGHT.blue } else { ColorScheme::LIGHT.border })).build(),
                        StyleBuilder::new().border_radius(Radius::MD).build(),
                        StyleBuilder::new().padding(Spacing::LG).build(),
                        StyleBuilder::new().cursor("pointer").build(),
                        StyleBuilder::new().background_color(if *drag_active.read() { ColorScheme::LIGHT.surface2 } else { ColorScheme::LIGHT.surface }).build(),
                        StyleBuilder::new().color(ColorScheme::LIGHT.text2).build(),
                        StyleBuilder::new().transition(Interaction::TRANSITION_FAST).build(),
                    ),
                    ondragenter: on_drag_enter,
                    ondragover: on_drag_over,
                    ondragleave: on_drag_leave,
                    ondrop: on_drop,

                    span {
                        style: StyleBuilder::new()
                            .font_size(Typography::BODY)
                            .font_weight("600")
                            .build(),
                        "📁 Drop JSON file here or click to browse"
                    }
                    span {
                        style: StyleBuilder::new()
                            .font_size(Typography::LABEL)
                            .font_weight("500")
                            .color(ColorScheme::LIGHT.text3)
                            .build(),
                        ".json files only"
                    }

                    input {
                        id: "json-upload",
                        r#type: "file",
                        accept: ".json",
                        disabled: *busy.read(),
                        onchange: on_file_change,
                        aria_describedby: "json-upload-help json-upload-status",
                        style: "position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer;",
                    }
                }

                if !file_name.read().is_empty() {
                    p {
                        style: StyleBuilder::new()
                            .margin(&format!("{} 0 0", Spacing::MD))
                            .color(ColorScheme::LIGHT.text2)
                            .font_size(Typography::BODY)
                            .build(),
                        "Selected: {file_name}"
                    }
                }

                p {
                    id: "json-upload-status",
                    role: "status",
                    aria_live: "polite",
                    aria_atomic: "true",
                    style: StyleBuilder::new()
                        .margin(&format!("{} 0 0", Spacing::MD))
                        .font_weight("600")
                        .color(if status.read().contains("Error") { ColorScheme::LIGHT.red } else { ColorScheme::LIGHT.text })
                        .build(),
                    "{status}"
                }

                if !results.read().is_empty() {
                    table {
                        "aria-labelledby": "results-heading",
                        style: StyleBuilder::new()
                            .width("100%")
                            .property("border-collapse", "collapse")
                            .margin(&format!("{} 0 0", Spacing::LG))
                            .build(),
                        caption {
                            id: "results-heading",
                            style: StyleBuilder::new()
                                .margin("0 0 0.5rem 0")
                                .font_size(Typography::H2)
                                .font_weight("600")
                                .color(ColorScheme::LIGHT.text)
                                .text_align("left")
                                .build(),
                            "Non-null counts by column"
                        }

                        thead {
                            tr {
                                th {
                                    scope: "col",
                                    style: StyleBuilder::new()
                                        .text_align("left")
                                        .border(&format!("2px solid {}", ColorScheme::LIGHT.border))
                                        .padding(Spacing::SM)
                                        .font_weight("600")
                                        .color(ColorScheme::LIGHT.text)
                                        .build(),
                                    "Column"
                                }
                                th {
                                    scope: "col",
                                    style: StyleBuilder::new()
                                        .text_align("right")
                                        .border(&format!("2px solid {}", ColorScheme::LIGHT.border))
                                        .padding(Spacing::SM)
                                        .font_weight("600")
                                        .color(ColorScheme::LIGHT.text)
                                        .build(),
                                    "Non-null count"
                                }
                            }
                        }

                        tbody {
                            for col in results.read().iter() {
                                tr {
                                    td {
                                        style: StyleBuilder::new()
                                            .padding(Spacing::SM)
                                            .border(&format!("1px solid {}", ColorScheme::LIGHT.border))
                                            .build(),
                                        "{col.key}"
                                    }
                                    td {
                                        style: StyleBuilder::new()
                                            .padding(Spacing::SM)
                                            .border(&format!("1px solid {}", ColorScheme::LIGHT.border))
                                            .text_align("right")
                                            .color(ColorScheme::LIGHT.green)
                                            .font_weight("600")
                                            .build(),
                                        "{col.count}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            style { ".skip-link:focus {{ top: 0 !important; outline: 3px solid #0b5cab; outline-offset: 2px; }}" }
        }
    }
}

// ---------------------------------------------------------------------------
// Streaming JSON scanner - uses BlobCursor from file-upload crate
// ---------------------------------------------------------------------------

/// Unescapes a raw (still-escaped) JSON string body.
/// Handles standard JSON escapes including `\uXXXX` (BMP).
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
                    if let Ok(hex) = std::str::from_utf8(&raw[i + 2..i + 6])
                        && let Ok(code) = u32::from_str_radix(hex, 16)
                        && let Some(c) = char::from_u32(code)
                    {
                        out.push(c);
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

/// Reads a JSON string key from the cursor using the shared BlobCursor.
#[cfg(target_arch = "wasm32")]
async fn read_json_key<F: FnMut(u64, u64)>(
    cursor: &mut BlobCursor<F>,
) -> Result<String, file_upload::ScanError> {
    use wasm_bindgen::JsValue;

    if cursor.next_byte().await? != Some(b'"') {
        return Err(file_upload::ScanError(JsValue::from_str(
            "Expected opening quote for string",
        )));
    }

    let mut raw = Vec::new();
    let mut escaped = false;
    loop {
        let start = cursor.pos();
        let buf = cursor.buffer();
        let mut i = start;
        let mut closed = false;

        while i < buf.len() {
            match buf[i] {
                _ if escaped => {
                    escaped = false;
                    i += 1;
                    continue;
                }
                b'\\' => escaped = true,
                b'"' => {
                    closed = true;
                    break;
                }
                _ => {
                    i += 1;
                }
            }
        }

        raw.extend_from_slice(&buf[start..i]);
        cursor.advance(i - start);

        if closed {
            cursor.advance(1); // consume closing quote
            break;
        }

        if !cursor.fill().await? {
            return Err(file_upload::ScanError(JsValue::from_str(
                "Unexpected EOF while reading string",
            )));
        }
    }

    Ok(unescape_json_string(&raw))
}

/// Skips over a JSON string (consuming opening/closing quotes) and
/// reports only whether it had at least one character. No allocation.
#[cfg(target_arch = "wasm32")]
async fn skip_string_nonempty<F: FnMut(u64, u64)>(
    cursor: &mut BlobCursor<F>,
) -> Result<bool, file_upload::ScanError> {
    use wasm_bindgen::JsValue;

    if cursor.next_byte().await? != Some(b'"') {
        return Err(file_upload::ScanError(JsValue::from_str(
            "Expected opening quote for string",
        )));
    }

    let mut escaped = false;
    let mut any = false;
    loop {
        let start = cursor.pos();
        let buf = cursor.buffer();
        let mut i = start;
        let mut closed = false;

        while i < buf.len() {
            match buf[i] {
                _ if escaped => {
                    escaped = false;
                    i += 1;
                    continue;
                }
                b'\\' => escaped = true,
                b'"' => {
                    closed = true;
                    break;
                }
                _ => {
                    i += 1;
                }
            }
        }

        if i > start {
            any = true;
        }
        cursor.advance(i - start);

        if closed {
            cursor.advance(1); // consume closing quote
            break;
        }

        if !cursor.fill().await? {
            return Err(file_upload::ScanError(JsValue::from_str(
                "Unexpected EOF while reading string",
            )));
        }
    }

    Ok(any)
}

/// Counts the number of non-null "leaf" values inside a JSON value.
/// Nested objects/arrays are flattened and counted recursively in a
/// single synchronous pass; strings count as 1 if non-empty; numbers
/// and booleans count as 1; `null` counts as 0.
#[cfg(target_arch = "wasm32")]
async fn count_value<F: FnMut(u64, u64)>(
    cursor: &mut BlobCursor<F>,
) -> Result<u64, file_upload::ScanError> {
    use wasm_bindgen::JsValue;

    if !cursor.ensure_any().await? {
        return Ok(0);
    }

    let first = cursor
        .current_byte()
        .ok_or_else(|| file_upload::ScanError(JsValue::from_str("Unexpected end of buffer")))?;

    if first == b'"' {
        return Ok(u64::from(skip_string_nonempty(cursor).await?));
    }

    if first == b'{' || first == b'[' {
        cursor.advance(1);
        let mut depth: i32 = 1;
        let mut count: u64 = 0;
        let mut in_string = false;
        let mut escaped = false;
        let mut in_token = false;
        let mut token_first_byte = 0u8;

        loop {
            let buf = cursor.buffer().to_vec(); // Copy to avoid borrow issues
            let mut i = cursor.pos();

            while i < buf.len() {
                let b = buf[i];

                if in_string {
                    if escaped {
                        escaped = false;
                    } else if b == b'\\' {
                        escaped = true;
                    } else if b == b'"' {
                        in_string = false;
                    }
                    i += 1;
                    continue;
                }

                if in_token {
                    if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                        if token_first_byte != b'n' {
                            count += 1;
                        }
                        in_token = false;
                    } else {
                        i += 1;
                        continue;
                    }
                }

                match b {
                    b'"' => {
                        in_string = true;
                        count += 1;
                        i += 1;
                    }
                    b'{' | b'[' => {
                        depth += 1;
                        i += 1;
                    }
                    b'}' | b']' => {
                        depth -= 1;
                        i += 1;
                        if depth == 0 {
                            let advance_by = i - cursor.pos();
                            cursor.advance(advance_by);
                            return Ok(count);
                        }
                    }
                    b':' | b',' | b' ' | b'\t' | b'\n' | b'\r' => {
                        i += 1;
                    }
                    _ => {
                        in_token = true;
                        token_first_byte = b;
                        i += 1;
                    }
                }
            }

            let advance_by = i - cursor.pos();
            cursor.advance(advance_by);

            if !cursor.fill().await? {
                return Err(file_upload::ScanError(JsValue::from_str(
                    "Unexpected EOF while scanning nested JSON value",
                )));
            }
        }
    }

    // Bare scalar at this position: number, true, false, or null.
    let first = cursor.current_byte().unwrap_or(b'n');
    cursor.advance(1);
    loop {
        let buf = cursor.buffer().to_vec(); // Copy to avoid borrow issues
        let i = cursor.pos();
        while i < buf.len() {
            let b = buf[i];
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                return Ok(u64::from(first != b'n'));
            }
            cursor.advance(1);
        }
        if !cursor.fill().await? {
            return Ok(u64::from(first != b'n'));
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)]
async fn scan_blob_with_progress(
    blob: &Blob,
    on_progress: impl FnMut(u64, u64),
) -> Result<Vec<ColumnResult>, String> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let total_bytes = blob.size() as u64;
    let mut cur = BlobCursor::new(blob, total_bytes, on_progress);

    cur.skip_ws().await.map_err(|e| e.to_string())?;
    let Some(open) = cur.next_byte().await.map_err(|e| e.to_string())? else {
        return Ok(Vec::new());
    };
    if open != b'{' {
        return Err("Expected a top-level JSON object".to_string());
    }

    let mut fields = Vec::new();
    loop {
        cur.skip_ws().await.map_err(|e| e.to_string())?;
        if cur.peek().await.map_err(|e| e.to_string())? == Some(b'}') {
            cur.next_byte().await.map_err(|e| e.to_string())?;
            break;
        }

        let key = read_json_key(&mut cur).await.map_err(|e| e.to_string())?;
        cur.skip_ws().await.map_err(|e| e.to_string())?;

        let colon = cur.next_byte().await.map_err(|e| e.to_string())?;
        if colon != Some(b':') {
            return Err("Expected ':' after object key".to_string());
        }

        cur.skip_ws().await.map_err(|e| e.to_string())?;
        let count = count_value(&mut cur).await.map_err(|e| e.to_string())?;
        fields.push(ColumnResult { key, count });

        cur.skip_ws().await.map_err(|e| e.to_string())?;
        match cur.peek().await.map_err(|e| e.to_string())? {
            Some(b',') => {
                cur.next_byte().await.map_err(|e| e.to_string())?;
            }
            Some(b'}') => {
                cur.next_byte().await.map_err(|e| e.to_string())?;
                break;
            }
            _ => break,
        }
    }

    Ok(fields)
}
