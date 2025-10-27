pub mod item_update_actor;
pub mod model;
pub mod properties_repository;
pub mod properties_actor;

use macros::define_id;

define_id!("users", UserID, String);
define_id!("workshop_items", ItemID, String);
