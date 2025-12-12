use surrealdb::{Surreal, engine::local::Db};
use tracing::error;

use crate::{
    db::{
        ItemID, UserID,
        model::{Property, User, WorkshopItemProperties},
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
    async fn list_users(&self) -> Result<Vec<User<String>>, AdminError> {
        match self
            .db
            .query("SELECT id.to_string() as id, * FROM users")
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
        let id = UserID::from(patch.id).into_recordid();
        if let Some(banned) = patch.banned {
            if let Err(e) = self
                .db
                .query("UPDATE $user SET banned=$banned")
                .bind(("user", id.clone()))
                .bind(("banned", banned))
                .await
            {
                error!(?e, "failed to update banned flag");
                return Err(AdminError::Internal);
            }
        }
        if let Some(admin) = patch.admin {
            if let Err(e) = self
                .db
                .query("UPDATE $user SET admin=$admin")
                .bind(("user", id))
                .bind(("admin", admin))
                .await
            {
                error!(?e, "failed to update admin flag");
                return Err(AdminError::Internal);
            }
        }
        Ok(())
    }

    async fn list_workshop_item_properties(
        &self,
    ) -> Result<Vec<WorkshopItemProperties<String, Property>>, AdminError> {
        match self
            .db
            .query(
                "SELECT record::id(in) as in, out.*.id.{class,value} as out, source.to_string(), \
                 id.to_string(), * FROM workshop_item_properties",
            )
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
            .query("LET $link = properties:{class: $class, value: $value}")
            .query(
                "UPDATE ONLY workshop_item_properties SET status=$status WHERE in = $item AND out \
                 = $link;",
            )
            .bind(("class", patch.property.class))
            .bind(("value", patch.property.value))
            .bind(("item", ItemID::from(patch.item).into_recordid()))
            .bind(("status", patch.status))
            .await;
        if let Err(e) = res {
            error!(?e, "failed to patch workshop item property");
            return Err(AdminError::Internal);
        }
        Ok(())
    }
}
