// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

use crate::unescape::unicode_code_unit;
use crate::unescape_json_string;

fn unescape(s: &str) -> String {
    unescape_json_string(s.as_bytes())
}

fn unicode_escape(raw: &[u8], at: usize) -> Option<char> {
    unicode_code_unit(raw, at).and_then(|code| char::from_u32(u32::from(code)))
}

#[test]
fn a_string_with_no_backslash_is_returned_unchanged() {
    assert_eq!(unescape("plain text"), "plain text", "no escapes to expand");
    assert_eq!(unescape(""), "", "and nothing at all");
}

#[test]
fn a_backslash_is_what_arms_the_scanner() {
    assert_eq!(
        unescape(r"a\zc"),
        "azc",
        "an unknown escape contributes only its character"
    );
    assert_eq!(
        unescape(r"\za\nb"),
        "za\nb",
        "and the scan carries on from just after it"
    );
}

#[test]
fn every_single_character_escape_is_expanded() {
    assert_eq!(unescape(r#"\""#), "\"", r#"\" → \""#);
    assert_eq!(unescape(r"\\"), r"\", r"\\ → \");
    assert_eq!(unescape(r"\/"), "/", r"\/ → /");
    assert_eq!(unescape(r"\b"), "\u{8}", r"\b → backspace");
    assert_eq!(unescape(r"\f"), "\u{c}", r"\f → form feed");
    assert_eq!(unescape(r"\n"), "\n", r"\n → newline");
    assert_eq!(unescape(r"\r"), "\r", r"\r → carriage return");
    assert_eq!(unescape(r"\t"), "\t", r"\t → tab");
}

#[test]
fn escapes_advance_past_both_characters() {
    assert_eq!(unescape(r"\n\n"), "\n\n", "two escapes, four bytes in");
    assert_eq!(unescape(r"a\nb"), "a\nb", "literals either side survive");
}

#[test]
fn a_unicode_escape_is_the_character_it_names() {
    assert_eq!(unescape(r"\u0041"), "A", r"\u0041 is A");
    assert_eq!(unescape(r"\u00e9"), "é", r"\u00e9 is e-acute");
}

#[test]
fn a_unicode_escape_is_expanded_wherever_it_sits_in_the_body() {
    assert_eq!(unescape(r"a\u0042b"), "aBb", "between literals");
    assert_eq!(unescape(r"\u0041\u0042"), "AB", "after another escape");
    assert_eq!(
        unescape(r"\u0041tail\u0042"),
        "AtailB",
        "two, with a run between"
    );
}

#[test]
fn the_four_digits_are_read_from_the_right_place() {
    assert_eq!(
        unicode_escape(b"\\u0041", 2),
        Some('A'),
        "the digits start after the backslash and the u"
    );
    assert_eq!(
        unicode_escape(b"0041", 0),
        Some('A'),
        "and reading from the start of a bare digit run works too"
    );
}

#[test]
fn a_unicode_escape_fails_when_the_digits_are_not_there() {
    assert_eq!(unicode_escape(b"", 0), None, "no bytes at all");
    assert_eq!(unicode_escape(b"00", 0), None, "two digits");
    assert_eq!(unicode_escape(b"004", 0), None, "three digits");
}

#[test]
fn a_unicode_escape_fails_when_the_digits_are_not_hex() {
    assert_eq!(unicode_escape(b"00zz", 0), None, "z is not a hex digit");
    assert_eq!(unicode_escape(b"  41", 0), None, "a space is not either");
}

#[test]
fn a_surrogate_is_not_a_character() {
    assert_eq!(
        unicode_escape(b"d800", 0),
        None,
        "a surrogate half is not a character"
    );
    assert!(char::from_u32(0xD800).is_none(), "which is the reason");
}

#[test]
fn a_truncated_unicode_escape_falls_through_to_the_literal_branch() {
    assert_eq!(
        unescape(r"\u00"),
        "u00",
        "the u is kept and the scan continues from the digits"
    );
}

#[test]
fn surrogate_pairs_decode_and_lone_surrogates_emit_replacement() {
    for (input, expected, label) in [
        (r"\ud83d\ude00", "😀", "emoji surrogate pair"),
        (r"\ud800\udc00", "\u{10000}", "minimum supplementary scalar"),
        (r"\udbff\udfff", "\u{10FFFF}", "maximum Unicode scalar"),
        (r"\ud800", "\u{FFFD}", "lone high surrogate at end"),
        (
            r"\ud800tail",
            "\u{FFFD}tail",
            "lone high surrogate before text",
        ),
        (
            r"\ud800\u0041",
            "\u{FFFD}A",
            "lone high surrogate before BMP escape keeps the BMP escape",
        ),
        (
            r"\udc00tail",
            "\u{FFFD}tail",
            "lone low surrogate before text",
        ),
        (r"\udbff\udc00", "\u{10FC00}", "high boundary DBFF + DC00"),
        (r"\udfff", "\u{FFFD}", "lone low surrogate max"),
        (r"\ud7ff", "\u{D7FF}", "BMP just below surrogates"),
        (r"\ue000", "\u{E000}", "BMP just above surrogates"),
        (r"\u00zzB", "B", "invalid hex digits still advance by 6"),
    ] {
        assert_eq!(unescape(input), expected, "{label}: {input}");
    }
}

#[test]
fn a_run_of_plain_characters_is_copied_in_one_go() {
    assert_eq!(
        unescape(r"hello\nworld"),
        "hello\nworld",
        "one run, one escape"
    );
    assert_eq!(unescape(r"\na\rb\tc"), "\na\rb\tc", "a leading escape");
    assert_eq!(
        unescape(r"a\tb\tc"),
        "a\tb\tc",
        "no run before the first escape"
    );
}

#[test]
fn a_run_stops_at_the_first_backslash() {
    assert_eq!(unescape(r"abc\n"), "abc\n", "and nothing past it");
}

#[test]
fn a_backslash_at_the_end_is_a_literal() {
    assert_eq!(
        unescape("abc\\"),
        "abc\\",
        "a trailing backslash is literal"
    );
    assert_eq!(unescape("\\"), "\\", "and a lone one");
}

#[test]
fn invalid_utf8_is_replaced_rather_than_rejected() {
    let out = unescape_json_string(&[b'a', 0xFF, b'b']);
    assert_eq!(
        out.chars().count(),
        3,
        "the bad byte becomes one replacement character"
    );
    assert!(
        out.contains('\u{FFFD}'),
        "and it is the replacement: {out:?}"
    );
}
