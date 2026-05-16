// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::Locale;

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

pub fn err_taxon_too_long(locale: Locale) -> String {
    match locale {
        Locale::En => "Taxon input is too long. Please keep it under 500 characters.".to_string(),
        Locale::Fr => {
            "La valeur du taxon est trop longue. Veuillez rester sous 500 caractères.".to_string()
        }
        Locale::De => "Die Taxon-Eingabe ist zu lang. Bitte unter 500 Zeichen bleiben.".to_string(),
        Locale::It => {
            "Il valore del taxon è troppo lungo. Mantienilo sotto i 500 caratteri.".to_string()
        }
    }
}

pub fn err_structure_too_long(locale: Locale) -> String {
    match locale {
        Locale::En => {
            "Structure input is too long. Please shorten the SMILES/Molfile text.".to_string()
        }
        Locale::Fr => {
            "La structure est trop longue. Raccourcissez le texte SMILES/Molfile.".to_string()
        }
        Locale::De => {
            "Die Struktur-Eingabe ist zu lang. Bitte den SMILES/Molfile-Text kürzen.".to_string()
        }
        Locale::It => {
            "L'input della struttura è troppo lungo. Riduci il testo SMILES/Molfile.".to_string()
        }
    }
}

pub fn err_mass_out_of_range(locale: Locale) -> String {
    match locale {
        Locale::En => "Mass values must be between 0 and 10000.".to_string(),
        Locale::Fr => "Les valeurs de masse doivent être comprises entre 0 et 10000.".to_string(),
        Locale::De => "Massenwerte müssen zwischen 0 und 10000 liegen.".to_string(),
        Locale::It => "I valori di massa devono essere compresi tra 0 e 10000.".to_string(),
    }
}

pub fn err_mass_range_invalid(locale: Locale) -> String {
    match locale {
        Locale::En => "Mass minimum cannot exceed mass maximum.".to_string(),
        Locale::Fr => "La masse minimale ne peut pas dépasser la masse maximale.".to_string(),
        Locale::De => "Die minimale Masse darf die maximale Masse nicht überschreiten.".to_string(),
        Locale::It => "La massa minima non può superare la massa massima.".to_string(),
    }
}

pub fn err_year_out_of_range(locale: Locale) -> String {
    match locale {
        Locale::En => "Year is outside the supported range.".to_string(),
        Locale::Fr => "L'année est hors de la plage prise en charge.".to_string(),
        Locale::De => "Das Jahr liegt außerhalb des unterstützten Bereichs.".to_string(),
        Locale::It => "L'anno è fuori dall'intervallo supportato.".to_string(),
    }
}

pub fn err_year_range_invalid(locale: Locale) -> String {
    match locale {
        Locale::En => "Year from cannot exceed year to.".to_string(),
        Locale::Fr => "L'année de début ne peut pas dépasser l'année de fin.".to_string(),
        Locale::De => "Das Startjahr darf nicht größer als das Endjahr sein.".to_string(),
        Locale::It => "L'anno iniziale non può superare l'anno finale.".to_string(),
    }
}

pub fn err_element_count_too_high(locale: Locale) -> String {
    match locale {
        Locale::En => "Formula element counts are too high.".to_string(),
        Locale::Fr => "Les comptages d'éléments de la formule sont trop élevés.".to_string(),
        Locale::De => "Die Elementanzahl in der Formel ist zu hoch.".to_string(),
        Locale::It => "I conteggi degli elementi della formula sono troppo alti.".to_string(),
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
