// The `tests` tests, extracted from `depict_simple.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `depict_simple` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn the_unreserved_set_is_left_alone() {
    assert_eq!(
        encode("abcXYZ019-_.~"),
        "abcXYZ019-_.~",
        "letters, digits, dash, underscore, dot and tilde are untouched"
    );
}

#[test]
fn a_space_is_a_percent_twenty_and_not_a_plus() {
    // The value goes in a query string, where `+` means a space only in a
    // form body. A `+` here would be part of the SMILES.
    assert_eq!(encode("a b"), "a%20b", "space");
}

#[test]
fn a_reserved_character_is_escaped() {
    assert_eq!(encode("a#b"), "a%23b", "hash");
    assert_eq!(encode("a/b"), "a%2Fb", "slash");
    assert_eq!(encode("a?b"), "a%3Fb", "question mark");
    assert_eq!(encode("C=C"), "C%3DC", "equals");
}

#[test]
fn a_multi_byte_character_is_encoded_byte_by_byte() {
    // The one the old encoder got wrong. U+00E9 is two bytes in UTF-8, so it
    // is two percent-escapes; `c as u8` gave one, for U+00E9's low byte.
    assert_eq!(encode("\u{e9}"), "%C3%A9", "two UTF-8 bytes, two escapes");
}

#[test]
fn a_three_byte_character_gives_three_escapes() {
    assert_eq!(
        encode("\u{20ac}"),
        "%E2%82%AC",
        "the euro sign, three bytes"
    );
}

#[test]
fn the_url_carries_the_encoded_smiles() {
    let html = render_svg("C=C");
    assert!(
        html.contains("smi=C%3DC"),
        "the escaped value is in the query string: {html}"
    );
    assert!(
        html.starts_with("<img src=\"https://"),
        "and it is an img: {html}"
    );
}
