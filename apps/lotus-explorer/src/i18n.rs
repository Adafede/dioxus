//! Minimal i18n helpers for user-facing count labels and status text.
//!
//! Keep this intentionally small: one locale switch and a couple of
//! count-aware labels. It is easy to extend without introducing a full
//! translation framework.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    En,
    Fr,
}

impl Locale {
    pub fn detect(lang_hint: &str) -> Self {
        let normalized = lang_hint.trim().to_ascii_lowercase();
        if normalized.starts_with("fr") {
            return Self::Fr;
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
                            if code.to_ascii_lowercase().starts_with("fr") {
                                return Self::Fr;
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
    Row,
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
    // Header
    PageTitle,
    PageSubtitle,
    ResolvedTaxon,
    QueryHash,
    ResultHash,
    TotalMatches,
    CopyTaxonQid,
    CopyFullQueryHash,
    CopyFullResultHash,
    CopyShareableLink,
    CopySparqlQuery,
    // Loading/welcome
    LoadingTitle,
    LoadingHint,
    WelcomeTitle,
    WelcomeTry,
    WelcomeLeadA,
    WelcomeLeadB,
    WelcomeLeadC,
    WelcomeLeadD,
    WelcomeLeadE,
    ExampleGentiana,
    ExampleCannabis,
    ExampleCitrusQid,
    ExampleAllTriples,
    ExampleSmilesOnly,
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
    FormulaFilter,
    ExactFormula,
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
    HeavyExportHint,
    // Table/export
    DatasetStatistics,
    DownloadResults,
    PreparingDownload,
    StartingCsvDownload,
    PreparingJsonDownload,
    PreparingTtlDownload,
    DownloadCsvTitle,
    DownloadJsonTitle,
    DownloadTtlTitle,
    DownloadMetadataTitle,
    Metadata,
    OpenInQlever,
    OpenInQleverTitle,
    SparqlQuery,
    NoResults,
    LoadMore,
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
    FooterCode,
    FooterTools,
    FooterLicense,
    FooterForData,
    FooterForCode,
    TableTriplesAria,
    OpenFullSizeDepiction,
    OpenInWikidata,
    OpenInScholia,
    OpenDoi,
}

pub fn t(locale: Locale, key: TextKey) -> &'static str {
    match locale {
        Locale::En => match key {
            TextKey::Share => "Share",
            TextKey::Copy => "Copy",
            TextKey::Copied => "Copied!",
            TextKey::CopyToClipboard => "Copy to clipboard",
            TextKey::Notice => "Notice",
            TextKey::Error => "Error",
            TextKey::DismissError => "Dismiss error",
            TextKey::FiltersShow => "Show filters",
            TextKey::FiltersHide => "Hide filters",
            TextKey::PageTitle => "LOTUS Wikidata Explorer",
            TextKey::PageSubtitle => "Natural product occurrences - compound, taxon, reference.",
            TextKey::ResolvedTaxon => "Resolved taxon",
            TextKey::QueryHash => "Query hash",
            TextKey::ResultHash => "Result hash",
            TextKey::TotalMatches => "Total matches",
            TextKey::CopyTaxonQid => "Copy taxon QID",
            TextKey::CopyFullQueryHash => "Copy full query hash (SHA-256)",
            TextKey::CopyFullResultHash => "Copy full result hash (SHA-256)",
            TextKey::CopyShareableLink => "Copy shareable link",
            TextKey::CopySparqlQuery => "Copy SPARQL query",
            TextKey::LoadingTitle => "Querying Wikidata via QLever...",
            TextKey::LoadingHint => "Large result sets may take several seconds.",
            TextKey::WelcomeTitle => "Browse natural product occurrences",
            TextKey::WelcomeTry => "Try",
            TextKey::WelcomeLeadA => {
                "Every row ties a compound to the organism it has been reported from, "
            }
            TextKey::WelcomeLeadB => {
                "together with the primary literature reference. Data comes from the "
            }
            TextKey::WelcomeLeadC => ", stored on ",
            TextKey::WelcomeLeadD => " and queried via ",
            TextKey::WelcomeLeadE => ".",
            TextKey::ExampleGentiana => "Compounds from yellow gentian",
            TextKey::ExampleCannabis => "Compounds from Cannabis sativa and subtaxa",
            TextKey::ExampleCitrusQid => "Citrus genus - enter a bare Wikidata QID",
            TextKey::ExampleAllTriples => "All LOTUS compound-taxon-reference triples",
            TextKey::ExampleSmilesOnly => "Paste a SMILES in the structure box - no taxon required",
            TextKey::SearchFilters => "Search filters",
            TextKey::Taxon => "Taxon",
            TextKey::TaxonPlaceholder => "Gentiana lutea - Q34317 - *",
            TextKey::TaxonHint => "Name, Wikidata QID or * for the full dataset.",
            TextKey::StructureSmilesOrMol => "Structure - SMILES or Molfile",
            TextKey::StructurePlaceholder => {
                "c1ccccc1   - or paste a Molfile (V2000 / V3000) block"
            }
            TextKey::StructureHintEmpty => {
                "Optional. One-line SMILES or a full Molfile - paste with trailing \"M  END\"."
            }
            TextKey::Substructure => "Substructure",
            TextKey::Similarity => "Similarity",
            TextKey::StructureSearchMode => "Structure search mode",
            TextKey::FormulaFilter => "Formula filter",
            TextKey::ExactFormula => "Exact formula",
            TextKey::Search => "Search",
            TextKey::Searching => "Searching...",
            TextKey::MolecularMass => "Molecular Mass (Da)",
            TextKey::Min => "Min",
            TextKey::Max => "Max",
            TextKey::PublicationYear => "Publication Year",
            TextKey::YearFrom => "From",
            TextKey::YearTo => "To",
            TextKey::RunSearch => "Run search",
            TextKey::KetcherSummary => "Structure editor (Ketcher)",
            TextKey::KetcherHintA => "Need to draw or look up a structure? Use the ",
            TextKey::KetcherHintB => " panel in the main view, then ",
            TextKey::KetcherHintC => " (or ",
            TextKey::KetcherHintD => ") and paste above.",
            TextKey::KetcherIframeTitle => "Ketcher structure editor",
            TextKey::KindNoteSmiles => "  Sent as a single-line SPARQL literal.",
            TextKey::KindNoteMol2000 => "  Forwarded verbatim to SACHEM scoredSubstructureSearch.",
            TextKey::KindNoteMol3000 => {
                "  Forwarded verbatim to SACHEM scoredSubstructureSearch (CTAB v3000)."
            }
            TextKey::HeavyExportHint => {
                "JSON/TTL disabled on wasm for very large result sets to avoid memory exhaustion. Use CSV export."
            }
            TextKey::DatasetStatistics => "Dataset statistics",
            TextKey::DownloadResults => "Download results",
            TextKey::PreparingDownload => "Preparing download...",
            TextKey::StartingCsvDownload => "Starting CSV download...",
            TextKey::PreparingJsonDownload => "Preparing JSON download...",
            TextKey::PreparingTtlDownload => "Preparing TTL download...",
            TextKey::DownloadCsvTitle => "Download all rows as CSV",
            TextKey::DownloadJsonTitle => {
                "Download all rows as newline-delimited JSON (can take time)"
            }
            TextKey::DownloadTtlTitle => "Download all rows as RDF Turtle (can take time)",
            TextKey::DownloadMetadataTitle => "Download Schema.org metadata (JSON-LD)",
            TextKey::Metadata => "Metadata",
            TextKey::OpenInQlever => "Open in QLever",
            TextKey::OpenInQleverTitle => "Open this query in the QLever web interface",
            TextKey::SparqlQuery => "SPARQL query",
            TextKey::NoResults => "No results. Try broadening your search.",
            TextKey::LoadMore => "Load more",
            TextKey::Structure => "Structure",
            TextKey::Compound => "Compound",
            TextKey::Mass => "Mass",
            TextKey::Formula => "Formula",
            TextKey::TaxonCol => "Taxon",
            TextKey::Reference => "Reference",
            TextKey::Year => "Year",
            TextKey::FooterData => "Data",
            TextKey::FooterCode => "Code",
            TextKey::FooterTools => "Tools",
            TextKey::FooterLicense => "License",
            TextKey::FooterForData => " for data ",
            TextKey::FooterForCode => " for code",
            TextKey::TableTriplesAria => "Compound-taxon-reference triples",
            TextKey::OpenFullSizeDepiction => "Open full-size depiction",
            TextKey::OpenInWikidata => "Open in Wikidata",
            TextKey::OpenInScholia => "Open in Scholia",
            TextKey::OpenDoi => "Open DOI",
        },
        Locale::Fr => match key {
            TextKey::Share => "Partager",
            TextKey::Copy => "Copier",
            TextKey::Copied => "Copié!",
            TextKey::CopyToClipboard => "Copier dans le presse-papiers",
            TextKey::Notice => "Note",
            TextKey::Error => "Erreur",
            TextKey::DismissError => "Fermer l'erreur",
            TextKey::FiltersShow => "Afficher les filtres",
            TextKey::FiltersHide => "Masquer les filtres",
            TextKey::PageTitle => "Explorateur LOTUS Wikidata",
            TextKey::PageSubtitle => {
                "Occurrences de produits naturels - composé, taxon, référence."
            }
            TextKey::ResolvedTaxon => "Taxon resolu",
            TextKey::QueryHash => "Hash de la requête",
            TextKey::ResultHash => "Hash du résultat",
            TextKey::TotalMatches => "Total",
            TextKey::CopyTaxonQid => "Copier le QID du taxon",
            TextKey::CopyFullQueryHash => "Copier le hash complet de la requête (SHA-256)",
            TextKey::CopyFullResultHash => "Copier le hash complet du résultat (SHA-256)",
            TextKey::CopyShareableLink => "Copier le lien à partager",
            TextKey::CopySparqlQuery => "Copier la requête SPARQL",
            TextKey::LoadingTitle => "Interrogation de Wikidata via QLever...",
            TextKey::LoadingHint => "Les grands jeux de résultats peuvent prendre du temps.",
            TextKey::WelcomeTitle => "Explorer les occurrences de produits naturels",
            TextKey::WelcomeTry => "Essayer",
            TextKey::WelcomeLeadA => {
                "Chaque ligne relie un compose à l'organisme dans lequel il est rapporté, "
            }
            TextKey::WelcomeLeadB => {
                "avec la reference bibliographique reliée. Les données viennent de "
            }
            TextKey::WelcomeLeadC => ", stockées sur ",
            TextKey::WelcomeLeadD => " et interrogées via ",
            TextKey::WelcomeLeadE => ".",
            TextKey::ExampleGentiana => "Composés de la gentiane jaune",
            TextKey::ExampleCannabis => "Composés de Cannabis sativa et sous-taxa",
            TextKey::ExampleCitrusQid => "Genre Citrus - saisir un QID Wikidata brut",
            TextKey::ExampleAllTriples => "Tous les triplets composé-taxon-reference LOTUS",
            TextKey::ExampleSmilesOnly => {
                "Collez un SMILES dans la zone structure - pas de taxon requis"
            }
            TextKey::SearchFilters => "Filtres de recherche",
            TextKey::Taxon => "Taxon",
            TextKey::TaxonPlaceholder => "Gentiana lutea - Q34317 - *",
            TextKey::TaxonHint => "Nom, QID Wikidata ou * pour tout le jeu de données.",
            TextKey::StructureSmilesOrMol => "Structure - SMILES ou Molfile",
            TextKey::StructurePlaceholder => "c1ccccc1   - ou collez un Molfile (V2000 / V3000)",
            TextKey::StructureHintEmpty => {
                "Optionnel. SMILES sur une ligne ou Molfile complet - finit par \"M  END\"."
            }
            TextKey::Substructure => "Sous-structure",
            TextKey::Similarity => "Similarité",
            TextKey::StructureSearchMode => "Mode de recherche structure",
            TextKey::FormulaFilter => "Filtre formule",
            TextKey::ExactFormula => "Formule brute",
            TextKey::Search => "Rechercher",
            TextKey::Searching => "Recherche...",
            TextKey::MolecularMass => "Masse moleculaire (Da)",
            TextKey::Min => "Min",
            TextKey::Max => "Max",
            TextKey::PublicationYear => "Année de publication",
            TextKey::YearFrom => "De",
            TextKey::YearTo => "À",
            TextKey::RunSearch => "Lancer la recherche",
            TextKey::KetcherSummary => "Editeur de structure (Ketcher)",
            TextKey::KetcherHintA => "Besoin de dessiner ou trouver une structure ? Utilisez le ",
            TextKey::KetcherHintB => " dans la vue principale, puis ",
            TextKey::KetcherHintC => " (ou ",
            TextKey::KetcherHintD => ") et collez ci-dessus.",
            TextKey::KetcherIframeTitle => "Editeur de structure Ketcher",
            TextKey::KindNoteSmiles => "  Envoyé comme littéral SPARQL sur une seule ligne.",
            TextKey::KindNoteMol2000 => "  Transmis tel quel à SACHEM scoredSubstructureSearch.",
            TextKey::KindNoteMol3000 => {
                "  Transmis tel quel $ SACHEM scoredSubstructureSearch (CTAB v3000)."
            }
            TextKey::HeavyExportHint => {
                "JSON/TTL désactivé sur wasm pour les très grands résultats afin d'éviter la saturation de la memoire. Utilisez CSV."
            }
            TextKey::DatasetStatistics => "Statistiques du jeu de donnees",
            TextKey::DownloadResults => "Télécharger résultats",
            TextKey::PreparingDownload => "Préparation du téléchargement...",
            TextKey::StartingCsvDownload => "Démarrage téléchargement CSV...",
            TextKey::PreparingJsonDownload => "Préparation téléchargement JSON...",
            TextKey::PreparingTtlDownload => "Préparation téléchargement TTL...",
            TextKey::DownloadCsvTitle => "Télécharger toutes les lignes en CSV",
            TextKey::DownloadJsonTitle => "Télécharger toutes les lignes en JSON (peut être long)",
            TextKey::DownloadTtlTitle => "Télécharger toutes les lignes en Turtle RDF",
            TextKey::DownloadMetadataTitle => "Télécharger les metadonnées",
            TextKey::Metadata => "Metadonnées",
            TextKey::OpenInQlever => "Ouvrir dans QLever",
            TextKey::OpenInQleverTitle => "Ouvrir cette requête dans QLever",
            TextKey::SparqlQuery => "Requête SPARQL",
            TextKey::NoResults => "Aucun résultat. Essayez une recherche plus large.",
            TextKey::LoadMore => "Charger plus",
            TextKey::Structure => "Structure",
            TextKey::Compound => "Composé",
            TextKey::Mass => "Masse",
            TextKey::Formula => "Formule brute",
            TextKey::TaxonCol => "Taxon",
            TextKey::Reference => "Référence",
            TextKey::Year => "Année",
            TextKey::FooterData => "Données",
            TextKey::FooterCode => "Code",
            TextKey::FooterTools => "Outils",
            TextKey::FooterLicense => "Licence",
            TextKey::FooterForData => " pour les données ",
            TextKey::FooterForCode => " pour le code",
            TextKey::TableTriplesAria => "Triplets composé-taxon-référence",
            TextKey::OpenFullSizeDepiction => "Ouvrir la dépiction en taille complète",
            TextKey::OpenInWikidata => "Ouvrir dans Wikidata",
            TextKey::OpenInScholia => "Ouvrir dans Scholia",
            TextKey::OpenDoi => "Ouvrir DOI",
        },
    }
}

pub fn threshold_label(locale: Locale, value: f64) -> String {
    match locale {
        Locale::En => format!("Threshold: {value:.2}"),
        Locale::Fr => format!("Seuil: {value:.2}"),
    }
}

pub fn err_invalid_search_input(locale: Locale) -> String {
    match locale {
        Locale::En => "Please enter a taxon name / QID, or a SMILES structure.".to_string(),
        Locale::Fr => "Veuillez saisir un nom de taxon / QID, ou une structure SMILES.".to_string(),
    }
}

pub fn err_taxon_not_found(locale: Locale, taxon: &str) -> String {
    match locale {
        Locale::En => format!("Taxon '{taxon}' not found in Wikidata."),
        Locale::Fr => format!("Taxon '{taxon}' introuvable dans Wikidata."),
    }
}

pub fn warn_input_standardized(locale: Locale, original: &str, normalized: &str) -> String {
    match locale {
        Locale::En => format!("Input standardized from '{original}' to '{normalized}'."),
        Locale::Fr => format!("Entreé standardiseé de '{original}' à '{normalized}'."),
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
    }
}

#[cfg(target_arch = "wasm32")]
pub fn err_wasm_large_query_fallback(locale: Locale, err_msg: &str) -> String {
    match locale {
        Locale::En => format!(
            "Large-query fallback disabled on wasm to avoid memory exhaustion ({err_msg}). Try adding filters or use a desktop browser for large result exports."
        ),
        Locale::Fr => format!(
            "Le repli sur grande requete est désactivé sur wasm pour éviter la saturation de la mémoire ({err_msg}). Essayez d'ajouter des filtres ou utilisez un navigateur desktop pour les grands exports."
        ),
    }
}

pub fn aria_wikidata_entity(locale: Locale, qid: &str) -> String {
    match locale {
        Locale::En => format!("Wikidata {qid}"),
        Locale::Fr => format!("Wikidata {qid}"),
    }
}

pub fn aria_search_inchikey(locale: Locale, ik: &str) -> String {
    match locale {
        Locale::En => format!("Search Wikidata for InChIKey {ik}"),
        Locale::Fr => format!("Rechercher dans Wikidata la cle InChIKey {ik}"),
    }
}

pub fn aria_wikidata_statement(locale: Locale, stmt: &str) -> String {
    match locale {
        Locale::En => format!("Wikidata statement {stmt}"),
        Locale::Fr => format!("Declaration Wikidata {stmt}"),
    }
}

pub fn count_label(locale: Locale, noun: CountNoun, count: usize) -> &'static str {
    match locale {
        Locale::En => match noun {
            CountNoun::Compound => {
                if count == 1 {
                    "Compound"
                } else {
                    "Compounds"
                }
            }
            CountNoun::Taxon => {
                if count == 1 {
                    "Taxon"
                } else {
                    "Taxa"
                }
            }
            CountNoun::Reference => {
                if count == 1 {
                    "Reference"
                } else {
                    "References"
                }
            }
            CountNoun::Entry => {
                if count == 1 {
                    "Entry"
                } else {
                    "Entries"
                }
            }
            CountNoun::Row => {
                if count == 1 {
                    "row"
                } else {
                    "rows"
                }
            }
        },
        Locale::Fr => match noun {
            CountNoun::Compound => {
                if count == 1 {
                    "Composé"
                } else {
                    "Composés"
                }
            }
            CountNoun::Taxon => {
                if count == 1 {
                    "Taxon"
                } else {
                    "Taxa"
                }
            }
            CountNoun::Reference => {
                if count == 1 {
                    "Référence"
                } else {
                    "Références"
                }
            }
            CountNoun::Entry => {
                if count == 1 {
                    "Entrée"
                } else {
                    "Entrées"
                }
            }
            CountNoun::Row => {
                if count == 1 {
                    "ligne"
                } else {
                    "lignes"
                }
            }
        },
    }
}

pub fn showing_rows_text(locale: Locale, visible: usize, total: usize) -> String {
    match locale {
        Locale::En => format!(
            "Showing {visible} of {total} {}",
            count_label(locale, CountNoun::Row, total)
        ),
        Locale::Fr => format!(
            "Affichage de {visible} sur {total} {}",
            count_label(locale, CountNoun::Row, total)
        ),
    }
}
