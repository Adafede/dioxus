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
