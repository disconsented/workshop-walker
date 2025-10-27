use std::{collections::HashMap, convert::Into, fmt};

use lingua::{
    Language,
    Language::{Chinese, English, Japanese, Korean, Portuguese, Russian, Spanish},
    LanguageDetector, LanguageDetectorBuilder,
};
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use salvo::prelude::ToSchema;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::processing::language_actor::DetectedLanguage::Unknown;

// The threshold of total words a language must be, to be considered valid for
// detection.
const WORD_PERCENTAGE: f32 = 0.2;

#[derive(
    Debug,
    Default,
    ToSchema,
    Copy,
    Clone,
    Serialize_repr,
    Deserialize_repr,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
#[repr(u8)]
pub enum DetectedLanguage {
    #[default]
    English = 1,
    Russian = 2,
    Chinese = 3,
    Japanese = 4,
    Korean = 5,
    Spanish = 6,
    Portuguese = 7,
    Unknown = 0,
}

impl fmt::Display for DetectedLanguage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
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
            Spanish => Self::Spanish,
            Portuguese => Self::Portuguese,
            _ => Unknown,
        }
    }
}

pub struct LanguageActor {}

pub struct LanguageArgs {}
pub struct LanguageState {
    detector: LanguageDetector,
}

pub enum LanguageMsg {
    Detect(String, RpcReplyPort<Vec<DetectedLanguage>>),
}
#[async_trait]
impl Actor for LanguageActor {
    type Arguments = LanguageArgs;
    type Msg = LanguageMsg;
    type State = LanguageState;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        _: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::State {
            detector: LanguageDetectorBuilder::from_languages(&[
                English, Russian, Chinese, Japanese, Korean,
            ])
            .with_minimum_relative_distance(0.9)
            .build(),
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            LanguageMsg::Detect(text, reply) => {
                let _ = reply.send(detect(&text, &state.detector));
            }
        }

        Ok(())
    }
}

/// Using heuristics, determine what languages are likely present in the text.
/// I'd noticed that mods sometimes have translated descriptions, hence, the
/// need to return N langs.
pub fn detect(text: &str, language_detector: &LanguageDetector) -> Vec<DetectedLanguage> {
    let mut detected_languages = HashMap::new();
    let mut total_words = 0;
    for language in language_detector.detect_multiple_languages_of(text) {
        detected_languages
            .entry(language.language())
            .and_modify(|e| *e += language.word_count())
            .or_insert(language.word_count());
        total_words += language.word_count();
    }
    detected_languages
        .into_iter()
        .filter(|&(_, words)| words as f32 > total_words as f32 * WORD_PERCENTAGE)
        .map(|(lang, _)| lang.into())
        .collect()
}

#[cfg(test)]
mod test {
    use lingua::{
        Language::{Chinese, English, Japanese, Korean, Russian},
        LanguageDetectorBuilder,
    };

    use crate::processing::language_actor::DetectedLanguage;

    #[test]
    fn test_lang_encode() {
        // ensures that language is always encoded into an int, catches a surreal
        // performance limitation
        assert_eq!(
            serde_json::to_string(&DetectedLanguage::English).unwrap(),
            "1"
        );
    }
    #[test]
    fn test_mixed() {
        let foo = r"

Adds preserved meals that do not cause food poisoning, lavish and fine meals crafted from them, harmless recreational drugs, and canned preserved foods that can be used as raw ingredients.

Recreational drugs become available after researching Drug Production, while rations, prepared meals, and canned food can be crafted after researching Packaged Survival Meal.

These meals provide greater mood benefits when consumed by Vigils from Misstall's Vigil Race.
https://steamcommunity.com/sharedfiles/filedetails/?id=3473101740

Canned food is processed in a way that prevents spoilage, allowing it to be used as a substitute for raw ingredients in cooking or other recipes that require raw materials. It provides more nutrition than raw food and can be stockpiled in large quantities within a compact space.


Translated with DeepL.com (free version)

==================================================



食中毒を起こさない保存食と､その保存食を材料に作成するlavish､Fine mealと､健康に害を及ぼさない娯楽用のドラッグと､生食材として使用可能な保存食の缶詰を追加します

娯楽用ドラッグ類はドラッグ製造の研究を､レーションや配膳食､缶詰は非常用食品の研究で作成可能です。

Misstall's Vigil Raceのヴィジルたちはこの味気ない食事を好みます。
https://steamcommunity.com/sharedfiles/filedetails/?id=3473101740

缶詰は生の食材を腐食しない形に加工し､料理やその他生食材を原料とする制作物に使用できます。生の食材よりも栄養価が高く、狭いスペースに多数備蓄することが可能です。";

        let detector =
            LanguageDetectorBuilder::from_languages(&[English, Russian, Chinese, Japanese, Korean])
                .with_minimum_relative_distance(0.9)
                .build();
        let mut r: Vec<DetectedLanguage> = dbg!(
            detector
                .detect_multiple_languages_of(foo)
                .into_iter()
                .map(|result| result.language().into())
                .collect()
        );
        r.sort_unstable();
        r.dedup();
        assert_eq!(
            vec![DetectedLanguage::English, DetectedLanguage::Japanese],
            r
        );
    }
}
