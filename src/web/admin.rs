use log::error;
use reqwest::StatusCode;
use salvo::{
    Depot, Response, Writer, handler,
    oapi::{ToSchema, extract::JsonBody},
    prelude::{Json, endpoint},
};
use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::local::Db};

use crate::{
    db::{ItemID, UserID},
    model::{Property, Status, User, WorkshopItemProperties},
};

#[endpoint]
pub async fn get_users(depot: &mut Depot, response: &mut Response) {
    match depot
        .obtain::<Surreal<Db>>()
        .expect("getting shared state")
        .query("SELECT record::id(id) as id, * FROM users")
        .await
        .map(|mut q| q.take(0))
    {
        Ok(Ok(results)) => {
            response.render(Json::<Vec<User<UserID>>>(results));
        }
        Ok(Err(e)) | Err(e) => {
            error!("{e:?}");
            response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[endpoint]
pub async fn patch_user(data: JsonBody<PatchUser>, depot: &mut Depot, response: &mut Response) {
    let id = UserID::from(data.0.id);
    if let Some(banned) = data.0.banned {
        let res = depot
            .obtain::<Surreal<Db>>()
            .expect("getting shared state")
            .query("UPDATE $user SET banned=$banned")
            .bind(("user", id.clone()))
            .bind(("banned", banned))
            .await;
        if let Err(e) = res {
            error!("{e:?}");
            response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            return;
        }
    }

    if let Some(admin) = data.0.admin {
        let res = depot
            .obtain::<Surreal<Db>>()
            .expect("getting shared state")
            .query("UPDATE $user SET $admin=admin")
            .bind(("user", id))
            .bind(("admin", admin))
            .await;
        if let Err(e) = res {
            error!("{e:?}");
            response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            return;
        }
    }
    response.status_code(StatusCode::NO_CONTENT);
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct PatchUser {
    pub id: String,
    pub banned: Option<bool>,
    pub admin: Option<bool>,
}

#[endpoint]
pub async fn get_workshop_item_properties(depot: &mut Depot, response: &mut Response) {
    match depot
        .obtain::<Surreal<Db>>()
        .expect("getting shared state")
        .query(
            "SELECT record::id(in) as in, out.*.id.{class,value} as out, source.to_string(), \
             id.to_string(), * FROM workshop_item_properties",
        )
        .await
        .map(|mut q| q.take(0))
    {
        Ok(Ok(results)) => {
            response.render(Json::<Vec<WorkshopItemProperties<String, Property>>>(
                results,
            ));
        }
        Ok(Err(e)) | Err(e) => {
            error!("{e:?}");
            response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[handler]
pub async fn patch_workshop_item_properties(
    data: JsonBody<PatchRelationship>,
    depot: &mut Depot,
    response: &mut Response,
) {
    let res = depot
        .obtain::<Surreal<Db>>()
        .expect("getting shared state")
        .query("LET $link = properties:{class: $class, value: $value}")
        .query(
            "UPDATE ONLY workshop_item_properties SET status=$status WHERE in = $item AND out = \
             $link;",
        )
        .bind(("class", data.0.property.class))
        .bind(("value", data.0.property.value))
        .bind(("item", ItemID::from(data.0.item).into_recordid()))
        .bind(("status", data.0.status))
        .await;
    if let Err(e) = res {
        error!("{e:?}");
        response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        return;
    }
    response.status_code(StatusCode::NO_CONTENT);
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct PatchRelationship {
    pub item: String,
    #[serde(flatten)]
    pub property: Property,
    pub status: Status,
}
