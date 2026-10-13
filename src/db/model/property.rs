use core::fmt::{Display, Formatter};

use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use surrealdb_core::types::SurrealValue;
use surrealdb_types::{Object, RecordIdKey};

use crate::db::model::class::Class;

/// Crowdsourced metadata for an item, private version
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    ToSchema,
    Eq,
    PartialEq,
    Hash,
    SurrealValue,
    Ord,
    PartialOrd,
)]
pub struct Property {
    pub class: Class,
    pub value: String,
}

impl Display for Property {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.class.to_string())?;
        f.write_str(":")?;
        f.write_str(&self.value)
    }
}

impl From<Property> for RecordIdKey {
    fn from(val: Property) -> Self {
        let mut obj = Object::new();
        obj.insert("class", val.class);
        obj.insert("value", val.value);
        RecordIdKey::Object(obj)
    }
}
