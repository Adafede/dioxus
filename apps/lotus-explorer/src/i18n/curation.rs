// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::Locale;

pub fn title_curation_explorer(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Curation Explorer beta",
        Locale::Fr => "Explorateur de curation bêta",
        Locale::De => "Curation Explorer beta",
        Locale::It => "Curation Explorer beta",
    }
}

pub fn subtitle_curation_explorer(locale: Locale) -> &'static str {
    match locale {
        Locale::En => {
            "Create copy-pastable QuickStatements for new compounds or missing metadata. This page runs in the browser with RDKit.js and shareable URL queries."
        }
        Locale::Fr => {
            "Créez des QuickStatements copiables pour de nouveaux composés ou des métadonnées manquantes. Cette page s'exécute dans le navigateur avec RDKit.js et des requêtes URL partageables."
        }
        Locale::De => {
            "Erstellen Sie kopierbare QuickStatements für neue Verbindungen oder fehlende Metadaten. Diese Seite läuft im Browser mit RDKit.js und teilbaren URL-Abfragen."
        }
        Locale::It => {
            "Crea QuickStatements copiabili per nuovi composti o metadati mancanti. Questa pagina viene eseguita nel browser con RDKit.js e query URL condivisibili."
        }
    }
}

pub fn heading_add_one_row(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Add one row",
        Locale::Fr => "Ajouter une ligne",
        Locale::De => "Eine Zeile hinzufügen",
        Locale::It => "Aggiungi una riga",
    }
}
pub fn heading_tsv_import(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "TSV import",
        Locale::Fr => "Import TSV",
        Locale::De => "TSV-Import",
        Locale::It => "Import TSV",
    }
}
pub fn heading_queued_rows(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Queued rows",
        Locale::Fr => "Lignes en file",
        Locale::De => "Wartende Zeilen",
        Locale::It => "Righe in coda",
    }
}
pub fn heading_results(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Curation results",
        Locale::Fr => "Résultats de curation",
        Locale::De => "Curation-Ergebnisse",
        Locale::It => "Risultati curation",
    }
}
pub fn heading_quickstatements(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "QuickStatements (copy-paste)",
        Locale::Fr => "QuickStatements (copier-coller)",
        Locale::De => "QuickStatements (kopieren/einfügen)",
        Locale::It => "QuickStatements (copia-incolla)",
    }
}
pub fn heading_quickstatements_dependencies(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "QuickStatements — prerequisites",
        Locale::Fr => "QuickStatements — prérequis",
        Locale::De => "QuickStatements — Voraussetzungen",
        Locale::It => "QuickStatements — prerequisiti",
    }
}
pub fn placeholder_molecule_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Molecule name",
        Locale::Fr => "Nom de molécule",
        Locale::De => "Molekülname",
        Locale::It => "Nome molecola",
    }
}
pub fn placeholder_taxon_optional(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Taxon (optional)",
        Locale::Fr => "Taxon (optionnel)",
        Locale::De => "Taxon (optional)",
        Locale::It => "Taxon (opzionale)",
    }
}
pub fn placeholder_doi_optional(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "DOI (optional)",
        Locale::Fr => "DOI (optionnel)",
        Locale::De => "DOI (optional)",
        Locale::It => "DOI (opzionale)",
    }
}
pub fn button_add_row(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Add row",
        Locale::Fr => "Ajouter",
        Locale::De => "Zeile hinzufügen",
        Locale::It => "Aggiungi",
    }
}
pub fn button_load_example_rows(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Load example rows",
        Locale::Fr => "Charger des exemples",
        Locale::De => "Beispielzeilen laden",
        Locale::It => "Carica esempi",
    }
}
pub fn button_append_tsv_rows(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Append TSV rows",
        Locale::Fr => "Ajouter les lignes TSV",
        Locale::De => "TSV-Zeilen anhängen",
        Locale::It => "Aggiungi righe TSV",
    }
}
pub fn button_generate_quickstatements(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Generate QuickStatements",
        Locale::Fr => "Générer les QuickStatements",
        Locale::De => "QuickStatements erzeugen",
        Locale::It => "Genera QuickStatements",
    }
}
pub fn button_generating(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Generating...",
        Locale::Fr => "Generation...",
        Locale::De => "Erzeuge...",
        Locale::It => "Generazione...",
    }
}
pub fn button_remove(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Remove",
        Locale::Fr => "Retirer",
        Locale::De => "Entfernen",
        Locale::It => "Rimuovi",
    }
}
pub fn col_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Name",
        Locale::Fr => "Nom",
        Locale::De => "Name",
        Locale::It => "Nome",
    }
}
pub fn col_action(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Action",
        Locale::Fr => "Action",
        Locale::De => "Aktion",
        Locale::It => "Azione",
    }
}
pub fn col_original_smiles(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Original SMILES",
        Locale::Fr => "SMILES original",
        Locale::De => "Original-SMILES",
        Locale::It => "SMILES originale",
    }
}
pub fn col_canonical_smiles(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Canonical SMILES",
        Locale::Fr => "SMILES canonique",
        Locale::De => "Kanonisches SMILES",
        Locale::It => "SMILES canonico",
    }
}
pub fn col_exact_mass(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Exact mass",
        Locale::Fr => "Masse exacte",
        Locale::De => "Exakte Masse",
        Locale::It => "Massa esatta",
    }
}
pub fn col_status(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Status",
        Locale::Fr => "Statut",
        Locale::De => "Status",
        Locale::It => "Stato",
    }
}
pub fn label_new_item(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "new item",
        Locale::Fr => "nouvel élément",
        Locale::De => "neuer Eintrag",
        Locale::It => "nuova voce",
    }
}
pub fn hint_expected_tsv_headers(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Expected headers: name, smiles, organism/taxon, doi",
        Locale::Fr => "En-têtes attendus : name, smiles, organism/taxon, doi",
        Locale::De => "Erwartete Header: name, smiles, organism/taxon, doi",
        Locale::It => "Intestazioni attese: name, smiles, organism/taxon, doi",
    }
}

