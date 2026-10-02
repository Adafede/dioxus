// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `tests` tests, extracted from `download.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `download` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::sanitize_filename;

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
    assert_eq!(sanitize_filename("   "), "");
    assert_eq!(sanitize_filename("."), "");
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
fn download_text_fails_on_native() {
    #[cfg(not(target_arch = "wasm32"))]
    assert!(crate::download_text("test", "test.txt").is_err());
}
