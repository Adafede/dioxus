// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! JSON string validation and unescaping including `\uXXXX` and UTF-16 surrogate pairs.

use crate::error::JsonCountError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum EscState {
    #[default]
    Normal,
    AfterBackslash,
    UnicodeHex(u8),
}

/// Incremental RFC 8259 JSON string byte validator (escapes, control chars, UTF-8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StringValidator {
    esc: EscState,
    utf8_buf: [u8; 4],
    utf8_len: u8,
    utf8_target: u8,
    has_backslash: bool,
}

impl StringValidator {
    #[must_use]
    pub(crate) const fn has_backslash(self) -> bool {
        self.has_backslash
    }

    /// Returns the length of the leading plain-ASCII prefix (`0x20..=0x7F`,
    /// excluding `"` and `\`) that needs no escape or multi-byte UTF-8 state.
    #[inline]
    #[must_use]
    pub(crate) fn plain_ascii_run(self, tail: &[u8]) -> usize {
        if self.utf8_len != 0 || !matches!(self.esc, EscState::Normal) {
            return 0;
        }
        tail.iter()
            .take_while(|&&b| (0x20..0x80).contains(&b) && b != b'"' && b != b'\\')
            .count()
    }

    /// Feeds one byte inside an open JSON string.
    ///
    /// Returns `Ok(true)` when an unescaped closing `"` ends the string.
    ///
    /// # Errors
    ///
    /// Returns [`JsonCountError`] on unescaped control chars (`< 0x20`),
    /// invalid escape sequences, or invalid UTF-8 sequences.
    pub(crate) fn step(&mut self, byte: u8) -> Result<bool, JsonCountError> {
        if self.utf8_len > 0 {
            if (byte & 0b1100_0000) != 0b1000_0000 {
                return Err(JsonCountError::InvalidUtf8);
            }
            let idx = usize::from(self.utf8_len);
            if let Some(slot) = self.utf8_buf.get_mut(idx) {
                *slot = byte;
            }
            self.utf8_len += 1;
            if self.utf8_len == self.utf8_target {
                let target = usize::from(self.utf8_target);
                let slice = self.utf8_buf.get(..target).unwrap_or_default();
                if std::str::from_utf8(slice).is_err() {
                    return Err(JsonCountError::InvalidUtf8);
                }
                self.utf8_len = 0;
                self.utf8_target = 0;
            }
            return Ok(false);
        }

        match self.esc {
            EscState::UnicodeHex(rem) => {
                if !byte.is_ascii_hexdigit() {
                    return Err(JsonCountError::InvalidEscape);
                }
                self.esc = if rem <= 1 {
                    EscState::Normal
                } else {
                    EscState::UnicodeHex(rem - 1)
                };
                Ok(false)
            }
            EscState::AfterBackslash => {
                if byte == b'u' {
                    self.esc = EscState::UnicodeHex(4);
                } else {
                    let _ = single_char_escape(byte)?;
                    self.esc = EscState::Normal;
                }
                Ok(false)
            }
            EscState::Normal => match byte {
                b'"' => Ok(true),
                b'\\' => {
                    self.has_backslash = true;
                    self.esc = EscState::AfterBackslash;
                    Ok(false)
                }
                0x00..=0x1F => Err(JsonCountError::UnescapedControlChar),
                0x20..=0x7F => Ok(false),
                0xC2..=0xDF => {
                    self.utf8_buf[0] = byte;
                    self.utf8_len = 1;
                    self.utf8_target = 2;
                    Ok(false)
                }
                0xE0..=0xEF => {
                    self.utf8_buf[0] = byte;
                    self.utf8_len = 1;
                    self.utf8_target = 3;
                    Ok(false)
                }
                0xF0..=0xF4 => {
                    self.utf8_buf[0] = byte;
                    self.utf8_len = 1;
                    self.utf8_target = 4;
                    Ok(false)
                }
                _ => Err(JsonCountError::InvalidUtf8),
            },
        }
    }
}

