// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Secondary i18n helpers split out of `i18n.rs` for maintainability.

use super::{CountNoun, Locale};

pub fn threshold_label(locale: Locale, value: f64) -> String {
    match locale {
        Locale::En => format!("Threshold: {value:.2}"),
        Locale::Fr => format!("Seuil: {value:.2}"),
        Locale::De => format!("Grenzwert: {value:.2}"),
        Locale::It => format!("Soglia: {value:.2}"),
    }
}

pub fn err_invalid_search_input(locale: Locale) -> String {
    match locale {
        Locale::En => "Please enter a taxon name / QID, or a SMILES structure.".to_string(),
        Locale::Fr => "Veuillez saisir un nom de taxon / QID, ou une structure SMILES.".to_string(),
        Locale::De => {
            "Bitte geben Sie einen Taxonnamen / eine QID oder eine SMILES-Struktur ein.".to_string()
        }
        Locale::It => "Inserisci un nome di taxon / QID oppure una struttura SMILES.".to_string(),
    }
}

pub fn err_unsupported_format(locale: Locale, fmt: &str) -> String {
    match locale {
        Locale::En => format!("Unsupported format '{fmt}'. Use csv, json, or rdf."),
        Locale::Fr => {
            format!("Format '{fmt}' non pris en charge. Utilisez csv, json ou rdf.")
        }
        Locale::De => {
            format!("Nicht unterstütztes Format '{fmt}'. Verwenden Sie csv, json oder rdf.")
        }
        Locale::It => {
            format!("Formato '{fmt}' non supportato. Usa csv, json o rdf.")
        }
    }
}

/// Retained as part of the public i18n API even if not currently called
/// from compiled paths.
#[allow(dead_code)]
pub fn err_taxon_search_failed(locale: Locale, detail: &str) -> String {
    match locale {
        Locale::En => format!("Taxon search failed: {detail}"),
        Locale::Fr => format!("Échec de la recherche de taxon : {detail}"),
        Locale::De => format!("Taxon-Suche fehlgeschlagen: {detail}"),
        Locale::It => format!("Ricerca del taxon non riuscita: {detail}"),
    }
}

pub fn err_taxon_parse_failed(locale: Locale, detail: &str) -> String {
    match locale {
        Locale::En => format!("Taxon parse failed: {detail}"),
        Locale::Fr => format!("Échec de l'analyse du taxon : {detail}"),
        Locale::De => format!("Taxon-Parsing fehlgeschlagen: {detail}"),
        Locale::It => format!("Parsing del taxon non riuscito: {detail}"),
    }
}

pub fn err_taxon_resolution_failed(locale: Locale) -> String {
    match locale {
        Locale::En => "Taxon resolution failed.".to_string(),
        Locale::Fr => "Échec de la résolution du taxon.".to_string(),
        Locale::De => "Taxon-Auflösung fehlgeschlagen.".to_string(),
        Locale::It => "Risoluzione del taxon non riuscita.".to_string(),
    }
}

pub fn err_query_stage_failed(locale: Locale, stage: &str, detail: &str) -> String {
    match locale {
        Locale::En => format!("{stage} failed: {detail}"),
        Locale::Fr => format!("Échec de l'étape {stage} : {detail}"),
        Locale::De => format!("Schritt {stage} fehlgeschlagen: {detail}"),
        Locale::It => format!("Fase {stage} non riuscita: {detail}"),
    }
}

pub fn err_taxon_not_found(locale: Locale, taxon: &str) -> String {
    match locale {
        Locale::En => format!("Taxon '{taxon}' not found in Wikidata."),
        Locale::Fr => format!("Taxon '{taxon}' introuvable dans Wikidata."),
        Locale::De => format!("Taxon '{taxon}' wurde in Wikidata nicht gefunden."),
        Locale::It => format!("Taxon '{taxon}' non trovato in Wikidata."),
    }
}

