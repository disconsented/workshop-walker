use crate::{
    db::model::Tag,
    domain::tags::{TagError, TagsPort},
};

pub struct TagsService<R: TagsPort> {
    repo: R,
}

impl<R: TagsPort> TagsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn update_tags(&self, app_id: u32, tags: Vec<Tag>) -> Result<(), TagError> {
        self.repo.upsert_tags(app_id, tags).await
    }
}