pub fn msg_name_smiles_required(locale: Locale) -> String {
    match locale {
        Locale::En => "Name and SMILES are required to add a row.".to_string(),
        Locale::Fr => "Le nom et le SMILES sont requis pour ajouter une ligne.".to_string(),
        Locale::De => "Name und SMILES sind erforderlich, um eine Zeile hinzuzufügen.".to_string(),
        Locale::It => "Nome e SMILES sono obbligatori per aggiungere una riga.".to_string(),
    }
}
pub fn msg_duplicate_row_skipped(locale: Locale) -> String {
    match locale {
        Locale::En => "Duplicate row skipped (same structure/taxon/reference).".to_string(),
        Locale::Fr => "Ligne en double ignorée (même structure/taxon/référence).".to_string(),
        Locale::De => "Doppelte Zeile übersprungen (gleiche Struktur/Taxon/Referenz).".to_string(),
        Locale::It => "Riga duplicata ignorata (stessa struttura/taxon/riferimento).".to_string(),
    }
}
pub fn msg_no_valid_tsv_rows(locale: Locale) -> String {
    match locale {
        Locale::En => "No valid rows found in TSV input.".to_string(),
        Locale::Fr => "Aucune ligne valide trouvée dans l'entrée TSV.".to_string(),
        Locale::De => "Keine gültigen Zeilen in der TSV-Eingabe gefunden.".to_string(),
        Locale::It => "Nessuna riga valida trovata nell'input TSV.".to_string(),
    }
}
pub fn msg_tsv_import_complete(locale: Locale, added: usize, skipped: usize) -> String {
    match locale {
        Locale::En => format!(
            "TSV import complete: added {added} unique row(s), skipped {skipped} duplicate(s)."
        ),
        Locale::Fr => format!(
            "Import TSV terminé : {added} ligne(s) unique(s) ajoutée(s), {skipped} doublon(s) ignoré(s)."
        ),
        Locale::De => format!(
            "TSV-Import abgeschlossen: {added} eindeutige Zeile(n) hinzugefügt, {skipped} Duplikat(e) übersprungen."
        ),
        Locale::It => format!(
            "Import TSV completato: aggiunte {added} riga(e) uniche, saltati {skipped} duplicati."
        ),
    }
}
pub fn msg_examples_loaded(locale: Locale, added: usize, skipped: usize) -> String {
    match locale {
        Locale::En => {
            format!("Examples loaded: added {added} unique row(s), skipped {skipped} duplicate(s).")
        }
        Locale::Fr => format!(
            "Exemples chargés : {added} ligne(s) unique(s) ajoutée(s), {skipped} doublon(s) ignoré(s)."
        ),
        Locale::De => format!(
            "Beispiele geladen: {added} eindeutige Zeile(n) hinzugefügt, {skipped} Duplikat(e) übersprungen."
        ),
        Locale::It => format!(
            "Esempi caricati: aggiunte {added} riga(e) uniche, saltati {skipped} duplicati."
        ),
    }
}
pub fn msg_add_row_before_generate(locale: Locale) -> String {
    match locale {
        Locale::En => "Add at least one row before generating QuickStatements.".to_string(),
        Locale::Fr => {
            "Ajoutez au moins une ligne avant de générer les QuickStatements.".to_string()
        }
        Locale::De => {
            "Fügen Sie mindestens eine Zeile hinzu, bevor Sie QuickStatements erzeugen.".to_string()
        }
        Locale::It => "Aggiungi almeno una riga prima di generare i QuickStatements.".to_string(),
    }
}
pub fn msg_running_checks(locale: Locale) -> String {
    match locale {
        Locale::En => "Running curation checks with RDKit.js and Wikidata...".to_string(),
        Locale::Fr => {
            "Exécution des vérifications de curation avec RDKit.js et Wikidata...".to_string()
        }
        Locale::De => {
            "Curation-Prüfungen mit RDKit.js und Wikidata werden ausgeführt...".to_string()
        }
        Locale::It => "Esecuzione dei controlli curation con RDKit.js e Wikidata...".to_string(),
    }
}
pub fn msg_done_review_copy(locale: Locale) -> String {
    match locale {
        Locale::En => "Done. Review generated rows and copy the QuickStatements block.".to_string(),
        Locale::Fr => {
            "Terminé. Vérifiez les lignes générées et copiez le bloc QuickStatements.".to_string()
        }
        Locale::De => {
            "Fertig. Prüfen Sie die erzeugten Zeilen und kopieren Sie den QuickStatements-Block."
                .to_string()
        }
        Locale::It => {
            "Fatto. Controlla le righe generate e copia il blocco QuickStatements.".to_string()
        }
    }
}
pub fn msg_curation_failed(locale: Locale, detail: &str) -> String {
    match locale {
        Locale::En => format!("Curation failed: {detail}"),
        Locale::Fr => format!("Échec de la curation : {detail}"),
        Locale::De => format!("Curation fehlgeschlagen: {detail}"),
        Locale::It => format!("Curation non riuscita: {detail}"),
    }
}

