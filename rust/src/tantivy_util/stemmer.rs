use tantivy::tokenizer::{Language, Stemmer};

pub fn create_stemmer(lang: &str) -> Stemmer {
    let stemmer_language = match lang {
        "ar" => Language::Arabic,
        "da" => Language::Danish,
        "nl" => Language::Dutch,
        "en" => Language::English,
        "fi" => Language::Finnish,
        "fr" => Language::French,
        "de" => Language::German,
        "el" => Language::Greek,
        "hu" => Language::Hungarian,
        "it" => Language::Italian,
        "no" => Language::Norwegian,
        "pt" => Language::Portuguese,
        "ro" => Language::Romanian,
        "ru" => Language::Russian,
        "es" => Language::Spanish,
        "sv" => Language::Swedish,
        "ta" => Language::Tamil,
        "tr" => Language::Turkish,
        _ => panic!("Unsupported language"),
    };

    Stemmer::new(stemmer_language)
}