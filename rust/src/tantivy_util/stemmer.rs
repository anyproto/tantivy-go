use crate::tantivy_util::TantivyGoError;
use lazy_static::lazy_static;
use phf::phf_map;
use tantivy::tokenizer::{Language, Stemmer};

lazy_static! {
    pub static ref LANGUAGES: phf::Map<&'static str, Language> = phf_map! {
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
    };
}

pub fn create_stemmer(lang: &str) -> Result<Stemmer, TantivyGoError> {
    let stemmer_language = LANGUAGES
        .get(lang)
        .ok_or_else(|| TantivyGoError(format!("{lang} is an unsupported language")))?;

    Ok(Stemmer::new(stemmer_language.to_owned()))
}