pub fn msg_curation_rate_limited(locale: Locale) -> &'static str {
    match locale {
        Locale::En => {
            "Rate limit reached on an upstream metadata service (HTTP 429). Wait about 60s and retry."
        }
        Locale::Fr => {
            "Limite de débit atteinte sur un service de métadonnées amont (HTTP 429). Attendez environ 60 s puis réessayez."
        }
        Locale::De => {
            "Ratenlimit bei einem vorgelagerten Metadatendienst erreicht (HTTP 429). Warten Sie etwa 60 Sekunden und versuchen Sie es erneut."
        }
        Locale::It => {
            "Limite di richieste raggiunto su un servizio di metadati upstream (HTTP 429). Attendi circa 60 secondi e riprova."
        }
    }
}
pub fn msg_prerequisites_pending(locale: Locale, count: usize) -> String {
    match locale {
        Locale::En => format!(
            "{count} row(s) still wait for prerequisite entities. Run prerequisites, create/merge them in Wikidata, then run second pass."
        ),
        Locale::Fr => format!(
            "{count} ligne(s) attend(ent) encore des entités préalables. Exécutez les prérequis, créez/fusionnez-les dans Wikidata, puis lancez la seconde passe."
        ),
        Locale::De => format!(
            "{count} Zeile(n) warten noch auf vorausgesetzte Entitäten. Führen Sie die Voraussetzungen aus, erstellen/vereinigen Sie sie in Wikidata und starten Sie dann den zweiten Durchlauf."
        ),
        Locale::It => format!(
            "{count} riga(e) sono ancora in attesa di entità prerequisito. Esegui i prerequisiti, crea/unisci in Wikidata, poi avvia il secondo passaggio."
        ),
    }
}

