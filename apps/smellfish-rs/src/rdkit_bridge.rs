// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Inline JavaScript bridge for `RDKit.js` + Ertl NP-likeness model, embedded from
//! `assets/rdkit_bridge.js`.

pub(crate) const RDKIT_BRIDGE_JS: &str = include_str!("../assets/rdkit_bridge.js");

#[cfg(test)]
mod tests {
    use super::RDKIT_BRIDGE_JS;

    #[test]
    fn compute_np_likeness_tests_fingerprint_bits_with_bitwise_and() {
        assert!(
            RDKIT_BRIDGE_JS.contains("(byte & (1 << b)) !== 0"),
            "fingerprint bit extraction must use bitwise `&`, not logical `&&`"
        );
        assert!(
            !RDKIT_BRIDGE_JS.contains("byte && (1 << b)"),
            "logical `&&` treats all 8 bits of any non-zero byte as set"
        );
    }

    #[test]
    fn bridge_uses_spelled_ertl_substituents_checks_ok_and_waits_for_motifs() {
        let typo = concat!("ERTL_", "SUBSTUENTS");
        assert!(
            !RDKIT_BRIDGE_JS.contains(typo),
            "RDKIT_BRIDGE_JS must not misspell ERTL_SUBSTITUENTS"
        );
        assert!(
            RDKIT_BRIDGE_JS.contains("ERTL_SUBSTITUENTS"),
            "RDKIT_BRIDGE_JS must destructure and map ERTL_SUBSTITUENTS"
        );
        assert!(
            RDKIT_BRIDGE_JS.contains("waitForSmellfishMotifs"),
            "RDKIT_BRIDGE_JS should wait briefly for motif-library.js if script execution races"
        );
        assert!(
            RDKIT_BRIDGE_JS.contains("response.ok") && RDKIT_BRIDGE_JS.contains("r.ok"),
            "all bridge fetches (np_model.bin and lotus_1percent_scaffolds.txt) must check .ok"
        );
    }
}
