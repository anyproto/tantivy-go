use tantivy::tokenizer::{Language, Stemmer};
use crate::tantivy_util::TantivyGoError;

pub fn create_stemmer(lang: &str) -> Result<Stemmer, TantivyGoError> {
    let stemmer_language = match lang {
        "ar" => Ok(Language::Arabic),
        "da" => Ok(Language::Danish),
        "nl" => Ok(Language::Dutch),
        "en" => Ok(Language::English),
        "fi" => Ok(Language::Finnish),
        "fr" => Ok(Language::French),
        "de" => Ok(Language::German),
        "el" => Ok(Language::Greek),
        "hu" => Ok(Language::Hungarian),
        "it" => Ok(Language::Italian),
        "no" => Ok(Language::Norwegian),
        "pt" => Ok(Language::Portuguese),
        "ro" => Ok(Language::Romanian),
        "ru" => Ok(Language::Russian),
        "es" => Ok(Language::Spanish),
        "sv" => Ok(Language::Swedish),
        "ta" => Ok(Language::Tamil),
        "tr" => Ok(Language::Turkish),
        _ => Err(TantivyGoError(format!("{} is an unsupported language", lang))),
    }?;

    Ok(Stemmer::new(stemmer_language))
}
