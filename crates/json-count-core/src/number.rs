// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! RFC 8259 bare scalar and number validation.

use crate::error::JsonCountError;

/// Maximum length of a single bare scalar token (`64 KiB`).
pub(crate) const MAX_SCALAR_BYTES: usize = 65_536;

pub(crate) fn push_scalar_byte(token_buf: &mut Vec<u8>, byte: u8) -> Result<(), JsonCountError> {
    if token_buf.len() >= MAX_SCALAR_BYTES {
        return Err(JsonCountError::ScalarTooLong);
    }
    token_buf.push(byte);
    Ok(())
}

pub(crate) fn validate_bare_scalar(token: &[u8]) -> Result<u64, JsonCountError> {
    if token == b"null" {
        return Ok(0);
    }
    if token == b"true" || token == b"false" {
        return Ok(1);
    }
    if is_valid_json_number(token) {
        return Ok(1);
    }
    Err(JsonCountError::InvalidBareScalar)
}

fn consume_digits(bytes: &[u8]) -> Option<&[u8]> {
    let count = bytes
        .iter()
        .position(|b| !b.is_ascii_digit())
        .unwrap_or(bytes.len());
    (count > 0).then(|| bytes.get(count..).unwrap_or_default())
}

/// Validates whether `bytes` conforms to the RFC 8259 JSON number grammar.
#[must_use]
pub(crate) fn is_valid_json_number(bytes: &[u8]) -> bool {
    let rest = bytes.strip_prefix(b"-").unwrap_or(bytes);
    let Some(rest) = rest.strip_prefix(b"0").or_else(|| consume_digits(rest)) else {
        return false;
    };

    let rest = if let Some(frac) = rest.strip_prefix(b".") {
        let Some(after_frac) = consume_digits(frac) else {
            return false;
        };
        after_frac
    } else {
        rest
    };

    let rest = if let Some(exp) = rest.strip_prefix(b"e").or_else(|| rest.strip_prefix(b"E")) {
        let exp = exp
            .strip_prefix(b"+")
            .or_else(|| exp.strip_prefix(b"-"))
            .unwrap_or(exp);
        let Some(after_exp) = consume_digits(exp) else {
            return false;
        };
        after_exp
    } else {
        rest
    };

    rest.is_empty()
}
