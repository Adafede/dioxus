use serde::Deserialize;
use std::{error::Error, fs, path::PathBuf};

#[derive(Debug, Deserialize)]
struct Metadata {
    site: Site,
    manifest: Manifest,
}

#[derive(Debug, Deserialize)]
struct Site {
    name: String,
    short_name: String,
    description: String,
    base_url: String,
    repo_url: String,
    issues_url: String,
    discussions_url: String,
    lotus_home_url: String,
    paper_doi_url: String,
    paper_landing_url: String,
    bibtex_path: String,
    app_license_url: String,
    data_license_url: String,
    security_contact_url: String,
    security_policy_url: String,
    source_path: String,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    start_url: String,
    scope: String,
    display: String,
    background_color: String,
    theme_color: String,
    lang: String,
    dir: String,
    screenshots: Vec<String>,
    icons: Vec<Icon>,
    categories: Vec<String>,
    prefer_related_applications: bool,
    shortcuts: Vec<Shortcut>,
}

#[derive(Debug, Deserialize)]
struct Icon {
    src: String,
    sizes: String,
    #[serde(rename = "type")]
    mime_type: String,
    purpose: String,
}

#[derive(Debug, Deserialize)]
struct Shortcut {
    name: String,
    short_name: String,
    description: String,
    url: String,
    icons: Vec<Icon>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let metadata_path = manifest_dir.join("metadata/site-metadata.json");
    let raw = fs::read_to_string(&metadata_path)?;
    let metadata: Metadata = serde_json::from_str(&raw)?;

    let public_dir = manifest_dir.join("public");
    let well_known_dir = public_dir.join(".well-known");
    fs::create_dir_all(&well_known_dir)?;

    write_if_changed(public_dir.join("llms.txt"), build_llms_txt(&metadata))?;
    write_if_changed(public_dir.join("humans.txt"), build_humans_txt(&metadata))?;
    write_if_changed(public_dir.join("robots.txt"), build_robots_txt(&metadata))?;
    write_if_changed(
        well_known_dir.join("security.txt"),
        build_security_txt(&metadata),
    )?;
    write_if_changed(public_dir.join("_headers"), build_headers_txt(&metadata))?;
    write_if_changed(
        public_dir.join("site.webmanifest"),
        serde_json::to_string_pretty(&build_manifest_json(&metadata))?,
    )?;

