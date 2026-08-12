// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document `<head>` management for lotus-explore-rs.
//!
//! Replaces the static `index.html` with Rust code that sets meta tags,
//! bundled styles, scripts (CDN + inline bridge code), and structured data.

use crate::lotus_styles::bundled_lotus_styles;
use dioxus::prelude::*;
use ui::prelude::*;

/// All inline JavaScript for the RDKit, citation-js, and toast bridges.
/// Stored as a raw string so it survives verbatim in the generated `<script>` tag.
const INLINE_SCRIPT: &str = r#"
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

/// All inline CSS for the Inter font and toast notification styles.
const INLINE_STYLE: &str = r#"
/* Inter Font */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap') layer;

#dx-toast-template {
    display: none;
    visibility: hidden;
}

.dx-toast {
    position: absolute;
    top: 10px;
    right: 0;
    padding-right: 10px;
    user-select: none;
    z-index: 2147483647;
}

.dx-toast .dx-toast-inner {
    position: fixed;
    background-color: #181B20;
    color: #ffffff;
    font-family: "Inter", sans-serif;
    display: grid;
    grid-template-columns: auto auto;
    max-width: 400px;
    min-height: 56px;
    border-radius: 5px;
}

.dx-toast .dx-toast-inner {
    cursor: pointer;
    margin-right: 10px;
}

.dx-toast .dx-toast-level-bar-container {
    height: 100%;
    width: 6px;
}

.dx-toast .dx-toast-level-bar-container .dx-toast-level-bar {
    width: 100%;
    height: 100%;
    border-radius: 5px 0 0 5px;
}

.dx-toast .dx-toast-content {
    padding: 8px;
}

.dx-toast .dx-toast-header {
    display: flex;
    flex-direction: row;
    justify-content: start;
    align-items: end;
    margin-bottom: 10px;
}

.dx-toast .dx-toast-header>svg {
    height: 18px;
    margin-right: 5px;
}

.dx-toast .dx-toast-header .dx-toast-header-text {
    font-size: 14px;
    font-weight: 700;
    padding: 0;
    margin: 0;
}

.dx-toast .dx-toast-msg {
    font-size: 11px;
    font-weight: 400;
    padding: 0;
    margin: 0;
}

.dx-toast-level-bar.info {
    background-color: #428EFF;
}

.dx-toast-level-bar.success {
    background-color: #42FF65;
}

.dx-toast-level-bar.error {
    background-color: #FF4242;
}
"#;

/// JSON-LD structured data for the LOTUS Knowledge Explorer.
const JSON_LD: &str = r#"{
  "@context": "https://schema.org",
  "@type": "WebApplication",
  "name": "LOTUS Knowledge Explorer",
  "description": "Explore LOTUS natural-product records with taxon filters, SMILES/Molfile structure search, and Wikidata curation workflows.",
  "applicationCategory": "ScienceApplication",
  "operatingSystem": "Any",
  "inLanguage": "en",
  "isAccessibleForFree": true,
  "license": "https://www.gnu.org/licenses/agpl-3.0.html",
  "codeRepository": "https://github.com/Adafede/dioxus",
  "author": {
    "@type": "Organization",
    "name": "Contributors to the dioxus-apps project"
  }
}"#;

