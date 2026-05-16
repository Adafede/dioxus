// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Minimal i18n helpers for user-facing labels and status text.
//!
//! Keep this intentionally small: one locale switch and a couple of
//! localized labels. It is easy to extend without introducing a full
//! translation framework.
//!
//! Two main systems:
//! - [`TextKey`] — Enumerated UI labels (returns `&'static str`)
//! - [`error_key::ErrorKey`] — Localized error messages (returns `String`)
//!
//! Translation tables live in per-locale submodules:
//! - [`en`] — English
//! - [`fr`] — French (with accents)
//! - [`de`] — German (with umlauts)
//! - [`it`] — Italian (with accents)

mod curation;
pub use curation::*;

mod de;
mod en;
mod fr;
mod it;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    En,
    Fr,
    De,
    It,
}

impl Locale {
    pub fn detect(lang_hint: &str) -> Self {
        let normalized = lang_hint.trim().to_ascii_lowercase();
        if normalized.starts_with("fr") {
            return Self::Fr;
        }
        if normalized.starts_with("de") {
            return Self::De;
        }
        if normalized.starts_with("it") {
            return Self::It;
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(win) = web_sys::window() {
                let win_js = wasm_bindgen::JsValue::from(win);
                if let Ok(nav) =
                    js_sys::Reflect::get(&win_js, &wasm_bindgen::JsValue::from_str("navigator"))
                {
                    if let Ok(lang) =
                        js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("language"))
                    {
                        if let Some(code) = lang.as_string() {
                            let code = code.to_ascii_lowercase();
                            if code.starts_with("fr") {
                                return Self::Fr;
                            }
                            if code.starts_with("de") {
                                return Self::De;
                            }
                            if code.starts_with("it") {
                                return Self::It;
                            }
                        }
                    }
                }
            }
        }

        Self::En
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountNoun {
    Compound,
    Taxon,
    Reference,
    Entry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKey {
    // Generic/meta
    Share,
    Copy,
    Copied,
    CopyToClipboard,
    Notice,
    Error,
    DismissError,
    FiltersShow,
    FiltersHide,
    Language,
    // Header
    PageTitle,
    GoToHomepage,
    PageSubtitle,
    ResolvedTaxon,
    QueryHash,
    ResultHash,
    CopyTaxonQid,
    CopyFullQueryHash,
    CopyFullResultHash,
    CopyShareableLink,
    CopySparqlQuery,
    ArchiveNotice,
    Unique,
    // Loading/welcome
    LoadingTitle,
    LoadingHint,
    LoadingResolvingTaxon,
    LoadingFetchingResults,
    LoadingProcessingResults,
    LoadingRendering,
    Retry,
    ErrorHintValidation,
    ErrorHintNetwork,
    ErrorHintParse,
    ErrorHintUnknown,
    SkipToResults,
    WelcomeLeadA,
    WelcomeLeadB,
    WelcomeLeadC,
    WelcomeLeadD,
    WelcomeLeadE,
    ExampleGentiana,
    ExampleAllTriples,
    ExampleSmilesOnly,
    ExampleQueryExecute,
    ExampleQueryTaxon,
    ExampleQueryStructure,
    ExampleQueryAdvanced,
    WelcomeProgrammaticDownload,
    LabelLanguagePolicy,
    // Search panel
    SearchFilters,
    Taxon,
    TaxonPlaceholder,
    TaxonHint,
    StructureSmilesOrMol,
    StructurePlaceholder,
    StructureHintEmpty,
    Substructure,
    Similarity,
    StructureSearchMode,
    EditCopyDaylightSmiles,
    CopyExtendedSmilesMol,
    FormulaFilter,
    ExactFormula,
    MinCount,
    MaxCount,
    MinCountAria,
    MaxCountAria,
    ElementRequirement,
    ElementStateAllowed,
    ElementStateRequired,
    ElementStateExcluded,
    Search,
    Searching,
    MolecularMass,
    Min,
    Max,
    PublicationYear,
    YearFrom,
    YearTo,
    RunSearch,
    KetcherSummary,
    KetcherHintA,
    KetcherHintB,
    KetcherHintC,
    KetcherHintD,
    KetcherIframeTitle,
    KindNoteSmiles,
    KindNoteMol2000,
    KindNoteMol3000,
    // Error stage labels (used in transport error messages)
    StageTaxonSearch,
    StageResultsQuery,
    // Table/export
    DatasetStatistics,
    DownloadResults,
    PreparingDownload,
    StartingCsvDownload,
    PreparingJsonDownload,
    PreparingRdfDownload,
    DownloadCsvTitle,
    DownloadCsvLabel,
    DownloadJsonTitle,
    DownloadJsonLabel,
    DownloadRdfTitle,
    DownloadRdfLabel,
    DownloadMetadataTitle,
    DownloadMetadataLabel,
    OpenInQlever,
    OpenInQleverTitle,
    SparqlQuery,
    NoResults,
    DisplayCappedHint,
    // Columns
    Structure,
    Compound,
    Mass,
    Formula,
    TaxonCol,
    Reference,
    Year,
    // Footer
    FooterData,
    FooterCitation,
    FooterCode,
    FooterArchive,
    FooterPrograms,
    FooterLicense,
    FooterForData,
    FooterForCode,
    TableTriplesAria,
    OpenFullSizeDepiction,
    OpenInWikidata,
    OpenInScholia,
    OpenDoi,
    Statement,
}

/// Resolve a [`TextKey`] for the given [`Locale`].
///
/// Delegates to the per-locale submodule functions so each translation table
/// lives in its own file and can be edited independently.
pub fn t(locale: Locale, key: TextKey) -> &'static str {
    match locale {
        Locale::En => en::en_t(key),
        Locale::Fr => fr::fr_t(key),
        Locale::De => de::de_t(key),
        Locale::It => it::it_t(key),
    }
}

mod helpers;

pub use helpers::*;

pub mod error_key;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_labels_exist() {
        assert!(!t(Locale::En, TextKey::Search).is_empty());
        assert!(!t(Locale::Fr, TextKey::Search).is_empty());
        assert!(!t(Locale::En, TextKey::SkipToResults).is_empty());
        assert!(!t(Locale::Fr, TextKey::SkipToResults).is_empty());
    }

