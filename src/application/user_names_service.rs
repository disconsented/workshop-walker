use chrono::{Duration, Utc};
use surrealdb::RecordId;
use tracing::debug;

use crate::domain::user_names::{UserName, UserNameError, UserNamesPort};

pub struct UserNamesService<R: UserNamesPort> {
    repo: R,
}

impl<R: UserNamesPort> UserNamesService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn update_user_name(&self, id: RecordId, name: String) -> Result<(), UserNameError> {
        if match self.repo.get_by_id(id.clone()).await? {
            Some(existing) => {
                // let elapsed = Utc::now().signed_duration_since(existing.last_updated);
                // debug!(elapsed = elapsed.as_seconds_f32(), %name, update = elapsed >
                // Duration::weeks(1),  "checking if username needs updating");
                Utc::now().signed_duration_since(existing.last_updated) > Duration::weeks(1)
            }
            None => true,
        } {
            debug!(?id, %name, "Upserting username");
            self.repo
                .upsert(UserName {
                    id,
                    last_updated: Utc::now(),
                    name,
                })
                .await?;
        }

        Ok(())
    }
}
