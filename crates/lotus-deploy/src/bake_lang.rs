//! Bake a synchronous `<html lang>` + language bootstrap into shipped
//! `index.html` files.
//!
//! # Why this exists
//!
//! dioxus-cli 0.7 only bakes its own title/toast into the static `index.html`;
//! the app's `DocumentHead` (in `apps/lotus-explore-rs/src/document_head.rs`)
//! applies `<html lang>` client-side *after* the async dioxus module hydrates.
//! WAVE therefore reports, for `/?lang=fr`:
//!
//! > "Language *en* as the change happens AFTER loading"
//!
//! This injects a tiny inline `<script>` as the very first child of `<head>`
//! (it runs during HTML parsing, before the deferred async module) that sets
//! `document.documentElement.lang` from the same `?lang=` / `?locale=` convention
//! the app already reads (`features::explore::url_state`), defaulting to `"en"`
//! (the app's default locale). `<html lang>` is thus correct on initial paint,
//! so no language change happens after load.
//!
//! # Usage
//!
//! Run from the site root that contains `_site/`, before
//! `actions/upload-pages-artifact`. Targets the root landing page and each
//! app's top-level `index.html` only — never the nested Ketcher iframe
//! `index.html` under `assets/ketcher/`.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// `<head>` replacement: keeps the `<head>` tag, then inserts the synchronous
/// language resolver as the first child (runs during parse, before the async
/// module). Reads `?lang=` / `?locale=` and defaults to `"en"`. (Raw string so
/// the double quotes in the embedded JS need no escaping.)
const LANG_HEAD: &str = r#"<head>
            <script id="dx-lang-bootstrap">try{var p=new URLSearchParams(location.search);var l=p.get("lang")||p.get("locale");document.documentElement.lang=l||"en";}catch(e){}</script>"#;

#[must_use]
fn targets(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root_index = root.join("index.html");
    if root_index.is_file() {
        out.push(root_index);
    }
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        // Skip the Ketcher iframe assets tree: its `index.html` is the
        // standalone editor's own shell and must be left untouched.
        if entry.file_name() == OsStr::new("assets") {
            continue;
        }
        let index = entry_path.join("index.html");
        if index.is_file() {
            out.push(index);
        }
    }
    out
}

fn bake(path: &Path) -> std::io::Result<()> {
    let mut html = fs::read_to_string(path)?;
    let needs_lang = html.contains("<html>") && !html.contains("<html lang=");
    if needs_lang {
        html = html.replacen("<html>", "<html lang=\"en\">", 1);
    }
    let needs_script = !html.contains("dx-lang-bootstrap");
    if needs_script {
        html = html.replacen("<head>", LANG_HEAD, 1);
    }
    if needs_lang || needs_script {
        fs::write(path, html.as_bytes())?;
        println!("  baked lang bootstrap into {}", path.display());
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let root = Path::new("_site");
    if !root.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("site root not found: {}", root.display()),
        ));
    }
    let root_targets = targets(root);
    let mut count = 0usize;
    for path in &root_targets {
        bake(path)?;
        count += 1;
    }
    println!("baked lang into {count} index.html file(s)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn injects_lang_and_bootstrap_idempotently() {
        let dir = std::env::temp_dir().join("lotus_deploy_bake_lang_test");
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("index.html");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(file, "<html><head><title>x</title></head></html>").unwrap();
        drop(file);

        bake(&path).unwrap();
        let html = std::fs::read_to_string(&path).unwrap();
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("dx-lang-bootstrap"));
        assert!(html.contains(LANG_HEAD));

        // Re-running must not duplicate the injection.
        bake(&path).unwrap();
        let html2 = std::fs::read_to_string(&path).unwrap();
        assert_eq!(html2, html);
        assert_eq!(html.matches("dx-lang-bootstrap").count(), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn targets_skips_ketcher_assets() {
        let root = std::env::temp_dir().join("lotus_deploy_targets_test");
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(root.join("lotus-explore-rs/assets/ketcher")).unwrap();
        std::fs::write(root.join("index.html"), "<html></html>").unwrap();
        std::fs::write(root.join("lotus-explore-rs/index.html"), "<html></html>").unwrap();
        std::fs::write(
            root.join("lotus-explore-rs/assets/ketcher/index.html"),
            "<html></html>",
        )
        .unwrap();

        // Root landing page + the app's index, but NOT the nested Ketcher
        // iframe index.html (under assets/).
        assert_eq!(targets(&root).len(), 2);

        std::fs::remove_dir_all(&root).ok();
    }
}
