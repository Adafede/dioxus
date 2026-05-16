// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Typed error message keys for structured, i18n-aware error formatting.
//!
//! This module provides an [`ErrorKey`] enum that enables type-safe error message
//! lookups across all supported locales (English, French, German, Italian).
//!
//! Error messages are discovered at compile time, making it impossible to
//! accidentally reference a non-existent error key. This complements the
//! [`crate::i18n::TextKey`] system for UI labels and notices.
//!
//! # Example
//!
//! ```ignore
//! use crate::i18n::{error_key, Locale};
//!
//! let msg = error_key::err(Locale::En, error_key::ErrorKey::TaxonTooLong);
//! println!("{}", msg);
//! ```

use crate::i18n::Locale;

/// Typed keys for localized error messages and validation feedback.
///
/// Each variant corresponds to a specific error condition that may arise
/// during search, validation, or data processing. Error messages are
/// fetched from localized implementations via [`err()`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Used throughout UI layer for error formatting and display
pub enum ErrorKey {
    // Validation errors
    InvalidSearchInput,
    TaxonTooLong,
    StructureTooLong,
    MassOutOfRange,
    MassRangeInvalid,
    YearOutOfRange,
    YearRangeInvalid,
    ElementCountTooHigh,

    // Taxon resolution errors
    TaxonNotFound,
    TaxonResolutionFailed,

    // Configuration errors
    ApiNotConfigured,

    // Format/parse errors
    UnsupportedFormat,
    TaxonParseFailed,

    // Query/processing errors
    QueryStageFailed,

    // Warnings
    InputStandardized,
    AmbiguousTaxon,

    // Platform-specific (wasm)
    #[cfg(target_arch = "wasm32")]
    WasmLargeQueryFallback,
    #[cfg(target_arch = "wasm32")]
    MemoryHint,
}