pub fn msg_two_step_hint(locale: Locale) -> &'static str {
    match locale {
        Locale::En => {
            "Two-step workflow: run prerequisites first, create/merge those items in Wikidata, then click Generate QuickStatements again so the main block uses resolved QIDs directly."
        }
        Locale::Fr => {
            "Flux en deux étapes : exécutez d'abord les prérequis, créez/fusionnez ces éléments dans Wikidata, puis cliquez à nouveau sur Générer les QuickStatements pour que le bloc principal utilise directement les QID résolus."
        }
        Locale::De => {
            "Zweistufiger Ablauf: Führen Sie zuerst die Voraussetzungen aus, erstellen/vereinigen Sie diese Einträge in Wikidata und klicken Sie dann erneut auf QuickStatements erzeugen, damit der Hauptblock direkt aufgelöste QIDs verwendet."
        }
        Locale::It => {
            "Flusso in due fasi: esegui prima i prerequisiti, crea/unisci quegli elementi in Wikidata, poi fai di nuovo clic su Genera QuickStatements così il blocco principale usa direttamente i QID risolti."
        }
    }
}

pub fn button_second_pass(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "I created the missing items, let's do the rest",
        Locale::Fr => "J'ai créé les éléments manquants, faisons le reste",
        Locale::De => "Ich habe die fehlenden Einträge erstellt, jetzt den Rest",
        Locale::It => "Ho creato gli elementi mancanti, facciamo il resto",
    }
}

pub fn msg_second_pass_running(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Running second pass on rows that depended on missing items...",
        Locale::Fr => {
            "Exécution de la seconde passe sur les lignes qui dépendaient d'éléments manquants..."
        }
        Locale::De => {
            "Zweiter Durchlauf für Zeilen mit fehlenden Abhängigkeiten wird ausgeführt..."
        }
        Locale::It => {
            "Esecuzione del secondo passaggio sulle righe che dipendevano da elementi mancanti..."
        }
    }
}

pub fn msg_second_pass_done(locale: Locale) -> &'static str {
    match locale {
        Locale::En => {
            "Second pass complete. Main QuickStatements are refreshed with resolved QIDs where available."
        }
        Locale::Fr => {
            "Seconde passe terminée. Les QuickStatements principaux sont rafraîchis avec les QID résolus lorsque disponibles."
        }
        Locale::De => {
            "Zweiter Durchlauf abgeschlossen. Haupt-QuickStatements wurden mit aufgelösten QIDs aktualisiert, sofern verfügbar."
        }
        Locale::It => {
            "Secondo passaggio completato. I QuickStatements principali sono stati aggiornati con i QID risolti quando disponibili."
        }
    }
}

pub fn msg_second_pass_still_pending_count(locale: Locale, count: usize) -> String {
    match locale {
        Locale::En => format!(
            "{count} prerequisite item(s) are still not found. Create/merge them and retry after about 30-120 seconds."
        ),
        Locale::Fr => format!(
            "{count} élément(s) prérequis sont encore introuvables. Créez/fusionnez-les puis réessayez après 30-120 secondes."
        ),
        Locale::De => format!(
            "{count} vorausgesetzte Einträge wurden noch nicht gefunden. Erstellen/vereinigen Sie sie und versuchen Sie es nach etwa 30-120 Sekunden erneut."
        ),
        Locale::It => format!(
            "{count} elemento/i prerequisito non è ancora stato trovato. Crea/unisci gli elementi e riprova dopo circa 30-120 secondi."
        ),
    }
}

pub fn curation_badge_prerequisite_pending(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Prerequisite pending",
        Locale::Fr => "Prerequis en attente",
        Locale::De => "Voraussetzung ausstehend",
        Locale::It => "Prerequisito in attesa",
    }
}

pub fn curation_badge_mass_missing(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Mass missing",
        Locale::Fr => "Masse manquante",
        Locale::De => "Masse fehlt",
        Locale::It => "Massa mancante",
    }
}

