use core::fmt::{Display, Formatter};

use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use surrealdb_types::{Error, Kind, KindLiteral, SurrealValue, Value};

use crate::db::{IUserID, model::InternalSource};

#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Class {
    /// Anything like addon, overhaul, bugfix, patch
    Type,
    /// Literary themes like mecha
    Theme,
    /// Literary genres like `CyberPunk`
    Genre,
    /// Mod features, like "new scenario" or "new clothes"
    Feature,
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            Class::Type => "Type",
            Class::Theme => "Theme",
            Class::Genre => "Genre",
            Class::Feature => "Feature",
        };
        f.write_str(txt)
    }
}

impl SurrealValue for InternalSource {
    fn kind_of() -> Kind {
        Kind::Either(vec![
            Kind::Literal(KindLiteral::String("system".to_string())),
            <IUserID as SurrealValue>::kind_of(),
        ])
    }

    fn is_value(value: &Value) -> bool {
        match value {
            Value::String(string) => string == "system",
            other => <IUserID as SurrealValue>::is_value(other),
        }
    }

    fn into_value(self) -> Value {
        // The schema declares `'system' | record<users>`, so `System` must go
        // out as the bare string, not as an enum-shaped object.
        match self {
            Self::System => Value::String("system".to_string()),
            Self::User(field_0) => Value::RecordId(field_0.into()),
        }
    }

    fn from_value(value: Value) -> Result<Self, Error> {
        match value {
            Value::Object(mut map) => {
                {
                    if map.get("System").is_some() {
                        return Ok(Self::System);
                    }
                }
                {
                    if let Some(value) = map.remove("User") {
                        {
                            let field_0 = <IUserID as SurrealValue>::from_value(value)?;
                            return Ok(Self::User(field_0));
                        }
                    }
                }
            }
            Value::String(string) => {
                if string == "system" {
                    return Ok(Self::System);
                }
            }
            other => {
                if let Ok(user_id) = IUserID::from_value(other) {
                    return Ok(Self::User(user_id));
                }
            }
        }

        Err(Error::internal(format!(
            "Failed to decode {}, no variants matched",
            "InternalSource"
        )))
    }
}

impl SurrealValue for Class {
    fn kind_of() -> Kind {
        Kind::Either(vec![
            Kind::Literal(KindLiteral::String("Type".to_string())),
            Kind::Literal(KindLiteral::String("Theme".to_string())),
            Kind::Literal(KindLiteral::String("Genre".to_string())),
            Kind::Literal(KindLiteral::String("Feature".to_string())),
        ])
    }

    fn is_value(value: &Value) -> bool {
        matches!(
            value,
            Value::String(s)
                if matches!(s.as_str(), "Type" | "Theme" | "Genre" | "Feature")
        )
    }

    fn into_value(self) -> Value {
        let variant = match self {
            Self::Type => "Type",
            Self::Theme => "Theme",
            Self::Genre => "Genre",
            Self::Feature => "Feature",
        };
        Value::String(variant.to_string())
    }

    fn from_value(value: Value) -> Result<Self, Error> {
        match value {
            Value::Object(map) => {
                {
                    if map.get("Type").is_some() {
                        return Ok(Self::Type);
                    }
                }
                {
                    if map.get("Theme").is_some() {
                        return Ok(Self::Theme);
                    }
                }
                {
                    if map.get("Genre").is_some() {
                        return Ok(Self::Genre);
                    }
                }
                {
                    if map.get("Feature").is_some() {
                        return Ok(Self::Feature);
                    }
                }
            }
            Value::String(value) => match value.as_str() {
                "Type" => return Ok(Self::Type),
                "Theme" => return Ok(Self::Theme),
                "Genre" => return Ok(Self::Genre),
                "Feature" => return Ok(Self::Feature),
                _ => {}
            },
            _ => {}
        }
        Err(Error::internal(format!(
            "Failed to decode {}, no variants matched",
            "Class"
        )))
    }
}
