use std::sync::OnceLock;

use ractor::{async_trait, call, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use salvo::{
    oapi::{endpoint, extract::PathParam}, prelude::{Json, StatusCode, StatusError},
    Depot,
    Writer,
};
use surrealdb::{engine::local::Db, Surreal};
use surrealdb_core::sql::{
    field::Selector, lookup::{LookupKind, LookupSubject}, part::DestructurePart, statements::SelectStatement, BinaryOperator, Closure, Dir, Expr, Field, Fields, Idiom, Kind,
    Literal,
    Lookup,
    Param,
    Part,
};
use surrealdb_types::{RecordId, SurrealValue, ToSql};
use tracing::{debug, error, instrument};

use crate::{
    db::{
        model::{ExternalFullWorkshopItem, InternalFullWorkshopItem, Status}, IItemID,
        IUserID,
    },
    web::auth,
};

static ITEM_ACTOR: OnceLock<ActorRef<ItemMsg>> = OnceLock::new();

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = StatusError;

#[derive(Debug)]
enum InnerError {
    NotFound,
    InternalError,
}

impl InnerError {
    fn status_code(&self) -> StatusCode {
        match self {
            InnerError::NotFound => StatusCode::NOT_FOUND,
            InnerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<InnerError> for StatusError {
    fn from(value: InnerError) -> Self {
        let mut error = StatusError::internal_server_error();
        error.code = value.status_code();
        error.name = value
            .status_code()
            .canonical_reason()
            .unwrap_or_default()
            .to_string();
        error.brief = format!("{value:?}");
        error.detail = None;
        error
    }
}

pub struct ItemActor;
pub struct ItemState {
    database: Surreal<Db>,
}
pub struct ItemArgs {
    pub database: Surreal<Db>,
}

pub enum ItemMsg {
    Get(
        IItemID,
        Option<IUserID>,
        RpcReplyPort<Result<InternalFullWorkshopItem>>,
    ),
}

#[async_trait]
impl Actor for ItemActor {
    type Arguments = ItemArgs;
    type Msg = ItemMsg;
    type State = ItemState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        ITEM_ACTOR.get_or_init(|| myself);
        Ok(ItemState {
            database: args.database,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ItemMsg::Get(id, user, reply) => {
                let res = get_item(&state.database, id, user).await;
                if reply.send(res).is_err() {
                    error!(message = "Get", "Failed to reply to message");
                }
            }
        }
        Ok(())
    }
}

async fn get_item(
    db: &Surreal<Db>,
    id: IItemID,
    user: Option<IUserID>,
) -> Result<InternalFullWorkshopItem> {
    let dep_fields = [
        DestructurePart::Field("app".to_string()),
        DestructurePart::Field("author".to_string()),
        DestructurePart::Field("description".to_string()),
        DestructurePart::Field("id".to_string()),
        DestructurePart::Field("languages".to_string()),
        DestructurePart::Field("last_updated".to_string()),
        DestructurePart::Field("preview_url".to_string()),
        DestructurePart::Field("score".to_string()),
        DestructurePart::Field("title".to_string()),
        DestructurePart::All("tags".to_string()),
    ];
    let mut stmt = SelectStatement::default();
    stmt.what = vec![Expr::from_public_value(
        RecordId::from(id.clone()).into_value(),
    )];

    stmt.fields = Fields::Select(vec![
        Field::All,
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![Part::Field("tags".to_string()), Part::All])),
            alias: None,
        }),
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![Part::Field("author".to_string()), Part::All])),
            alias: None,
        }),
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![
                Part::Graph(Lookup {
                    kind: LookupKind::Graph(Dir::Out),
                    what: vec![LookupSubject::Table {
                        table: "workshop_item_properties".to_string(),
                        referencing_field: None,
                    }],
                    ..Default::default()
                }),
                Part::All,
            ])),
            // ToDo: Filter for user submitted props
            alias: Some(Idiom(vec![
                Part::Field("properties".to_string()),
                Part::Method(
                    "filter".to_string(),
                    vec![Expr::Closure(Box::new(Closure {
                        args: vec![(Param::new("prop".to_string()), Kind::Any)],
                        returns: None,
                        body: Expr::Binary {
                            left: Box::new(Expr::Idiom(Idiom(vec![
                                Part::Start(Expr::Param(Param::new("prop".to_string()))),
                                Part::Field("status".to_string()),
                            ]))),
                            op: BinaryOperator::ExactEqual,
                            right: Box::new(Expr::Literal(Literal::Integer(
                                Status::Accepted as i64,
                            ))),
                        },
                    }))],
                ),
            ])),
        }),
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![
                Part::Graph(Lookup {
                    kind: LookupKind::Graph(Dir::Out),
                    what: vec![LookupSubject::Table {
                        table: "item_dependencies".to_string(),
                        referencing_field: None,
                    }],
                    ..Default::default()
                }),
                Part::All,
                Part::Field("in".to_string()),
                Part::All,
                Part::Destructure(dep_fields.to_vec()),
            ])),
            alias: Some(Idiom::field("dependencies".to_string())),
        }),
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![
                Part::Graph(Lookup {
                    kind: LookupKind::Graph(Dir::In),
                    what: vec![LookupSubject::Table {
                        table: "item_dependencies".to_string(),
                        referencing_field: None,
                    }],
                    ..Default::default()
                }),
                Part::All,
                Part::Field("in".to_string()),
                Part::All,
                Part::Destructure(dep_fields.to_vec()),
            ])),
            alias: Some(Idiom::field("dependants".to_string())),
        }),
    ]);

    debug!(sql = stmt.to_sql(), "item query");

    let result: Option<InternalFullWorkshopItem> = db
        .query(stmt)
        .bind(("id", RecordId::from(id)))
        .await
        .inspect_err(|error| error!(message = "get_item", ?error, "Failed to query database"))
        .map_err(|_| InnerError::InternalError)?
        .take(0)
        .inspect_err(|error| error!(message = "get_item", ?error, "Failed to take result"))
        .map_err(|_| InnerError::InternalError)?;
    result.ok_or(Error::from(InnerError::NotFound))
}

/// GET /api/item/{id}
/// Retrieves a full workshop item by id, including dependencies and dependants.
#[endpoint]
#[instrument(skip_all)]
pub async fn get(id: PathParam<i64>, depot: &mut Depot) -> Result<Json<ExternalFullWorkshopItem>> {
    // Lazily spawn the actor on first use and keep a global reference like auth.rs
    let actor = ITEM_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;

    let user = auth::get_user_from_depot(depot).map(Into::into);
    let data = call!(actor, |reply| { ItemMsg::Get(id.0.into(), user, reply) })
        .map_err(|_| InnerError::InternalError)??;
    Ok(Json(
        data.try_into().map_err(|_| InnerError::InternalError)?,
    ))
}

/// GET /api/item/{id}/app
/// Retrieves the app
#[endpoint]
#[instrument(skip_all)]
pub async fn app_from_item(
    id: PathParam<i64>,
    depot: &mut Depot,
) -> Result<Json<ExternalFullWorkshopItem>> {
    // Lazily spawn the actor on first use and keep a global reference like auth.rs
    let actor = ITEM_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;

    let user = auth::get_user_from_depot(depot).map(Into::into);
    let data = call!(actor, |reply| { ItemMsg::Get(id.0.into(), user, reply) })
        .map_err(|_| InnerError::InternalError)??;
    Ok(Json(
        data.try_into().map_err(|_| InnerError::InternalError)?,
    ))
}