/// Link specifications for resource hints and favicons.
const LINKS: &[LinkSpec] = &[
    // Discovery links
    LinkSpec {
        rel: "manifest",
        href: "/site.webmanifest",
        r#type: Some("application/manifest+json"),
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "llmstxt",
        href: "/llms.txt",
        r#type: Some("text/plain"),
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "api-catalog",
        href: "/.well-known/api-catalog.json",
        r#type: Some("application/json"),
        media: None,
        crossorigin: None,
        sizes: Some("64x64"),
        hreflang: None,
    },
    LinkSpec {
        rel: "agent-skills",
        href: "/.well-known/agent-skills.json",
        r#type: Some("application/json"),
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    // Resource hints
    LinkSpec {
        rel: "preconnect",
        href: "https://qlever.dev",
        r#type: None,
        media: None,
        crossorigin: Some("anonymous"),
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "preconnect",
        href: "https://www.simolecule.com",
        r#type: None,
        media: None,
        crossorigin: Some("anonymous"),
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "dns-prefetch",
        href: "https://idsm.elixir-czech.cz",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "preconnect",
        href: "https://unpkg.com",
        r#type: None,
        media: None,
        crossorigin: Some("anonymous"),
        sizes: None,
        hreflang: None,
    },
    // Favicon
    LinkSpec {
        rel: "icon",
        href: "favicon.svg",
        r#type: Some("image/svg+xml"),
        media: None,
        crossorigin: None,
        sizes: Some("any"),
        hreflang: None,
    },
];

/// Hreflang alternate links.
const HREFLANGS: &[(&str, &str)] = &[
    ("x-default", ""),
    ("en", "?locale=en"),
    ("fr", "?locale=fr"),
    ("de", "?locale=de"),
    ("it", "?locale=it"),
];

/// Base URL for the app (used in hreflang and canonical links).
const BASE_URL: &str = "https://adafede.github.io/dioxus/lotus-explore-rs/";

/// Renders the complete document `<head>` using `dioxus::document` instead of
/// a static `index.html`.  Includes SEO meta tags, OG tags, JSON-LD, CDN scripts,
/// inline bridge JS, toast CSS, and resource hint `<link>`s.
#[component]
pub fn LotusDocumentHead(lang: String) -> Element {
    let description = "Explore LOTUS natural-product records with taxon filters, SMILES/Molfile structure search, and Wikidata curation workflows.";
    let scripts = vec![
        "https://unpkg.com/@rdkit/rdkit/dist/RDKit_minimal.js".to_string(),
        "https://cdn.jsdelivr.net/npm/citation-js@0.8.2/build/citation.min.js".to_string(),
        // Privacy-respecting analytics (async, no cookies, GDPR-compliant)
        "https://scripts.simpleanalyticscdn.com/latest.js".to_string(),
    ];

    let mut links: Vec<LinkSpec> = LINKS.to_vec();
    // Canonical
    links.push(LinkSpec {
        rel: "canonical",
        href: BASE_URL,
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    });
    // Hreflang alternates — need owned strings for formatted URLs
    for (lang, suffix) in HREFLANGS {
        links.push(LinkSpec {
            rel: "alternate",
            href: Box::leak(format!("{BASE_URL}{suffix}").into_boxed_str()),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: Some(*lang),
        });
    }
    let inline_style = format!("{}\n\n{}", bundled_lotus_styles(), INLINE_STYLE);

    rsx! {
        DocumentHead {
            title: "LOTUS Knowledge Explorer".to_string(),
            lang,
            description: Some(description.to_string()),
            og_type: Some("website".to_string()),
            og_url: Some(BASE_URL.to_string()),
            og_site_name: Some("LOTUS Knowledge Explorer".to_string()),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts,
            inline_style: Some(inline_style),
            inline_script: Some(INLINE_SCRIPT.to_string()),
            json_ld: Some(JSON_LD.to_string()),
            canonical: Some(BASE_URL.to_string()),
        }

        DocumentLinks { links }
    }
}

/// The toast notification template — placed in the component tree so it
/// renders before the main content, mirroring the original `index.html` body.
#[component]
pub fn ToastTemplate() -> Element {
    rsx! {
        div { id: "dx-toast-template", style: "display:none;visibility:hidden" }
        div {
            id: "__dx-toast",
            class: "dx-toast",
            "aria-hidden": "true",
            div {
                id: "__dx-toast-inner",
                class: "dx-toast-inner",
                style: "right:-1000px;",
                div { class: "dx-toast-level-bar-container",
                    div { id: "__dx-toast-decor", class: "dx-toast-level-bar __info" }
                }
                div { class: "dx-toast-content",
                    div { class: "dx-toast-header",
                        svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 32 32", "preserveAspectRatio": "none", "aria-hidden": "true",
                            path { fill: "#e96020", d: "M22.158 1.783c0 3.077-.851 5.482-2.215 7.377s-3.32 3.557-5.447 5.33-4.425 3.657-6.252 6.195-3.102 5.515-3.102 9.532h4.699c0-3.077.853-5.377 2.217-7.272s3.32-3.557 5.447-5.33 4.425-3.657 6.252-6.195 3.102-5.62 3.102-9.637z" }
                            path { fill: "#2d323b", d: "M9.531 25.927c-.635 0-1.021.515-1.02 1.15s.385 1.151 1.02 1.15H22.47a1.151 1.151 0 1 0 0-2.301zm1.361-4.076c-.608 0-.954.558-.953 1.166s.346 1.035.953 1.035h10.217a1.101 1.101 0 1 0 0-2.201zm0-13.594a1.101 1.101 0 1 0 0 2.201h10.217c.607 0 .953-.598.953-1.205s-.345-.996-.953-.996zM9.531 4.021A1.15 1.15 0 0 0 8.38 5.17a1.15 1.15 0 0 1 1.15 1.15h12.94c.635 0 1.021-.498 1.02-1.133s-.386-1.166-1.02-1.166z" }
                            path { fill: "#00a8d6", d: "M5.142 1.783c0 4.016 1.275 7.099 3.102 9.637s4.125 4.422 6.252 6.195 4.083 3.656 5.447 5.551 2.215 3.974 2.215 7.051h4.701c0-4.016-1.275-7.038-3.102-9.576s-4.125-4.422-6.252-6.195-4.083-3.435-5.447-5.33S9.841 4.86 9.841 1.783z" }
                        }
                    }
                }
                p { id: "__dx-toast-msg", class: "dx-toast-msg", "A non-hot-reloadable change occurred and we must rebuild." }
            }
        }
    }
}
