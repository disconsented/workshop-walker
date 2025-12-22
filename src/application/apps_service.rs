use crate::{
    db::model::App,
    domain::apps::{AppError, AppsPort},
};

pub struct AppsService<R: AppsPort> {
    repo: R,
}

impl<R: AppsPort> AppsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list_available(&self) -> Result<Vec<App>, AppError> {
        self.repo.list_available().await
    }

    pub async fn upsert(&self, app: App) -> Result<(), AppError> {
        self.repo.upsert(app).await
    }

    pub async fn remove(&self, id: u32) -> Result<(), AppError> {
        self.repo.remove(id).await
    }

    pub async fn list(&self) -> Result<Vec<App>, AppError> {
        self.repo.list().await
    }

    pub async fn get(&self, id: u32) -> Result<App, AppError> {
        self.repo.get(id).await
    }
}