pub fn curation_badge_second_pass_required(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Second pass required",
        Locale::Fr => "Seconde passe requise",
        Locale::De => "Zweiter Durchlauf erforderlich",
        Locale::It => "Secondo passaggio richiesto",
    }
}

pub fn curation_mass_warning_title(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Exact mass could not be resolved from descriptor endpoints",
        Locale::Fr => {
            "La masse exacte n'a pas pu être résolue depuis les points de terminaison de descripteurs"
        }
        Locale::De => {
            "Die exakte Masse konnte von den Descriptor-Endpunkten nicht aufgelöst werden"
        }
        Locale::It => "La massa esatta non è stata risolta dagli endpoint dei descrittori",
    }
}

pub fn msg_delay_advice(locale: Locale) -> &'static str {
    match locale {
        Locale::En => {
            "Advice: Wikidata and query endpoints may need 30-120 seconds to expose newly created items."
        }
        Locale::Fr => {
            "Conseil : Wikidata et les points d'accès de requête peuvent nécessiter 30 à 120 secondes pour exposer les nouveaux éléments."
        }
        Locale::De => {
            "Hinweis: Wikidata und Abfrage-Endpunkte benötigen oft 30-120 Sekunden, bis neu erstellte Einträge sichtbar sind."
        }
        Locale::It => {
            "Suggerimento: Wikidata e gli endpoint di query possono richiedere 30-120 secondi per rendere visibili i nuovi elementi."
        }
    }
}

pub fn curation_qs_dev_label(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Open QS-Dev",
        Locale::Fr => "Ouvrir QS-Dev",
        Locale::De => "QS-Dev öffnen",
        Locale::It => "Apri QS-Dev",
    }
}

pub fn curation_qs_dev_prereq_hint(locale: Locale) -> &'static str {
    match locale {
        Locale::En => {
            "Open QS-Dev, paste the prerequisites block, run it, create or merge the new items in Wikidata, wait briefly, then return here for the second pass."
        }
        Locale::Fr => {
            "Ouvrez QS-Dev, collez le bloc de prérequis, exécutez-le, créez ou fusionnez les nouveaux éléments dans Wikidata, attendez un instant, puis revenez ici pour la seconde passe."
        }
        Locale::De => {
            "Öffnen Sie QS-Dev, fügen Sie den Voraussetzungen-Block ein, führen Sie ihn aus, erstellen oder vereinigen Sie die neuen Einträge in Wikidata, warten Sie kurz und kehren Sie dann für den zweiten Durchlauf hierher zurück."
        }
        Locale::It => {
            "Apri QS-Dev, incolla il blocco dei prerequisiti, eseguilo, crea o unisci i nuovi elementi in Wikidata, attendi un momento e poi torna qui per il secondo passaggio."
        }
    }
}

pub fn curation_qs_dev_main_hint(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Open QS-Dev, paste the main block, review the commands, then execute them.",
        Locale::Fr => {
            "Ouvrez QS-Dev, collez le bloc principal, vérifiez les commandes, puis exécutez-les."
        }
        Locale::De => {
            "Öffnen Sie QS-Dev, fügen Sie den Hauptblock ein, prüfen Sie die Befehle und führen Sie sie dann aus."
        }
        Locale::It => {
            "Apri QS-Dev, incolla il blocco principale, controlla i comandi e poi eseguili."
        }
    }
}

pub fn curation_note_existing_complete(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Entry already exists on Wikidata and no missing fields were detected.",
        Locale::Fr => "L'entrée existe déjà dans Wikidata et aucun champ manquant n'a été détecté.",
        Locale::De => {
            "Der Eintrag existiert bereits in Wikidata und es wurden keine fehlenden Felder erkannt."
        }
        Locale::It => "La voce esiste già su Wikidata e non sono stati rilevati campi mancanti.",
    }
}

pub fn curation_note_existing_updates(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Existing Wikidata entry found: generated update QuickStatements.",
        Locale::Fr => "Entrée Wikidata existante trouvée : QuickStatements de mise à jour générés.",
        Locale::De => {
            "Vorhandener Wikidata-Eintrag gefunden: Update-QuickStatements wurden erstellt."
        }
        Locale::It => {
            "Trovata una voce Wikidata esistente: generati QuickStatements di aggiornamento."
        }
    }
}

