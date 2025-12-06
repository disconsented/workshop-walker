use crate::{
    db::model::Source,
    domain::properties::{NewProperty, PropertiesError, PropertiesPort, VoteData},
};

pub struct PropertiesService<R: PropertiesPort> {
    repo: R,
}

impl<R: PropertiesPort> PropertiesService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn new_property(
        &self,
        mut new_property: NewProperty,
        source: Source<String>,
    ) -> Result<(), PropertiesError> {
        new_property.value = new_property.value.to_ascii_lowercase();

        // Some short/valid entries include UI and art
        if !(2..=32).contains(&new_property.value.len()) {
            return Err(PropertiesError::BadRequest {
                msg: format!(
                    "Property be between 2 and 32 characters in length; is {}",
                    new_property.value.len()
                ),
            });
        }

        if !new_property
            .value
            .chars()
            .all(|c| c.is_alphabetic() || c.is_ascii_whitespace() || c.is_ascii_punctuation())
        {
            return Err(PropertiesError::BadRequest {
                msg: "Property value must be ascii alphabetic, whitespace or punctuation \
                      characters only"
                    .into(),
            });
        }

        self.repo
            .create_or_link_property(new_property, source)
            .await
    }

    pub async fn vote(&self, vote: VoteData, userid: String) -> Result<(), PropertiesError> {
        if vote.score != 1 && vote.score != -1 {
            return Err(PropertiesError::InvalidVoteScore);
        }
        self.repo.vote(vote, userid).await
    }

    pub async fn remove_vote(&self, vote: VoteData, userid: String) -> Result<(), PropertiesError> {
        self.repo.remove_vote(vote, userid).await
    }
}
