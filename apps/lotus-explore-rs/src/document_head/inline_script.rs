// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Inline JavaScript for the document head, split into small bridge snippets.
//!
//! Handles synchronous app bootstrap (Service Worker registration & non-blocking analytics)
//! alongside lazy-loaded bridges for RDKit and Citation.js.

const BOOTSTRAP_INLINE_SCRIPT: &str = r#"
(function() {
    // 1. Service Worker: Cache-First strategy for WASM and JS assets
    // Overrides GitHub Pages default 10-minute (600s) Cache-Control header.
    if ('serviceWorker' in navigator) {
    // Wait until idle or after DOMContentLoaded to register sw.js
    var registerSw = function() {
        navigator.serviceWorker.register('./sw.js').catch(function() {});
    };
    if ('requestIdleCallback' in window) {
        requestIdleCallback(registerSw);
    } else {
        setTimeout(registerSw, 2000);
    }
}

    // 2. Non-blocking Analytics Initialization
    // Defers loading Simple Analytics to unblock initial WASM instantiation and LCP.
    var loadAnalytics = function() {
        if (document.querySelector('script[src*="simpleanalyticscdn.com"]')) return;
        var s = document.createElement('script');
        s.async = true;
        s.defer = true;
        s.src = 'https://scripts.simpleanalyticscdn.com/latest.js';
        document.head.appendChild(s);
    };

    if ('requestIdleCallback' in window) {
        requestIdleCallback(loadAnalytics, { timeout: 2000 });
    } else {
        setTimeout(loadAnalytics, 1500);
    }
})();
"#;

const RDKIT_BRIDGE_SCRIPT: &str = r#"
const RDKIT_JS_SRC = "https://unpkg.com/@rdkit/rdkit/dist/RDKit_minimal.js";
let rdkitScriptLoadPromise = null;
let rdkitReadyPromise = null;

function waitForInitRDKitModule(timeoutMs = 12000) {
    const start = Date.now();
    return new Promise((resolve, reject) => {
        function poll() {
            if (typeof initRDKitModule === "function") {
                resolve(initRDKitModule);
                return;
            }
            if (Date.now() - start >= timeoutMs) {
                reject(new Error("RDKit_minimal.js is not loaded"));
                return;
            }
            setTimeout(poll, 16);
        }
        poll();
    });
}

function loadRdkitJs() {
    if (typeof initRDKitModule === "function") {
        return Promise.resolve(initRDKitModule);
    }
    if (rdkitScriptLoadPromise) {
        return rdkitScriptLoadPromise;
    }

    rdkitScriptLoadPromise = new Promise((resolve, reject) => {
        const existing = document.querySelector(`script[src="${RDKIT_JS_SRC}"]`);
        const complete = () => {
            waitForInitRDKitModule().then(resolve, reject);
        };

        if (existing) {
            complete();
            return;
        }

        const script = document.createElement("script");
        script.src = RDKIT_JS_SRC;
        script.async = true;
        script.crossOrigin = "anonymous";
        script.addEventListener("load", complete, { once: true });
        script.addEventListener(
            "error",
            () => reject(new Error("RDKit_minimal.js failed to load")),
            { once: true }
        );
        document.head.appendChild(script);
    }).catch((error) => {
        rdkitScriptLoadPromise = null;
        throw error;
    });

    return rdkitScriptLoadPromise;
}

function ensureRdkitReady() {
    if (rdkitReadyPromise) {
        return rdkitReadyPromise;
    }

    rdkitReadyPromise = (async () => {
        const init = await loadRdkitJs();
        const RDKit = await init();

        function withMol(smiles, callback) {
            const trimmed = String(smiles || "").trim();
            if (!trimmed) {
                throw new Error("smiles is required");
            }
            const mol = RDKit.get_mol(trimmed);
            if (!mol) {
                throw new Error("rdkit.js could not parse the structure");
            }
            try {
                return callback(mol);
            } finally {
                if (typeof mol.delete === "function") {
                    mol.delete();
                }
            }
        }

        function descriptorExactMass(descriptors) {
            const keys = [
                "exact_molecular_weight",
                "ExactMolWt",
                "exactmw",
                "exact_mw",
            ];
            for (const key of keys) {
                const value = descriptors?.[key];
                if (typeof value === "number" && Number.isFinite(value)) {
                    return value;
                }
            }
            throw new Error("rdkit.js descriptors did not include exact mass");
        }

        function stripStereoFromSmiles(smiles) {
            return smiles.replace(/@{1,2}/g, "").replace(/[/\\]/g, "");
        }

        return {
            convert(smiles) {
                return withMol(smiles, (mol) => {
                    const isomericsmiles = mol.get_smiles(
                        JSON.stringify({ canonical: true, isomericSmiles: true })
                    );
                    let canonicalsmiles = mol.get_smiles(
                        JSON.stringify({ canonical: true, isomericSmiles: false })
                    );
                    if (
                        canonicalsmiles.includes("@") ||
                        canonicalsmiles.includes("/") ||
                        canonicalsmiles.includes("\\")
                    ) {
                        canonicalsmiles = stripStereoFromSmiles(canonicalsmiles);
                    }
                    const inchi = mol.get_inchi();
                    if (!inchi) {
                        throw new Error("rdkit.js could not generate InChI");
                    }
                    const inchikey = RDKit.get_inchikey_for_inchi(inchi);
                    if (!inchikey) {
                        throw new Error("rdkit.js could not generate InChIKey");
                    }
                    return { canonicalsmiles, isomericsmiles, inchi, inchikey };
                });
            },
            exactMass(smiles) {
                return withMol(smiles, (mol) => {
                    const descriptors = JSON.parse(mol.get_descriptors());
                    return descriptorExactMass(descriptors);
                });
            },
            hasUndefinedStereo(smiles) {
                return withMol(smiles, (mol) => {
                    const tags = JSON.parse(mol.get_stereo_tags() || "[]");
                    if (!Array.isArray(tags)) {
                        return false;
                    }
                    return tags.some((tag) => {
                        const text = String(tag || "").toLowerCase();
                        return text.includes("unspecified") || text.includes("unknown");
                    });
                });
            },
        };
    })().catch((error) => {
        rdkitReadyPromise = null;
        throw error;
    });

    return rdkitReadyPromise;
}