pub fn curation_note_new_compound(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "No Wikidata entry found by InChIKey: generated creation QuickStatements.",
        Locale::Fr => {
            "Aucune entrée Wikidata trouvée via InChIKey : QuickStatements de création générés."
        }
        Locale::De => {
            "Kein Wikidata-Eintrag über InChIKey gefunden: Erstellungs-QuickStatements wurden erzeugt."
        }
        Locale::It => {
            "Nessuna voce Wikidata trovata tramite InChIKey: generati QuickStatements di creazione."
        }
    }
}

pub fn curation_note_dependencies_pending(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Prerequisite entities are still unresolved.",
        Locale::Fr => "Les entités préalables ne sont pas encore résolues.",
        Locale::De => "Vorausgesetzte Entitäten sind noch nicht aufgelöst.",
        Locale::It => "Le entità prerequisito non sono ancora risolte.",
    }
}

pub fn curation_pending_taxon(locale: Locale, taxon: &str) -> String {
    match locale {
        Locale::En => format!("Taxon '{taxon}' was not found yet in Wikidata."),
        Locale::Fr => format!("Le taxon '{taxon}' n'a pas encore été trouvé dans Wikidata."),
        Locale::De => format!("Taxon '{taxon}' wurde in Wikidata noch nicht gefunden."),
        Locale::It => format!("Il taxon '{taxon}' non è ancora stato trovato in Wikidata."),
    }
}

pub fn curation_pending_reference(locale: Locale, doi: &str) -> String {
    match locale {
        Locale::En => format!("Reference for DOI '{doi}' is not available yet."),
        Locale::Fr => format!("La référence pour le DOI '{doi}' n'est pas encore disponible."),
        Locale::De => format!("Die Referenz für DOI '{doi}' ist noch nicht verfügbar."),
        Locale::It => format!("La referenza per il DOI '{doi}' non è ancora disponibile."),
    }
}

pub fn curation_status_label(locale: Locale, status_key: &str) -> &'static str {
    match (locale, status_key) {
        (Locale::En, "existing_complete") => "already complete",
        (Locale::En, "existing_updates") => "existing item, updates generated",
        (Locale::En, "new_compound") => "new item, creation generated",
        (Locale::En, "pending_dependencies") => "waiting for prerequisite entities",
        (Locale::En, "error") => "error",
        (Locale::Fr, "existing_complete") => "déjà complet",
        (Locale::Fr, "existing_updates") => "élément existant, mises à jour générées",
        (Locale::Fr, "new_compound") => "nouvel élément, création générée",
        (Locale::Fr, "pending_dependencies") => "en attente des entités prérequises",
        (Locale::Fr, "error") => "erreur",
        (Locale::De, "existing_complete") => "bereits vollständig",
        (Locale::De, "existing_updates") => "vorhandener eintrag, updates erzeugt",
        (Locale::De, "new_compound") => "neuer Eintrag, Erstellung erzeugt",
        (Locale::De, "pending_dependencies") => "wartet auf vorausgesetzte entitäten",
        (Locale::De, "error") => "fehler",
        (Locale::It, "existing_complete") => "già completo",
        (Locale::It, "existing_updates") => "voce esistente, aggiornamenti generati",
        (Locale::It, "new_compound") => "nuova voce, creazione generata",
        (Locale::It, "pending_dependencies") => "in attesa delle entità prerequisito",
        (Locale::It, "error") => "errore",
        (_, _) => "status",
    }
}

pub fn view_switch_aria(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "View",
        Locale::Fr => "Vue",
        Locale::De => "Ansicht",
        Locale::It => "Vista",
    }
}

pub fn view_label_explorer(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Explorer",
        Locale::Fr => "Explorateur",
        Locale::De => "Explorer",
        Locale::It => "Explorer",
    }
}

pub fn view_label_curation_explorer(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Curation Explorer",
        Locale::Fr => "Explorateur de curation",
        Locale::De => "Curation Explorer",
        Locale::It => "Curation Explorer",
    }
}

pub fn view_label_draw(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Draw",
        Locale::Fr => "Dessiner",
        Locale::De => "Zeichnen",
        Locale::It => "Disegna",
    }
}
