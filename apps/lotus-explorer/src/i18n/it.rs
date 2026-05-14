// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Italian translation table.

use crate::i18n::TextKey;

pub fn it_t(key: TextKey) -> &'static str {
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
        TextKey::PageTitle => "LOTUS Knowledge Explorer",
        TextKey::GoToHomepage => "Vai alla home page",
        TextKey::PageSubtitle => "Occorrenze di prodotti naturali - composto, taxon, riferimento.",
        TextKey::ResolvedTaxon => "Taxon risolto",
        TextKey::QueryHash => "Hash della query",
        TextKey::ResultHash => "Hash del risultato",
        TextKey::CopyTaxonQid => "Copia QID del taxon",
        TextKey::CopyFullQueryHash => "Copia hash completo della query (SHA-256)",
        TextKey::CopyFullResultHash => "Copia hash completo del risultato (SHA-256)",
        TextKey::CopyShareableLink => "Copia link condivisibile",
        TextKey::CopySparqlQuery => "Copia query SPARQL",
        TextKey::ArchiveNotice => "Archivio congelato:",
        TextKey::Unique => "Uniche",
        TextKey::LoadingTitle => "Interrogazione di Wikidata tramite QLever...",
        TextKey::LoadingHint => {
            "I set di risultati di grandi dimensioni possono richiedere alcuni secondi."
        }
        TextKey::LoadingResolvingTaxon => "Risoluzione del taxon...",
        TextKey::LoadingCounting => "Conteggio delle corrispondenze...",
        TextKey::LoadingFetchingPreview => "Recupero righe di anteprima...",
        TextKey::LoadingRendering => "Rendering della tabella...",
        TextKey::Retry => "Riprova",
        TextKey::ErrorHintValidation => "Controlla l'input e riprova.",
        TextKey::ErrorHintNetwork => "Problema di rete rilevato. Riprova.",
        TextKey::ErrorHintParse => {
            "Impossibile interpretare la risposta. Riprova o affina la query."
        }
        TextKey::ErrorHintUnknown => "Errore inatteso. Riprova.",
        TextKey::SkipToResults => "Vai ai risultati",
        TextKey::WelcomeLeadA => {
            "Ogni riga collega un composto all'organismo da cui è stato segnalato, "
        }
        TextKey::WelcomeLeadB => "con il riferimento bibliografico. I dati provengono dalla ",
        TextKey::WelcomeLeadC => ", archiviati su ",
        TextKey::WelcomeLeadD => " e interrogati tramite ",
        TextKey::WelcomeLeadE => ".",
        TextKey::ExampleGentiana => "Inserisci un nome di taxon o un QID Wikidata",
        TextKey::ExampleAllTriples => "Tutte le triple LOTUS composto-taxon-riferimento",
        TextKey::ExampleSmilesOnly => "Incolla uno SMILES o un Molfile nel campo struttura",
        TextKey::ExampleQueryExecute => "Esegui",
        TextKey::ExampleQueryTaxon => "Scarica CSV",
        TextKey::ExampleQueryStructure => "Scarica JSON",
        TextKey::ExampleQueryAdvanced => "Scarica RDF",
        TextKey::WelcomeProgrammaticDownload => {
            "Parametri URL programmatici (eseguire o scaricare CSV / JSON / RDF):"
        }
        TextKey::LabelLanguagePolicy => {
            "Le etichette usano prima 'mul' e poi 'en' per risultati confrontabili."
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
        TextKey::EditCopyDaylightSmiles => "Edit -> Copy as Daylight SMILES",
        TextKey::CopyExtendedSmilesMol => "Copy as Extended SMILES / MOL V3000",
        TextKey::FormulaFilter => "Filtro formula",
        TextKey::ExactFormula => "Formula bruta",
        TextKey::MinCount => "min",
        TextKey::MaxCount => "max",
        TextKey::MinCountAria => "conteggio minimo",
        TextKey::MaxCountAria => "conteggio massimo",
        TextKey::ElementRequirement => "vincolo",
        TextKey::ElementStateAllowed => "consentito",
        TextKey::ElementStateRequired => "richiesto",
        TextKey::ElementStateExcluded => "escluso",
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
        TextKey::KetcherHintA => "Devi disegnare o cercare una struttura? Apri la scheda ",
        TextKey::KetcherHintB => " e poi copia con ",
        TextKey::KetcherHintC => " (oppure ",
        TextKey::KetcherHintD => ") e usalo nel campo struttura dell'esploratore.",
        TextKey::KetcherIframeTitle => "Editor di strutture Ketcher",
        TextKey::KindNoteSmiles => "  Inviato come letterale SPARQL su una singola riga.",
        TextKey::KindNoteMol2000 => {
            "  Inoltrato senza modifiche a SACHEM scoredSubstructureSearch."
        }
        TextKey::KindNoteMol3000 => {
            "  Inoltrato senza modifiche a SACHEM scoredSubstructureSearch (CTAB v3000)."
        }
        TextKey::DatasetStatistics => "Statistiche del dataset",
        TextKey::DownloadResults => "Scarica i risultati",
        TextKey::PreparingDownload => "Preparazione download...",
        TextKey::StartingCsvDownload => "Avvio download CSV...",
        TextKey::PreparingJsonDownload => "Preparazione download JSON...",
        TextKey::PreparingRdfDownload => "Preparazione download RDF...",
        TextKey::DownloadCsvTitle => "Scarica i risultati in CSV",
        TextKey::DownloadCsvLabel => "Scarica CSV",
        TextKey::DownloadJsonTitle => "Scarica i risultati in JSON",
        TextKey::DownloadJsonLabel => "Scarica JSON",
        TextKey::DownloadRdfTitle => "Scarica i risultati in RDF (Turtle)",
        TextKey::DownloadRdfLabel => "Scarica RDF",
        TextKey::DownloadMetadataTitle => "Scarica metadati Schema.org (JSON-LD)",
        TextKey::DownloadMetadataLabel => "Scarica metadati",
        TextKey::OpenInQlever => "Apri in QLever",
        TextKey::OpenInQleverTitle => "Apri questa query nell'interfaccia web di QLever",
        TextKey::SparqlQuery => "Query SPARQL",
        TextKey::NoResults => "Nessun risultato. Prova ad ampliare la ricerca.",
        TextKey::DisplayCappedHint => {
            "Per sicurezza di memoria su questo dispositivo vengono mostrate solo le prime righe. I conteggi restano esatti."
        }
        TextKey::Structure => "Struttura",
        TextKey::Compound => "Composto",
        TextKey::Mass => "Massa",
        TextKey::Formula => "Formula",
        TextKey::TaxonCol => "Taxon",
        TextKey::Reference => "Riferimento",
        TextKey::Year => "Anno",
        TextKey::FooterData => "Dati",
        TextKey::FooterCitation => "Citazione",
        TextKey::FooterCode => "Codice",
        TextKey::FooterArchive => "Archivio",
        TextKey::FooterPrograms => "Programmi",
        TextKey::FooterLicense => "Licenza",
        TextKey::FooterForData => " per i dati ",
        TextKey::FooterForCode => " per il codice",
        TextKey::TableTriplesAria => "Triple composto-taxon-riferimento",
        TextKey::OpenFullSizeDepiction => "Apri la rappresentazione a dimensione piena",
        TextKey::OpenInWikidata => "Apri in Wikidata",
        TextKey::OpenInScholia => "Apri in Scholia",
        TextKey::OpenDoi => "Apri DOI",
        TextKey::Statement => "Dichiarazione",
    }
}
