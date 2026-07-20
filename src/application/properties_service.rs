use crate::{
    db::{
        IUserID,
        model::{InternalSource, Status},
    },
    domain::properties::{InternalNewProperty, InternalVoteData, PropertiesError, PropertiesPort},
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
        mut new_property: InternalNewProperty,
        source: InternalSource,
        status: Status,
    ) -> Result<(), PropertiesError> {
        new_property.value = new_property.value.to_ascii_lowercase();

        // Some short/valid entries include UI and art
        if !(2..=32).contains(&new_property.value.len()) {
            return Err(PropertiesError::BadRequest {
                msg: format!(
                    "Property must be between 2 and 32 characters in length; is {}",
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
            .create_or_link_property(new_property, source, status)
            .await
    }

    pub async fn vote(
        &self,
        vote: InternalVoteData,
        userid: IUserID,
    ) -> Result<(), PropertiesError> {
        if vote.score != 1 && vote.score != -1 {
            return Err(PropertiesError::InvalidVoteScore);
        }
        self.repo.vote(vote, userid).await
    }

    pub async fn remove_vote(
        &self,
        vote: InternalVoteData,
        userid: IUserID,
    ) -> Result<(), PropertiesError> {
        self.repo.remove_vote(vote, userid).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::PropertiesService;
    use crate::{
        db::{IItemID, IUserID, model::Class},
        domain::properties::{
            InternalNewProperty, InternalVoteData, PropertiesError, PropertiesPort,
        },
    };

    /// A `PropertiesPort` that records how many times each method was called so
    /// tests can assert whether the service delegated to the repository.
    #[derive(Default)]
    struct SpyRepo {
        vote_calls: AtomicUsize,
        remove_calls: AtomicUsize,
    }

    impl PropertiesPort for SpyRepo {
        async fn create_or_link_property(
            &self,
            _: InternalNewProperty,
            _: crate::db::model::InternalSource,
            _: crate::db::model::Status,
        ) -> Result<(), PropertiesError> {
            Ok(())
        }

        async fn vote(&self, _: InternalVoteData, _: IUserID) -> Result<(), PropertiesError> {
            self.vote_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn remove_vote(
            &self,
            _: InternalVoteData,
            _: IUserID,
        ) -> Result<(), PropertiesError> {
            self.remove_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn vote(score: i32) -> InternalVoteData {
        InternalVoteData {
            item: IItemID::from(1_i64),
            class: Class::Feature,
            value: "ffff".to_string(),
            score,
        }
    }

    #[tokio::test]
    async fn valid_scores_are_forwarded_to_the_repo() {
        let service = PropertiesService::new(SpyRepo::default());
        service.vote(vote(1), IUserID::from(1_i64)).await.unwrap();
        service.vote(vote(-1), IUserID::from(1_i64)).await.unwrap();
        assert_eq!(service.repo.vote_calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn score_zero_is_rejected_before_reaching_the_repo() {
        let service = PropertiesService::new(SpyRepo::default());
        let err = service
            .vote(vote(0), IUserID::from(1_i64))
            .await
            .unwrap_err();
        assert!(matches!(err, PropertiesError::InvalidVoteScore));
        assert_eq!(service.repo.vote_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn out_of_range_scores_are_rejected() {
        let service = PropertiesService::new(SpyRepo::default());
        assert!(matches!(
            service
                .vote(vote(2), IUserID::from(1_i64))
                .await
                .unwrap_err(),
            PropertiesError::InvalidVoteScore
        ));
        assert!(matches!(
            service
                .vote(vote(-5), IUserID::from(1_i64))
                .await
                .unwrap_err(),
            PropertiesError::InvalidVoteScore
        ));
        assert_eq!(service.repo.vote_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn remove_vote_is_forwarded_to_the_repo() {
        let service = PropertiesService::new(SpyRepo::default());
        service
            .remove_vote(vote(1), IUserID::from(1_i64))
            .await
            .unwrap();
        assert_eq!(service.repo.remove_calls.load(Ordering::SeqCst), 1);
    }
}
