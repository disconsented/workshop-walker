use crate::{
    db::model::{App},
    domain::apps::{AppError, AppsPort},
};
use crate::db::{IAppID, ITagID};

pub struct AppsService<R: AppsPort> {
    repo: R,
}

impl<R: AppsPort> AppsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list_available(&self) -> Result<Vec<App<ITagID>>, AppError> {
        self.repo.list_available().await
    }

    pub async fn upsert(&self, app: App<ITagID>) -> Result<(), AppError> {
        self.repo.upsert(app).await
    }

    pub async fn remove(&self, id: IAppID) -> Result<(), AppError> {
        self.repo.remove(id).await
    }

    pub async fn list(&self) -> Result<Vec<App<ITagID>>, AppError> {
        self.repo.list().await
    }

    pub async fn get(&self, id: IAppID) -> Result<App<ITagID>, AppError> {
        self.repo.get(id).await
    }
}
