// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! German translation table.

use crate::i18n::TextKey;

pub fn de_t(key: TextKey) -> &'static str {
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
        TextKey::PageTitle => "LOTUS Knowledge Search",
        TextKey::GoToHomepage => "Zur Startseite",
        TextKey::PageSubtitle => "Naturstoff-Vorkommen - Verbindung, Taxon, Referenz.",
        TextKey::ResolvedTaxon => "Aufgelöstes Taxon",
        TextKey::QueryHash => "Abfrage-Hash",
        TextKey::ResultHash => "Ergebnis-Hash",
        TextKey::CopyTaxonQid => "Taxon-QID kopieren",
        TextKey::CopyFullQueryHash => "Vollständigen Abfrage-Hash kopieren (SHA-256)",
        TextKey::CopyFullResultHash => "Vollständigen Ergebnis-Hash kopieren (SHA-256)",
        TextKey::CopyShareableLink => "Freigabelink kopieren",
        TextKey::CopySparqlQuery => "SPARQL-Abfrage kopieren",
        TextKey::ArchiveNotice => "Eingefrorenes Archiv:",
        TextKey::Unique => "Eindeutig",
        TextKey::LoadingTitle => "Wikidata wird über QLever abgefragt...",
        TextKey::LoadingHint => "Große Ergebnismengen können einige Sekunden dauern.",
        TextKey::LoadingResolvingTaxon => "Taxon wird aufgelöst...",
        TextKey::LoadingCounting => "Treffer werden gezählt...",
        TextKey::LoadingFetchingPreview => "Vorschauzeilen werden geladen...",
        TextKey::LoadingRendering => "Tabelle wird gerendert...",
        TextKey::Retry => "Erneut versuchen",
        TextKey::ErrorHintValidation => "Bitte Eingaben prüfen, dann erneut versuchen.",
        TextKey::ErrorHintNetwork => "Netzwerkproblem erkannt. Ein erneuter Versuch kann helfen.",
        TextKey::ErrorHintParse => {
            "Antwort konnte nicht verarbeitet werden. Erneut versuchen oder Abfrage verfeinern."
        }
        TextKey::ErrorHintUnknown => "Unerwarteter Fehler. Ein erneuter Versuch kann helfen.",
        TextKey::SkipToResults => "Zu den Ergebnissen springen",
        TextKey::WelcomeLeadA => {
            "Jede Zeile verknüpft eine Verbindung mit dem Organismus, aus dem sie gemeldet wurde, "
        }
        TextKey::WelcomeLeadB => "mit der Literaturreferenz. Die Daten stammen aus der ",
        TextKey::WelcomeLeadC => ", gespeichert auf ",
        TextKey::WelcomeLeadD => " und abgefragt über ",
        TextKey::WelcomeLeadE => ".",
        TextKey::ExampleGentiana => "Taxonname oder Wikidata-QID eingeben",
        TextKey::ExampleAllTriples => "Alle LOTUS-Verbindung-Taxon-Referenz-Tripel",
        TextKey::ExampleSmilesOnly => "SMILES oder Molfile in das Strukturfeld einfügen",
        TextKey::ExampleQueryExecute => "Ausführen",
        TextKey::ExampleQueryTaxon => "CSV herunterladen",
        TextKey::ExampleQueryStructure => "JSON herunterladen",
        TextKey::ExampleQueryAdvanced => "RDF herunterladen",
        TextKey::WelcomeProgrammaticDownload => {
            "Programmgesteuerte URL-Parameter (Abfrage ausführen oder CSV / JSON / RDF laden):"
        }
        TextKey::LabelLanguagePolicy => {
            "Beschriftungen werden zuerst aus 'mul' und dann 'en' aufgelöst, damit Ergebnisse vergleichbar bleiben."
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
        TextKey::EditCopyDaylightSmiles => "Bearbeiten -> Als Daylight SMILES kopieren",
        TextKey::CopyExtendedSmilesMol => "Als erweiterte SMILES / MOL V3000 kopieren",
        TextKey::FormulaFilter => "Formelfilter",
        TextKey::ExactFormula => "Summenformel",
        TextKey::MinCount => "min",
        TextKey::MaxCount => "max",
        TextKey::MinCountAria => "Mindestanzahl",
        TextKey::MaxCountAria => "Maximalanzahl",
        TextKey::ElementRequirement => "Anforderung",
        TextKey::ElementStateAllowed => "erlaubt",
        TextKey::ElementStateRequired => "erforderlich",
        TextKey::ElementStateExcluded => "ausgeschlossen",
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
        TextKey::KetcherHintA => {
            "Sie möchten eine Struktur zeichnen oder suchen? Öffnen Sie den Tab "
        }
        TextKey::KetcherHintB => " und kopieren Sie dann mit ",
        TextKey::KetcherHintC => " (oder ",
        TextKey::KetcherHintD => {
            ") und verwenden Sie den Inhalt im Strukturfeld der Registerkarte Suche."
        }
        TextKey::KetcherIframeTitle => "Ketcher-Struktureditor",
        TextKey::KindNoteSmiles => "  Wird als einzeiliges SPARQL-Literal gesendet.",
        TextKey::KindNoteMol2000 => {
            "  Wird unverändert an SACHEM scoredSubstructureSearch weitergegeben."
        }
        TextKey::KindNoteMol3000 => {
            "  Wird unverändert an SACHEM scoredSubstructureSearch weitergegeben (CTAB v3000)."
        }
        TextKey::DatasetStatistics => "Datensatz-Statistiken",
        TextKey::DownloadResults => "Ergebnisse herunterladen",
        TextKey::PreparingDownload => "Download wird vorbereitet...",
        TextKey::StartingCsvDownload => "CSV-Download wird gestartet...",
        TextKey::PreparingJsonDownload => "JSON-Download wird vorbereitet...",
        TextKey::PreparingRdfDownload => "RDF-Download wird vorbereitet...",
        TextKey::DownloadCsvTitle => "Ergebnisse als CSV herunterladen",
        TextKey::DownloadCsvLabel => "CSV herunterladen",
        TextKey::DownloadJsonTitle => "Ergebnisse als JSON herunterladen",
        TextKey::DownloadJsonLabel => "JSON herunterladen",
        TextKey::DownloadRdfTitle => "Ergebnisse als RDF (Turtle) herunterladen",
        TextKey::DownloadRdfLabel => "RDF herunterladen",
        TextKey::DownloadMetadataTitle => "Schema.org-Metadaten herunterladen (JSON-LD)",
        TextKey::DownloadMetadataLabel => "Metadaten herunterladen",
        TextKey::OpenInQlever => "In QLever öffnen",
        TextKey::OpenInQleverTitle => "Diese Abfrage in der QLever-Weboberfläche öffnen",
        TextKey::SparqlQuery => "SPARQL-Abfrage",
        TextKey::NoResults => "Keine Ergebnisse. Bitte erweitern Sie die Suche.",
        TextKey::StageTaxonSearch => "Taxon-Auflösung",
        TextKey::StageCountQuery => "Ergebniszählung",
        TextKey::StageDisplayQuery => "Vorschauabruf",
        TextKey::StageFallbackQuery => "Fallback-Abruf",
        TextKey::DisplayCappedHint => {
            "Aus Speichergründen werden auf diesem Gerät nur die ersten Zeilen angezeigt. Die Gesamtzahlen bleiben exakt."
        }
        TextKey::Structure => "Struktur",
        TextKey::Compound => "Verbindung",
        TextKey::Mass => "Masse",
        TextKey::Formula => "Formel",
        TextKey::TaxonCol => "Taxon",
        TextKey::Reference => "Referenz",
        TextKey::Year => "Jahr",
        TextKey::FooterData => "Daten",
        TextKey::FooterCitation => "Zitat",
        TextKey::FooterCode => "Code",
        TextKey::FooterArchive => "Archiv",
        TextKey::FooterPrograms => "Programme",
        TextKey::FooterLicense => "Lizenz",
        TextKey::FooterForData => " für Daten ",
        TextKey::FooterForCode => " für Code",
        TextKey::TableTriplesAria => "Verbindung-Taxon-Referenz-Tripel",
        TextKey::OpenFullSizeDepiction => "Darstellung in voller Größe öffnen",
        TextKey::OpenInWikidata => "In Wikidata öffnen",
        TextKey::OpenInScholia => "In Scholia öffnen",
        TextKey::OpenDoi => "DOI öffnen",
        TextKey::Statement => "Aussage",
    }
}
