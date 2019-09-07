use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::{JsonSchema, Map, Result};
use serde_json::json;

macro_rules! simple_impl {
    ($type:tt => $instance_type:ident) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                stringify!($instance_type).to_owned()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Result {
                Ok(SchemaObject {
                    instance_type: Some(InstanceType::$instance_type.into()),
                    ..Default::default()
                }
                .into())
            }
        }
    };
}

simple_impl!(str => String);
simple_impl!(String => String);
simple_impl!(bool => Boolean);
simple_impl!(f32 => Number);
simple_impl!(f64 => Number);
simple_impl!(i8 => Integer);
simple_impl!(i16 => Integer);
simple_impl!(i32 => Integer);
simple_impl!(i64 => Integer);
simple_impl!(i128 => Integer);
simple_impl!(isize => Integer);
simple_impl!(u8 => Integer);
simple_impl!(u16 => Integer);
simple_impl!(u32 => Integer);
simple_impl!(u64 => Integer);
simple_impl!(u128 => Integer);
simple_impl!(usize => Integer);
simple_impl!(() => Null);

impl JsonSchema for char {
    no_ref_schema!();

    fn schema_name() -> String {
        "Character".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Result {
        let mut extensions = Map::new();
        extensions.insert("minLength".to_owned(), json!(1));
        extensions.insert("maxLength".to_owned(), json!(1));
        Ok(SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            extensions,
            ..Default::default()
        }
        .into())
    }
}