window.__lotusRdkit = {
    ready() {
        return ensureRdkitReady();
    },
    async convert(smiles) {
        const bridge = await ensureRdkitReady();
        return bridge.convert(smiles);
    },
    async exactMass(smiles) {
        const bridge = await ensureRdkitReady();
        return bridge.exactMass(smiles);
    },
    async hasUndefinedStereo(smiles) {
        const bridge = await ensureRdkitReady();
        return bridge.hasUndefinedStereo(smiles);
    },
};
"#;

const CITATION_BRIDGE_SCRIPT: &str = r#"
const CITATION_JS_SRC = "https://tools-static.wmflabs.org/scholia/js/citation.js";
let citationJsLoadPromise = null;

function loadCitationJs() {
    if (typeof globalThis.__Cite !== "undefined") {
        return Promise.resolve(globalThis.__Cite);
    }
    if (citationJsLoadPromise) {
        return citationJsLoadPromise;
    }

    citationJsLoadPromise = new Promise((resolve, reject) => {
        const existing = document.querySelector(`script[src="${CITATION_JS_SRC}"]`);
        const complete = () => {
            try {
                const CiteCtor = require("citation-js").Cite;
                if (typeof CiteCtor === "function") {
                    globalThis.__Cite = CiteCtor;
                    resolve(CiteCtor);
                } else {
                    reject(new Error("citation.js loaded but did not expose a Cite constructor"));
                }
            } catch (_error) {
                reject(new Error("Scholia citation.js bundle failed to expose Cite"));
            }
        };

        if (existing) {
            existing.addEventListener("load", complete, { once: true });
            existing.addEventListener(
                "error",
                () => reject(new Error("citation.js failed to load")),
                { once: true }
            );
            return;
        }

        const script = document.createElement("script");
        script.src = CITATION_JS_SRC;
        script.async = true;
        script.crossOrigin = "anonymous";
        script.addEventListener("load", complete, { once: true });
        script.addEventListener(
            "error",
            () => reject(new Error("citation.js failed to load")),
            { once: true }
        );
        document.head.appendChild(script);
    }).catch((error) => {
        citationJsLoadPromise = null;
        throw error;
    });

    return citationJsLoadPromise;
}

async function resolveCitationBridge() {
    if (typeof globalThis.__Cite === "function") {
        return globalThis.__Cite;
    }
    return await loadCitationJs();
}

async function fetchDoiCslJson(doi) {
    const response = await fetch(`https://doi.org/${encodeURIComponent(doi)}`, {
        headers: {
            Accept: "application/vnd.citationstyles.csl+json, application/json;q=0.9",
        },
    });
    if (!response.ok) {
        throw new Error(`DOI metadata fetch failed: HTTP ${response.status}`);
    }
    return await response.json();
}

window.__lotusCitation = {
    async quickStatements(doi) {
        const trimmed = String(doi || "").trim();
        if (!trimmed) {
            return "";
        }
        const CiteCtor = await resolveCitationBridge();
        const cslJson = await fetchDoiCslJson(trimmed);
        let cite;
        try {
            cite = new CiteCtor(cslJson);
        } catch (_ctorError) {
            cite = await CiteCtor.async(cslJson);
        }
        const output = cite.format('quickstatements');
        return typeof output === "string" ? output : "";
    },
};
"#;

/// Primary bootstrap script passed to `DocumentHead { inline_script }`.
/// Registers the Service Worker and schedules analytics injection during browser idle time.
pub fn build_bootstrap_inline_script() -> String {
    BOOTSTRAP_INLINE_SCRIPT.to_string()
}

/// Lazy-loaded bridge code for RDKit and Citation.js on curation routes.
pub fn build_curation_inline_script() -> String {
    [RDKIT_BRIDGE_SCRIPT, CITATION_BRIDGE_SCRIPT].join("\n\n")
}
