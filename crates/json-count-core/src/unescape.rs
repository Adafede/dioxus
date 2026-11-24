// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! JSON string unescaping including `\uXXXX` and UTF-16 surrogate pairs.

/// Parses four ASCII hex digits starting at `at` into a UTF-16 code unit.
#[must_use]
pub(crate) fn unicode_code_unit(raw: &[u8], at: usize) -> Option<u16> {
    let digits = raw.get(at..at + 4)?;
    let hex = std::str::from_utf8(digits).ok()?;
    u16::from_str_radix(hex, 16).ok()
}

fn decode_unicode_escape(raw: &[u8], at: usize, out: &mut String) -> usize {
    let Some(code) = unicode_code_unit(raw, at + 2) else {
        return 6;
    };
    if (0xD800..=0xDBFF).contains(&code)
        && raw.get(at + 6..at + 8) == Some(b"\\u")
        && let Some(low) = unicode_code_unit(raw, at + 8)
        && (0xDC00..=0xDFFF).contains(&low)
    {
        let scalar = 0x10000 + ((u32::from(code) - 0xD800) << 10) + (u32::from(low) - 0xDC00);
        out.push(char::from_u32(scalar).unwrap_or('\u{FFFD}'));
        return 12;
    }
    if (0xD800..=0xDFFF).contains(&code) {
        out.push('\u{FFFD}');
        return 6;
    }
    out.push(char::from_u32(u32::from(code)).unwrap_or('\u{FFFD}'));
    6
}

const fn single_char_escape(escaped: u8) -> char {
    match escaped {
        b'"' => '"',
        b'\\' => '\\',
        b'/' => '/',
        b'b' => '\u{8}',
        b'f' => '\u{c}',
        b'n' => '\n',
        b'r' => '\r',
        b't' => '\t',
        other => other as char,
    }
}

/// Unescapes `raw` JSON string bytes into `out` without allocating if `out` has capacity.
pub(crate) fn unescape_json_string_into(raw: &[u8], out: &mut String) {
    out.clear();
    if !raw.contains(&b'\\') {
        out.push_str(&String::from_utf8_lossy(raw));
        return;
    }

    out.reserve(raw.len());
    let mut i = 0;
    while let Some(&byte) = raw.get(i) {
        if byte != b'\\' {
            let rest = raw.get(i..).unwrap_or_default();
            let next = rest
                .iter()
                .position(|&c| c == b'\\')
                .map_or(raw.len(), |p| i + p);
            if let Some(chunk) = raw.get(i..next) {
                out.push_str(&String::from_utf8_lossy(chunk));
            }
            i = next;
            continue;
        }
        let Some(&escaped) = raw.get(i + 1) else {
            if let Some(chunk) = raw.get(i..) {
                out.push_str(&String::from_utf8_lossy(chunk));
            }
            break;
        };
        if escaped == b'u' && raw.get(i + 2..i + 6).is_some() {
            i += decode_unicode_escape(raw, i, out);
        } else {
            out.push(single_char_escape(escaped));
            i += 2;
        }
    }
}

/// Unescapes a raw JSON string body, including `\uXXXX` and UTF-16 surrogate pairs.
#[cfg(test)]
#[must_use]
pub(crate) fn unescape_json_string(raw: &[u8]) -> String {
    let mut out = String::new();
    unescape_json_string_into(raw, &mut out);
    out
}
