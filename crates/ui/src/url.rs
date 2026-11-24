// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! URL encoding and hostname validation helpers.

use std::fmt::Write as _;

/// Percent-encodes a URL path segment or query parameter value per RFC 3986 unreserved bytes.
pub(crate) fn percent_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

/// Validates a DNS hostname according to RFC 1035/1123 label rules before interpolating into URLs.
pub(crate) fn is_valid_domain(domain: &str) -> bool {
    !domain.is_empty()
        && domain.len() <= 253
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encode_path_encodes_slashes_and_special_chars() {
        assert_eq!(percent_encode_path("/"), "%2F");
        assert_eq!(
            percent_encode_path("/dioxus/cxsmiles-yoga/"),
            "%2Fdioxus%2Fcxsmiles-yoga%2F"
        );
    }

    #[test]
    fn validates_hostnames_and_rejects_injection_payloads() {
        assert!(is_valid_domain("tools.example.org"));
        assert!(!is_valid_domain(""));
        assert!(!is_valid_domain("-bad"));
        assert!(!is_valid_domain("a..b"));
        assert!(!is_valid_domain("foo/bar"));
        assert!(!is_valid_domain("evil.example\" onload=\"alert(1)"));
    }
}