    #[test]
    fn wikidata_entity_aria_labels_include_an_action() {
        assert_eq!(
            aria_wikidata_entity(Locale::En, "Q42"),
            "Open Wikidata entity Q42"
        );
        assert!(aria_wikidata_entity(Locale::Fr, "Q42").contains("Ouvrir"));
        assert!(aria_wikidata_entity(Locale::De, "Q42").contains("öffnen"));
        assert!(aria_wikidata_entity(Locale::It, "Q42").contains("Apri"));
    }

    #[test]
    fn sort_toggle_aria_includes_column_and_direction() {
        let en = aria_sort_toggle(Locale::En, "Mass", true);
        assert!(en.contains("Mass"));
        assert!(en.contains("descending"));

        let de = aria_sort_toggle(Locale::De, "Jahr", false);
        assert!(de.contains("Jahr"));
        assert!(de.contains("aufsteigend"));
    }

    #[test]
    fn format_count_uses_locale_separators() {
        assert_eq!(format_count(Locale::En, 1_234_567), "1,234,567");
        assert_eq!(format_count(Locale::Fr, 1_234_567), "1 234 567");
        assert_eq!(format_count(Locale::De, 1_234_567), "1.234.567");
        assert_eq!(format_count(Locale::It, 1_234_567), "1.234.567");
        assert_eq!(format_count(Locale::En, 42), "42");
    }

    #[test]
    fn error_key_messages_exist_for_all_locales() {
        let keys = [
            error_key::ErrorKey::InvalidSearchInput,
            error_key::ErrorKey::TaxonTooLong,
            error_key::ErrorKey::StructureTooLong,
            error_key::ErrorKey::MassOutOfRange,
            error_key::ErrorKey::TaxonNotFound,
        ];

        for key in &keys {
            for locale in &[Locale::En, Locale::Fr, Locale::De, Locale::It] {
                let msg = error_key::err(*locale, *key);
                assert!(
                    !msg.is_empty(),
                    "Error message for {:?} in {:?} should not be empty",
                    key,
                    locale
                );
            }
        }
    }
}
