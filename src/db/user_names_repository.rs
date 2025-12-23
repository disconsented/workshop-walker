use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::error;

use crate::domain::user_names::{UserName, UserNameError, UserNamesPort};

pub struct UserNamesSilo {
    db: Surreal<Db>,
}

impl UserNamesSilo {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }
}

impl UserNamesPort for UserNamesSilo {
    async fn upsert(&self, username: UserName) -> Result<(), UserNameError> {
        match self
            .db
            .query("UPSERT usernames CONTENT $username")
            .bind(("username", username))
            .await
            .map(surrealdb::Response::check)
        {
            Ok(Ok(..)) => Ok(()),
            Err(error) | Ok(Err(error)) => {
                error!(?error, "failed to upsert username");
                Err(UserNameError::Internal {
                    msg: error.to_string(),
                })
            }
        }
    }

    async fn get_by_id(&self, id: RecordId) -> Result<Option<UserName>, UserNameError> {
        match self
            .db
            .query("SELECT * FROM $id")
            .bind(("id", id))
            .await
            .map(|r| r.check().map(|mut r| r.take(0)))
        {
            Ok(Ok(Ok(username))) => Ok(username),
            Ok(Err(error) | Ok(Err(error))) | Err(error) => {
                error!(?error, "failed to get user_name by id");
                Err(UserNameError::Internal {
                    msg: error.to_string(),
                })
            }
        }
    }
}
