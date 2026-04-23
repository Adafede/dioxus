//! Minimal i18n helpers for user-facing count labels and status text.
//!
//! Keep this intentionally small: one locale switch and a couple of
//! count-aware labels. It is easy to extend without introducing a full
//! translation framework.

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
    Language,
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
    LoadingResolvingTaxon,
    LoadingCounting,
    LoadingFetchingPreview,
    LoadingRendering,
    Retry,
    ErrorHintValidation,
    ErrorHintNetwork,
    ErrorHintServer,
    ErrorHintParse,
    ErrorHintMemory,
    ErrorHintUnknown,
    SkipToResults,
    WelcomeTitle,
    WelcomeTry,
    WelcomeLeadA,
    WelcomeLeadB,
    WelcomeLeadC,
    WelcomeLeadD,
    WelcomeLeadE,
    ExampleGentiana,
    ExampleAllTriples,
    ExampleSmilesOnly,
    WelcomeProgrammaticDownload,
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
            TextKey::Language => "Language",
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
            TextKey::LoadingResolvingTaxon => "Resolving taxon...",
            TextKey::LoadingCounting => "Counting matches...",
            TextKey::LoadingFetchingPreview => "Fetching preview rows...",
            TextKey::LoadingRendering => "Rendering table...",
            TextKey::Retry => "Retry",
            TextKey::ErrorHintValidation => "Please adjust your query input and try again.",
            TextKey::ErrorHintNetwork => "Network issue detected. Retry may succeed.",
            TextKey::ErrorHintServer => "Remote endpoint error. Retry in a few seconds.",
            TextKey::ErrorHintParse => "Response parsing failed. Retry or refine query.",
            TextKey::ErrorHintMemory => "Result too large for current device memory.",
            TextKey::ErrorHintUnknown => "Unexpected error. Retry may help.",
            TextKey::SkipToResults => "Skip to results",
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
            TextKey::ExampleGentiana => "Enter a taxon name or a Wikidata QID",
            TextKey::ExampleAllTriples => "All LOTUS compound-taxon-reference triples",
            TextKey::ExampleSmilesOnly => "Paste a SMILES or Molfile in the structure box",
            TextKey::WelcomeProgrammaticDownload => "Programmatic download URL patterns:",
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
            TextKey::Language => "Langue",
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
            TextKey::LoadingResolvingTaxon => "Résolution du taxon...",
            TextKey::LoadingCounting => "Comptage des correspondances...",
            TextKey::LoadingFetchingPreview => "Récupération de l'aperçu...",
            TextKey::LoadingRendering => "Rendu du tableau...",
            TextKey::Retry => "Réessayer",
            TextKey::ErrorHintValidation => "Veuillez ajuster la saisie puis réessayer.",
            TextKey::ErrorHintNetwork => "Problème réseau détecté. Réessayer peut aider.",
            TextKey::ErrorHintServer => {
                "Erreur du service distant. Réessayez dans quelques secondes."
            }
            TextKey::ErrorHintParse => {
                "Echec de lecture de la réponse. Réessayez ou affinez la requête."
            }
            TextKey::ErrorHintMemory => "Résultat trop volumineux pour la mémoire de l'appareil.",
            TextKey::ErrorHintUnknown => "Erreur inattendue. Réessayer peut aider.",
            TextKey::SkipToResults => "Aller aux résultats",
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
            TextKey::ExampleGentiana => "Saisir un nom de taxon ou un QID Wikidata",
            TextKey::ExampleAllTriples => "Tous les triplets composé-taxon-reference LOTUS",
            TextKey::ExampleSmilesOnly => "Collez un SMILES ou un Molfile dans la zone structure",
            TextKey::WelcomeProgrammaticDownload => {
                "Modèles d'URL pour téléchargement programmatique :"
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
            TextKey::OpenFullSizeDepiction => "Ouvrir la représentation en taille complète",
            TextKey::OpenInWikidata => "Ouvrir dans Wikidata",
            TextKey::OpenInScholia => "Ouvrir dans Scholia",
            TextKey::OpenDoi => "Ouvrir DOI",
        },
        Locale::De => de_t(key),
        Locale::It => it_t(key),
    }
}

