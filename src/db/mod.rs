pub mod admin_actor;
pub mod admin_repository;
pub mod item_update_actor;
pub mod model;
pub mod properties_actor;
pub mod properties_repository;

use macros::define_id;

define_id!("users", UserID, String);
define_id!("workshop_items", ItemID, String);
