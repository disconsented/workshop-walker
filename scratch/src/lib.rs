use macros::define_id;

define_id!("users", Internal, External, String);

#[cfg(test)]
mod tests {
    use proc_macros::ConvertId;

    use crate::{External, Internal};

    #[derive(ConvertId)]
    struct TestStruct<T> {
        id: T,
        something: String,
    }

    #[test]
    fn test_type_conversion() {
        let external = External::from("test".to_string());
        let internal = Internal::from(external);
        let external2: External = internal.try_into().unwrap();
        let raw: String = external2.into();
        assert_eq!(raw, "test");
    }

    #[test]
    fn test_struct_conversion() {
        let external = TestStruct::<External> {
            id: String::from("test").into(),
            something: "".to_string(),
        };

        let internal: TestStruct<Internal> = external.into();
        let _external2: TestStruct<External> = internal.try_into().unwrap();
    }
}
