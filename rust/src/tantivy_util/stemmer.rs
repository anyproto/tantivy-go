use crate::tantivy_util::TantivyGoError;
use lazy_static::lazy_static;
use phf::phf_map;
use tantivy::tokenizer::{Language, Stemmer};

lazy_static! {
    pub static ref LANGUAGES: phf::Map<&'static str, Language> = phf_map! {
        "ar" => Language::Arabic,
        "hy" => Language::Armenian,
        "eu" => Language::Basque,
        "ca" => Language::Catalan,
        "da" => Language::Danish,
        "nl" => Language::Dutch,
        "en" => Language::English,
        "et" => Language::Estonian,
        "fi" => Language::Finnish,
        "fr" => Language::French,
        "de" => Language::German,
        "el" => Language::Greek,
        "hi" => Language::Hindi,
        "hu" => Language::Hungarian,
        "id" => Language::Indonesian,
        "ga" => Language::Irish,
        "it" => Language::Italian,
        "lt" => Language::Lithuanian,
        "ne" => Language::Nepali,
        "no" => Language::Norwegian,
        "pt" => Language::Portuguese,
        "ro" => Language::Romanian,
        "ru" => Language::Russian,
        "sr" => Language::Serbian,
        "es" => Language::Spanish,
        "sv" => Language::Swedish,
        "ta" => Language::Tamil,
        "tr" => Language::Turkish,
        "yi" => Language::Yiddish,
    };
}

pub fn create_stemmer(lang: &str) -> Result<Stemmer, TantivyGoError> {
    let stemmer_language = LANGUAGES
        .get(lang)
        .ok_or_else(|| TantivyGoError(format!("{lang} is an unsupported language")))?;

    Ok(Stemmer::new(stemmer_language.to_owned()))
}
