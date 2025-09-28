pub mod item_update_actor;
pub mod model;

use macros::define_id;

define_id!("users", UserID, String);
define_id!("workshop_items", ItemID, String);
