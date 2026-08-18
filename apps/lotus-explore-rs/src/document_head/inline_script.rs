// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Inline JavaScript for the document head, split into small bridge snippets.

const LANG_BOOTSTRAP_SCRIPT: &str = r#"
(function installTrustedTypesPolicy() {
    const trusted = window.trustedTypes;
    if (!trusted || typeof trusted.getPolicy !== "function" || typeof trusted.createPolicy !== "function") {
        return;
    }
    try {
        if (!trusted.getPolicy("default")) {
            trusted.createPolicy("default", {
                createHTML: (value) => String(value),
                createScript: (value) => String(value),
                createScriptURL: (value) => String(value),
                createURL: (value) => String(value),
            });
        }
    } catch (_error) {
        // Browsers without Trusted Types support will ignore the policy registration.
    }
})();

(function syncDocumentLangFromQuery() {
    try {
        const params = new URL(window.location.href).searchParams;
        const lang = params.get("lang") || params.get("locale");
        if (lang) {
            document.documentElement.lang = lang;
        }
    } catch (_error) {
        // Keep the app running if the browser blocks URL parsing for any reason.
    }
})();
"#;

const RDKIT_BRIDGE_SCRIPT: &str = r#"
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

const rdkitReady = (async () => {
    const init = await waitForInitRDKitModule();
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
})();

window.__lotusRdkit = {
    ready: rdkitReady,
    async convert(smiles) {
        const bridge = await rdkitReady;
        return bridge.convert(smiles);
    },
    async exactMass(smiles) {
        const bridge = await rdkitReady;
        return bridge.exactMass(smiles);
    },
    async hasUndefinedStereo(smiles) {
        const bridge = await rdkitReady;
        return bridge.hasUndefinedStereo(smiles);
    },
};
"#;

const CITATION_BRIDGE_SCRIPT: &str = r#"
async function resolveCitationBridge() {
    let CiteCtor = globalThis.Cite;
    if (typeof CiteCtor !== "function" && typeof require === "function") {
        for (const moduleId of ["citation-js", "@citation-js/core"]) {
            try {
                const mod = require(moduleId);
                CiteCtor = mod?.default?.default || mod?.default || mod?.Cite || mod;
                if (typeof CiteCtor === "function") {
                    break;
                }
            } catch (_error) {
                // Try the next module id.
            }
        }
    }
    if (typeof CiteCtor !== "function") {
        throw new Error("citation.js is not loaded or did not expose a Cite constructor");
    }
    return CiteCtor;
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

const TOAST_BRIDGE_SCRIPT: &str = r#"
const STORAGE_KEY = "SCHEDULED-DX-TOAST";
let currentTimeout = null;
let currentToastId = 0;

function showDXToast(headerText, message, progressLevel, durationMs) {
    const decor = document.getElementById("__dx-toast-decor");
    const text = document.getElementById("__dx-toast-text");
    const msg = document.getElementById("__dx-toast-msg");
    const inner = document.getElementById("__dx-toast-inner");
    const toast = document.getElementById("__dx-toast");

    if (decor) decor.className = `dx-toast-level-bar ${progressLevel}`;
    if (text) text.innerText = headerText;
    if (msg) msg.innerText = message;
    if (inner) inner.style.right = "0";
    if (toast) {
        toast.removeAttribute("aria-hidden");
        toast.addEventListener("click", closeDXToast);
    }

    setTimeout(
        () => {
            let ourToastId = currentToastId;
            currentTimeout = setTimeout(() => {
                if (ourToastId === currentToastId) {
                    closeDXToast();
                }
            }, durationMs);
        },
        100
    );

    currentToastId += 1;
}

function scheduleDXToast(headerText, message, level, durationMs) {
    let data = {
        headerText,
        message,
        level,
        durationMs,
    };

    let jsonData = JSON.stringify(data);
    sessionStorage.setItem(STORAGE_KEY, jsonData);
}

function closeDXToast() {
    document.getElementById("__dx-toast-inner").style.right = "-1000px";
    document.getElementById("__dx-toast").setAttribute("aria-hidden", "true");
    clearTimeout(currentTimeout);
}

let potentialData = sessionStorage.getItem(STORAGE_KEY);
if (potentialData) {
    sessionStorage.removeItem(STORAGE_KEY);
    let data = JSON.parse(potentialData);
    showDXToast(data.headerText, data.message, data.level, data.durationMs);
}

window.scheduleDXToast = scheduleDXToast;
window.showDXToast = showDXToast;
window.closeDXToast = closeDXToast;
"#;

pub fn build_core_inline_script() -> String {
    [LANG_BOOTSTRAP_SCRIPT, TOAST_BRIDGE_SCRIPT].join("\n\n")
}

/// Inline scripts only required on the curation page: the RDKit and
/// citation.js bridges. Loaded lazily via [`ui::document::DocumentScripts`]
/// (mounted inside the curation view) so the bridge code — and, by extension,
/// the heavy `RDKit_minimal.js` / `citation.min.js` CDN payloads — are never
/// downloaded by visitors who only explore results or draw structures.
pub fn build_curation_inline_script() -> String {
    [RDKIT_BRIDGE_SCRIPT, CITATION_BRIDGE_SCRIPT].join("\n\n")
}
