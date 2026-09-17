use crate::{
    db::{IAppID, ITagID, model::InternalTag},
    domain::tags::{TagError, TagsPort},
};

pub struct TagsService<R: TagsPort> {
    repo: R,
}

impl<R: TagsPort> TagsService<R> {
    #[tracing::instrument(level = "trace", skip(repo))]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    #[tracing::instrument(level = "trace", skip(self, app, tags))]
    pub async fn update_tags(&self, app: IAppID, tags: Vec<InternalTag>) -> Result<(), TagError> {
        self.repo.upsert_tags(app, tags).await
    }

    #[tracing::instrument(level = "trace", skip(self, tag, members))]
    pub async fn set_tag_known_members(&self, tag: ITagID, members: i64) -> Result<(), TagError> {
        self.repo.set_tag_known_members(tag, members).await
    }
}