/// Parses four ASCII hex digits starting at `at` into a UTF-16 code unit.
#[must_use]
pub(crate) fn unicode_code_unit(raw: &[u8], at: usize) -> Option<u16> {
    let digits = raw.get(at..at + 4)?;
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let hex = std::str::from_utf8(digits).ok()?;
    u16::from_str_radix(hex, 16).ok()
}

fn decode_unicode_escape(raw: &[u8], at: usize, out: &mut String) -> Result<usize, JsonCountError> {
    let code = unicode_code_unit(raw, at + 2).ok_or(JsonCountError::InvalidEscape)?;
    if (0xD800..=0xDBFF).contains(&code)
        && raw.get(at + 6..at + 8) == Some(b"\\u")
        && let Some(low) = unicode_code_unit(raw, at + 8)
        && (0xDC00..=0xDFFF).contains(&low)
    {
        let scalar = 0x10000 + ((u32::from(code) - 0xD800) << 10) + (u32::from(low) - 0xDC00);
        out.push(char::from_u32(scalar).unwrap_or('\u{FFFD}'));
        return Ok(12);
    }
    if (0xD800..=0xDFFF).contains(&code) {
        out.push('\u{FFFD}');
        return Ok(6);
    }
    out.push(char::from_u32(u32::from(code)).unwrap_or('\u{FFFD}'));
    Ok(6)
}

const fn single_char_escape(escaped: u8) -> Result<char, JsonCountError> {
    match escaped {
        b'"' => Ok('"'),
        b'\\' => Ok('\\'),
        b'/' => Ok('/'),
        b'b' => Ok('\u{8}'),
        b'f' => Ok('\u{c}'),
        b'n' => Ok('\n'),
        b'r' => Ok('\r'),
        b't' => Ok('\t'),
        _ => Err(JsonCountError::InvalidEscape),
    }
}

fn push_verified_slice(chunk: &[u8], out: &mut String) -> Result<(), JsonCountError> {
    if chunk.iter().any(|&b| b < 0x20) {
        return Err(JsonCountError::UnescapedControlChar);
    }
    let text = std::str::from_utf8(chunk).map_err(|_| JsonCountError::InvalidUtf8)?;
    out.push_str(text);
    Ok(())
}

/// Unescapes `raw` bytes already validated by [`StringValidator`], skipping
/// backslash and control-character rescans when `has_backslash` is `false`.
pub(crate) fn unescape_validated_json_string_into(
    raw: &[u8],
    has_backslash: bool,
    out: &mut String,
) -> Result<(), JsonCountError> {
    out.clear();
    if !has_backslash {
        let text = std::str::from_utf8(raw).map_err(|_| JsonCountError::InvalidUtf8)?;
        out.push_str(text);
        return Ok(());
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
                push_verified_slice(chunk, out)?;
            }
            i = next;
            continue;
        }
        let Some(&escaped) = raw.get(i + 1) else {
            return Err(JsonCountError::InvalidEscape);
        };
        if escaped == b'u' {
            i += decode_unicode_escape(raw, i, out)?;
        } else {
            out.push(single_char_escape(escaped)?);
            i += 2;
        }
    }
    Ok(())
}

/// Validates and unescapes `raw` JSON string bytes into `out`.
///
/// # Errors
///
/// Returns [`JsonCountError`] on unescaped control bytes, invalid escape sequences,
/// or invalid UTF-8.
#[cfg(test)]
pub(crate) fn unescape_json_string_into(
    raw: &[u8],
    out: &mut String,
) -> Result<(), JsonCountError> {
    if !raw.contains(&b'\\') {
        out.clear();
        return push_verified_slice(raw, out);
    }
    unescape_validated_json_string_into(raw, true, out)
}

/// Unescapes a raw JSON string body, including `\uXXXX` and UTF-16 surrogate pairs.
#[cfg(test)]
pub(crate) fn unescape_json_string(raw: &[u8]) -> Result<String, JsonCountError> {
    let mut out = String::new();
    unescape_json_string_into(raw, &mut out)?;
    Ok(out)
}
