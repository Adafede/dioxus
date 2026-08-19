//! Bake a synchronous `<html lang>` + language bootstrap into shipped
//! `index.html` files.
//!
//! # Why this exists
//!
//! dioxus-cli 0.7 only bakes its own title/toast into the static `index.html`;
//! the app's `DocumentHead` (in `apps/lotus-explore-rs/src/document_head.rs`)
//! applies `<html lang>` client-side *after* the async dioxus module hydrates.
//! WAVE therefore reports, for `/?lang=fr`:
//!
//! > "Language *en* as the change happens AFTER loading"
//!
//! This injects a tiny inline `<script>` as the very first child of `<head>`
//! (it runs during HTML parsing, before the deferred async module) that sets
//! `document.documentElement.lang` from the same `?lang=` / `?locale=` convention
//! the app already reads (`features::explore::url_state`), defaulting to `"en"`
//! (the app's default locale). `<html lang>` is thus correct on initial paint,
//! so no language change happens after load.
//!
//! # Lotus pre-bake (mobile LCP)
//!
//! The landing-page hero `<p>` ("This app demonstrates the power of linked open
//! data…") is the mobile Lighthouse LCP, but the lotus app is pure CSR: the
//! shell ships an *empty* `<div id="main">` and the hero only paints once the
//! ~1.7 MB wasm downloads + compiles — on a 3G-throttled phone that is the
//! reported ~910 ms "Element render delay". dx 0.7 has no SSR/static pre-render
//! for a CSR app (`--platform web` writes an empty mount, `--static` serves
//! routes from a fullstack `/static_routes` endpoint the lotus app does not
//! expose), so hydration-based pre-render is off the table.
//!
//! To get the hero to first paint instead, this pass — **for the lotus shell
//! only** — additionally:
//!
//! 1. inlines the critical CSS custom properties + reset into `<head>` so the
//!    design tokens are "directly reachable" on first paint (the app only
//!    injects its stylesheet after the wasm loads); and
//! 2. pre-bakes the 4-language hero `<p>` (with its constant Wikidata/QLever
//!    links and the language-policy note) into the shell *before*
//!    `<div id="main">`, guarded by a tiny inline route-gate that mirrors the
//!    app's `ContentPhase::Welcome` condition (`Explore` view, no `taxon`/
//!    `structure`/`execute` query). The app no longer renders the hero itself
//!    (see `components/welcome.rs`), so there is no double paint.
//!
//! # Usage
//!
//! Run from the site root that contains `_site/`, before
//! `actions/upload-pages-artifact`. Targets the root landing page and each
//! app's top-level `index.html` only — never the nested Ketcher iframe
//! `index.html` under `assets/`.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// `<head>` replacement: keeps the `<head>` tag, then inserts the synchronous
/// language resolver as the first child (runs during parse, before the async
/// module). Reads `?lang=` / `?locale=` and defaults to `"en"`. (Raw string so
/// the double quotes in the embedded JS need no escaping.)
const LANG_HEAD: &str = r#"<head>
            <script id="dx-lang-bootstrap">try{var p=new URLSearchParams(location.search);var l=p.get("lang")||p.get("locale");document.documentElement.lang=l||"en";}catch(e){}</script>"#;

