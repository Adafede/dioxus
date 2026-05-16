// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! French translation table.

use crate::i18n::TextKey;

pub fn fr_t(key: TextKey) -> &'static str {
    match key {
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
        TextKey::PageTitle => "LOTUS Knowledge Search",
        TextKey::GoToHomepage => "Aller à la page d'accueil",
        TextKey::PageSubtitle => "Occurrences de produits naturels - composé, taxon, référence.",
        TextKey::ResolvedTaxon => "Taxon résolu",
        TextKey::QueryHash => "Hash de la requête",
        TextKey::ResultHash => "Hash du résultat",
        TextKey::CopyTaxonQid => "Copier le QID du taxon",
        TextKey::CopyFullQueryHash => "Copier le hash complet de la requête (SHA-256)",
        TextKey::CopyFullResultHash => "Copier le hash complet du résultat (SHA-256)",
        TextKey::CopyShareableLink => "Copier le lien à partager",
        TextKey::CopySparqlQuery => "Copier la requête SPARQL",
        TextKey::ArchiveNotice => "Archive figée :",
        TextKey::Unique => "Uniques",
        TextKey::LoadingTitle => "Interrogation de Wikidata via QLever...",
        TextKey::LoadingHint => "Les grands jeux de résultats peuvent prendre du temps.",
        TextKey::LoadingResolvingTaxon => "Résolution du taxon...",
        TextKey::LoadingFetchingResults => "Récupération des résultats...",
        TextKey::LoadingProcessingResults => "Traitement des comptages de résultats...",
        TextKey::LoadingRendering => "Rendu du tableau...",
        TextKey::Retry => "Réessayer",
        TextKey::ErrorHintValidation => "Veuillez ajuster la saisie puis réessayer.",
        TextKey::ErrorHintNetwork => "Problème réseau détecté. Réessayer peut aider.",
        TextKey::ErrorHintParse => {
            "Échec de lecture de la réponse. Réessayez ou affinez la requête."
        }
        TextKey::ErrorHintUnknown => "Erreur inattendue. Réessayer peut aider.",
        TextKey::SkipToResults => "Aller aux résultats",
        TextKey::WelcomeLeadA => {
            "Chaque ligne relie un composé à l'organisme où il a été rapporté, "
        }
        TextKey::WelcomeLeadB => "avec la référence bibliographique. Les données proviennent de ",
        TextKey::WelcomeLeadC => ", stockées sur ",
        TextKey::WelcomeLeadD => " et interrogées via ",
        TextKey::WelcomeLeadE => ".",
        TextKey::ExampleGentiana => "Saisir un nom de taxon ou un QID Wikidata",
        TextKey::ExampleAllTriples => "Tous les triplets LOTUS composé-taxon-référence",
        TextKey::ExampleSmilesOnly => "Collez un SMILES ou un Molfile dans la zone structure",
        TextKey::ExampleQueryExecute => "Exécuter",
        TextKey::ExampleQueryTaxon => "Télécharger CSV",
        TextKey::ExampleQueryStructure => "Télécharger JSON",
        TextKey::ExampleQueryAdvanced => "Télécharger RDF",
        TextKey::WelcomeProgrammaticDownload => {
            "Modèles d'URL programmatiques (lancer ou télécharger CSV / JSON / RDF) :"
        }
        TextKey::LabelLanguagePolicy => {
            "Les libellés utilisent d'abord 'mul', puis 'en', pour des résultats comparables."
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
        TextKey::EditCopyDaylightSmiles => "Édition -> Copier en tant que SMILES Daylight",
        TextKey::CopyExtendedSmilesMol => "Copier en tant que SMILES étendus / MOL V3000",
        TextKey::FormulaFilter => "Filtre formule",
        TextKey::ExactFormula => "Formule brute",
        TextKey::MinCount => "min",
        TextKey::MaxCount => "max",
        TextKey::MinCountAria => "compte minimum",
        TextKey::MaxCountAria => "compte maximum",
        TextKey::ElementRequirement => "contrainte",
        TextKey::ElementStateAllowed => "autorisé",
        TextKey::ElementStateRequired => "requis",
        TextKey::ElementStateExcluded => "exclu",
        TextKey::Search => "Rechercher",
        TextKey::Searching => "Recherche...",
        TextKey::MolecularMass => "Masse moléculaire (Da)",
        TextKey::Min => "Min",
        TextKey::Max => "Max",
        TextKey::PublicationYear => "Année de publication",
        TextKey::YearFrom => "De",
        TextKey::YearTo => "À",
        TextKey::RunSearch => "Lancer la recherche",
        TextKey::KetcherSummary => "Éditeur de structure (Ketcher)",
        TextKey::KetcherHintA => "Besoin de dessiner ou trouver une structure ? Ouvrez l'onglet ",
        TextKey::KetcherHintB => ", puis copiez avec ",
        TextKey::KetcherHintC => " (ou ",
        TextKey::KetcherHintD => {
            ") puis utilisez-la dans le champ structure de l'onglet Recherche."
        }
        TextKey::KetcherIframeTitle => "Éditeur de structure Ketcher",
        TextKey::KindNoteSmiles => "  Envoyé comme littéral SPARQL sur une seule ligne.",
        TextKey::KindNoteMol2000 => "  Transmis tel quel à SACHEM scoredSubstructureSearch.",
        TextKey::KindNoteMol3000 => {
            "  Transmis tel quel à SACHEM scoredSubstructureSearch (CTAB v3000)."
        }
        TextKey::DatasetStatistics => "Statistiques du jeu de données",
        TextKey::DownloadResults => "Télécharger les résultats",
        TextKey::PreparingDownload => "Préparation du téléchargement...",
        TextKey::StartingCsvDownload => "Démarrage du téléchargement CSV...",
        TextKey::PreparingJsonDownload => "Préparation du téléchargement JSON...",
        TextKey::PreparingRdfDownload => "Préparation du téléchargement RDF...",
        TextKey::DownloadCsvTitle => "Télécharger les résultats en CSV",
        TextKey::DownloadCsvLabel => "Télécharger CSV",
        TextKey::DownloadJsonTitle => "Télécharger les résultats en JSON",
        TextKey::DownloadJsonLabel => "Télécharger JSON",
        TextKey::DownloadRdfTitle => "Télécharger les résultats en RDF (Turtle)",
        TextKey::DownloadRdfLabel => "Télécharger RDF",
        TextKey::DownloadMetadataTitle => "Télécharger les métadonnées",
        TextKey::DownloadMetadataLabel => "Télécharger les métadonnées",
        TextKey::OpenInQlever => "Ouvrir dans QLever",
        TextKey::OpenInQleverTitle => "Ouvrir cette requête dans l'interface web de QLever",
        TextKey::SparqlQuery => "Requête SPARQL",
        TextKey::NoResults => "Aucun résultat. Essayez une recherche plus large.",
        TextKey::StageTaxonSearch => "résolution du taxon",
        TextKey::StageResultsQuery => "récupération des résultats",
        TextKey::DisplayCappedHint => {
            "Affichage des premières lignes uniquement pour préserver la mémoire de l'appareil. Les totaux restent exacts."
        }
        TextKey::Structure => "Structure",
        TextKey::Compound => "Composé",
        TextKey::Mass => "Masse",
        TextKey::Formula => "Formule brute",
        TextKey::TaxonCol => "Taxon",
        TextKey::Reference => "Référence",
        TextKey::Year => "Année",
        TextKey::FooterData => "Données",
        TextKey::FooterCitation => "Citation",
        TextKey::FooterCode => "Code",
        TextKey::FooterArchive => "Archive",
        TextKey::FooterPrograms => "Programmes",
        TextKey::FooterLicense => "Licence",
        TextKey::FooterForData => " pour les données ",
        TextKey::FooterForCode => " pour le code",
        TextKey::TableTriplesAria => "Triplets composé-taxon-référence",
        TextKey::OpenFullSizeDepiction => "Ouvrir la représentation en taille complète",
        TextKey::OpenInWikidata => "Ouvrir dans Wikidata",
        TextKey::OpenInScholia => "Ouvrir dans Scholia",
        TextKey::OpenDoi => "Ouvrir DOI",
        TextKey::Statement => "Déclaration",
    }
}
