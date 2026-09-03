use surrealdb::{Surreal, engine::local::Db};
use tracing::error;

use crate::{
    db::{
        IItemID, IUserID,
        model::{InternalUser, InternalWorkshopItemProperties},
    },
    domain::admin::{AdminError, AdminPort, PatchRelationshipData, PatchUserData},
};

pub struct AdminSilo {
    pub db: Surreal<Db>,
}

impl AdminSilo {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }
}

impl AdminPort for AdminSilo {
    async fn list_users(&self) -> Result<Vec<InternalUser>, AdminError> {
        match self
            .db
            .query("SELECT * FROM users")
            .await
            .map(|mut q| q.take(0))
        {
            Ok(Ok(results)) => Ok(results),
            Ok(Err(e)) | Err(e) => {
                error!(?e, "failed to list users");
                Err(AdminError::Internal)
            }
        }
    }

    async fn patch_user(&self, patch: PatchUserData) -> Result<(), AdminError> {
        let id: IUserID = patch.id.clone().into();
        if let Some(banned) = patch.banned
            && let Err(e) = self
                .db
                .query("UPDATE $user SET banned=$banned")
                .bind(("user", id.clone()))
                .bind(("banned", banned))
                .await
        {
            error!(?e, "failed to update banned flag");
            return Err(AdminError::Internal);
        }
        if let Some(admin) = patch.admin
            && let Err(e) = self
                .db
                .query("UPDATE $user SET admin=$admin")
                .bind(("user", id))
                .bind(("admin", admin))
                .await
        {
            error!(?e, "failed to update admin flag");
            return Err(AdminError::Internal);
        }
        Ok(())
    }

    async fn list_workshop_item_properties(
        &self,
    ) -> Result<Vec<InternalWorkshopItemProperties>, AdminError> {
        // ToDo: Pagination and sorting by pending
        match self
            .db
            .query("SELECT out.id().{class,value} as out, source, * FROM workshop_item_properties")
            .await
            .map(|mut q| q.take(0))
        {
            Ok(Ok(results)) => Ok(results),
            Ok(Err(e)) | Err(e) => {
                error!(?e, "failed to list workshop item properties");
                Err(AdminError::Internal)
            }
        }
    }

    async fn patch_workshop_item_property(
        &self,
        patch: PatchRelationshipData,
    ) -> Result<(), AdminError> {
        let res = self
            .db
            .query("BEGIN")
            .query("LET $link = properties:{class: $class, value: $value}")
            .query(
                "UPDATE ONLY workshop_item_properties SET status=$status WHERE in = $item AND out \
                 = $link;",
            )
            .query("COMMIT")
            .bind(("class", patch.property.class))
            .bind(("value", patch.property.value))
            .bind(("item", IItemID::from(patch.item)))
            .bind(("status", patch.status))
            .await;
        if let Err(e) = res {
            error!(?e, "failed to patch workshop item property");
            return Err(AdminError::Internal);
        }
        Ok(())
    }
}
