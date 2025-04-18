use std::convert::Into;

use lingua::{
    Language,
    Language::{Chinese, English, Russian},
    LanguageDetectorBuilder,
};
use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, ToSchema, Copy, Clone, Serialize, Deserialize,)]
pub enum DetectedLanguage {
    #[default]
    English,
    Russian,
    Chinese,
}

impl From<Language> for DetectedLanguage {
    fn from(value: Language) -> Self {
        match value {
            Chinese => Self::Chinese,
            Russian => Self::Russian,
            _ => Self::English,
        }
    }
}
pub fn detect(text: &str) -> DetectedLanguage {
    let detector = LanguageDetectorBuilder::from_languages(&[English, Russian, Chinese])
        .with_minimum_relative_distance(0.9)
        .build();
    detector
        .detect_language_of(text)
        .map(Into::into)
        .unwrap_or_default()
}
