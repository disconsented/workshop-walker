pub use proc_macros::dual_struct;

/// A helper macro to define a newtype for a SurrealDB record ID.
/// Makes it not a total pain in the ass to deal with.
///
/// Implemented with 2 structs because, we can't maintain a transparent newtype
/// with an enum, which, makes sense as its a different type.
///
/// ## Args:
/// Table name: The name of the SurrealDB table\
/// Internal name: The name of the internal newtype (interfaces with the DB)\
/// External name: The name of the external newtype (exposed to users)\
/// External type: The underlying type of the external newtype
#[macro_export]
macro_rules! define_id {
    ($table:literal, $internal:ident, $external:ident, $external_type:ty) => {
        /// Externally facing newtype over $external_type
        #[derive(
            serde::Serialize,
            serde::Deserialize,
            Clone,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            salvo::prelude::ToSchema,
        )]
        #[serde(transparent)]
        pub struct $external($external_type);
        impl From<$external> for $external_type {
            fn from(id: $external) -> $external_type {
                id.0
            }
        }

        impl From<$external_type> for $external {
            fn from(id: $external_type) -> Self {
                Self(id)
            }
        }
        /// Internally facing newtype over RecordID
        #[derive(
            serde::Serialize,
            serde::Deserialize,
            Clone,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            surrealdb_types::SurrealValue,
        )]
        #[serde(transparent)]
        pub struct $internal(surrealdb_types::RecordId);
        impl $internal {
            const TABLE_NAME: &'static str = $table;

            // An alias of TryInto<$external> for $internal to make it easier to use
            pub fn try_into_external(self) -> Result<$external, surrealdb_types::Error> {
                self.try_into()
            }
        }
        impl From<$external> for $internal {
            fn from(id: $external) -> Self {
                #[allow(
                    clippy::disallowed_methods,
                    reason = "Macro use only, this is considered correct"
                )]
                Self(surrealdb_types::RecordId::new(Self::TABLE_NAME, id.0))
            }
        }
        impl From<$external_type> for $internal {
            fn from(id: $external_type) -> Self {
                #[allow(
                    clippy::disallowed_methods,
                    reason = "Macro use only, this is considered correct"
                )]
                Self(surrealdb_types::RecordId::new(Self::TABLE_NAME, id))
            }
        }

        impl From<$internal> for surrealdb_types::RecordId {
            fn from(id: $internal) -> surrealdb_types::RecordId {
                id.0
            }
        }

        impl TryFrom<$internal> for $external {
            type Error = surrealdb_types::Error;

            fn try_from(id: $internal) -> Result<$external, Self::Error> {
                Ok($external(id.0.key.into_value().into_t()?))
            }
        }

        impl std::ops::Deref for $internal {
            type Target = surrealdb_types::RecordId;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