fn de_t(key: TextKey) -> &'static str {
    match key {
        TextKey::Share => "Teilen",
        TextKey::Copy => "Kopieren",
        TextKey::Copied => "Kopiert!",
        TextKey::CopyToClipboard => "In die Zwischenablage kopieren",
        TextKey::Notice => "Hinweis",
        TextKey::Error => "Fehler",
        TextKey::DismissError => "Fehler schließen",
        TextKey::FiltersShow => "Filter anzeigen",
        TextKey::FiltersHide => "Filter ausblenden",
        TextKey::Language => "Sprache",
        TextKey::PageTitle => "LOTUS Wikidata Explorer",
        TextKey::PageSubtitle => "Naturstoff-Vorkommen - Verbindung, Taxon, Referenz.",
        TextKey::ResolvedTaxon => "Aufgelöstes Taxon",
        TextKey::QueryHash => "Abfrage-Hash",
        TextKey::ResultHash => "Ergebnis-Hash",
        TextKey::TotalMatches => "Treffer gesamt",
        TextKey::CopyTaxonQid => "Taxon-QID kopieren",
        TextKey::CopyFullQueryHash => "Vollständigen Abfrage-Hash kopieren (SHA-256)",
        TextKey::CopyFullResultHash => "Vollständigen Ergebnis-Hash kopieren (SHA-256)",
        TextKey::CopyShareableLink => "Freigabelink kopieren",
        TextKey::CopySparqlQuery => "SPARQL-Abfrage kopieren",
        TextKey::LoadingTitle => "Wikidata wird über QLever abgefragt...",
        TextKey::LoadingHint => "Große Ergebnismengen können einige Sekunden dauern.",
        TextKey::LoadingResolvingTaxon => "Taxon wird aufgelöst...",
        TextKey::LoadingCounting => "Treffer werden gezählt...",
        TextKey::LoadingFetchingPreview => "Vorschauzeilen werden geladen...",
        TextKey::LoadingRendering => "Tabelle wird gerendert...",
        TextKey::Retry => "Erneut versuchen",
        TextKey::ErrorHintValidation => "Bitte Eingaben prüfen, dann erneut versuchen.",
        TextKey::ErrorHintNetwork => "Netzwerkproblem erkannt. Ein erneuter Versuch kann helfen.",
        TextKey::ErrorHintServer => {
            "Fehler am entfernten Dienst. Versuchen Sie es in einigen Sekunden erneut."
        }
        TextKey::ErrorHintParse => {
            "Antwort konnte nicht verarbeitet werden. Erneut versuchen oder Abfrage verfeinern."
        }
        TextKey::ErrorHintMemory => "Ergebnis ist zu groß für den verfügbaren Gerätespeicher.",
        TextKey::ErrorHintUnknown => "Unerwarteter Fehler. Ein erneuter Versuch kann helfen.",
        TextKey::SkipToResults => "Zu den Ergebnissen springen",
        TextKey::WelcomeTitle => "Naturstoff-Vorkommen durchsuchen",
        TextKey::WelcomeTry => "Beispiele",
        TextKey::WelcomeLeadA => {
            "Jede Zeile verknüpft eine Verbindung mit dem Organismus, aus dem sie berichtet wurde, "
        }
        TextKey::WelcomeLeadB => {
            "zusammen mit der zugehörigen Primärliteratur. Die Daten stammen aus der "
        }
        TextKey::WelcomeLeadC => ", gespeichert in ",
        TextKey::WelcomeLeadD => " und abgefragt über ",
        TextKey::WelcomeLeadE => ".",
        TextKey::ExampleGentiana => "Taxonname oder Wikidata-QID eingeben",
        TextKey::ExampleAllTriples => "Alle LOTUS Verbindung-Taxon-Referenz-Tripel",
        TextKey::ExampleSmilesOnly => "SMILES oder Molfile in das Strukturfeld einfügen",
        TextKey::WelcomeProgrammaticDownload => {
            "Programmgesteuerte Downloads per URL-Parameter (Beispiele):"
        }
        TextKey::SearchFilters => "Suchfilter",
        TextKey::Taxon => "Taxon",
        TextKey::TaxonPlaceholder => "Gentiana lutea - Q34317 - *",
        TextKey::TaxonHint => "Name, Wikidata-QID oder * für den gesamten Datensatz.",
        TextKey::StructureSmilesOrMol => "Struktur - SMILES oder Molfile",
        TextKey::StructurePlaceholder => {
            "c1ccccc1   - oder einen Molfile-Block (V2000 / V3000) einfügen"
        }
        TextKey::StructureHintEmpty => {
            "Optional. Einzeiliges SMILES oder vollständiges Molfile - mit \"M  END\" abschließen."
        }
        TextKey::Substructure => "Substruktur",
        TextKey::Similarity => "Ähnlichkeit",
        TextKey::StructureSearchMode => "Struktursuchmodus",
        TextKey::FormulaFilter => "Formelfilter",
        TextKey::ExactFormula => "Summenformel",
        TextKey::Search => "Suchen",
        TextKey::Searching => "Suche...",
        TextKey::MolecularMass => "Molekulare Masse (Da)",
        TextKey::Min => "Min",
        TextKey::Max => "Max",
        TextKey::PublicationYear => "Publikationsjahr",
        TextKey::YearFrom => "Von",
        TextKey::YearTo => "Bis",
        TextKey::RunSearch => "Suche starten",
        TextKey::KetcherSummary => "Struktureditor (Ketcher)",
        TextKey::KetcherHintA => "Sie möchten eine Struktur zeichnen oder suchen? Nutzen Sie das ",
        TextKey::KetcherHintB => "-Panel in der Hauptansicht und dann ",
        TextKey::KetcherHintC => " (oder ",
        TextKey::KetcherHintD => ") und fügen Sie den Inhalt oben ein.",
        TextKey::KetcherIframeTitle => "Ketcher-Struktureditor",
        TextKey::KindNoteSmiles => "  Wird als einzeiliges SPARQL-Literal gesendet.",
        TextKey::KindNoteMol2000 => {
            "  Wird unverändert an SACHEM scoredSubstructureSearch weitergegeben."
        }
        TextKey::KindNoteMol3000 => {
            "  Wird unverändert an SACHEM scoredSubstructureSearch weitergegeben (CTAB v3000)."
        }
        TextKey::HeavyExportHint => {
            "JSON/TTL ist in wasm bei sehr großen Ergebnissen deaktiviert, um Speicherprobleme zu vermeiden. Bitte CSV verwenden."
        }
        TextKey::DatasetStatistics => "Datensatz-Statistiken",
        TextKey::DownloadResults => "Ergebnisse herunterladen",
        TextKey::PreparingDownload => "Download wird vorbereitet...",
        TextKey::StartingCsvDownload => "CSV-Download wird gestartet...",
        TextKey::PreparingJsonDownload => "JSON-Download wird vorbereitet...",
        TextKey::PreparingTtlDownload => "TTL-Download wird vorbereitet...",
        TextKey::DownloadCsvTitle => "Alle Zeilen als CSV herunterladen",
        TextKey::DownloadJsonTitle => "Alle Zeilen als NDJSON herunterladen (kann dauern)",
        TextKey::DownloadTtlTitle => "Alle Zeilen als RDF Turtle herunterladen (kann dauern)",
        TextKey::DownloadMetadataTitle => "Schema.org-Metadaten herunterladen (JSON-LD)",
        TextKey::Metadata => "Metadaten",
        TextKey::OpenInQlever => "In QLever öffnen",
        TextKey::OpenInQleverTitle => "Diese Abfrage in der QLever-Weboberfläche öffnen",
        TextKey::SparqlQuery => "SPARQL-Abfrage",
        TextKey::NoResults => "Keine Ergebnisse. Bitte erweitern Sie die Suche.",
        TextKey::Structure => "Struktur",
        TextKey::Compound => "Verbindung",
        TextKey::Mass => "Masse",
        TextKey::Formula => "Formel",
        TextKey::TaxonCol => "Taxon",
        TextKey::Reference => "Referenz",
        TextKey::Year => "Jahr",
        TextKey::FooterData => "Daten",
        TextKey::FooterCode => "Code",
        TextKey::FooterTools => "Werkzeuge",
        TextKey::FooterLicense => "Lizenz",
        TextKey::FooterForData => " für Daten ",
        TextKey::FooterForCode => " für Code",
        TextKey::TableTriplesAria => "Verbindung-Taxon-Referenz-Tripel",
        TextKey::OpenFullSizeDepiction => "Darstellung in voller Größe öffnen",
        TextKey::OpenInWikidata => "In Wikidata öffnen",
        TextKey::OpenInScholia => "In Scholia öffnen",
        TextKey::OpenDoi => "DOI öffnen",
    }
}