pub fn warn_input_standardized(locale: Locale, original: &str, normalized: &str) -> String {
    match locale {
        Locale::En => format!("Input standardized from '{original}' to '{normalized}'."),
        Locale::Fr => format!("Entrée standardisée de '{original}' à '{normalized}'."),
        Locale::De => format!("Eingabe von '{original}' zu '{normalized}' standardisiert."),
        Locale::It => format!("Input standardizzato da '{original}' a '{normalized}'."),
    }
}

pub fn warn_ambiguous_taxon(
    locale: Locale,
    best_name: &str,
    best_qid: &str,
    names: &str,
) -> String {
    match locale {
        Locale::En => {
            format!("Ambiguous taxon name; using {best_name} ({best_qid}). Candidates: {names}")
        }
        Locale::Fr => format!(
            "Nom de taxon ambigu; utilisation de {best_name} ({best_qid}). Candidats : {names}"
        ),
        Locale::De => format!(
            "Mehrdeutiger Taxonname; verwende {best_name} ({best_qid}). Kandidaten: {names}"
        ),
        Locale::It => {
            format!("Nome taxon ambiguo; uso {best_name} ({best_qid}). Candidati: {names}")
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn err_wasm_large_query_fallback(locale: Locale, err_msg: &str) -> String {
    match locale {
        Locale::En => format!(
            "Large-query fallback disabled on wasm to avoid memory exhaustion ({err_msg}). Try adding filters or use a desktop browser for large result exports."
        ),
        Locale::Fr => format!(
            "Le repli sur grande requête est désactivé sur wasm pour éviter la saturation de la mémoire ({err_msg}). Essayez d'ajouter des filtres ou utilisez un navigateur desktop pour les grands exports."
        ),
        Locale::De => format!(
            "Große-Query-Fallback auf wasm deaktiviert, um Speicherprobleme zu vermeiden ({err_msg}). Bitte Filter verfeinern oder für sehr große Exporte einen Desktop-Browser nutzen."
        ),
        Locale::It => format!(
            "Fallback per query grandi disabilitato su wasm per evitare esaurimento memoria ({err_msg}). Aggiungi filtri o usa un browser desktop per export molto grandi."
        ),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn error_hint_memory(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Result too large for current device memory.",
        Locale::Fr => "Résultat trop volumineux pour la mémoire de l'appareil.",
        Locale::De => "Ergebnis ist zu groß für den verfügbaren Gerätespeicher.",
        Locale::It => "Risultato troppo grande per la memoria disponibile sul dispositivo.",
    }
}

pub fn aria_wikidata_entity(locale: Locale, qid: &str) -> String {
    match locale {
        Locale::En => format!("Open Wikidata entity {qid}"),
        Locale::Fr => format!("Ouvrir l'entité Wikidata {qid}"),
        Locale::De => format!("Wikidata-Entität {qid} öffnen"),
        Locale::It => format!("Apri l'entità Wikidata {qid}"),
    }
}

pub fn aria_search_inchikey(locale: Locale, ik: &str) -> String {
    match locale {
        Locale::En => format!("Search Wikidata for InChIKey {ik}"),
        Locale::Fr => format!("Rechercher dans Wikidata la cle InChIKey {ik}"),
        Locale::De => format!("InChIKey {ik} in Wikidata suchen"),
        Locale::It => format!("Cerca InChIKey {ik} in Wikidata"),
    }
}

pub fn aria_chemical_structure(locale: Locale, compound_name: &str) -> String {
    match locale {
        Locale::En => format!("Chemical structure of {compound_name}"),
        Locale::Fr => format!("Structure chimique de {compound_name}"),
        Locale::De => format!("Chemische Struktur von {compound_name}"),
        Locale::It => format!("Struttura chimica di {compound_name}"),
    }
}

pub fn aria_wikidata_statement(locale: Locale, stmt: &str) -> String {
    match locale {
        Locale::En => format!("Wikidata statement {stmt}"),
        Locale::Fr => format!("Déclaration Wikidata {stmt}"),
        Locale::De => format!("Wikidata-Aussage {stmt}"),
        Locale::It => format!("Dichiarazione Wikidata {stmt}"),
    }
}

pub fn aria_sort_toggle(locale: Locale, column: &str, next_descending: bool) -> String {
    match locale {
        Locale::En => {
            let dir = if next_descending {
                "descending"
            } else {
                "ascending"
            };
            format!("Sort by {column}, {dir}")
        }
        Locale::Fr => {
            let dir = if next_descending {
                "décroissant"
            } else {
                "croissant"
            };
            format!("Trier par {column}, ordre {dir}")
        }
        Locale::De => {
            let dir = if next_descending {
                "absteigend"
            } else {
                "aufsteigend"
            };
            format!("Nach {column} sortieren, {dir}")
        }
        Locale::It => {
            let dir = if next_descending {
                "decrescente"
            } else {
                "crescente"
            };
            format!("Ordina per {column}, ordine {dir}")
        }
    }
}

fn group_digits(mut value: usize, sep: char) -> String {
    if value < 1000 {
        return value.to_string();
    }

    let mut groups: Vec<usize> = Vec::new();
    while value >= 1000 {
        groups.push(value % 1000);
        value /= 1000;
    }

    let mut out = value.to_string();
    for group in groups.iter().rev() {
        out.push(sep);
        out.push_str(&format!("{group:03}"));
    }
    out
}

pub fn format_count(locale: Locale, value: usize) -> String {
    let sep = match locale {
        Locale::En => ',',
        Locale::Fr => ' ',
        Locale::De => '.',
        Locale::It => '.',
    };
    group_digits(value, sep)
}

pub fn count_label(locale: Locale, noun: CountNoun, count: usize) -> &'static str {
    match (locale, noun, count == 1) {
        (Locale::En, CountNoun::Compound, true) => "Compound",
        (Locale::En, CountNoun::Compound, false) => "Compounds",
        (Locale::En, CountNoun::Taxon, true) => "Taxon",
        (Locale::En, CountNoun::Taxon, false) => "Taxa",
        (Locale::En, CountNoun::Reference, true) => "Reference",
        (Locale::En, CountNoun::Reference, false) => "References",
        (Locale::En, CountNoun::Entry, true) => "Entry",
        (Locale::En, CountNoun::Entry, false) => "Entries",
        (Locale::Fr, CountNoun::Compound, true) => "Composé",
        (Locale::Fr, CountNoun::Compound, false) => "Composés",
        (Locale::Fr, CountNoun::Taxon, true) => "Taxon",
        (Locale::Fr, CountNoun::Taxon, false) => "Taxa",
        (Locale::Fr, CountNoun::Reference, true) => "Référence",
        (Locale::Fr, CountNoun::Reference, false) => "Références",
        (Locale::Fr, CountNoun::Entry, true) => "Entrée",
        (Locale::Fr, CountNoun::Entry, false) => "Entrées",
        (Locale::De, CountNoun::Compound, true) => "Verbindung",
        (Locale::De, CountNoun::Compound, false) => "Verbindungen",
        (Locale::De, CountNoun::Taxon, true) => "Taxon",
        (Locale::De, CountNoun::Taxon, false) => "Taxa",
        (Locale::De, CountNoun::Reference, true) => "Referenz",
        (Locale::De, CountNoun::Reference, false) => "Referenzen",
        (Locale::De, CountNoun::Entry, true) => "Eintrag",
        (Locale::De, CountNoun::Entry, false) => "Einträge",
        (Locale::It, CountNoun::Compound, true) => "Composto",
        (Locale::It, CountNoun::Compound, false) => "Composti",
        (Locale::It, CountNoun::Taxon, true) => "Taxon",
        (Locale::It, CountNoun::Taxon, false) => "Taxa",
        (Locale::It, CountNoun::Reference, true) => "Riferimento",
        (Locale::It, CountNoun::Reference, false) => "Riferimenti",
        (Locale::It, CountNoun::Entry, true) => "Voce",
        (Locale::It, CountNoun::Entry, false) => "Voci",
    }
}
