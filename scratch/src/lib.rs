use macros::define_id;

define_id!("users", Internal, External, String);

// ///   Externally facing newtype over $external_type
// #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
// #[serde(transparent)]
// pub struct External(String);
// impl Into<String> for External {
//     fn into(self) -> String {
//         self.0
//     }
// }
// impl From<String> for External {
//     fn from(id: String) -> Self {
//         Self(id)
//     }
// }
// ///   Internally facing newtype over RecordID
// #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
// #[serde(transparent)]
// pub struct Internal(surrealdb_types::RecordId);
// impl Internal {
//     const TABLE_NAME: &'static str = "users";
// }
// impl From<External> for Internal {
//     fn from(id: External) -> Self {
//         Self(surrealdb_types::RecordId::new(Self::TABLE_NAME, id.0))
//     }
// }
// impl Into<surrealdb_types::RecordId> for Internal {
//     fn into(self) -> surrealdb_types::RecordId {
//         self.0
//     }
// }
// impl TryInto<External> for Internal {
//     type Error = surrealdb_types::Error;
//
//     fn try_into(self) -> Result<External, Self::Error> {
//         Ok(External(
//             surrealdb_types::SurrealValue::into_value(self.0.key).into_t()?,
//         ))
//     }
// }

#[cfg(test)]
mod tests {
    use derive_more::Into;

    // use macros::define_id;
    use crate::{External, Internal};

    #[derive(Into)]
    struct TestStruct<T> {
        id: T,
        something: String,
    }

    #[test]
    fn it_works() {
        let external = External::from("test".to_string());
        let internal = Internal::from(external);
        let external2: External = internal.try_into().unwrap();
        let raw: String = external2.into();
        assert_eq!(raw, "test");

    }

    #[test]
    fn it_works_2() {
        let external = TestStruct::<External> {
            id: String::from("test").into(),
            something: "".to_string(),
        };

        let internal: TestStruct<Internal> = external.into();
    }
}