fn it_t(key: TextKey) -> &'static str {
    match key {
        TextKey::Share => "Condividi",
        TextKey::Copy => "Copia",
        TextKey::Copied => "Copiato!",
        TextKey::CopyToClipboard => "Copia negli appunti",
        TextKey::Notice => "Nota",
        TextKey::Error => "Errore",
        TextKey::DismissError => "Chiudi errore",
        TextKey::FiltersShow => "Mostra filtri",
        TextKey::FiltersHide => "Nascondi filtri",
        TextKey::Language => "Lingua",
        TextKey::PageTitle => "Esploratore LOTUS Wikidata",
        TextKey::PageSubtitle => "Occorrenze di prodotti naturali - composto, taxon, riferimento.",
        TextKey::ResolvedTaxon => "Taxon risolto",
        TextKey::QueryHash => "Hash della query",
        TextKey::ResultHash => "Hash del risultato",
        TextKey::TotalMatches => "Totale corrispondenze",
        TextKey::CopyTaxonQid => "Copia QID del taxon",
        TextKey::CopyFullQueryHash => "Copia hash completo della query (SHA-256)",
        TextKey::CopyFullResultHash => "Copia hash completo del risultato (SHA-256)",
        TextKey::CopyShareableLink => "Copia link condivisibile",
        TextKey::CopySparqlQuery => "Copia query SPARQL",
        TextKey::LoadingTitle => "Interrogazione di Wikidata tramite QLever...",
        TextKey::LoadingHint => "I set di risultati grandi possono richiedere alcuni secondi.",
        TextKey::LoadingResolvingTaxon => "Risoluzione del taxon...",
        TextKey::LoadingCounting => "Conteggio delle corrispondenze...",
        TextKey::LoadingFetchingPreview => "Recupero righe di anteprima...",
        TextKey::LoadingRendering => "Rendering della tabella...",
        TextKey::Retry => "Riprova",
        TextKey::ErrorHintValidation => "Controlla l'input e riprova.",
        TextKey::ErrorHintNetwork => "Problema di rete rilevato. Riprova.",
        TextKey::ErrorHintServer => "Errore del servizio remoto. Riprova tra qualche secondo.",
        TextKey::ErrorHintParse => {
            "Impossibile interpretare la risposta. Riprova o affina la query."
        }
        TextKey::ErrorHintMemory => {
            "Risultato troppo grande per la memoria disponibile sul dispositivo."
        }
        TextKey::ErrorHintUnknown => "Errore inatteso. Riprova.",
        TextKey::SkipToResults => "Vai ai risultati",
        TextKey::WelcomeTitle => "Esplora le occorrenze di prodotti naturali",
        TextKey::WelcomeTry => "Esempi",
        TextKey::WelcomeLeadA => {
            "Ogni riga collega un composto all'organismo da cui è stato riportato, "
        }
        TextKey::WelcomeLeadB => {
            "insieme al riferimento bibliografico primario. I dati provengono dalla "
        }
        TextKey::WelcomeLeadC => ", archiviati su ",
        TextKey::WelcomeLeadD => " e interrogati tramite ",
        TextKey::WelcomeLeadE => ".",
        TextKey::ExampleGentiana => "Inserisci un nome di taxon o un QID Wikidata",
        TextKey::ExampleAllTriples => "Tutte le triple LOTUS composto-taxon-riferimento",
        TextKey::ExampleSmilesOnly => "Incolla uno SMILES o un Molfile nel campo struttura",
        TextKey::WelcomeProgrammaticDownload => {
            "Download programmatico con parametri URL (esempi):"
        }
        TextKey::SearchFilters => "Filtri di ricerca",
        TextKey::Taxon => "Taxon",
        TextKey::TaxonPlaceholder => "Gentiana lutea - Q34317 - *",
        TextKey::TaxonHint => "Nome, QID Wikidata oppure * per l'intero dataset.",
        TextKey::StructureSmilesOrMol => "Struttura - SMILES o Molfile",
        TextKey::StructurePlaceholder => {
            "c1ccccc1   - oppure incolla un blocco Molfile (V2000 / V3000)"
        }
        TextKey::StructureHintEmpty => {
            "Opzionale. SMILES su una riga oppure Molfile completo - termina con \"M  END\"."
        }
        TextKey::Substructure => "Sottostruttura",
        TextKey::Similarity => "Somiglianza",
        TextKey::StructureSearchMode => "Modalità di ricerca struttura",
        TextKey::FormulaFilter => "Filtro formula",
        TextKey::ExactFormula => "Formula bruta",
        TextKey::Search => "Cerca",
        TextKey::Searching => "Ricerca...",
        TextKey::MolecularMass => "Massa molecolare (Da)",
        TextKey::Min => "Min",
        TextKey::Max => "Max",
        TextKey::PublicationYear => "Anno di pubblicazione",
        TextKey::YearFrom => "Da",
        TextKey::YearTo => "A",
        TextKey::RunSearch => "Avvia ricerca",
        TextKey::KetcherSummary => "Editor di strutture (Ketcher)",
        TextKey::KetcherHintA => "Devi disegnare o cercare una struttura? Usa il pannello ",
        TextKey::KetcherHintB => " nella vista principale, poi ",
        TextKey::KetcherHintC => " (oppure ",
        TextKey::KetcherHintD => ") e incolla sopra.",
        TextKey::KetcherIframeTitle => "Editor strutture Ketcher",
        TextKey::KindNoteSmiles => "  Inviato come letterale SPARQL su una singola riga.",
        TextKey::KindNoteMol2000 => {
            "  Inoltrato senza modifiche a SACHEM scoredSubstructureSearch."
        }
        TextKey::KindNoteMol3000 => {
            "  Inoltrato senza modifiche a SACHEM scoredSubstructureSearch (CTAB v3000)."
        }
        TextKey::HeavyExportHint => {
            "JSON/TTL disabilitato su wasm per risultati molto grandi, per evitare esaurimento memoria. Usa CSV."
        }
        TextKey::DatasetStatistics => "Statistiche del dataset",
        TextKey::DownloadResults => "Scarica risultati",
        TextKey::PreparingDownload => "Preparazione download...",
        TextKey::StartingCsvDownload => "Avvio download CSV...",
        TextKey::PreparingJsonDownload => "Preparazione download JSON...",
        TextKey::PreparingTtlDownload => "Preparazione download TTL...",
        TextKey::DownloadCsvTitle => "Scarica tutte le righe in CSV",
        TextKey::DownloadJsonTitle => "Scarica tutte le righe in NDJSON (può richiedere tempo)",
        TextKey::DownloadTtlTitle => "Scarica tutte le righe in RDF Turtle (può richiedere tempo)",
        TextKey::DownloadMetadataTitle => "Scarica metadati Schema.org (JSON-LD)",
        TextKey::Metadata => "Metadati",
        TextKey::OpenInQlever => "Apri in QLever",
        TextKey::OpenInQleverTitle => "Apri questa query nell'interfaccia web di QLever",
        TextKey::SparqlQuery => "Query SPARQL",
        TextKey::NoResults => "Nessun risultato. Prova ad ampliare la ricerca.",
        TextKey::Structure => "Struttura",
        TextKey::Compound => "Composto",
        TextKey::Mass => "Massa",
        TextKey::Formula => "Formula",
        TextKey::TaxonCol => "Taxon",
        TextKey::Reference => "Riferimento",
        TextKey::Year => "Anno",
        TextKey::FooterData => "Dati",
        TextKey::FooterCode => "Codice",
        TextKey::FooterTools => "Strumenti",
        TextKey::FooterLicense => "Licenza",
        TextKey::FooterForData => " per i dati ",
        TextKey::FooterForCode => " per il codice",
        TextKey::TableTriplesAria => "Triple composto-taxon-riferimento",
        TextKey::OpenFullSizeDepiction => "Apri rappresentazione a dimensione piena",
        TextKey::OpenInWikidata => "Apri in Wikidata",
        TextKey::OpenInScholia => "Apri in Scholia",
        TextKey::OpenDoi => "Apri DOI",
    }
}

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

