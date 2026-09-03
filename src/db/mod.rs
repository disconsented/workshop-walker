pub mod admin_actor;
pub mod admin_repository;
pub mod apps_actor;
pub mod apps_repository;
pub mod item_update_actor;
pub mod model;
pub mod properties_actor;
pub mod properties_repository;
pub mod tags_repository;
pub mod user_names_repository;

use macros::define_id;
use surrealdb_types::SurrealValue;

use crate::db::model::Property;

define_id!("users", IUserID, UserID, i64);
define_id!("workshop_items", IItemID, ItemID, i64);
define_id!("apps", IAppID, AppID, i64);
define_id!("tags", ITagID, TagID, String);
define_id!("usernames", IUsernameID, UsernameID, i64);
define_id!("properties", IPropertyID, PropertyID, Property);
define_id!("votes", IVoteID, VoteID, String);
