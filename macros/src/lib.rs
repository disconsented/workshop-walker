/// A helper macro to define a newtype for a SurrealDB record ID
/// Makes it not a total pain in the ass to deal with
///
/// Args:
/// Table name: The name of the SurrealDB table
/// Internal name: The name of the internal newtype (interfaces with the DB)
/// External name: The name of the external newtype (exposed to users)
/// External type: The underlying type of the external newtype
#[macro_export]
macro_rules! define_id {
    ($table:literal, $internal:ident, $external:ident, $external_type:ty) => {
        /// Externally facing newtype over $external_type
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        #[serde(transparent)]
        pub struct $external($external_type);
        impl Into<$external_type> for $external {
            fn into(self) -> $external_type {
                self.0
            }
        }

        impl From<$external_type> for $external {
            fn from(id: $external_type) -> Self {
                Self(id)
            }
        }
        /// Internally facing newtype over RecordID
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        #[serde(transparent)]
        pub struct $internal(surrealdb_types::RecordId);
        impl $internal {
            const TABLE_NAME: &'static str = $table;
        }
        impl From<$external> for $internal {
            fn from(id: $external) -> Self {
                Self(surrealdb_types::RecordId::new(Self::TABLE_NAME, id.0))
            }
        }
        impl Into<surrealdb_types::RecordId> for $internal {
            fn into(self) -> surrealdb_types::RecordId {
                self.0
            }
        }

        impl TryInto<$external> for $internal {
            type Error = surrealdb_types::Error;
            fn try_into(self) -> Result<$external, Self::Error> {
                Ok($external(surrealdb_types::SurrealValue::into_value(self.0.key).into_t()?))
            }
        }
    };
}

