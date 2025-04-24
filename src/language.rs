use std::{convert::Into, fmt};

use lingua::{
    Language,
    Language::{Chinese, English, Japanese, Korean, Russian},
    LanguageDetectorBuilder,
};
use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use crate::language::DetectedLanguage::Unknown;

#[derive(Debug, Default, ToSchema, Copy, Clone, Serialize, Deserialize)]
pub enum DetectedLanguage {
    #[default]
    English,
    Russian,
    Chinese,
    Japanese,
    Korean,
    Unknown
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
