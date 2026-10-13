use std::collections::BTreeMap;

use salvo::oapi::ToSchema;
use serde_repr::{Deserialize_repr, Serialize_repr};
use surrealdb_types::{Kind, KindLiteral, Number, SurrealValue};

#[derive(
    Debug,
    Default,
    ToSchema,
    Copy,
    Clone,
    Serialize_repr,
    Deserialize_repr,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
#[repr(i8)]
pub enum Status {
    Hidden = -2,
    Rejected = -1,
    #[default]
    Pending = 0,
    Accepted = 1,
}

// Horrendous manual hack thanks to _another_ surreal bug
impl SurrealValue for Status {
    fn kind_of() -> Kind {
        Kind::Either(vec![
            {
                let mut obj = BTreeMap::new();
                obj.insert(
                    "Hidden".to_string(),
                    Kind::Literal(KindLiteral::Object(BTreeMap::new())),
                );
                Kind::Literal(KindLiteral::Object(obj))
            },
            {
                let mut obj = BTreeMap::new();
                obj.insert(
                    "Rejected".to_string(),
                    Kind::Literal(KindLiteral::Object(BTreeMap::new())),
                );
                Kind::Literal(KindLiteral::Object(obj))
            },
            {
                let mut obj = BTreeMap::new();
                obj.insert(
                    "Pending".to_string(),
                    Kind::Literal(KindLiteral::Object(BTreeMap::new())),
                );
                Kind::Literal(KindLiteral::Object(obj))
            },
            {
                let mut obj = BTreeMap::new();
                obj.insert(
                    "Accepted".to_string(),
                    Kind::Literal(KindLiteral::Object(BTreeMap::new())),
                );
                Kind::Literal(KindLiteral::Object(obj))
            },
        ])
    }

    fn is_value(value: &surrealdb::types::Value) -> bool {
        if let surrealdb::types::Value::Object(map) = value {
            {
                if map
                    .get("Hidden")
                    .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                {
                    return true;
                }
            }
            {
                if map
                    .get("Rejected")
                    .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                {
                    return true;
                }
            }
            {
                if map
                    .get("Pending")
                    .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                {
                    return true;
                }
            }
            {
                if map
                    .get("Accepted")
                    .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                {
                    return true;
                }
            }
        }
        false
    }

    fn into_value(self) -> surrealdb::types::Value {
        surrealdb::types::Value::Number(Number::Int(self as i64))
    }

    fn from_value(value: surrealdb::types::Value) -> Result<Self, surrealdb::types::Error> {
        match value {
            surrealdb::types::Value::Object(map) => {
                {
                    if map.get("Hidden").is_some() {
                        return Ok(Self::Hidden);
                    }
                }

                {
                    if map.get("Rejected").is_some() {
                        return Ok(Self::Rejected);
                    }
                }
                {
                    if map.get("Pending").is_some() {
                        return Ok(Self::Pending);
                    }
                }
                {
                    if map.get("Accepted").is_some() {
                        return Ok(Self::Accepted);
                    }
                }
            }
            surrealdb::types::Value::Number(number) => match number {
                Number::Int(-2) => {
                    return Ok(Self::Hidden);
                }
                Number::Int(-1) => {
                    return Ok(Self::Rejected);
                }
                Number::Int(0) => {
                    return Ok(Self::Pending);
                }
                Number::Int(1) => {
                    return Ok(Self::Accepted);
                }
                _ => {}
            },
            _ => {}
        }
        Err(surrealdb::types::Error::internal(format!(
            "Failed to decode {}, no variants matched",
            "Status"
        )))
    }
}
