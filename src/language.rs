use std::{convert::Into, fmt};

use lingua::{
    Language,
    Language::{Chinese, English, Japanese, Korean, Russian},
    LanguageDetectorBuilder,
};
use salvo::prelude::ToSchema;
use serde_repr::{Deserialize_repr, Serialize_repr};
use crate::language::DetectedLanguage::Unknown;

#[derive(Debug, Default, ToSchema, Copy, Clone, Serialize_repr, Deserialize_repr,)]
#[repr(u8)]
pub enum DetectedLanguage {
    #[default]
    English = 1,
    Russian = 2,
    Chinese = 3,
    Japanese = 4,
    Korean = 5,
    Unknown = 0,
}

impl fmt::Display for DetectedLanguage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl From<Language> for DetectedLanguage {
    fn from(value: Language) -> Self {
        match value {
            Chinese => Self::Chinese,
            Russian => Self::Russian,
            Japanese => Self::Japanese,
            Korean => Self::Korean,
            English => Self::English,
            _ => Unknown
        }
    }
}
pub fn detect(text: &str) -> DetectedLanguage {
    let detector =
        LanguageDetectorBuilder::from_languages(&[English, Russian, Chinese, Japanese, Korean])
            .with_minimum_relative_distance(0.9)
            .build();
    detector
        .detect_language_of(text)
        .map(Into::into)
        .unwrap_or_default()
}


#[cfg(test)]
mod test{
    use crate::language::DetectedLanguage;

    #[test]
    fn test_lang_encode(){
        // ensures that language is always encoded into an int, catches a surreal performance limitation
        assert_eq!(serde_json::to_string(&DetectedLanguage::English).unwrap(), "1")
    }
}