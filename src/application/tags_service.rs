use crate::{
    domain::tags::{TagError, TagsPort},
};
use crate::db::IAppID;
use crate::db::model::InternalTag;

pub struct TagsService<R: TagsPort> {
    repo: R,
}

impl<R: TagsPort> TagsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn update_tags(&self, app_id: IAppID, tags: Vec<InternalTag>) -> Result<(), TagError> {
        self.repo.upsert_tags(app_id, tags).await
    }
}
