use crate::{
    db::{IAppID, ITagID, model::InternalTag},
    domain::tags::{TagError, TagsPort},
};

pub struct TagsService<R: TagsPort> {
    repo: R,
}

impl<R: TagsPort> TagsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn update_tags(&self, app: IAppID, tags: Vec<InternalTag>) -> Result<(), TagError> {
        self.repo.upsert_tags(app, tags).await
    }

    pub async fn set_tag_known_members(&self, tag: ITagID, members: i64) -> Result<(), TagError> {
        self.repo.set_tag_known_members(tag, members).await
    }
}
