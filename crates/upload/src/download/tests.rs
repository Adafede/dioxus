// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `tests` tests, extracted from `download.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `download` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::{sanitize_filename, sanitize_filename_with_fallback};

#[test]
fn sanitize_removes_path_separators() {
    assert_eq!(sanitize_filename("a/b\\c"), "a_b_c");
}

#[test]
fn sanitize_strips_control_chars() {
    assert_eq!(sanitize_filename("file\x00name"), "filename");
}

#[test]
fn sanitize_strips_leading_dots() {
    assert_eq!(sanitize_filename("...file.txt"), "file.txt");
}

#[test]
fn sanitize_empty_input() {
    assert_eq!(sanitize_filename("   "), "download.txt");
    assert_eq!(sanitize_filename("."), "download.txt");
    assert_eq!(sanitize_filename("..."), "download.txt");
    assert_eq!(sanitize_filename("\x00\n"), "download.txt");
}

#[test]
fn sanitize_replaces_windows_reserved_chars() {
    assert_eq!(sanitize_filename("a:b*c?d<e>f|g"), "a_b_c_d_e_f_g");
}

#[test]
fn sanitize_replaces_quotes_with_underscore() {
    assert_eq!(sanitize_filename("file\"name"), "file_name");
    assert_eq!(sanitize_filename("file'name"), "file_name");
}

#[test]
fn sanitize_preserves_safe_names() {
    assert_eq!(sanitize_filename("lotus_results.csv"), "lotus_results.csv");
    assert_eq!(sanitize_filename("my_file-01.json"), "my_file-01.json");
}

#[test]
fn sanitize_strips_trailing_whitespace() {
    assert_eq!(sanitize_filename("file.txt "), "file.txt");
    assert_eq!(sanitize_filename(" file.txt"), "file.txt");
}

#[test]
fn sanitize_drops_newlines_rather_than_underscoring_them() {
    // `is_control` runs before the match, so `\n` and `\r` never reach the
    // `_`-substitution arm and are removed, not replaced.
    assert_eq!(sanitize_filename("a\nb"), "ab");
    assert_eq!(sanitize_filename("a\r\nb"), "ab");
    assert_ne!(sanitize_filename("a\nb"), "a_b");
}

#[test]
fn sanitize_unicode_passthrough() {
    assert_eq!(sanitize_filename("résultats.csv"), "résultats.csv");
    assert_eq!(sanitize_filename("α-β-γ.rdf"), "α-β-γ.rdf");
}

#[test]
fn download_helpers_fail_on_native() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        assert_eq!(
            crate::download_text("test", "test.txt"),
            Err(crate::UploadError::BrowserOnly)
        );
        assert_eq!(
            crate::download_svg("<svg/>", "plot.svg"),
            Err(crate::UploadError::BrowserOnly)
        );
        assert_eq!(
            super::download_with_mime("{}", "out.json", "application/json", "download.txt"),
            Err(crate::UploadError::BrowserOnly)
        );
    }
}

#[test]
fn sanitize_prefixes_windows_reserved_device_stems() {
    assert_eq!(sanitize_filename("CON"), "_CON");
    assert_eq!(sanitize_filename("con.txt"), "_con.txt");
    assert_eq!(sanitize_filename("NUL.tar.gz"), "_NUL.tar.gz");
    assert_eq!(sanitize_filename("COM1.csv"), "_COM1.csv");
    assert_eq!(sanitize_filename("LPT9"), "_LPT9");
    assert_eq!(sanitize_filename("console.txt"), "console.txt");
}

#[test]
fn sanitize_caps_utf8_byte_length_preserving_extension() {
    let long = format!("{}.csv", "é".repeat(200));
    let sanitized = sanitize_filename(&long);
    assert!(
        sanitized.len() <= 255,
        "sanitized length {} exceeded 255 bytes",
        sanitized.len()
    );
    assert!(
        std::path::Path::new(&sanitized)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("csv")),
        "extension must be preserved: {sanitized}"
    );
}

#[test]
fn sanitize_trims_dots_whitespace_and_trimmed_reserved_stems() {
    assert_eq!(sanitize_filename(". . ."), "download.txt");
    assert_eq!(sanitize_filename("a. ."), "a");
    assert_eq!(sanitize_filename("CON .txt"), "_CON .txt");
    assert_eq!(
        sanitize_filename_with_fallback("...", "download.svg"),
        "download.svg"
    );
}