pub fn aria_wikidata_entity(locale: Locale, qid: &str) -> String {
    match locale {
        Locale::En => format!("Wikidata {qid}"),
        Locale::Fr => format!("Wikidata {qid}"),
        Locale::De => format!("Wikidata {qid}"),
        Locale::It => format!("Wikidata {qid}"),
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

pub fn aria_wikidata_statement(locale: Locale, stmt: &str) -> String {
    match locale {
        Locale::En => format!("Wikidata statement {stmt}"),
        Locale::Fr => format!("Déclaration Wikidata {stmt}"),
        Locale::De => format!("Wikidata-Aussage {stmt}"),
        Locale::It => format!("Dichiarazione Wikidata {stmt}"),
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
        Locale::De => match noun {
            CountNoun::Compound => {
                if count == 1 {
                    "Verbindung"
                } else {
                    "Verbindungen"
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
                    "Referenz"
                } else {
                    "Referenzen"
                }
            }
            CountNoun::Entry => {
                if count == 1 {
                    "Eintrag"
                } else {
                    "Einträge"
                }
            }
            CountNoun::Row => {
                if count == 1 {
                    "Zeile"
                } else {
                    "Zeilen"
                }
            }
        },
        Locale::It => match noun {
            CountNoun::Compound => {
                if count == 1 {
                    "Composto"
                } else {
                    "Composti"
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
                    "Riferimento"
                } else {
                    "Riferimenti"
                }
            }
            CountNoun::Entry => {
                if count == 1 {
                    "Voce"
                } else {
                    "Voci"
                }
            }
            CountNoun::Row => {
                if count == 1 {
                    "riga"
                } else {
                    "righe"
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
        Locale::De => format!(
            "Anzeige {visible} von {total} {}",
            count_label(locale, CountNoun::Row, total)
        ),
        Locale::It => format!(
            "Visualizzazione {visible} di {total} {}",
            count_label(locale, CountNoun::Row, total)
        ),
    }
}

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
    fn pluralization_smoke() {
        assert_eq!(count_label(Locale::En, CountNoun::Taxon, 1), "Taxon");
        assert_eq!(count_label(Locale::En, CountNoun::Taxon, 2), "Taxa");
        assert_eq!(count_label(Locale::Fr, CountNoun::Entry, 1), "Entrée");
        assert_eq!(count_label(Locale::Fr, CountNoun::Entry, 2), "Entrées");
    }
}