/// Critical CSS inlined into the lotus shell `<head>` so the first paint is
/// styled and the design tokens are reachable *before* the wasm module injects
/// its stylesheet. Covers exactly the reset + tokens the static hero and shell
/// chrome use; keep roughly in sync with `crates/ui/src/styles/lotus/base.rs`
/// `reset_and_tokens()` (the app re-applies the full token set at runtime, so
/// an out-of-band token only risks a sub-millisecond flash, never a hang).
const LOTUS_CRITICAL_CSS: &str = r#"/* lotus-critical: inlined so vars are reachable on first paint (keep ~sync with crates/ui/src/styles/lotus/base.rs) */
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
html,body{height:100%}
html,body,#main{width:100%;max-width:100%;overflow-x:hidden}
img,svg,canvas,video{max-width:100%;height:auto}
body{background:var(--bg);color:var(--text);font-family:var(--sans);-webkit-font-smoothing:antialiased}
:root{color-scheme:light dark;--bg:#f7fafc;--bg2:#f7fafc;--surface:#ffffff;--surface2:#ffffff;--border:#c3cfdd;--text:#111827;--text2:#233548;--text3:#516274;--accent:#0b5cab;--accent2:#084b8a;--radius:10px;--radius-sm:4px;--fs-0:clamp(.75rem,.725rem + .17vw,.875rem);--fs-1:clamp(.875rem,.845rem + .2vw,.9375rem);--fs-body:clamp(.875rem,.845rem + .2vw,.9375rem);--mono:'Fira Code',ui-monospace,sfmono-regular,'JetBrains Mono',consolas,monospace;--sans:'Inter',-apple-system,blinkmacsystemfont,'Segoe UI',roboto,'Helvetica Neue',arial,sans-serif;--tap-target-min:40px}
@media (prefers-color-scheme:dark){:root{--bg:#0f172a;--bg2:#0f172a;--surface:#111827;--surface2:#111827;--border:#38475a;--text:#eef4fb;--text2:#d5deea;--text3:#a7b4c7;--accent:#8cbcff;--accent2:#5e98f3}}
/* pre-baked LCP hero: painted from the static shell, no wasm wait.
   Container hidden unless the route-gate opts in (default landing). */
.lotus-hero-skeleton{display:none;margin:0;padding:0;width:100%;max-width:none}
.lotus-hero-skeleton .lx-hero{display:none}
html[data-lotus-hero="1"] .lotus-hero-skeleton{display:block}
html[lang="en"] .lotus-hero-skeleton .lx-en{display:block}
html[lang="fr"] .lotus-hero-skeleton .lx-fr{display:block}
html[lang="de"] .lotus-hero-skeleton .lx-de{display:block}
html[lang="it"] .lotus-hero-skeleton .lx-it{display:block}
"#;

/// Inline route-gate: opts the pre-baked hero into first paint only on the
/// default landing (Explore view + no search query), mirroring the app's
/// `ContentPhase::Welcome` so the hero is not duplicated/visible on results,
/// draw or curation pages. Runs during head parsing, before the body hero.
const LOTUS_HERO_GATE: &str = r#"<script id="lotus-hero-gate">try{var p=new URLSearchParams(location.search);var v=p.get("view");var explore=!v||v==="explore";var hasSearch=!!p.get("taxon")||!!p.get("structure")||!!p.get("execute");if(explore&&!hasSearch){document.documentElement.setAttribute("data-lotus-hero","1");}}catch(e){}</script>"#;

/// Localized hero lead text for one language. The hero is rendered from the
/// static shell (see [`bake_lotus_shell`]), so this table is the single source
/// of truth for the baked LCP copy — it is *not* part of the runtime i18n
/// resolver (the `WelcomeLeadA..E` / `LabelLanguagePolicy` `TextKey` arms were
/// removed once the hero stopped being rendered client-side). The link labels
/// are proper nouns and intentionally NOT translated.
struct HeroStrings {
    lang: &'static str,
    cls: &'static str,
    lead_a: &'static str,
    lead_b: &'static str,
    lead_c: &'static str,
    lead_d: &'static str,
    lead_e: &'static str,
    policy: &'static str,
}

const LOTUS_HERO_LANGS: &[HeroStrings] = &[
    HeroStrings {
        lang: "en",
        cls: "lx-en",
        lead_a: "This app demonstrates the power of linked open data by connecting natural products to organisms and scientific literature. ",
        lead_b: "The data model links compounds, taxa, and references—sourced from the ",
        lead_c: ", published as linked data on ",
        lead_d: ", and queried via SPARQL through ",
        lead_e: ".",
        policy: "Labels use 'mul' first, then 'en' fallback, for comparable results.",
    },
    HeroStrings {
        lang: "fr",
        cls: "lx-fr",
        lead_a: "Cette application démontre la puissance des données ouvertes liées en connectant les produits naturels aux organismes et à la littérature scientifique. ",
        lead_b: "Le modèle de données relie les composés, les taxa et les références—provenant de ",
        lead_c: ", publiées en tant que données ouvertes liées sur ",
        lead_d: " et interrogées via SPARQL par ",
        lead_e: ".",
        policy: "Les libellés utilisent d'abord 'mul', puis 'en', pour des résultats comparables.",
    },
    HeroStrings {
        lang: "de",
        cls: "lx-de",
        lead_a: "Diese Anwendung demonstriert die Leistungsfähigkeit verknüpfter offener Daten durch Verbindung natürlicher Produkte mit Organismen und wissenschaftlicher Literatur. ",
        lead_b: "Das Datenmodell verknüpft Verbindungen, Taxa und Referenzen—aus der ",
        lead_c: ", veröffentlicht als verknüpfte offene Daten auf ",
        lead_d: " und abgefragt via SPARQL durch ",
        lead_e: ".",
        policy: "Beschriftungen werden zuerst aus 'mul' und dann 'en' aufgelöst, damit Ergebnisse vergleichbar bleiben.",
    },
    HeroStrings {
        lang: "it",
        cls: "lx-it",
        lead_a: "Questa applicazione dimostra la potenza dei dati aperti collegati collegando i prodotti naturali agli organismi e alla letteratura scientifica. ",
        lead_b: "Il modello di dati collega gli composti, i taxon e i riferimenti—provenienti da ",
        lead_c: ", pubblicati come dati aperti collegati su ",
        lead_d: " e interrogati tramite SPARQL tramite ",
        lead_e: ".",
        policy: "Le etichette usano prima 'mul' e poi 'en' per risultati confrontabili.",
    },
];

/// Constant anchor markup for the three (proper-noun) links — mirrors the app's
/// `inline_link_style()`.
const LOTUS_LINK_LOTUS: &str = r#"<a href="https://www.wikidata.org/wiki/Q104225190" target="_blank" rel="noopener noreferrer" style="text-decoration:underline;text-underline-offset:2px;font-weight:600">LOTUS initiative</a>"#;
const LOTUS_LINK_WIKIDATA: &str = r#"<a href="https://www.wikidata.org/" target="_blank" rel="noopener noreferrer" style="text-decoration:underline;text-underline-offset:2px;font-weight:600">Wikidata</a>"#;
const LOTUS_LINK_QLVER: &str = r#"<a href="https://qlever.dev/wikidata" target="_blank" rel="noopener noreferrer" style="text-decoration:underline;text-underline-offset:2px;font-weight:600">QLever</a>"#;

/// Escape text for safe embedding in the baked HTML.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Build one language's hero `<p>` (links are constant proper nouns).
fn lotus_hero_p(hero: &HeroStrings) -> String {
    let lang = hero.lang;
    let cls = hero.cls;
    let lead_a = escape_html(hero.lead_a);
    let lead_b = escape_html(hero.lead_b);
    let lead_c = escape_html(hero.lead_c);
    let lead_d = escape_html(hero.lead_d);
    let lead_e = escape_html(hero.lead_e);
    let policy = escape_html(hero.policy);
    format!(
        r#"<p lang="{lang}" class="lx-hero {cls}" style="font-size:var(--fs-1);color:var(--text2);margin:6px 0 0 0;line-height:1.6;max-width:none;overflow-wrap:anywhere">{lead_a}{lead_b}{LOTUS_LINK_LOTUS}{lead_c}{LOTUS_LINK_WIKIDATA}{lead_d}{LOTUS_LINK_QLVER}{lead_e} <span style="font-size:var(--fs-1);color:var(--text2);margin-top:10px;max-width:72ch;line-height:1.55">{policy}</span></p>"#
    )
}

/// The pre-baked hero block (4 language `<p>`s). The language shown is selected
/// by the `[lang="…"]` rule in `LOTUS_CRITICAL_CSS`; only the one matching
/// `document.documentElement.lang` (set by the `dx-lang-bootstrap` script) is
/// displayed. Guarded by `data-lotus-hero="1"` (set by `LOTUS_HERO_GATE`) so it
/// only paints on the default landing.
fn lotus_hero_html() -> String {
    let mut out = String::from(
        r#"<div class="lotus-hero-skeleton" aria-hidden="true"><div style="max-width:none;padding:16px 22px 0">"#,
    );
    for hero in LOTUS_HERO_LANGS {
        out.push_str(&lotus_hero_p(hero));
    }
    out.push_str("</div></div>");
    out
}

/// True for the lotus landing shell (`_site/lotus-explore-rs/index.html`).
fn is_lotus_shell(path: &Path) -> bool {
    path.to_string_lossy()
        .ends_with("lotus-explore-rs/index.html")
}

/// Pre-bake the critical CSS vars + route-gate + hero into the lotus shell.
/// Idempotent (a no-op once `lotus-hero-skeleton` is present).
fn bake_lotus_shell(path: &Path) -> std::io::Result<()> {
    let mut html = fs::read_to_string(path)?;
    if html.contains("lotus-hero-skeleton") {
        return Ok(());
    }

    // Inject critical `<style>` + the route-gate `<script>` as early as
    // possible (right after the dx-lang-bootstrap script, which is the first
    // script in `<head>` after the lang pass).
    let style = format!("<style>\n{LOTUS_CRITICAL_CSS}\n</style>");
    let injection = format!("\n            {style}\n            {LOTUS_HERO_GATE}\n            ");
    if html.contains("</script>") {
        html = html.replacen(
            "</script>",
            &format!("</script>\n            {injection}"),
            1,
        );
    } else {
        html = html.replacen("<head>", &format!("<head>\n            {injection}"), 1);
    }

    // Pre-bake the hero just before the dioxus mount so it paints with the
    // first chunk of the document, independent of the wasm.
    let hero = lotus_hero_html();
    html = html.replacen(
        "<div id=\"main\"",
        &format!("{hero}\n            <div id=\"main\""),
        1,
    );

    fs::write(path, html.as_bytes())?;
    println!("  baked lotus critical css + hero into {}", path.display());
    Ok(())
}

#[must_use]
fn targets(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root_index = root.join("index.html");
    if root_index.is_file() {
        out.push(root_index);
    }
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        // Skip the Ketcher iframe assets tree: its `index.html` is the
        // standalone editor's own shell and must be left untouched.
        if entry.file_name() == OsStr::new("assets") {
            continue;
        }
        let index = entry_path.join("index.html");
        if index.is_file() {
            out.push(index);
        }
    }
    out
}

fn bake(path: &Path) -> std::io::Result<()> {
    let mut html = fs::read_to_string(path)?;
    let needs_lang = html.contains("<html>") && !html.contains("<html lang=");
    if needs_lang {
        html = html.replacen("<html>", "<html lang=\"en\">", 1);
    }
    let needs_script = !html.contains("dx-lang-bootstrap");
    if needs_script {
        html = html.replacen("<head>", LANG_HEAD, 1);
    }
    if needs_lang || needs_script {
        fs::write(path, html.as_bytes())?;
        println!("  baked lang bootstrap into {}", path.display());
    }
    // Pre-bake the lotus LCP hero + critical CSS (no-op for non-lotus shells).
    if is_lotus_shell(path) {
        bake_lotus_shell(path)?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let root = Path::new("_site");
    if !root.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("site root not found: {}", root.display()),
        ));
    }
    let root_targets = targets(root);
    let mut count = 0usize;
    for path in &root_targets {
        bake(path)?;
        count += 1;
    }
    println!("baked lang into {count} index.html file(s)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn injects_lang_and_bootstrap_idempotently() {
        let dir = std::env::temp_dir().join("lotus_deploy_bake_lang_test");
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("index.html");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(file, "<html><head><title>x</title></head></html>").unwrap();
        drop(file);

        bake(&path).unwrap();
        let html = std::fs::read_to_string(&path).unwrap();
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("dx-lang-bootstrap"));
        assert!(html.contains(LANG_HEAD));

        // Re-running must not duplicate the injection.
        bake(&path).unwrap();
        let html2 = std::fs::read_to_string(&path).unwrap();
        assert_eq!(html2, html);
        assert_eq!(html.matches("dx-lang-bootstrap").count(), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn targets_skips_ketcher_assets() {
        let root = std::env::temp_dir().join("lotus_deploy_targets_test");
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(root.join("lotus-explore-rs/assets/ketcher")).unwrap();
        std::fs::write(root.join("index.html"), "<html></html>").unwrap();
        std::fs::write(root.join("lotus-explore-rs/index.html"), "<html></html>").unwrap();
        std::fs::write(
            root.join("lotus-explore-rs/assets/ketcher/index.html"),
            "<html></html>",
        )
        .unwrap();

        // Root landing page + the app's index, but NOT the nested Ketcher
        // iframe index.html (under assets/).
        assert_eq!(targets(&root).len(), 2);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn bakes_lotus_critical_css_and_hero() {
        let root = std::env::temp_dir().join("lotus_deploy_lotus_shell_test");
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(root.join("lotus-explore-rs")).unwrap();
        let path = root.join("lotus-explore-rs/index.html");
        // Mimic a dioxus 0.7 web shell: bootstrap already injected by the lang
        // pass, empty dioxus mount, async module.
        std::fs::write(&path, "<html lang=\"en\">\n<head>\n<script id=\"dx-lang-bootstrap\">try{}</script>\n</head>\n<body>\n<div id=\"main\">\n</div>\n<script type=\"module\" async src=\"/dioxus/lotus-explore-rs/assets/lotus-explore-rs-dxabc.js\"></script>\n</body>\n</html>").unwrap();

        bake(&path).unwrap();
        let html = std::fs::read_to_string(&path).unwrap();

        // Critical CSS vars are "directly reachable" on first paint.
        assert!(html.contains("--fs-1"));
        assert!(html.contains("--text2"));
        // Reset reaches the body.
        assert!(html.contains("box-sizing:border-box"));
        // The route-gate + hero skeleton are present.
        assert!(html.contains("data-lotus-hero"));
        assert!(html.contains("lotus-hero-gate"));
        // All four language variants are baked.
        assert!(html.contains("class=\"lx-hero lx-en\""));
        assert!(html.contains("class=\"lx-hero lx-fr\""));
        assert!(html.contains("class=\"lx-hero lx-de\""));
        assert!(html.contains("class=\"lx-hero lx-it\""));
        // The en hero copy (single source of truth = LOTUS_HERO_LANGS).
        assert!(html.contains("This app demonstrates the power of linked open data"));
        // Links kept constant (proper nouns) across languages.
        assert!(html.contains("qlever.dev/wikidata"));
        // Exactly one hero container div; the other occurrences live in the CSS.
        assert_eq!(html.matches("<div class=\"lotus-hero-skeleton").count(), 1);
        // Hero div lands before the dioxus mount.
        let hero_idx = html.find("<div class=\"lotus-hero-skeleton\"").unwrap();
        let main_idx = html.find("<div id=\"main\"").unwrap();
        assert!(hero_idx < main_idx);

        // Idempotent: re-running the full bake must not duplicate.
        let before = html;
        bake(&path).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(before, after);

        std::fs::remove_dir_all(&root).ok();
    }
}