/// Format parameterized messages. Parameters are provided as separate arguments.
#[allow(dead_code)] // Public API for parameterized error message formatting
pub enum ErrorParams<'a> {
    None,
    Single(&'a str),
    Pair(&'a str, &'a str),
    Triple(&'a str, &'a str, &'a str),
}

/// Resolve an error key to its localized message.
///
/// This function acts as the primary dispatcher for error message lookup.
/// For parameterized messages (e.g., those containing taxon names or format strings),
/// use [`err_with_params()`] instead.
#[allow(dead_code)] // Used throughout UI layer for error formatting
pub fn err(locale: Locale, key: ErrorKey) -> String {
    match locale {
        Locale::En => lookup_en(key, ErrorParams::None),
        Locale::Fr => lookup_fr(key, ErrorParams::None),
        Locale::De => lookup_de(key, ErrorParams::None),
        Locale::It => lookup_it(key, ErrorParams::None),
    }
}

/// Resolve a parameterized error key to its localized message.
///
/// Use this for error messages containing dynamic content like taxon names,
/// format specifications, or stage descriptions.
#[allow(dead_code)] // Used for parameterized error formatting
pub fn err_with_params(locale: Locale, key: ErrorKey, params: ErrorParams) -> String {
    match locale {
        Locale::En => lookup_en(key, params),
        Locale::Fr => lookup_fr(key, params),
        Locale::De => lookup_de(key, params),
        Locale::It => lookup_it(key, params),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Locale-specific lookup implementations
// ─────────────────────────────────────────────────────────────────────────────

fn lookup_en(key: ErrorKey, params: ErrorParams) -> String {
    match key {
        ErrorKey::InvalidSearchInput => {
            "Please enter a taxon name / QID, or a SMILES structure.".to_string()
        }
        ErrorKey::TaxonTooLong => {
            "Taxon input is too long. Please keep it under 500 characters.".to_string()
        }
        ErrorKey::StructureTooLong => {
            "Structure input is too long. Please shorten the SMILES/Molfile text.".to_string()
        }
        ErrorKey::MassOutOfRange => "Mass values must be between 0 and 10000.".to_string(),
        ErrorKey::MassRangeInvalid => "Mass minimum cannot exceed mass maximum.".to_string(),
        ErrorKey::YearOutOfRange => "Year is outside the supported range.".to_string(),
        ErrorKey::YearRangeInvalid => "Year from cannot exceed year to.".to_string(),
        ErrorKey::ElementCountTooHigh => "Formula element counts are too high.".to_string(),
        ErrorKey::TaxonNotFound => match params {
            ErrorParams::Single(taxon) => {
                format!("Taxon '{taxon}' not found in Wikidata.")
            }
            _ => "Taxon not found.".to_string(),
        },
        ErrorKey::TaxonResolutionFailed => "Taxon resolution failed.".to_string(),
        ErrorKey::ApiNotConfigured => "LOTUS API is not configured.".to_string(),
        ErrorKey::UnsupportedFormat => match params {
            ErrorParams::Single(fmt) => {
                format!("Unsupported format '{fmt}'. Use csv, json, or rdf.")
            }
            _ => "Unsupported format.".to_string(),
        },
        ErrorKey::TaxonParseFailed => match params {
            ErrorParams::Single(detail) => format!("Taxon parse failed: {detail}"),
            _ => "Taxon parse failed.".to_string(),
        },
        ErrorKey::QueryStageFailed => match params {
            ErrorParams::Pair(stage, detail) => format!("{stage} failed: {detail}"),
            _ => "Query stage failed.".to_string(),
        },
        ErrorKey::InputStandardized => match params {
            ErrorParams::Pair(original, normalized) => {
                format!("Input standardized from '{original}' to '{normalized}'.")
            }
            _ => "Input was standardized.".to_string(),
        },
        ErrorKey::AmbiguousTaxon => match params {
            ErrorParams::Triple(best_name, best_qid, names) => {
                format!("Ambiguous taxon name; using {best_name} ({best_qid}). Candidates: {names}")
            }
            _ => "Ambiguous taxon name.".to_string(),
        },
        #[cfg(target_arch = "wasm32")]
        ErrorKey::WasmLargeQueryFallback => match params {
            ErrorParams::Single(err_msg) => {
                format!(
                    "Large-query fallback disabled on wasm to avoid memory exhaustion ({err_msg}). Try adding filters or use a desktop browser for large result exports."
                )
            }
            _ => "Large query fallback disabled.".to_string(),
        },
        #[cfg(target_arch = "wasm32")]
        ErrorKey::MemoryHint => "Result too large for current device memory.".to_string(),
    }
}

fn lookup_fr(key: ErrorKey, params: ErrorParams) -> String {
    match key {
        ErrorKey::InvalidSearchInput => {
            "Veuillez entrer un nom de taxon / QID, ou une structure SMILES.".to_string()
        }
        ErrorKey::TaxonTooLong => {
            "L'entrée du taxon est trop longue. Veuillez rester sous 500 caractères.".to_string()
        }
        ErrorKey::StructureTooLong => {
            "L'entrée de structure est trop longue. Veuillez raccourcir le texte SMILES/Molfile."
                .to_string()
        }
        ErrorKey::MassOutOfRange => {
            "Les valeurs de masse doivent être comprises entre 0 et 10000.".to_string()
        }
        ErrorKey::MassRangeInvalid => {
            "La masse minimale ne peut pas dépasser la masse maximale.".to_string()
        }
        ErrorKey::YearOutOfRange => {
            "L'année est en dehors de la plage prise en charge.".to_string()
        }
        ErrorKey::YearRangeInvalid => {
            "L'année de n'excessif ne peut pas dépasser l'année à.".to_string()
        }
        ErrorKey::ElementCountTooHigh => {
            "Les décomptes d'éléments de formule sont trop élevés.".to_string()
        }
        ErrorKey::TaxonNotFound => match params {
            ErrorParams::Single(taxon) => {
                format!("Le taxon '{taxon}' n'a pas été trouvé dans Wikidata.")
            }
            _ => "Taxon non trouvé.".to_string(),
        },
        ErrorKey::TaxonResolutionFailed => "La résolution du taxon a échoué.".to_string(),
        ErrorKey::ApiNotConfigured => "L'API LOTUS n'est pas configurée.".to_string(),
        ErrorKey::UnsupportedFormat => match params {
            ErrorParams::Single(fmt) => {
                format!("Format non pris en charge '{fmt}'. Utilisez csv, json ou rdf.")
            }
            _ => "Format non pris en charge.".to_string(),
        },
        ErrorKey::TaxonParseFailed => match params {
            ErrorParams::Single(detail) => {
                format!("L'analyse du taxon a échoué : {detail}")
            }
            _ => "L'analyse du taxon a échoué.".to_string(),
        },
        ErrorKey::QueryStageFailed => match params {
            ErrorParams::Pair(stage, detail) => format!("{stage} a échoué : {detail}"),
            _ => "L'étape de requête a échoué.".to_string(),
        },
        ErrorKey::InputStandardized => match params {
            ErrorParams::Pair(original, normalized) => {
                format!("L'entrée a été normalisée de '{original}' à '{normalized}'.")
            }
            _ => "L'entrée a été normalisée.".to_string(),
        },
        ErrorKey::AmbiguousTaxon => match params {
            ErrorParams::Triple(best_name, best_qid, names) => {
                format!(
                    "Nom de taxon ambigu ; utilisation de {best_name} ({best_qid}). Candidats : {names}"
                )
            }
            _ => "Nom de taxon ambigu.".to_string(),
        },
        #[cfg(target_arch = "wasm32")]
        ErrorKey::WasmLargeQueryFallback => match params {
            ErrorParams::Single(err_msg) => {
                format!(
                    "Le secours à grande requête est désactivé sur wasm pour éviter l'épuisement de la mémoire ({err_msg}). Essayez d'ajouter des filtres ou utilisez un navigateur de bureau pour les grandes exportations de résultats."
                )
            }
            _ => "Le secours à grande requête est désactivé.".to_string(),
        },
        #[cfg(target_arch = "wasm32")]
        ErrorKey::MemoryHint => {
            "Résultat trop volumineux pour la mémoire actuelle de l'appareil.".to_string()
        }
    }
}

fn lookup_de(key: ErrorKey, params: ErrorParams) -> String {
    match key {
        ErrorKey::InvalidSearchInput => {
            "Bitte geben Sie einen Taxonnamen / QID oder eine SMILES-Struktur ein.".to_string()
        }
        ErrorKey::TaxonTooLong => {
            "Taxoneingabe ist zu lang. Bitte halten Sie sich unter 500 Zeichen.".to_string()
        }
        ErrorKey::StructureTooLong => {
            "Struktureingabe ist zu lang. Bitte kürzen Sie den SMILES/Molfile-Text.".to_string()
        }
        ErrorKey::MassOutOfRange => "Massewerte müssen zwischen 0 und 10000 liegen.".to_string(),
        ErrorKey::MassRangeInvalid => {
            "Mindestmasse darf Maximalmasse nicht überschreiten.".to_string()
        }
        ErrorKey::YearOutOfRange => "Jahr liegt außerhalb des Unterstützungsbereichs.".to_string(),
        ErrorKey::YearRangeInvalid => "Anfangsjahr darf Endjahr nicht überschreiten.".to_string(),
        ErrorKey::ElementCountTooHigh => "Elementzahlen in der Formel sind zu hoch.".to_string(),
        ErrorKey::TaxonNotFound => match params {
            ErrorParams::Single(taxon) => {
                format!("Taxon '{taxon}' nicht in Wikidata gefunden.")
            }
            _ => "Taxon nicht gefunden.".to_string(),
        },
        ErrorKey::TaxonResolutionFailed => "Taxonauflösung fehlgeschlagen.".to_string(),
        ErrorKey::ApiNotConfigured => "LOTUS-API ist nicht konfiguriert.".to_string(),
        ErrorKey::UnsupportedFormat => match params {
            ErrorParams::Single(fmt) => {
                format!("Nicht unterstütztes Format '{fmt}'. Verwenden Sie csv, json oder rdf.")
            }
            _ => "Nicht unterstütztes Format.".to_string(),
        },
        ErrorKey::TaxonParseFailed => match params {
            ErrorParams::Single(detail) => {
                format!("Taxonanalyse fehlgeschlagen: {detail}")
            }
            _ => "Taxonanalyse fehlgeschlagen.".to_string(),
        },
        ErrorKey::QueryStageFailed => match params {
            ErrorParams::Pair(stage, detail) => format!("{stage} fehlgeschlagen: {detail}"),
            _ => "Abfragestadium fehlgeschlagen.".to_string(),
        },
        ErrorKey::InputStandardized => match params {
            ErrorParams::Pair(original, normalized) => {
                format!("Eingabe standardisiert von '{original}' zu '{normalized}'.")
            }
            _ => "Eingabe wurde standardisiert.".to_string(),
        },
        ErrorKey::AmbiguousTaxon => match params {
            ErrorParams::Triple(best_name, best_qid, names) => {
                format!(
                    "Mehrdeutiger Taxonname; verwende {best_name} ({best_qid}). Kandidaten: {names}"
                )
            }
            _ => "Mehrdeutiger Taxonname.".to_string(),
        },
        #[cfg(target_arch = "wasm32")]
        ErrorKey::WasmLargeQueryFallback => match params {
            ErrorParams::Single(err_msg) => {
                format!(
                    "Große-Abfrage-Fallback auf wasm ist deaktiviert, um Speichererschöpfung zu vermeiden ({err_msg}). Versuchen Sie, Filter hinzuzufügen oder verwenden Sie einen Desktop-Browser für große Ergebnis-Exporte."
                )
            }
            _ => "Große-Abfrage-Fallback ist deaktiviert.".to_string(),
        },
        #[cfg(target_arch = "wasm32")]
        ErrorKey::MemoryHint => "Ergebnis zu groß für den aktuellen Gerätespeicher.".to_string(),
    }
}

fn lookup_it(key: ErrorKey, params: ErrorParams) -> String {
    match key {
        ErrorKey::InvalidSearchInput => {
            "Inserisci un nome di taxon / QID o una struttura SMILES.".to_string()
        }
        ErrorKey::TaxonTooLong => {
            "L'ingresso del taxon è troppo lungo. Mantienilo sotto i 500 caratteri.".to_string()
        }
        ErrorKey::StructureTooLong => {
            "L'ingresso della struttura è troppo lungo. Abbrevia il testo SMILES/Molfile."
                .to_string()
        }
        ErrorKey::MassOutOfRange => {
            "I valori di massa devono essere compresi tra 0 e 10000.".to_string()
        }
        ErrorKey::MassRangeInvalid => {
            "La massa minima non può superare la massa massima.".to_string()
        }
        ErrorKey::YearOutOfRange => "L'anno è fuori dall'intervallo supportato.".to_string(),
        ErrorKey::YearRangeInvalid => "L'anno da non può superare l'anno a.".to_string(),
        ErrorKey::ElementCountTooHigh => {
            "I conteggi degli elementi della formula sono troppo alti.".to_string()
        }
        ErrorKey::TaxonNotFound => match params {
            ErrorParams::Single(taxon) => {
                format!("Taxon '{taxon}' non trovato in Wikidata.")
            }
            _ => "Taxon non trovato.".to_string(),
        },
        ErrorKey::TaxonResolutionFailed => "Risoluzione del taxon non riuscita.".to_string(),
        ErrorKey::ApiNotConfigured => "L'API LOTUS non è configurata.".to_string(),
        ErrorKey::UnsupportedFormat => match params {
            ErrorParams::Single(fmt) => {
                format!("Formato non supportato '{fmt}'. Usa csv, json o rdf.")
            }
            _ => "Formato non supportato.".to_string(),
        },
        ErrorKey::TaxonParseFailed => match params {
            ErrorParams::Single(detail) => {
                format!("Analisi del taxon non riuscita: {detail}")
            }
            _ => "Analisi del taxon non riuscita.".to_string(),
        },
        ErrorKey::QueryStageFailed => match params {
            ErrorParams::Pair(stage, detail) => format!("{stage} non riuscito: {detail}"),
            _ => "Fase di query non riuscita.".to_string(),
        },
        ErrorKey::InputStandardized => match params {
            ErrorParams::Pair(original, normalized) => {
                format!("Input standardizzato da '{original}' a '{normalized}'.")
            }
            _ => "L'ingresso è stato standardizzato.".to_string(),
        },
        ErrorKey::AmbiguousTaxon => match params {
            ErrorParams::Triple(best_name, best_qid, names) => {
                format!(
                    "Nome di taxon ambiguo; utilizzo di {best_name} ({best_qid}). Candidati: {names}"
                )
            }
            _ => "Nome di taxon ambiguo.".to_string(),
        },
        #[cfg(target_arch = "wasm32")]
        ErrorKey::WasmLargeQueryFallback => match params {
            ErrorParams::Single(err_msg) => {
                format!(
                    "Il fallback di query di grandi dimensioni è disabilitato su wasm per evitare l'esaurimento della memoria ({err_msg}). Prova ad aggiungere filtri o utilizza un browser desktop per grandi esportazioni di risultati."
                )
            }
            _ => "Il fallback di query di grandi dimensioni è disabilitato.".to_string(),
        },
        #[cfg(target_arch = "wasm32")]
        ErrorKey::MemoryHint => {
            "Risultato troppo grande per la memoria del dispositivo corrente.".to_string()
        }
    }
}
