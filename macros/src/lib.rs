/// Table name, type name, underlying key type
#[macro_export]
macro_rules! define_id {
    ($table:literal, $name:ident, $type:ty) => {
        /// A SurrealDB ID abstraction to work around some points of pain
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        #[serde(transparent)]
        pub struct $name($type);
        impl $name {
            const TABLE_NAME: &'static str = $table;

            pub fn into_recordid(self) -> surrealdb::RecordId {
                Into::<surrealdb::RecordId>::into(self)
            }
        }
        impl Into<surrealdb::RecordId> for $name {
            fn into(self) -> surrealdb::RecordId {
                surrealdb::RecordId::from_table_key(Self::TABLE_NAME, self.0)
            }
        }
        impl Into<$type> for $name {
            fn into(self) -> $type {
                self.0
            }
        }

        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                Self(value)
            }
        }
    };
}
// #[macro_export]
// macro_rules! define_dual {
//     ($($id:ident : $type:ty),* $(,)?) => {
//         #[derive(serde::Serialize, serde::Deserialize)]
//         struct Test {
//             pub id: String,
//             $($id : $type),*
//         }
//     };
// }
