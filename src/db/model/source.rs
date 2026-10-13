use salvo::oapi::ToSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use serde_content::{Value, ValueVisitor};
use serde_hack::ValueRefDeserializer;

use crate::db::{IUserID, UserID};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InternalSource {
    /// Auto-generated
    System,
    /// User submitted
    User(IUserID),
}

impl serde::Serialize for InternalSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            InternalSource::System => serializer.serialize_str("system"),
            InternalSource::User(t) => t.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for InternalSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <Value as serde::Deserialize>::deserialize(deserializer)?;
        let deserializer = ValueRefDeserializer::<D::Error>::new(&value);
        let value = deserializer.deserialize_any(ValueVisitor)?;

        match value {
            Value::String(str) if str == "system" => Ok(InternalSource::System),
            _ => <IUserID as serde::Deserialize>::deserialize(deserializer)
                .map(InternalSource::User)
                .map_err(|_| {
                    Error::custom("data did not match any variant of untagged enum Source")
                }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ToSchema, Serialize, Deserialize)]
pub enum ExternalSource {
    /// Auto-generated
    System,
    /// User submitted
    User(UserID),
}

impl TryFrom<InternalSource> for ExternalSource {
    type Error = surrealdb_types::Error;

    fn try_from(value: InternalSource) -> Result<Self, Self::Error> {
        match value {
            InternalSource::System => Ok(ExternalSource::System),
            InternalSource::User(t) => Ok(ExternalSource::User(t.try_into()?)),
        }
    }
}

impl From<ExternalSource> for InternalSource {
    fn from(value: ExternalSource) -> Self {
        match value {
            ExternalSource::System => Self::System,
            ExternalSource::User(t) => Self::User(t.into()),
        }
    }
}
