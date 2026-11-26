// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

use crate::JsonCountError;
use crate::unescape::{unescape_json_string, unicode_code_unit};

fn unescape(s: &str) -> Result<String, JsonCountError> {
    unescape_json_string(s.as_bytes())
}

fn unicode_escape(raw: &[u8], at: usize) -> Option<char> {
    unicode_code_unit(raw, at).and_then(|code| char::from_u32(u32::from(code)))
}

#[test]
fn a_string_with_no_backslash_is_returned_unchanged() {
    assert_eq!(
        unescape("plain text"),
        Ok("plain text".to_string()),
        "no escapes to expand"
    );
    assert_eq!(unescape(""), Ok(String::new()), "and nothing at all");
}

#[test]
fn unknown_escapes_and_control_bytes_are_rejected() {
    assert_eq!(unescape(r"a\zc"), Err(JsonCountError::InvalidEscape));
    assert_eq!(unescape(r"\za\nb"), Err(JsonCountError::InvalidEscape));
    assert_eq!(
        unescape("line\nbreak"),
        Err(JsonCountError::UnescapedControlChar)
    );
    assert_eq!(
        unescape("nul\x00byte"),
        Err(JsonCountError::UnescapedControlChar)
    );
}

#[test]
fn every_single_character_escape_is_expanded() {
    assert_eq!(unescape(r#"\""#), Ok("\"".to_string()), r#"\" → \""#);
    assert_eq!(unescape(r"\\"), Ok(r"\".to_string()), r"\\ → \");
    assert_eq!(unescape(r"\/"), Ok("/".to_string()), r"\/ → /");
    assert_eq!(unescape(r"\b"), Ok("\u{8}".to_string()), r"\b → backspace");
    assert_eq!(unescape(r"\f"), Ok("\u{c}".to_string()), r"\f → form feed");
    assert_eq!(unescape(r"\n"), Ok("\n".to_string()), r"\n → newline");
    assert_eq!(
        unescape(r"\r"),
        Ok("\r".to_string()),
        r"\r → carriage return"
    );
    assert_eq!(unescape(r"\t"), Ok("\t".to_string()), r"\t → tab");
}

#[test]
fn escapes_advance_past_both_characters() {
    assert_eq!(
        unescape(r"\n\n"),
        Ok("\n\n".to_string()),
        "two escapes, four bytes in"
    );
    assert_eq!(
        unescape(r"a\nb"),
        Ok("a\nb".to_string()),
        "literals either side survive"
    );
}

#[test]
fn a_unicode_escape_is_the_character_it_names() {
    assert_eq!(unescape(r"\u0041"), Ok("A".to_string()), r"\u0041 is A");
    assert_eq!(
        unescape(r"\u00e9"),
        Ok("é".to_string()),
        r"\u00e9 is e-acute"
    );
}

#[test]
fn a_unicode_escape_is_expanded_wherever_it_sits_in_the_body() {
    assert_eq!(
        unescape(r"a\u0042b"),
        Ok("aBb".to_string()),
        "between literals"
    );
    assert_eq!(
        unescape(r"\u0041\u0042"),
        Ok("AB".to_string()),
        "after another escape"
    );
    assert_eq!(
        unescape(r"\u0041tail\u0042"),
        Ok("AtailB".to_string()),
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
    assert_eq!(unicode_escape(b"+041", 0), None, "a sign is not hex");
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
fn truncated_or_malformed_unicode_escapes_are_rejected() {
    assert_eq!(unescape(r"\u00"), Err(JsonCountError::InvalidEscape));
    assert_eq!(unescape(r"\u+041"), Err(JsonCountError::InvalidEscape));
    assert_eq!(unescape(r"\u00zzB"), Err(JsonCountError::InvalidEscape));
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
    ] {
        assert_eq!(
            unescape(input),
            Ok(expected.to_string()),
            "{label}: {input}"
        );
    }
}

#[test]
fn a_run_of_plain_characters_is_copied_in_one_go() {
    assert_eq!(
        unescape(r"hello\nworld"),
        Ok("hello\nworld".to_string()),
        "one run, one escape"
    );
    assert_eq!(
        unescape(r"\na\rb\tc"),
        Ok("\na\rb\tc".to_string()),
        "a leading escape"
    );
    assert_eq!(
        unescape(r"a\tb\tc"),
        Ok("a\tb\tc".to_string()),
        "no run before the first escape"
    );
}

#[test]
fn a_run_stops_at_the_first_backslash() {
    assert_eq!(
        unescape(r"abc\n"),
        Ok("abc\n".to_string()),
        "and nothing past it"
    );
}

#[test]
fn trailing_backslash_and_invalid_utf8_are_rejected() {
    assert_eq!(unescape("abc\\"), Err(JsonCountError::InvalidEscape));
    assert_eq!(unescape("\\"), Err(JsonCountError::InvalidEscape));
    assert_eq!(
        unescape_json_string(&[b'a', 0xFF, b'b']),
        Err(JsonCountError::InvalidUtf8)
    );
}
