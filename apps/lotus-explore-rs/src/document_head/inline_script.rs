// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Inline JavaScript for the document head, split into small bridge snippets.
//!
//! The critical language/trusted-types bootstrap that was previously injected
//! via `DocumentHead` (running after WASM hydration) has been moved to the
//! `index.html` template so it executes synchronously during HTML parsing —
//! eliminating the lang flicker that delayed LCP.

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
// citation.js + @citation-js/plugin-quickstatements
// Bundled together by Scholia (which uses the plugin in production for its
// DOI → QuickStatements feature). The 2.1 MB bundle includes citation.js
// (v0.8.2) and ALL its plugins, including @citation-js/plugin-quickstatements.
// After loading, require('citation-js').Cite exposes the Cite constructor
// with the 'quickstatements' output format already registered.
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

/// Inline scripts only required on the curation page: the RDKit bridge and the
/// citation bridge. Loaded lazily via [`ui::document::DocumentScripts`]
/// (mounted inside the curation view) so the bridge code stays off other views,
/// and both RDKit and citation.js only load on first use. The citation bridge
/// uses Scholia's pre-built citation.js bundle which includes citation.js
/// (v0.8.2) and the @citation-js/plugin-quickstatements output format, and
/// calls cite.format('quickstatements') to generate QuickStatements from CSL
/// JSON fetched via doi.org.
pub fn build_curation_inline_script() -> String {
    [RDKIT_BRIDGE_SCRIPT, CITATION_BRIDGE_SCRIPT].join("\n\n")
}
