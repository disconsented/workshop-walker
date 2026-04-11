use crate::{
    domain::admin::{AdminError, AdminPort, PatchRelationshipData, PatchUserData},
};
use crate::db::IUserID;
use crate::db::model::{InternalUser, InternalWorkshopItemProperties};

pub struct AdminService<R: AdminPort> {
    repo: R,
}

impl<R: AdminPort> AdminService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list_users(&self) -> Result<Vec<InternalUser>, AdminError> {
        self.repo.list_users().await
    }

    pub async fn patch_user(&self, patch: PatchUserData) -> Result<(), AdminError> {
        if patch.admin.is_none() && patch.banned.is_none() {
            return Err(AdminError::BadRequest {
                msg: "Must set at least one of 'admin' or 'banned'".into(),
            });
        }
        self.repo.patch_user(patch).await
    }

    pub async fn list_workshop_item_properties(
        &self,
    ) -> Result<Vec<InternalWorkshopItemProperties>, AdminError> {
        self.repo.list_workshop_item_properties().await
    }

    pub async fn patch_workshop_item_property(
        &self,
        patch: PatchRelationshipData,
    ) -> Result<(), AdminError> {
        self.repo.patch_workshop_item_property(patch).await
    }
}
