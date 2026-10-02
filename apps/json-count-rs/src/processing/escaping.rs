// The `escaping` tests, extracted from `processing.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `processing` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::{unescape_json_string, unicode_escape};

fn unescape(s: &str) -> String {
    unescape_json_string(s.as_bytes())
}

// ── the plain pass-through ───────────────────────────────────────────────

#[test]
fn a_string_with_no_backslash_is_returned_unchanged() {
    // The early return: without a `\\` there is nothing to unescape, and
    // scanning for one costs a pass over the bytes.
    assert_eq!(unescape("plain text"), "plain text", "no escapes to expand");
    assert_eq!(unescape(""), "", "and nothing at all");
}

#[test]
fn a_backslash_is_what_arms_the_scanner() {
    // One backslash anywhere means the string is escaped, so the rest of the
    // function has to survive the mixed case: escapes and literals together.
    // `z` is not one of the eight escapes, so it stands for an unknown one.
    //
    // The backslash is *dropped* and only the character kept. That is lossy:
    // `a\zb` and `azb` both unescape to `azb`, so a malformed file cannot be
    // told from a well-formed one after this. Keeping the backslash would be
    // the other defensible choice, but that is a behaviour change and not
    // one to make while adding tests — so it is pinned here as what the code
    // does, which is the first step to changing it deliberately.
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

// ── the two-character escapes ───────────────────────────────────────────

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
    // Two characters per escape: advancing one would leave the second to be
    // emitted as a literal, and this is the assertion that catches it.
    assert_eq!(unescape(r"\n\n"), "\n\n", "two escapes, four bytes in");
    assert_eq!(unescape(r"a\nb"), "a\nb", "literals either side survive");
}

// ── \uXXXX ───────────────────────────────────────────────────────────────

#[test]
fn a_unicode_escape_is_the_character_it_names() {
    // 0041 is 'A'; 00e9 is 'é'. Both are BMP, which is all this handles.
    assert_eq!(unescape(r"A"), "A", r"A is A");
    assert_eq!(unescape(r"é"), "é", "00e9 is e-acute");
}

#[test]
fn the_four_digits_are_read_from_the_right_place() {
    // `at` is the offset of the first digit, so reading from `at` rather
    // than `at - 1` would pick up the `u` and fail to parse as hex.
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
    // Fewer than four bytes left: `get` returns None rather than slicing
    // past the end, and the caller emits nothing.
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
    // D800 is a surrogate half. `char::from_u32` refuses it, and the escape
    // is dropped rather than emitting an unpaired surrogate, which would
    // produce a string Rust cannot represent.
    assert_eq!(
        unicode_escape(b"d800", 0),
        None,
        "a surrogate half is not a character"
    );
    assert!(char::from_u32(0xD800).is_none(), "which is the reason");
}

#[test]
fn a_truncated_unicode_escape_falls_through_to_the_literal_branch() {
    // The `b'u'` arm is guarded on four digits being present. Without the
    // guard, a truncated escape would consume six bytes and emit nothing;
    // with it, the `u` is treated as an unknown escape and kept.
    assert_eq!(
        unescape(r"\u00"),
        "u00",
        "the u is kept and the scan continues from the digits"
    );
}

#[test]
fn an_invalid_unicode_escape_emits_nothing_and_still_advances() {
    // Four digits are present but are not a character: `i += 6` happens
    // either way, so a bad escape cannot make the scan loop forever.
    assert_eq!(
        unescape(r"\ud800"),
        "",
        "an unpaired surrogate contributes no character"
    );
    assert_eq!(
        unescape(r"\ud800tail"),
        "tail",
        "and the scan carries on from after the escape"
    );
}

// ── the literal run between escapes ─────────────────────────────────────

#[test]
fn a_run_of_plain_characters_is_copied_in_one_go() {
    // The `else` branch finds the next backslash and copies everything up to
    // it. A run with no backslash at all is the early return; this is the
    // run that stops at one.
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
    // `position` finds the *first* backslash, so a run cannot swallow the
    // start of the next escape.
    assert_eq!(unescape(r"abc\n"), "abc\n", "and nothing past it");
}

#[test]
fn a_backslash_at_the_end_is_a_literal() {
    // There is no second byte to pair with, so the condition fails and the
    // backslash is copied as ordinary text rather than panicking on
    // `raw[i + 1]`.
    assert_eq!(
        unescape("abc\\"),
        "abc\\",
        "a trailing backslash is literal"
    );
    assert_eq!(unescape("\\"), "\\", "and a lone one");
}

#[test]
fn invalid_utf8_is_replaced_rather_than_rejected() {
    // The body comes from a JSON file, so it can hold bytes that are not
    // UTF-8. `from_utf8_lossy` is what keeps one bad byte from failing the
    // whole column; the replacement character is the documented cost.
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