    println!("cargo:rerun-if-changed={}", metadata_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn write_if_changed(path: PathBuf, contents: String) -> Result<(), Box<dyn Error>> {
    let should_write = fs::read_to_string(&path).map_or(true, |current| current != contents);
    if should_write {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
    }
    Ok(())
}

fn build_llms_txt(meta: &Metadata) -> String {
    let site = &meta.site;
    format!(
        concat!(
            "# {name}\n\n",
            "> {description}\n\n",
            "## Core information\n\n",
            "- **Official name**: {name}\n",
            "- **Short name**: {short_name}\n",
            "- **Purpose**: Interactive exploration of natural-product occurrence data\n",
            "- **Data domain**: Natural products, compounds, taxonomy, scientific references\n",
            "- **Access model**: Free, web-based, no authentication required\n\n",
            "## Features\n\n",
            "- Search by taxon filters and structure input (SMILES or Molfile V2000/V3000)\n",
            "- Use Ketcher to draw structures, then copy Daylight SMILES or MOL V3000 back into search\n",
            "- Filter by mass, year, and formula\n",
            "- Browse taxonomy and references\n",
            "- Export CSV, JSON, and SPARQL results for downstream analysis\n",
            "- Import TSV rows and generate QuickStatements for Wikidata curation\n\n",
            "## Links\n\n",
            "- [Home page]({base_url})\n",
            "- [Repository]({repo_url})\n",
            "- [Source path]({source_path})\n",
            "- [Paper landing page]({paper_landing_url})\n",
            "- [Paper DOI]({paper_doi_url})\n",
            "- [BibTeX references]({bibtex_path})\n",
            "- [LOTUS initiative]({lotus_home_url})\n",
            "- [Agent skills](/.well-known/agent-skills.json)\n",
            "- [API catalog](/.well-known/api-catalog.json)\n\n",
            "## Data sources\n\n",
            "- [Wikidata SPARQL](https://query.wikidata.org/)\n",
            "- [QLever](https://qlever.cs.uni-freiburg.de/wikidata)\n",
            "- [DOI metadata](https://doi.org/)\n",
            "- [LOTUS initiative]({lotus_home_url})\n\n",
            "## Discovery\n\n",
            "- [Agent skills](/.well-known/agent-skills.json)\n",
            "- [API catalog](/.well-known/api-catalog.json)\n",
            "- Structured data: JSON-LD in page head\n",
            "- Link headers: advertise llms, robots, security, sitemap\n\n",
            "## Citation\n\n",
            "- [Paper DOI]({paper_doi_url})\n",
            "- [Paper landing page]({paper_landing_url})\n",
            "- [BibTeX]({bibtex_path})\n\n",
            "## Licensing\n\n",
            "- [App license]({app_license_url}) (AGPL-3.0)\n",
            "- [Data license]({data_license_url}) (CC0 1.0)\n",
            "- [Source path]({source_path})\n\n",
            "## Contact\n\n",
            "- [Issues]({issues_url})\n",
            "- [Discussions]({discussions_url})\n",
        ),
        name = site.name,
        short_name = site.short_name,
        description = site.description,
        base_url = site.base_url,
        repo_url = site.repo_url,
        issues_url = site.issues_url,
        discussions_url = site.discussions_url,
        lotus_home_url = site.lotus_home_url,
        paper_doi_url = site.paper_doi_url,
        paper_landing_url = site.paper_landing_url,
        bibtex_path = site.bibtex_path,
        app_license_url = site.app_license_url,
        data_license_url = site.data_license_url,
        source_path = site.source_path,
    )
}

fn build_humans_txt(meta: &Metadata) -> String {
    let site = &meta.site;
    format!(
        concat!(
            "/* Humans are welcome — https://humanstxt.org/ */\n",
            "/* Reference: https://specification.website/spec/foundations/ */\n\n",
            "/* TEAM */\n",
            "  Name: Adriano Rutz (Adafede) and contributors\n",
            "  GitHub: https://github.com/Adafede\n",
            "  Location: Switzerland\n",
            "  Email: Contact via {issues_url}\n\n",
            "/* THANKS */\n",
            "  LOTUS initiative — {lotus_home_url}\n",
            "  Wikidata community — https://wikidata.org/\n",
            "  Dioxus framework — https://dioxuslabs.com/\n",
            "  RDKit.js — https://www.rdkitjs.com/\n",
            "  Citation.js — https://citation.js.org/\n\n",
            "/* SITE */\n",
            "  Product: {name}\n",
            "  Short name: {short_name}\n",
            "  Description: {description}\n",
            "  Language: English with French, German, Italian localizations\n",
            "  Standards: HTML5, CSS3, WebAssembly, WCAG 2.1 AA, JSON-LD\n",
            "  Components: Rust + Dioxus compiled to WASM\n",
            "  Infrastructure: GitHub Pages ({base_url})\n",
            "  Repository: {repo_url}\n",
            "  Search inputs: taxon filters, SMILES, Molfile V2000/V3000\n",
            "  Curation: TSV import, QuickStatements generation, structure resolution lookup\n",
            "  APIs: Wikidata SPARQL, QLever, DOI, RDKit.js, Citation.js, Ketcher\n",
            "  License (app): AGPL-3.0 — {app_license_url}\n",
            "  License (data): CC0 1.0 — {data_license_url}\n\n",
            "/* SPECIFICATION COMPLIANCE */\n",
            "  SEO: robots.txt, sitemap.xml, structured data, hreflang\n",
            "  Accessibility: semantic HTML, ARIA, keyboard navigation, visible focus\n",
            "  Security: HTTPS, CSP, HSTS, security.txt, Permissions-Policy\n",
            "  Agent Readiness: llms.txt, agent-skills, API catalog, Link headers\n",
            "  Resilience: web app manifest, graceful error handling, offline detection\n",
        ),
        name = site.name,
        short_name = site.short_name,
        description = site.description,
        base_url = site.base_url,
        app_license_url = site.app_license_url,
        data_license_url = site.data_license_url,
        lotus_home_url = site.lotus_home_url,
        issues_url = site.issues_url,
        repo_url = site.repo_url,
    )
}

fn build_robots_txt(meta: &Metadata) -> String {
    let site = &meta.site;
    format!(
        concat!(
            "# robots.txt — https://www.rfc-editor.org/rfc/rfc9309\n",
            "# Allow all well-behaved crawlers to index public site pages and generated metadata.\n",
            "# Reference: https://specification.website/spec/seo/robots-txt/\n",
            "# Reference: https://specification.website/spec/agent-readiness/robots-for-ai-crawlers/\n\n",
            "User-agent: *\n",
            "Allow: /\n",
            "Disallow: /target/\n\n",
            "User-agent: GPTBot\nAllow: /\n\n",
            "User-agent: ClaudeBot\nAllow: /\n\n",
            "User-agent: Claude-Web\nAllow: /\n\n",
            "User-agent: Gemini\nAllow: /\n\n",
            "User-agent: Perplexity\nAllow: /\n\n",
            "User-agent: APIBot\nAllow: /\n\n",
            "User-agent: CCBot\nAllow: /\n\n",
            "User-agent: anthropic-ai\nAllow: /\n\n",
            "User-agent: Applebot\nAllow: /\n\n",
            "User-agent: Googlebot\nAllow: /\n\n",
            "Content-Signal: *\n",
            "  Disallow-Search: false\n",
            "  Disallow-Ingest: false\n",
            "  Disallow-Train: false\n\n",
            "Sitemap: {base_url}sitemap.xml\n",
        ),
        base_url = site.base_url,
    )
}

fn build_security_txt(meta: &Metadata) -> String {
    let site = &meta.site;
    format!(
        concat!(
            "# security.txt — https://securitytxt.org/ (RFC 9116)\n",
            "# Report security vulnerabilities to the project maintainers.\n\n",
            "Contact: {security_contact_url}\n",
            "Expires: 2027-01-01T00:00:00.000Z\n",
            "Preferred-Languages: en, fr\n",
            "Canonical: {base_url}.well-known/security.txt\n",
            "Policy: {security_policy_url}\n",
        ),
        base_url = site.base_url,
        security_contact_url = site.security_contact_url,
        security_policy_url = site.security_policy_url,
    )
}

fn build_headers_txt(_meta: &Metadata) -> String {
    concat!(
    "# Netlify / Cloudflare Pages / compatible CDN — HTTP security headers\n",
    "# Reference: https://specification.website/spec/security/\n",
    "# Reference: https://specification.website/spec/agent-readiness/link-headers/\n",
    "#\n",
    "# These headers are applied to every response served from this origin.\n",
    "# Adjust the Content-Security-Policy for your specific needs.\n\n",
    "/*\n",
    "  Strict-Transport-Security: max-age=63072000; includeSubDomains; preload\n",
    "  X-Frame-Options: DENY\n",
    "  Content-Security-Policy: default-src 'self'; base-uri 'self'; form-action 'self'; script-src 'self' 'wasm-unsafe-eval' https://scripts.simpleanalyticscdn.com; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; connect-src 'self' https://qlever.dev https://query.wikidata.org https://www.wikidata.org https://www.simolecule.com https://idsm.elixir-czech.cz https://doi.org https://api.naturalproducts.net https://pubchem.ncbi.nlm.nih.gov https://api.semanticscholar.org https://api.openalex.org; worker-src 'self' blob:; object-src 'none'; frame-ancestors 'none'; require-trusted-types-for 'script'; trusted-types default\n",
    "  X-Content-Type-Options: nosniff\n",
    "  Referrer-Policy: strict-origin-when-cross-origin\n",
    "  Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=()\n",
    "  Cross-Origin-Opener-Policy: same-origin\n",
    "  Cross-Origin-Embedder-Policy: credentialless\n",
    "  Cross-Origin-Resource-Policy: same-origin\n",
    "  Link: </llms.txt>; rel=\"http://llmstxt.org/llms.txt\"; type=\"text/plain\"\n",
    "  Link: </sitemap.xml>; rel=\"sitemap\"; type=\"application/xml\"\n",
    "  Link: </robots.txt>; rel=\"robots\"; type=\"text/plain\"\n",
    "  Link: </.well-known/security.txt>; rel=\"security.txt\"; type=\"text/plain\"\n\n",
    "/.well-known/*\n",
    "  Cache-Control: no-cache, must-revalidate\n\n",
    "/robots.txt\n",
    "  Cache-Control: no-cache, must-revalidate\n\n",
    "/sitemap.xml\n",
    "  Cache-Control: public, max-age=2592000\n\n",
    "/llms.txt\n",
    "  Cache-Control: no-cache, must-revalidate\n\n",
    "/site.webmanifest\n",
    "  Cache-Control: public, max-age=2592000\n\n",
    "/favicon.svg\n",
    "  Cache-Control: public, max-age=2592000\n\n",
    "**/assets/*\n",
    "  Cache-Control: public, max-age=31536000, immutable\n\n",
    "/index.html\n",
    "  No-Vary-Search: key-order, params, except=(\"locale\")\n",
    )
        .to_string()
}

fn build_manifest_json(meta: &Metadata) -> serde_json::Value {
    let site = &meta.site;
    let manifest = &meta.manifest;
    serde_json::json!({
        "name": site.name,
        "short_name": site.short_name,
        "description": site.description,
        "start_url": manifest.start_url,
        "scope": manifest.scope,
        "display": manifest.display,
        "background_color": manifest.background_color,
        "theme_color": manifest.theme_color,
        "lang": manifest.lang,
        "dir": manifest.dir,
        "screenshots": manifest.screenshots,
        "icons": manifest.icons.iter().map(|icon| serde_json::json!({
            "src": icon.src,
            "sizes": icon.sizes,
            "type": icon.mime_type,
            "purpose": icon.purpose,
        })).collect::<Vec<_>>(),
        "categories": manifest.categories,
        "prefer_related_applications": manifest.prefer_related_applications,
        "shortcuts": manifest.shortcuts.iter().map(|shortcut| serde_json::json!({
            "name": shortcut.name,
            "short_name": shortcut.short_name,
            "description": shortcut.description,
            "url": shortcut.url,
            "icons": shortcut.icons.iter().map(|icon| serde_json::json!({
                "src": icon.src,
                "sizes": icon.sizes,
                "type": icon.mime_type,
                "purpose": icon.purpose,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}
