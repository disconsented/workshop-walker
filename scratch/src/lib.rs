use surrealdb_types::SurrealValue;
use surrealdb_types::Value;

#[derive(Debug, SurrealValue, PartialEq)]
pub enum Class {
    Type,
    Theme,
    Genre,
    Feature,
}

#[test]
fn test_reproduce_class_deserialization_bug() {
    let val = Value::String("Type".to_string());
    
    // This is expected to fail with the current SurrealValue derive implementation
    let result = Class::from_value(val);
    
    match result {
        Ok(c) => println!("Successfully deserialized: {:?}", c),
        Err(e) => {
            panic!("Failed to deserialize Class from String: {}", e);
        }
    }
}
